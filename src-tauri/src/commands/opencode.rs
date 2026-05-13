use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command;
use tauri::command;

use crate::commands::git::{GitInfo, get_git_info};
use crate::commands::projects::ContextFiles;
use crate::commands::settings::load_settings;

#[derive(Debug, Serialize, Deserialize)]
pub struct LaunchPromptResult {
    pub path: String,
    pub content: String,
}

fn build_launch_prompt(
    project_name: &str,
    project_path: &str,
    git_info: &GitInfo,
    context_files: &ContextFiles,
) -> String {
    let is_git_repo = if git_info.is_repo { "yes" } else { "no" };
    let branch = git_info.branch.as_deref().unwrap_or("unknown");

    let current_goal = context_files
        .current
        .as_ref()
        .and_then(|c| {
            let trimmed = c.trim();
            if trimmed.is_empty() || trimmed == "# Current Focus" || trimmed == "# Current Focus\n" {
                None
            } else {
                Some(trimmed.to_string())
            }
        })
        .unwrap_or_else(|| "No current focus has been set yet.".to_string());

    let architecture_text = context_files
        .architecture
        .as_ref()
        .map(|a| a.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "(No architecture saved yet.)".to_string());

    let recent_work_text = context_files
        .recent_work
        .as_ref()
        .map(|r| r.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "(No recent work saved yet.)".to_string());

    let changed_files_section = if !git_info.is_clean {
        let mut lines = Vec::new();
        lines.push("### Changed Files\n".to_string());

        if !git_info.staged_files.is_empty() {
            lines.push(format!("**Staged ({}):**", git_info.staged_files.len()));
            for f in git_info.staged_files.iter().take(10) {
                lines.push(format!("- {}", f));
            }
            lines.push(String::new());
        }

        if !git_info.unstaged_files.is_empty() {
            lines.push(format!("**Modified ({}):**", git_info.unstaged_files.len()));
            for f in git_info.unstaged_files.iter().take(10) {
                lines.push(format!("- {}", f));
            }
            lines.push(String::new());
        }

        if !git_info.untracked_files.is_empty() {
            lines.push(format!("**Untracked ({}):**", git_info.untracked_files.len()));
            for f in git_info.untracked_files.iter().take(10) {
                lines.push(format!("- {}", f));
            }
            lines.push(String::new());
        }

        lines.join("\n")
    } else {
        "### Changed Files\n\n(No changes)".to_string()
    };

    let recent_commits_section = if !git_info.recent_commits.is_empty() {
        let mut lines = Vec::new();
        lines.push("## Recent Commits\n".to_string());
        for commit in git_info.recent_commits.iter().take(5) {
            lines.push(format!(
                "- `{}` - {} ({})",
                &commit.hash[..7.min(commit.hash.len())],
                commit.message,
                commit.date
            ));
        }
        lines.join("\n")
    } else {
        "## Recent Commits\n\n(No commits)".to_string()
    };

    format!(r#"# OpenCode Launch Prompt

## Project

- Name: {project_name}
- Path: {project_path}
- Git repository: {is_git_repo}
- Branch: {branch}

## Current Goal

{current_goal}

## Architecture

{architecture_text}

## Recent Work

{recent_work_text}

## Git Status

- Branch: {branch}
- Changed files: {changed_files_count}

{changed_files_section}

{recent_commits_section}

## Instructions for OpenCode

You are working inside this local project.

Follow these rules:
- Preserve the existing architecture.
- Do not redesign the app unless explicitly requested.
- Do not add heavy dependencies.
- Do not add external APIs unless explicitly requested.
- Prefer small, reviewable changes.
- Explain what files you changed and why.
- After changes, run the project's normal validation commands if available.
- Do not touch deployment configuration unless explicitly requested.

## Requested Task

Continue from the Current Goal above. Implement the requested task described there using the project context, architecture notes, and recent work.
"#, project_name=project_name, project_path=project_path, is_git_repo=is_git_repo, branch=branch, current_goal=current_goal, architecture_text=architecture_text, recent_work_text=recent_work_text, changed_files_count=git_info.changed_files_count, changed_files_section=changed_files_section, recent_commits_section=recent_commits_section)
}

#[command]
pub fn generate_opencode_launch_prompt(
    project_path: String,
) -> Result<LaunchPromptResult, String> {
    let path = Path::new(&project_path);

    if !path.exists() {
        return Err(format!("Project path does not exist: {}", project_path));
    }

    let context_dir = path.join(".context-bridge");
    if !context_dir.exists() {
        return Err(".context-bridge directory not initialized. Please initialize context first.".to_string());
    }

    let project_name = path.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let git_info = get_git_info(project_path.clone())?;

    let context_files = {
        let meta = if context_dir.join("meta.json").exists() {
            fs::read_to_string(context_dir.join("meta.json"))
                .ok()
                .and_then(|c| serde_json::from_str::<crate::commands::projects::ProjectMeta>(&c).ok())
        } else {
            None
        };

        let read_md = |name: &str| -> Option<String> {
            let p = context_dir.join(name);
            if p.exists() { fs::read_to_string(&p).ok() } else { None }
        };

        ContextFiles {
            meta,
            current: read_md("current.md"),
            architecture: read_md("architecture.md"),
            recent_work: read_md("recent-work.md"),
        }
    };

    let prompt_content = build_launch_prompt(&project_name, &project_path, &git_info, &context_files);

    let prompt_path = context_dir.join("launch-prompt.md");
    fs::write(&prompt_path, &prompt_content).map_err(|e| e.to_string())?;

    Ok(LaunchPromptResult {
        path: prompt_path.to_string_lossy().to_string(),
        content: prompt_content,
    })
}

#[command]
pub fn read_launch_prompt(project_path: String) -> Result<String, String> {
    let path = Path::new(&project_path).join(".context-bridge").join("launch-prompt.md");

    if !path.exists() {
        return Err("No launch prompt found. Please generate one first.".to_string());
    }

    fs::read_to_string(&path).map_err(|e| e.to_string())
}

#[command]
pub fn launch_opencode(project_path: String) -> Result<(), String> {
    let settings = load_settings()?;

    let opencode_cmd = settings.opencode_command.trim();
    if opencode_cmd.is_empty() {
        return Err("OpenCode command not configured. Please set it in Settings.".to_string());
    }

    let path = Path::new(&project_path);
    if !path.exists() {
        return Err(format!("Project path does not exist: {}", project_path));
    }

    let prompt_path = path.join(".context-bridge").join("launch-prompt.md");
    let has_prompt = prompt_path.exists();

    let mut cmd = Command::new("cmd");
    cmd.args(["/C", "start", opencode_cmd]);

    if has_prompt {
        let prompt_str = prompt_path.to_string_lossy().to_string();
        if opencode_cmd.contains("opencode") {
            cmd = Command::new("cmd");
            cmd.args(["/C", "start", "", opencode_cmd, &format!("\"{}\"", prompt_str)]);
        }
    }

    cmd.current_dir(path);

    let output = cmd.output().map_err(|e| format!("Failed to launch OpenCode: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.is_empty() {
            return Err("OpenCode launch failed. Please check your opencode_command setting.".to_string());
        }
        return Err(format!("OpenCode error: {}", stderr.trim()));
    }

    Ok(())
}