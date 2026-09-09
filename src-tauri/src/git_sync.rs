use std::path::{Path, PathBuf};
use std::process::Command;

use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::error::AppError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitStatus {
    pub is_repo: bool,
    pub branch: String,
    pub status_kind: String, // "clean", "modified", "ahead", "behind", "diverged", "conflict", "no_remote", "untracked"
    pub ahead: usize,
    pub behind: usize,
    pub staged_files: Vec<String>,
    pub unstaged_files: Vec<String>,
    pub untracked_files: Vec<String>,
    pub has_conflicts: bool,
    pub conflict_files: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitCommit {
    pub hash: String,
    pub author: String,
    pub date: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectGitSettings {
    pub project_id: String,
    pub repo_path: Option<String>,
    pub remote_url: Option<String>,
    pub branch: String,
    pub auto_sync: bool,
    pub github_token: Option<String>,
    pub last_sync_at: Option<String>,
}

pub struct GitService;

impl GitService {
    pub fn is_git_installed() -> bool {
        Command::new("git")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    pub fn is_repo(dir: &Path) -> bool {
        if !dir.exists() || !dir.is_dir() {
            return false;
        }
        dir.join(".git").exists()
    }

    pub fn run_git_cmd(args: &[&str], cwd: &Path) -> Result<String, AppError> {
        let output = Command::new("git")
            .args(args)
            .current_dir(cwd)
            .output()
            .map_err(|e| AppError::Storage(format!("Failed to run git command: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let msg = if !stderr.trim().is_empty() {
                stderr.trim()
            } else {
                stdout.trim()
            };
            return Err(AppError::Validation(format!("Git error: {msg}")));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Initializes a Git repository in `dir` and writes a safe default `.gitignore` (LP-0702).
    pub fn init_repo(dir: &Path) -> Result<(), AppError> {
        if !dir.exists() {
            std::fs::create_dir_all(dir)
                .map_err(|e| AppError::Storage(format!("Failed to create directory: {e}")))?;
        }

        Self::run_git_cmd(&["init"], dir)?;

        let gitignore_path = dir.join(".gitignore");
        if !gitignore_path.exists() {
            let default_gitignore = "# Light Postman gitignore\n\
                *.local.json\n\
                .env\n\
                .secrets\n\
                *.sqlite\n\
                cookies.sqlite\n";
            let _ = std::fs::write(gitignore_path, default_gitignore);
        }

        Ok(())
    }

    /// Computes full Git status including ahead/behind, branch name, modified and conflicting files (LP-0710, LP-0712).
    pub fn get_status(dir: &Path) -> Result<GitStatus, AppError> {
        if !Self::is_repo(dir) {
            return Ok(GitStatus {
                is_repo: false,
                branch: "".into(),
                status_kind: "no_repo".into(),
                ahead: 0,
                behind: 0,
                staged_files: vec![],
                unstaged_files: vec![],
                untracked_files: vec![],
                has_conflicts: false,
                conflict_files: vec![],
            });
        }

        let output = Self::run_git_cmd(&["status", "--porcelain=v1", "-b"], dir)?;
        Ok(Self::parse_porcelain_status(&output))
    }

    pub fn parse_porcelain_status(output: &str) -> GitStatus {
        let mut branch = "main".to_string();
        let mut ahead = 0;
        let mut behind = 0;
        let mut staged = Vec::new();
        let mut unstaged = Vec::new();
        let mut untracked = Vec::new();
        let mut conflicts = Vec::new();

        for line in output.lines() {
            if line.starts_with("##") {
                // e.g. "## main...origin/main [ahead 1, behind 2]" or "## main"
                let branch_part = line.trim_start_matches("##").trim();
                if let Some(space_idx) = branch_part.find(' ') {
                    let name_part = &branch_part[..space_idx];
                    branch = name_part
                        .split("...")
                        .next()
                        .unwrap_or("main")
                        .to_string();

                    let bracket_part = &branch_part[space_idx..];
                    if bracket_part.contains("ahead ") {
                        if let Some(pos) = bracket_part.find("ahead ") {
                            let s = &bracket_part[pos + 6..];
                            let num_str: String =
                                s.chars().take_while(|c| c.is_ascii_digit()).collect();
                            ahead = num_str.parse().unwrap_or(0);
                        }
                    }
                    if bracket_part.contains("behind ") {
                        if let Some(pos) = bracket_part.find("behind ") {
                            let s = &bracket_part[pos + 7..];
                            let num_str: String =
                                s.chars().take_while(|c| c.is_ascii_digit()).collect();
                            behind = num_str.parse().unwrap_or(0);
                        }
                    }
                } else {
                    branch = branch_part
                        .split("...")
                        .next()
                        .unwrap_or("main")
                        .to_string();
                }
                continue;
            }

            if line.len() < 3 {
                continue;
            }

            let index_status = line.chars().next().unwrap();
            let worktree_status = line.chars().nth(1).unwrap();
            let filename = line[3..].trim().to_string();

            // Check for conflict states: UU, AA, UD, DU, etc.
            if (index_status == 'U' || worktree_status == 'U')
                || (index_status == 'A' && worktree_status == 'A')
                || (index_status == 'D' && worktree_status == 'D')
            {
                conflicts.push(filename.clone());
                continue;
            }

            if index_status == '?' && worktree_status == '?' {
                untracked.push(filename);
            } else {
                if index_status != ' ' && index_status != '?' {
                    staged.push(filename.clone());
                }
                if worktree_status != ' ' && worktree_status != '?' {
                    unstaged.push(filename);
                }
            }
        }

        let has_conflicts = !conflicts.is_empty();
        let status_kind = if has_conflicts {
            "conflict"
        } else if ahead > 0 && behind > 0 {
            "diverged"
        } else if ahead > 0 {
            "ahead"
        } else if behind > 0 {
            "behind"
        } else if !staged.is_empty() || !unstaged.is_empty() || !untracked.is_empty() {
            "modified"
        } else {
            "clean"
        };

        GitStatus {
            is_repo: true,
            branch,
            status_kind: status_kind.to_string(),
            ahead,
            behind,
            staged_files: staged,
            unstaged_files: unstaged,
            untracked_files: untracked,
            has_conflicts,
            conflict_files: conflicts,
        }
    }

    /// Stages all working directory changes (`git add -A`).
    pub fn stage_all(dir: &Path) -> Result<(), AppError> {
        Self::run_git_cmd(&["add", "-A"], dir)?;
        Ok(())
    }

    /// Commits staged changes with the provided message (LP-0702).
    pub fn commit(dir: &Path, message: &str) -> Result<String, AppError> {
        if message.trim().is_empty() {
            return Err(AppError::Validation("Commit message cannot be empty".into()));
        }

        Self::run_git_cmd(&["commit", "-m", message.trim()], dir)?;
        let sha = Self::run_git_cmd(&["rev-parse", "HEAD"], dir)?;
        Ok(sha.trim().to_string())
    }

    /// Returns the unified diff of working tree vs HEAD (LP-0702).
    pub fn diff(dir: &Path) -> Result<String, AppError> {
        Self::run_git_cmd(&["diff", "HEAD"], dir)
    }

    /// Returns recent Git commit history (LP-0702).
    pub fn log(dir: &Path, limit: usize) -> Result<Vec<GitCommit>, AppError> {
        let limit_arg = format!("-n{}", limit.max(1));
        let format_arg = "--pretty=format:%H%x09%an%x09%ai%x09%s";
        let output = match Self::run_git_cmd(&["log", &limit_arg, format_arg], dir) {
            Ok(o) => o,
            Err(_) => return Ok(vec![]), // repo might have no commits yet
        };

        let mut commits = Vec::new();
        for line in output.lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 4 {
                commits.push(GitCommit {
                    hash: parts[0].to_string(),
                    author: parts[1].to_string(),
                    date: parts[2].to_string(),
                    message: parts[3].to_string(),
                });
            }
        }

        Ok(commits)
    }

    /// Pulls changes from remote repository (LP-0706).
    pub fn pull(dir: &Path, remote: &str, branch: &str) -> Result<String, AppError> {
        Self::run_git_cmd(&["pull", remote, branch], dir)
    }

    /// Pushes changes to remote repository (LP-0706).
    pub fn push(dir: &Path, remote: &str, branch: &str) -> Result<String, AppError> {
        Self::run_git_cmd(&["push", remote, branch], dir)
    }

    /// Resolves conflict on a file by choosing "ours" or "theirs" (LP-0711).
    pub fn resolve_conflict(dir: &Path, file: &str, choice: &str) -> Result<(), AppError> {
        let flag = match choice {
            "ours" | "mine" | "local" => "--ours",
            "theirs" | "remote" => "--theirs",
            _ => {
                return Err(AppError::Validation(
                    "Choice must be 'ours' or 'theirs'".into(),
                ))
            }
        };

        Self::run_git_cmd(&["checkout", flag, file], dir)?;
        Self::run_git_cmd(&["add", file], dir)?;
        Ok(())
    }

    /// Writes `light-postman.json` into the target directory.
    pub fn export_to_repo(
        conn: &Connection,
        project_id: &str,
        dir: &Path,
        include_secrets: bool,
    ) -> Result<PathBuf, AppError> {
        if !dir.exists() {
            std::fs::create_dir_all(dir)
                .map_err(|e| AppError::Storage(format!("Failed to create repo folder: {e}")))?;
        }
        let json_content = crate::project_file::export_project_to_json(conn, project_id, include_secrets)?;
        let file_path = dir.join(crate::project_file::PROJECT_FILE_DEFAULT_NAME);
        std::fs::write(&file_path, json_content)
            .map_err(|e| AppError::Storage(format!("Failed to write project file: {e}")))?;
        Ok(file_path)
    }

    /// Reads `light-postman.json` from directory and updates SQLite.
    pub fn import_from_repo(
        conn: &mut Connection,
        dir: &Path,
        target_project_id: Option<&str>,
    ) -> Result<crate::models::Project, AppError> {
        let file_path = dir.join(crate::project_file::PROJECT_FILE_DEFAULT_NAME);
        if !file_path.exists() {
            return Err(AppError::NotFound(format!(
                "No {} found in {}",
                crate::project_file::PROJECT_FILE_DEFAULT_NAME,
                dir.display()
            )));
        }
        let json_content = std::fs::read_to_string(&file_path)
            .map_err(|e| AppError::Storage(format!("Failed to read project file: {e}")))?;
        crate::project_file::import_project_from_json(conn, &json_content, target_project_id)
    }
}

// ---------------------------------------------------------------------------
// Project Git Settings Store Operations
// ---------------------------------------------------------------------------

pub fn get_project_git_settings(
    conn: &Connection,
    project_id: &str,
) -> Result<Option<ProjectGitSettings>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT project_id, repo_path, remote_url, branch, auto_sync, github_token, last_sync_at
         FROM project_git_settings WHERE project_id = ?1",
    )?;

    let res = stmt
        .query_row(params![project_id], |row| {
            let auto_sync_int: i64 = row.get(4)?;
            Ok(ProjectGitSettings {
                project_id: row.get(0)?,
                repo_path: row.get(1)?,
                remote_url: row.get(2)?,
                branch: row.get(3)?,
                auto_sync: auto_sync_int != 0,
                github_token: row.get(5)?,
                last_sync_at: row.get(6)?,
            })
        })
        .optional()?;

    Ok(res)
}

pub fn save_project_git_settings(
    conn: &Connection,
    settings: &ProjectGitSettings,
) -> Result<(), AppError> {
    let auto_sync_int = if settings.auto_sync { 1 } else { 0 };
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO project_git_settings (project_id, repo_path, remote_url, branch, auto_sync, github_token, last_sync_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(project_id) DO UPDATE SET
            repo_path = excluded.repo_path,
            remote_url = excluded.remote_url,
            branch = excluded.branch,
            auto_sync = excluded.auto_sync,
            github_token = excluded.github_token,
            last_sync_at = excluded.last_sync_at;",
        params![
            settings.project_id,
            settings.repo_path,
            settings.remote_url,
            settings.branch,
            auto_sync_int,
            settings.github_token,
            settings.last_sync_at.as_deref().unwrap_or(&now),
        ],
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_porcelain_clean_status() {
        let raw = "## main...origin/main\n";
        let status = GitService::parse_porcelain_status(raw);
        assert_eq!(status.branch, "main");
        assert_eq!(status.status_kind, "clean");
        assert_eq!(status.ahead, 0);
        assert_eq!(status.behind, 0);
        assert!(!status.has_conflicts);
    }

    #[test]
    fn parses_porcelain_ahead_behind_and_modified() {
        let raw = "## feature/sync...origin/feature/sync [ahead 3, behind 1]\n\
                   M  light-postman.json\n\
                   ?? .env\n";
        let status = GitService::parse_porcelain_status(raw);
        assert_eq!(status.branch, "feature/sync");
        assert_eq!(status.ahead, 3);
        assert_eq!(status.behind, 1);
        assert_eq!(status.status_kind, "diverged");
        assert_eq!(status.staged_files, vec!["light-postman.json"]);
        assert_eq!(status.untracked_files, vec![".env"]);
        assert!(!status.has_conflicts);
    }

    #[test]
    fn parses_porcelain_conflict_status() {
        let raw = "## main...origin/main [ahead 1, behind 1]\n\
                   UU light-postman.json\n";
        let status = GitService::parse_porcelain_status(raw);
        assert_eq!(status.status_kind, "conflict");
        assert!(status.has_conflicts);
        assert_eq!(status.conflict_files, vec!["light-postman.json"]);
    }
}
