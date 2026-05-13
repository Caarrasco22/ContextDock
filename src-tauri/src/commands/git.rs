use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;
use tauri::command;

#[derive(Debug, Serialize, Deserialize)]
pub struct CommitInfo {
    pub hash: String,
    pub message: String,
    pub date: String,
    pub author: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitInfo {
    pub is_repo: bool,
    pub branch: Option<String>,
    pub is_clean: bool,
    pub changed_files_count: usize,
    pub staged_files: Vec<String>,
    pub unstaged_files: Vec<String>,
    pub untracked_files: Vec<String>,
    pub last_commit_hash: Option<String>,
    pub last_commit_message: Option<String>,
    pub last_commit_date: Option<String>,
    pub recent_commits: Vec<CommitInfo>,
}

fn run_git_command(cwd: &Path, args: &[&str]) -> Option<String> {
    Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
}

#[command]
pub fn get_git_info(project_path: String) -> Result<GitInfo, String> {
    let path = Path::new(&project_path);

    if !path.exists() {
        return Err(format!("Project path does not exist: {}", project_path));
    }

    let git_dir = path.join(".git");
    if !git_dir.exists() || !git_dir.is_dir() {
        return Ok(GitInfo {
            is_repo: false,
            branch: None,
            is_clean: true,
            changed_files_count: 0,
            staged_files: vec![],
            unstaged_files: vec![],
            untracked_files: vec![],
            last_commit_hash: None,
            last_commit_message: None,
            last_commit_date: None,
            recent_commits: vec![],
        });
    }

    let branch = run_git_command(path, &["rev-parse", "--abbrev-ref", "HEAD"]);

    let status_output = run_git_command(path, &["status", "--porcelain"]);
    let is_clean = status_output.as_ref().map(|s| s.is_empty()).unwrap_or(true);

    let staged_files: Vec<String> = run_git_command(path, &["diff", "--cached", "--name-only"])
        .map(|s| s.lines().map(|l| l.to_string()).collect())
        .unwrap_or_default();

    let unstaged_files: Vec<String> = run_git_command(path, &["diff", "--name-only"])
        .map(|s| s.lines().map(|l| l.to_string()).collect())
        .unwrap_or_default();

    let untracked_files: Vec<String> = run_git_command(path, &["ls-files", "--others", "--exclude-standard"])
        .map(|s| s.lines().map(|l| l.to_string()).collect())
        .unwrap_or_default();

    let changed_files_count = status_output
        .as_ref()
        .map(|s| s.lines().count())
        .unwrap_or(0);

    let last_commit_hash = run_git_command(path, &["rev-parse", "HEAD"]);
    let last_commit_message = run_git_command(path, &["log", "-1", "--format=%s"]);
    let last_commit_date = run_git_command(path, &["log", "-1", "--format=%ar"]);

    let recent_commits: Vec<CommitInfo> = run_git_command(path, &["log", "--format=%H|%s|%ar|%an", "-5"])
        .map(|output| {
            output
                .lines()
                .filter_map(|line| {
                    let parts: Vec<&str> = line.split('|').collect();
                    if parts.len() >= 4 {
                        Some(CommitInfo {
                            hash: parts[0].to_string(),
                            message: parts[1].to_string(),
                            date: parts[2].to_string(),
                            author: parts[3].to_string(),
                        })
                    } else {
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(GitInfo {
        is_repo: true,
        branch,
        is_clean,
        changed_files_count,
        staged_files,
        unstaged_files,
        untracked_files,
        last_commit_hash,
        last_commit_message,
        last_commit_date,
        recent_commits,
    })
}

#[command]
pub fn is_git_repo(project_path: String) -> bool {
    let path = Path::new(&project_path);
    let git_dir = path.join(".git");
    git_dir.exists() && git_dir.is_dir()
}