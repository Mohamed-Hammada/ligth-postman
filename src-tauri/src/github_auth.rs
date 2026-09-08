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
}
