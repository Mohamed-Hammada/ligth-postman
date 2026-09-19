use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, USER_AGENT};
use serde::{Deserialize, Serialize};

use crate::error::AppError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitHubUser {
    pub login: String,
    pub id: u64,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub avatar_url: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitHubRepoPermissions {
    #[serde(default)]
    pub admin: bool,
    #[serde(default)]
    pub push: bool,
    #[serde(default)]
    pub pull: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitHubRepoInfo {
    pub full_name: String,
    pub private: bool,
    pub default_branch: String,
    #[serde(default)]
    pub permissions: Option<GitHubRepoPermissions>,
}

/// Repository access level to grant when inviting someone. Maps onto GitHub's permission names:
/// viewer = `pull` (read-only), collaborator = `push` (read + write), admin = `admin`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollaboratorRole {
    Viewer,
    Collaborator,
    Admin,
}

impl CollaboratorRole {
    pub fn parse(role: &str) -> Result<Self, AppError> {
        match role.trim().to_ascii_lowercase().as_str() {
            "viewer" | "read" | "pull" => Ok(Self::Viewer),
            "collaborator" | "editor" | "write" | "push" => Ok(Self::Collaborator),
            "admin" => Ok(Self::Admin),
            other => Err(AppError::Validation(format!(
                "Unknown role '{other}' - use viewer, collaborator or admin"
            ))),
        }
    }

    pub fn github_permission(self) -> &'static str {
        match self {
            Self::Viewer => "pull",
            Self::Collaborator => "push",
            Self::Admin => "admin",
        }
    }
}

/// Outcome of an invite: GitHub answers 201 with an invitation for people who must accept first,
/// and 204 when the account already had access (the permission is simply updated).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InviteResult {
    /// "invited" (pending acceptance) or "updated" (already had access).
    pub status: String,
    pub username: String,
    pub repo: String,
    pub role: String,
    #[serde(default)]
    pub invitation_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitHubCollaborator {
    pub login: String,
    #[serde(default)]
    pub avatar_url: Option<String>,
    /// GitHub's repo-level role name (read / triage / write / maintain / admin).
    #[serde(default)]
    pub role: String,
    /// True for invitations that have not been accepted yet.
    #[serde(default)]
    pub pending: bool,
}

/// Extracts `(owner, repo)` from a GitHub remote - https (`https://github.com/o/r(.git)`, with or
/// without embedded credentials) or ssh (`git@github.com:o/r.git`). `None` for non-GitHub remotes.
pub fn parse_github_remote(remote_url: &str) -> Option<(String, String)> {
    let url = remote_url.trim();
    let path = if let Some(rest) = url.strip_prefix("git@github.com:") {
        rest
    } else {
        let after_scheme = url.split("://").nth(1)?;
        let after_creds = after_scheme.rsplit('@').next()?;
        after_creds.strip_prefix("github.com/")?
    };
    let path = path.trim_matches('/').trim_end_matches(".git");
    let mut parts = path.split('/');
    let owner = parts.next()?.trim();
    let repo = parts.next()?.trim();
    if owner.is_empty() || repo.is_empty() || parts.next().is_some() {
        return None;
    }
    Some((owner.to_string(), repo.to_string()))
}

/// GitHub logins are 1-39 chars of ASCII alphanumerics and single hyphens (no leading/trailing
/// hyphen). Emails are rejected explicitly because the collaborators API only accepts usernames.
pub fn validate_github_username(username: &str) -> Result<String, AppError> {
    let name = username.trim().trim_start_matches('@');
    if name.contains('@') {
        return Err(AppError::Validation(
            "Enter the person's GitHub username - GitHub can't add collaborators by email address".into(),
        ));
    }
    let valid = !name.is_empty()
        && name.len() <= 39
        && !name.starts_with('-')
        && !name.ends_with('-')
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-');
    if !valid {
        return Err(AppError::Validation(format!("'{name}' is not a valid GitHub username")));
    }
    Ok(name.to_string())
}

pub struct GitHubService;

impl GitHubService {
    fn build_client(token: &str) -> Result<reqwest::Client, AppError> {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("light-postman/1.0"));

        let auth_val = format!("Bearer {}", token.trim());
        let mut auth_header = HeaderValue::from_str(&auth_val)
            .map_err(|e| AppError::Validation(format!("Invalid GitHub token header: {e}")))?;
        auth_header.set_sensitive(true);
        headers.insert(AUTHORIZATION, auth_header);

        reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .map_err(|e| AppError::Network(format!("Failed to build HTTP client: {e}")))
    }

    /// Validates GitHub personal access token or OAuth token against the GitHub API (LP-0704).
    pub async fn verify_token(token: &str) -> Result<GitHubUser, AppError> {
        if token.trim().is_empty() {
            return Err(AppError::Validation("GitHub token cannot be empty".into()));
        }

        let client = Self::build_client(token)?;
        let resp = client
            .get("https://api.github.com/user")
            .send()
            .await
            .map_err(|e| AppError::Network(format!("Failed to reach GitHub API: {e}")))?;

        if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(AppError::Validation("Invalid or expired GitHub token".into()));
        }

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(AppError::Network(format!(
                "GitHub API error ({status}): {body}"
            )));
        }

        let user: GitHubUser = resp
            .json()
            .await
            .map_err(|e| AppError::Network(format!("Failed to parse GitHub user response: {e}")))?;

        Ok(user)
    }

    /// Fetches repository information and verified user permissions (LP-0703).
    pub async fn get_repo_info(
        token: &str,
        owner: &str,
        repo: &str,
    ) -> Result<GitHubRepoInfo, AppError> {
        let client = Self::build_client(token)?;
        let url = format!("https://api.github.com/repos/{owner}/{repo}");

        let resp = client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::Network(format!("Failed to reach GitHub API: {e}")))?;

        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(AppError::NotFound(format!(
                "Repository {owner}/{repo} not found or inaccessible"
            )));
        }

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(AppError::Network(format!(
                "GitHub API error ({status}): {body}"
            )));
        }

        let info: GitHubRepoInfo = resp
            .json()
            .await
            .map_err(|e| AppError::Network(format!("Failed to parse repository response: {e}")))?;

        Ok(info)
    }

    /// Adds `username` to `owner/repo` with the given role. The token's owner needs admin rights on
    /// the repository (GitHub answers 403/404 otherwise, which is reported as such).
    pub async fn invite_collaborator(
        token: &str,
        owner: &str,
        repo: &str,
        username: &str,
        role: CollaboratorRole,
    ) -> Result<InviteResult, AppError> {
        let client = Self::build_client(token)?;
        let url = format!("https://api.github.com/repos/{owner}/{repo}/collaborators/{username}");
        let resp = client
            .put(&url)
            .header("Accept", "application/vnd.github+json")
            .json(&serde_json::json!({ "permission": role.github_permission() }))
            .send()
            .await
            .map_err(|e| AppError::Network(format!("Failed to reach GitHub API: {e}")))?;

        let status = resp.status();
        let repo_full = format!("{owner}/{repo}");
        match status {
            reqwest::StatusCode::CREATED => {
                let body: serde_json::Value = resp.json().await.unwrap_or_default();
                Ok(InviteResult {
                    status: "invited".into(),
                    username: username.to_string(),
                    repo: repo_full,
                    role: role.github_permission().into(),
                    invitation_url: body.get("html_url").and_then(|v| v.as_str()).map(String::from),
                })
            }
            reqwest::StatusCode::NO_CONTENT => Ok(InviteResult {
                status: "updated".into(),
                username: username.to_string(),
                repo: repo_full,
                role: role.github_permission().into(),
                invitation_url: None,
            }),
            reqwest::StatusCode::UNAUTHORIZED => {
                Err(AppError::Validation("Invalid or expired GitHub token".into()))
            }
            reqwest::StatusCode::FORBIDDEN => Err(AppError::Validation(format!(
                "GitHub refused the invite - the token needs admin access to {repo_full} (and the 'repo' scope)"
            ))),
            reqwest::StatusCode::NOT_FOUND => Err(AppError::NotFound(format!(
                "Repository {repo_full} or user '{username}' not found (or the token can't see the repository)"
            ))),
            reqwest::StatusCode::UNPROCESSABLE_ENTITY => {
                let body = resp.text().await.unwrap_or_default();
                Err(AppError::Validation(format!(
                    "GitHub couldn't add '{username}' to {repo_full}: {body}"
                )))
            }
            _ => {
                let body = resp.text().await.unwrap_or_default();
                Err(AppError::Network(format!("GitHub API error ({status}): {body}")))
            }
        }
    }

    /// Current collaborators plus not-yet-accepted invitations, for the Invite dialog.
    pub async fn list_collaborators(
        token: &str,
        owner: &str,
        repo: &str,
    ) -> Result<Vec<GitHubCollaborator>, AppError> {
        let client = Self::build_client(token)?;
        let base = format!("https://api.github.com/repos/{owner}/{repo}");
        let mut out = Vec::new();

        let resp = client
            .get(format!("{base}/collaborators?per_page=100"))
            .header("Accept", "application/vnd.github+json")
            .send()
            .await
            .map_err(|e| AppError::Network(format!("Failed to reach GitHub API: {e}")))?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(AppError::Network(format!("GitHub API error ({status}): {body}")));
        }
        let list: Vec<serde_json::Value> = resp
            .json()
            .await
            .map_err(|e| AppError::Network(format!("Failed to parse collaborators: {e}")))?;
        for c in list {
            out.push(GitHubCollaborator {
                login: c.get("login").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
                avatar_url: c.get("avatar_url").and_then(|v| v.as_str()).map(String::from),
                role: c.get("role_name").and_then(|v| v.as_str()).unwrap_or("read").to_string(),
                pending: false,
            });
        }

        // Pending invitations are best-effort: a token that can list collaborators but not
        // invitations shouldn't make the whole dialog fail.
        if let Ok(resp) = client
            .get(format!("{base}/invitations?per_page=100"))
            .header("Accept", "application/vnd.github+json")
            .send()
            .await
        {
            if resp.status().is_success() {
                if let Ok(list) = resp.json::<Vec<serde_json::Value>>().await {
                    for inv in list {
                        let invitee = inv.get("invitee");
                        out.push(GitHubCollaborator {
                            login: invitee
                                .and_then(|i| i.get("login"))
                                .and_then(|v| v.as_str())
                                .unwrap_or_default()
                                .to_string(),
                            avatar_url: invitee
                                .and_then(|i| i.get("avatar_url"))
                                .and_then(|v| v.as_str())
                                .map(String::from),
                            role: inv.get("permissions").and_then(|v| v.as_str()).unwrap_or("read").to_string(),
                            pending: true,
                        });
                    }
                }
            }
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_mock_github_user() {
        let json = r#"{
            "login": "octocat",
            "id": 1,
            "name": "The Octocat",
            "avatar_url": "https://github.com/images/error/octocat_happy.gif",
            "email": "octocat@github.com"
        }"#;

        let user: GitHubUser = serde_json::from_str(json).unwrap();
        assert_eq!(user.login, "octocat");
        assert_eq!(user.name.as_deref(), Some("The Octocat"));
        assert_eq!(user.id, 1);
    }

    #[test]
    fn parses_mock_repo_permissions() {
        let json = r#"{
            "full_name": "octocat/Hello-World",
            "private": false,
            "default_branch": "main",
            "permissions": {
                "admin": false,
                "push": true,
                "pull": true
            }
        }"#;

        let info: GitHubRepoInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.full_name, "octocat/Hello-World");
        assert!(!info.private);
        assert_eq!(info.default_branch, "main");
        let perms = info.permissions.unwrap();
        assert!(perms.push);
        assert!(perms.pull);
        assert!(!perms.admin);
    }

    #[test]
    fn parses_github_remotes() {
        let ok = Some(("acme".to_string(), "api-specs".to_string()));
        assert_eq!(parse_github_remote("https://github.com/acme/api-specs.git"), ok);
        assert_eq!(parse_github_remote("https://github.com/acme/api-specs"), ok);
        assert_eq!(parse_github_remote("https://ghp_token@github.com/acme/api-specs.git"), ok);
        assert_eq!(parse_github_remote("https://user:pass@github.com/acme/api-specs/"), ok);
        assert_eq!(parse_github_remote("git@github.com:acme/api-specs.git"), ok);
        assert_eq!(parse_github_remote("https://gitlab.com/acme/api-specs.git"), None);
        assert_eq!(parse_github_remote("https://github.com/acme"), None);
        assert_eq!(parse_github_remote("https://github.com/acme/api/extra"), None);
        assert_eq!(parse_github_remote("not a url"), None);
    }

    #[test]
    fn maps_roles_to_github_permissions() {
        assert_eq!(CollaboratorRole::parse("viewer").unwrap().github_permission(), "pull");
        assert_eq!(CollaboratorRole::parse("Collaborator").unwrap().github_permission(), "push");
        assert_eq!(CollaboratorRole::parse("editor").unwrap().github_permission(), "push");
        assert_eq!(CollaboratorRole::parse("admin").unwrap().github_permission(), "admin");
        assert!(CollaboratorRole::parse("owner").is_err());
    }

    #[test]
    fn validates_github_usernames() {
        assert_eq!(validate_github_username("  @octocat ").unwrap(), "octocat");
        assert_eq!(validate_github_username("a-b-1").unwrap(), "a-b-1");
        assert!(validate_github_username("dev@company.com").is_err());
        assert!(validate_github_username("").is_err());
        assert!(validate_github_username("-bad").is_err());
        assert!(validate_github_username("bad-").is_err());
        assert!(validate_github_username("has space").is_err());
        assert!(validate_github_username(&"a".repeat(40)).is_err());
    }
}
