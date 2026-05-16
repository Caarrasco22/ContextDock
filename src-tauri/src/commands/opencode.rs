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

const DEFAULT_REQUESTED_TASK: &str = "\
Continue from the Current Goal above. Implement the requested task described \
there using the project context, architecture notes, and recent work.";

fn build_launch_prompt(
    project_name: &str,
    project_path: &str,
    git_info: &GitInfo,
    context_files: &ContextFiles,
    requested_task: Option<&str>,
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

{requested_task}
"#, project_name=project_name, project_path=project_path, is_git_repo=is_git_repo, branch=branch, current_goal=current_goal, architecture_text=architecture_text, recent_work_text=recent_work_text, changed_files_count=git_info.changed_files_count, changed_files_section=changed_files_section, recent_commits_section=recent_commits_section, requested_task=requested_task.unwrap_or(DEFAULT_REQUESTED_TASK))
}

#[command]
pub fn generate_opencode_launch_prompt(
    project_path: String,
    requested_task: Option<String>,
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

    let task = requested_task.as_deref().filter(|t| !t.trim().is_empty());
    let prompt_content = build_launch_prompt(&project_name, &project_path, &git_info, &context_files, task);

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

#[cfg(target_os = "macos")]
fn shell_escape_single_quotes(s: &str) -> String {
    s.replace('\'', "'\\''")
}

#[cfg(target_os = "windows")]
fn launch_in_terminal(cmd_str: &str, cwd: &Path, prompt_path: Option<&Path>) -> Result<(), String> {
    let mut args: Vec<String> = vec!["/C".to_string(), "start".to_string(), "OpenCode".to_string()];

    let full_cmd = if let Some(p) = prompt_path {
        let pp_escaped = p.to_string_lossy().replace('\'', "''");
        format!(
            "powershell -NoProfile -Command \"cd '{}'; {} --prompt (Get-Content '{}' -Raw); pause\"",
            cwd.to_string_lossy().replace('\'', "''"),
            cmd_str,
            pp_escaped
        )
    } else {
        format!("cd /D \"{}\" && {}", cwd.to_string_lossy(), cmd_str)
    };

    args.push(full_cmd);

    Command::new("cmd")
        .args(&args)
        .current_dir(cwd)
        .spawn()
        .map_err(|e| format!("Failed to launch OpenCode: {}", e))?;

    Ok(())
}

#[cfg(target_os = "macos")]
fn launch_in_terminal(cmd_str: &str, cwd: &Path, prompt_path: Option<&Path>) -> Result<(), String> {
    let cwd_escaped = shell_escape_single_quotes(&cwd.to_string_lossy());
    let pp_escaped = prompt_path.map(|p| shell_escape_single_quotes(&p.to_string_lossy()));
    let command_line = build_shell_command(cmd_str, pp_escaped.as_deref());

    let script = format!(
        "tell application \"Terminal\" to do script \"cd '{}' && {}; exit\"",
        cwd_escaped.replace('\\', "\\\\").replace('"', "\\\""),
        command_line.replace('\\', "\\\\").replace('"', "\\\"")
    );

    let output = Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .map_err(|e| format!("Failed to launch OpenCode: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("OpenCode launch error: {}", stderr.trim()));
    }

    Ok(())
}

#[cfg(target_os = "linux")]
fn launch_in_terminal(cmd_str: &str, cwd: &Path, prompt_path: Option<&Path>) -> Result<(), String> {
    let pp_escaped = prompt_path.map(|p| p.to_string_lossy().replace('\'', "'\\''"));
    let full_cmd = build_shell_command(cmd_str, pp_escaped.as_deref());

    let terminals: &[(&str, &[&str])] = &[
        ("gnome-terminal", &["--", "bash", "-c"]),
        ("konsole", &["-e"]),
        ("xfce4-terminal", &["-e"]),
        ("xterm", &["-e"]),
    ];

    let shell_cmd = format!("cd \"{}\" && {}; exec $SHELL", cwd.to_string_lossy(), full_cmd);

    for (term, prefix_args) in terminals {
        let mut cmd = Command::new(term);
        cmd.args(*prefix_args);
        cmd.arg(&shell_cmd);

        match cmd.spawn() {
            Ok(_) => return Ok(()),
            Err(_) => continue,
        }
    }

    let mut parts: Vec<&str> = full_cmd.split_whitespace().collect();
    if parts.is_empty() {
        return Err("OpenCode command is empty.".to_string());
    }
    let exe = parts.remove(0);
    Command::new(exe)
        .args(&parts)
        .current_dir(cwd)
        .spawn()
        .map_err(|e| format!("Failed to launch OpenCode: {}", e))?;

    Ok(())
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
fn launch_in_terminal(cmd_str: &str, cwd: &Path, prompt_path: Option<&Path>) -> Result<(), String> {
    let pp_escaped = prompt_path.map(|p| p.to_string_lossy().replace('\'', "'\\''"));
    let full_cmd = build_shell_command(cmd_str, pp_escaped.as_deref());
    let mut parts: Vec<&str> = full_cmd.split_whitespace().collect();
    if parts.is_empty() {
        return Err("OpenCode command is empty.".to_string());
    }
    let exe = parts.remove(0);
    Command::new(exe)
        .args(&parts)
        .current_dir(cwd)
        .spawn()
        .map_err(|e| format!("Failed to launch OpenCode: {}", e))?;
    Ok(())
}

fn build_shell_command(opencode_cmd: &str, prompt_path_escaped: Option<&str>) -> String {
    let mut cmd = opencode_cmd.to_string();
    if let Some(p) = prompt_path_escaped {
        cmd = format!("{} --prompt \"$(cat '{}')\"", cmd, p);
    }
    cmd
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

    launch_in_terminal(
        opencode_cmd,
        path,
        if has_prompt {
            Some(prompt_path.as_path())
        } else {
            None
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_shell_command_without_prompt() {
        let result = build_shell_command("opencode", None);
        assert_eq!(result, "opencode");
    }

    #[test]
    fn test_build_shell_command_with_prompt() {
        let escaped_path = "/test/project/.context-bridge/launch-prompt.md";
        let result = build_shell_command("opencode", Some(escaped_path));
        assert!(
            result.contains("--prompt"),
            "Command should contain --prompt flag, got: {}",
            result
        );
        assert!(
            result.contains("$(cat '"),
            "Command should use $(cat) for prompt content, got: {}",
            result
        );
    }

    #[test]
    fn test_build_shell_command_no_prompt_path_as_positional() {
        let escaped_path = "/test/project/.context-bridge/launch-prompt.md";
        let result = build_shell_command("opencode", Some(escaped_path));

        let after_opencode = result.strip_prefix("opencode ").unwrap_or("");
        assert!(
            after_opencode.starts_with("--prompt"),
            "First argument after opencode should be --prompt, not a path. Got: {}",
            result
        );
    }

    #[test]
    fn test_generate_launch_prompt_creates_file() {
        let dir = std::env::temp_dir().join(format!("contextdock-opencode-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join(".context-bridge")).unwrap();
        std::fs::write(dir.join(".context-bridge").join("meta.json"), "{}").unwrap();
        std::fs::write(dir.join(".context-bridge").join("current.md"), "# Current Focus\n\nTesting launch prompt.").unwrap();
        std::fs::write(dir.join(".context-bridge").join("architecture.md"), "# Architecture\n\nTest arch.").unwrap();
        std::fs::write(dir.join(".context-bridge").join("recent-work.md"), "# Recent Work\n\nTest work.").unwrap();

        let result = generate_opencode_launch_prompt(dir.to_string_lossy().to_string(), None);
        let _ = std::fs::remove_dir_all(&dir);

        assert!(result.is_ok());
        let launch = result.unwrap();
        assert!(launch.path.ends_with("launch-prompt.md"));
        assert!(launch.content.contains("Current Goal"));
        assert!(launch.content.contains("Testing launch prompt"));
        assert!(launch.content.contains(DEFAULT_REQUESTED_TASK));
    }

    #[test]
    fn test_generate_launch_prompt_with_custom_task() {
        let dir = std::env::temp_dir().join(format!("contextdock-customtask-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join(".context-bridge")).unwrap();
        std::fs::write(dir.join(".context-bridge").join("meta.json"), "{}").unwrap();
        std::fs::write(dir.join(".context-bridge").join("current.md"), "# Current Focus\n\nTesting.").unwrap();
        std::fs::write(dir.join(".context-bridge").join("architecture.md"), "").unwrap();
        std::fs::write(dir.join(".context-bridge").join("recent-work.md"), "").unwrap();

        let custom = "Fix the login bug in auth module.";
        let result = generate_opencode_launch_prompt(dir.to_string_lossy().to_string(), Some(custom.to_string()));
        let _ = std::fs::remove_dir_all(&dir);

        assert!(result.is_ok());
        let launch = result.unwrap();
        assert!(launch.content.contains(custom));
        assert!(!launch.content.contains(DEFAULT_REQUESTED_TASK));
    }

    #[test]
    fn test_generate_launch_prompt_empty_task_uses_default() {
        let dir = std::env::temp_dir().join(format!("contextdock-emptytask-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join(".context-bridge")).unwrap();
        std::fs::write(dir.join(".context-bridge").join("meta.json"), "{}").unwrap();
        std::fs::write(dir.join(".context-bridge").join("current.md"), "# Current Focus\n\nTesting.").unwrap();
        std::fs::write(dir.join(".context-bridge").join("architecture.md"), "").unwrap();
        std::fs::write(dir.join(".context-bridge").join("recent-work.md"), "").unwrap();

        let result = generate_opencode_launch_prompt(dir.to_string_lossy().to_string(), Some("   ".to_string()));
        let _ = std::fs::remove_dir_all(&dir);

        assert!(result.is_ok());
        let launch = result.unwrap();
        assert!(launch.content.contains(DEFAULT_REQUESTED_TASK));
    }

    #[test]
    fn test_generate_launch_prompt_missing_context_dir() {
        let dir = std::env::temp_dir().join(format!("contextdock-noctx-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let result = generate_opencode_launch_prompt(dir.to_string_lossy().to_string(), None);
        let _ = std::fs::remove_dir_all(&dir);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains(".context-bridge"));
    }
}