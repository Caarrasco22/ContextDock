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

    // Save history snapshot
    let history_dir = context_dir.join("history");
    fs::create_dir_all(&history_dir).map_err(|e| e.to_string())?;

    let timestamp = chrono::Local::now().format("%Y-%m-%d_%H%M%S");
    let mut snapshot_name = format!("{}-launch-prompt.md", timestamp);
    let mut snapshot_path = history_dir.join(&snapshot_name);
    let mut counter = 1u32;
    while snapshot_path.exists() {
        counter += 1;
        snapshot_name = format!("{}-{}-launch-prompt.md", timestamp, counter);
        snapshot_path = history_dir.join(&snapshot_name);
    }
    fs::write(&snapshot_path, &prompt_content).map_err(|e| e.to_string())?;

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

#[derive(Debug, Serialize, Deserialize)]
pub struct PromptHistoryEntry {
    pub filename: String,
    pub path: String,
    pub size_bytes: u64,
    pub modified: String,
}

#[command]
pub fn list_prompt_history(project_path: String) -> Result<Vec<PromptHistoryEntry>, String> {
    let path = Path::new(&project_path);
    if !path.exists() {
        return Err(format!("Project path does not exist: {}", project_path));
    }

    let history_dir = path.join(".context-bridge").join("history");
    if !history_dir.exists() {
        return Ok(Vec::new());
    }

    let mut entries: Vec<PromptHistoryEntry> = Vec::new();

    let dir_entries = fs::read_dir(&history_dir).map_err(|e| e.to_string())?;
    for entry in dir_entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let file_path = entry.path();

        if !file_path.is_file() {
            continue;
        }

        let file_name = match file_path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };

        if !file_name.ends_with(".md") {
            continue;
        }

        let metadata = match file_path.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        let size_bytes = metadata.len();
        let modified = match metadata.modified() {
            Ok(t) => {
                let duration = t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
                let secs = duration.as_secs();
                let naive = chrono::DateTime::from_timestamp(secs as i64, 0)
                    .unwrap_or_default();
                naive.format("%Y-%m-%d %H:%M:%S").to_string()
            }
            Err(_) => String::from("unknown"),
        };

        entries.push(PromptHistoryEntry {
            filename: file_name,
            path: file_path.to_string_lossy().to_string(),
            size_bytes,
            modified,
        });
    }

    entries.sort_by(|a, b| b.filename.cmp(&a.filename));

    Ok(entries)
}

#[command]
pub fn read_prompt_history_file(project_path: String, filename: String) -> Result<String, String> {
    if filename.is_empty() || filename.contains('/') || filename.contains('\\') || filename.contains("..") {
        return Err("Invalid filename.".to_string());
    }

    if !filename.ends_with(".md") {
        return Err("Only .md files can be read.".to_string());
    }

    let path = Path::new(&project_path);
    if !path.exists() {
        return Err(format!("Project path does not exist: {}", project_path));
    }

    let file_path = path.join(".context-bridge").join("history").join(&filename);

    if !file_path.exists() {
        return Err(format!("History file not found: {}", filename));
    }

    let canonical = file_path.canonicalize().map_err(|_| "Cannot resolve file path.".to_string())?;
    let canonical_base = path.join(".context-bridge").join("history").canonicalize().map_err(|_| "Cannot resolve history directory.".to_string())?;

    if !canonical.starts_with(&canonical_base) {
        return Err("Access denied: file is outside the history directory.".to_string());
    }

    fs::read_to_string(&file_path).map_err(|e| e.to_string())
}

#[cfg(target_os = "macos")]
fn shell_escape_single_quotes(s: &str) -> String {
    s.replace('\'', "'\\''")
}

#[cfg(any(target_os = "windows", test))]
fn escape_powershell_single_quoted(s: &str) -> String {
    format!("'{}'", s.replace('\'', "''"))
}

#[cfg(any(target_os = "windows", test))]
fn generate_windows_launch_script(opencode_cmd: &str, project_path: &Path, has_prompt: bool) -> String {
    let project_path_escaped = escape_powershell_single_quoted(&project_path.to_string_lossy());
    let prompt_rel = ".context-bridge\\launch-prompt.md";
    let prompt_escaped = escape_powershell_single_quoted(prompt_rel);
    let cmd = opencode_cmd.trim();

    let mut script = String::new();
    script.push_str(&format!("Set-Location -LiteralPath {}\n\n", project_path_escaped));

    if has_prompt {
        script.push_str(&format!(
            "if (Test-Path -LiteralPath {pl}) {{\n    $prompt = Get-Content -LiteralPath {pl} -Raw\n    {cmd} --prompt $prompt\n}} else {{\n    {cmd}\n}}\n\n",
            pl = prompt_escaped,
            cmd = cmd,
        ));
    } else {
        script.push_str(&format!("{}\n\n", cmd));
    }

    script.push_str("Write-Host \"\"\n");
    script.push_str("Write-Host \"OpenCode session ended. Press Enter to close...\"\n");
    script.push_str("Read-Host\n");

    script
}

#[cfg(target_os = "windows")]
fn launch_in_terminal(cmd_str: &str, cwd: &Path, _prompt_path: Option<&Path>) -> Result<(), String> {
    let has_prompt = cwd.join(".context-bridge").join("launch-prompt.md").exists();
    let script_path = cwd.join(".context-bridge").join("launch-opencode.ps1");

    let script_content = generate_windows_launch_script(cmd_str, cwd, has_prompt);
    fs::write(&script_path, &script_content)
        .map_err(|e| format!("Failed to write launch script: {}", e))?;

    let script_abs = script_path
        .canonicalize()
        .unwrap_or_else(|_| script_path.clone());

    let script_str = script_abs.to_string_lossy().to_string();

    let wt_result = Command::new("wt.exe")
        .args([
            "powershell.exe",
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
            &script_str,
        ])
        .spawn();

    match wt_result {
        Ok(_) => return Ok(()),
        Err(_) => {}
    }

    Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
            &script_str,
        ])
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

#[cfg(any(target_os = "linux", test))]
fn escape_single_quoted_bash(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

#[cfg(any(target_os = "linux", test))]
fn generate_linux_launch_script(opencode_cmd: &str, project_path: &Path, has_prompt: bool) -> String {
    let cd_path = escape_single_quoted_bash(&project_path.to_string_lossy());
    let prompt_path = escape_single_quoted_bash(".context-bridge/launch-prompt.md");

    let opencode_line = if has_prompt {
        format!("{} --prompt \"$(cat {})\"", opencode_cmd.trim(), prompt_path)
    } else {
        opencode_cmd.trim().to_string()
    };

    format!(
        "#!/usr/bin/env bash\ncd {}\n\nif [ -f {prompt} ]; then\n  {with_prompt}\nelse\n  {without_prompt}\nfi\n\necho\nread -r -p \"OpenCode session ended. Press Enter to close...\"\n",
        cd_path,
        prompt = prompt_path,
        with_prompt = opencode_line,
        without_prompt = opencode_cmd.trim(),
    )
}

#[cfg(any(target_os = "linux", test))]
fn terminal_launch_args(term: &str, script_path: &Path) -> Option<(&'static str, Vec<String>)> {
    let sp = script_path.to_string_lossy().to_string();
    match term {
        "gnome-terminal" => Some(("gnome-terminal", vec!["--".into(), "bash".into(), sp])),
        "konsole" => Some(("konsole", vec!["-e".into(), "bash".into(), sp])),
        "xfce4-terminal" => Some(("xfce4-terminal", vec!["--command".into(), format!("bash '{}'", sp)])),
        "mate-terminal" => Some(("mate-terminal", vec!["--".into(), "bash".into(), sp])),
        "alacritty" => Some(("alacritty", vec!["-e".into(), "bash".into(), sp])),
        "kitty" => Some(("kitty", vec!["bash".into(), sp])),
        "xterm" => Some(("xterm", vec!["-e".into(), "bash".into(), sp])),
        _ => None,
    }
}

#[cfg(target_os = "linux")]
fn launch_in_terminal(cmd_str: &str, cwd: &Path, _prompt_path: Option<&Path>) -> Result<(), String> {
    let has_prompt = cwd.join(".context-bridge").join("launch-prompt.md").exists();
    let script_path = cwd.join(".context-bridge").join("launch-opencode.sh");

    let script_content = generate_linux_launch_script(cmd_str, cwd, has_prompt);
    fs::write(&script_path, &script_content)
        .map_err(|e| format!("Failed to write launch script: {}", e))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&script_path)
            .map_err(|e| format!("Failed to read script permissions: {}", e))?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms)
            .map_err(|e| format!("Failed to set script executable: {}", e))?;
    }

    let terminal_names = [
        "gnome-terminal", "konsole", "xfce4-terminal", "mate-terminal",
        "alacritty", "kitty", "xterm",
    ];

    for name in terminal_names {
        if let Some((bin, args)) = terminal_launch_args(name, &script_path) {
            let mut cmd = Command::new(bin);
            cmd.args(&args);
            match cmd.spawn() {
                Ok(_) => return Ok(()),
                Err(_) => continue,
            }
        }
    }

    Command::new("bash")
        .arg(script_path.to_string_lossy().as_ref())
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

    fn setup_test_context_dir(label: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("contextdock-{}-{}", label, std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join(".context-bridge")).unwrap();
        std::fs::write(dir.join(".context-bridge").join("meta.json"), "{}").unwrap();
        std::fs::write(dir.join(".context-bridge").join("current.md"), "# Current Focus\n\nTesting launch prompt.").unwrap();
        std::fs::write(dir.join(".context-bridge").join("architecture.md"), "# Architecture\n\nTest arch.").unwrap();
        std::fs::write(dir.join(".context-bridge").join("recent-work.md"), "# Recent Work\n\nTest work.").unwrap();
        dir
    }

    fn count_history_snapshots(dir: &std::path::Path) -> usize {
        let history = dir.join(".context-bridge").join("history");
        if !history.exists() {
            return 0;
        }
        std::fs::read_dir(&history)
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .filter(|e| e.file_name().to_string_lossy().ends_with("-launch-prompt.md"))
                    .count()
            })
            .unwrap_or(0)
    }

    #[test]
    fn test_generate_launch_prompt_creates_file() {
        let dir = setup_test_context_dir("opencode-test");
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
        let dir = setup_test_context_dir("customtask-test");
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
        let dir = setup_test_context_dir("emptytask-test");
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

    #[test]
    fn test_history_snapshot_created() {
        let dir = setup_test_context_dir("history-test");
        let result = generate_opencode_launch_prompt(dir.to_string_lossy().to_string(), None);

        assert!(result.is_ok());
        let count = count_history_snapshots(&dir);
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(count, 1, "Expected 1 history snapshot, found {}", count);
    }

    #[test]
    fn test_history_snapshot_content_matches() {
        let dir = setup_test_context_dir("history-content");
        let result = generate_opencode_launch_prompt(dir.to_string_lossy().to_string(), None);

        assert!(result.is_ok());
        let launch = result.unwrap();
        assert_eq!(count_history_snapshots(&dir), 1);

        let history_dir = dir.join(".context-bridge").join("history");
        assert!(history_dir.exists(), "history dir should exist at {}", history_dir.display());
        let snapshot_content = std::fs::read_dir(&history_dir)
            .ok()
            .and_then(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .find(|e| e.file_name().to_string_lossy().ends_with("-launch-prompt.md"))
                    .and_then(|e| std::fs::read_to_string(e.path()).ok())
            })
            .expect("snapshot file should be readable");

        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(snapshot_content, launch.content);
    }

    #[test]
    fn test_history_two_generations_two_snapshots() {
        let dir = setup_test_context_dir("history-two");
        let _ = generate_opencode_launch_prompt(dir.to_string_lossy().to_string(), Some("First task.".to_string()));
        std::thread::sleep(std::time::Duration::from_secs(1));
        let result2 = generate_opencode_launch_prompt(dir.to_string_lossy().to_string(), Some("Second task.".to_string()));

        assert!(result2.is_ok());
        let launch2 = result2.unwrap();
        let count = count_history_snapshots(&dir);
        let _ = std::fs::remove_dir_all(&dir);

        assert_eq!(count, 2, "Expected 2 history snapshots, found {}", count);
        assert!(launch2.content.contains("Second task."));
    }

    // --- prompt history listing / reading tests ---

    fn write_history_file(dir: &std::path::Path, filename: &str, content: &str) {
        let history = dir.join(".context-bridge").join("history");
        std::fs::create_dir_all(&history).unwrap();
        std::fs::write(history.join(filename), content).unwrap();
    }

    #[test]
    fn test_list_prompt_history_returns_snapshots() {
        let dir = setup_test_context_dir("list-history");
        write_history_file(&dir, "2025-01-01_120000-launch-prompt.md", "old");
        write_history_file(&dir, "2025-06-01_120000-launch-prompt.md", "newer");
        write_history_file(&dir, "2025-03-15_093000-launch-prompt.md", "middle");

        let result = list_prompt_history(dir.to_string_lossy().to_string());
        let _ = std::fs::remove_dir_all(&dir);

        assert!(result.is_ok(), "Expected Ok, got {:?}", result.err());
        let entries = result.unwrap();
        assert_eq!(entries.len(), 3, "Expected 3 entries, got {}", entries.len());
        assert!(entries[0].filename > entries[1].filename, "Should be sorted newest first");
        assert!(entries[1].filename > entries[2].filename, "Should be sorted newest first");
    }

    #[test]
    fn test_list_prompt_history_empty_when_no_history_dir() {
        let dir = setup_test_context_dir("no-history");
        let result = list_prompt_history(dir.to_string_lossy().to_string());
        let _ = std::fs::remove_dir_all(&dir);

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_list_prompt_history_ignores_non_md_files() {
        let dir = setup_test_context_dir("non-md-filter");
        write_history_file(&dir, "2025-01-01_120000-launch-prompt.md", "ok");
        std::fs::write(
            dir.join(".context-bridge").join("history").join("notes.txt"),
            "not-md",
        )
        .unwrap();

        let result = list_prompt_history(dir.to_string_lossy().to_string());
        let _ = std::fs::remove_dir_all(&dir);

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1, "Should only list .md files");
    }

    #[test]
    fn test_read_prompt_history_file_returns_content() {
        let dir = setup_test_context_dir("read-history");
        write_history_file(&dir, "2025-06-15_080000-launch-prompt.md", "Hello history!");

        let result = read_prompt_history_file(
            dir.to_string_lossy().to_string(),
            "2025-06-15_080000-launch-prompt.md".to_string(),
        );
        let _ = std::fs::remove_dir_all(&dir);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello history!");
    }

    #[test]
    fn test_read_prompt_history_rejects_slash() {
        let dir = setup_test_context_dir("reject-slash");
        let result = read_prompt_history_file(
            dir.to_string_lossy().to_string(),
            "../secret.md".to_string(),
        );
        let _ = std::fs::remove_dir_all(&dir);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid"));
    }

    #[test]
    fn test_read_prompt_history_rejects_backslash() {
        let dir = setup_test_context_dir("reject-backslash");
        let result = read_prompt_history_file(
            dir.to_string_lossy().to_string(),
            "..\\secret.md".to_string(),
        );
        let _ = std::fs::remove_dir_all(&dir);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid"));
    }

    #[test]
    fn test_read_prompt_history_rejects_dot_dot() {
        let dir = setup_test_context_dir("reject-dotdot");
        let result = read_prompt_history_file(
            dir.to_string_lossy().to_string(),
            "../launch-prompt.md".to_string(),
        );
        let _ = std::fs::remove_dir_all(&dir);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid"));
    }

    #[test]
    fn test_read_prompt_history_rejects_non_md() {
        let dir = setup_test_context_dir("reject-non-md");
        write_history_file(&dir, "notes.txt", "not markdown");

        let result = read_prompt_history_file(
            dir.to_string_lossy().to_string(),
            "notes.txt".to_string(),
        );
        let _ = std::fs::remove_dir_all(&dir);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_prompt_history_rejects_nonexistent_file() {
        let dir = setup_test_context_dir("reject-missing");
        let result = read_prompt_history_file(
            dir.to_string_lossy().to_string(),
            "nonexistent.md".to_string(),
        );
        let _ = std::fs::remove_dir_all(&dir);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_prompt_history_rejects_symlink_escape() {
        // Verify canonicalize check catches path traversal even if basic string checks pass.
        let dir = setup_test_context_dir("symlink-test");
        let history = dir.join(".context-bridge").join("history");
        std::fs::create_dir_all(&history).unwrap();

        // Write a file outside the history dir
        let outside = dir.join("secret.txt");
        std::fs::write(&outside, "leaked").unwrap();

        // Create a symlink inside history to the outside file
        let symlink_path = history.join("innocent.md");
        std::os::unix::fs::symlink(&outside, &symlink_path).unwrap();

        let result = read_prompt_history_file(
            dir.to_string_lossy().to_string(),
            "innocent.md".to_string(),
        );
        let _ = std::fs::remove_dir_all(&dir);
        assert!(result.is_err(), "Should reject access via symlink to outside file");
    }

    // --- Linux launcher helper tests ---

    #[test]
    fn test_escape_single_quoted_bash_plain() {
        let result = escape_single_quoted_bash("/home/user/projects");
        assert_eq!(result, "'/home/user/projects'");
    }

    #[test]
    fn test_escape_single_quoted_bash_with_spaces() {
        let result = escape_single_quoted_bash("/home/user/my projects");
        assert_eq!(result, "'/home/user/my projects'");
    }

    #[test]
    fn test_escape_single_quoted_bash_with_single_quote() {
        let result = escape_single_quoted_bash("/home/user/it's a project");
        assert_eq!(result, "'/home/user/it'\\''s a project'");
    }

    #[test]
    fn test_escape_single_quoted_bash_empty() {
        let result = escape_single_quoted_bash("");
        assert_eq!(result, "''");
    }

    #[test]
    fn test_generate_linux_script_with_prompt() {
        let script = generate_linux_launch_script("opencode", Path::new("/tmp/proj"), true);
        assert!(script.contains("#!/usr/bin/env bash"), "Script should have shebang");
        assert!(script.contains("cd '/tmp/proj'"), "Script should cd to project dir, got: {}", script);
        assert!(script.contains("--prompt"), "Script should use --prompt flag, got: {}", script);
        assert!(script.contains("$(cat"), "Script should use cat for prompt, got: {}", script);
        assert!(script.contains("read -r -p"), "Script should have read prompt to keep terminal open");
    }

    #[test]
    fn test_generate_linux_script_without_prompt() {
        let script = generate_linux_launch_script("opencode", Path::new("/tmp/proj"), false);
        assert!(script.contains("#!/usr/bin/env bash"));
        assert!(!script.contains("--prompt"), "Script should NOT use --prompt when no prompt exists, got: {}", script);
        assert!(!script.contains("$(cat"), "Script should NOT use cat when no prompt exists");
        assert!(script.contains("read -r -p"), "Script should have read prompt to keep terminal open");
    }

    #[test]
    fn test_generate_linux_script_no_positional_prompt_path() {
        let script = generate_linux_launch_script("opencode", Path::new("/tmp/proj"), true);
        let lines: Vec<&str> = script.lines().collect();

        let mut in_then = false;
        for line in &lines {
            if line.trim() == "then" {
                in_then = true;
                continue;
            }
            if line.trim() == "else" || line.trim() == "fi" {
                in_then = false;
                continue;
            }
            if in_then && line.contains("opencode") && !line.starts_with('#') {
                assert!(
                    line.contains("--prompt"),
                    "OpenCode invocation in then-branch should use --prompt flag. Got: {}",
                    line
                );
            }
        }
    }

    #[test]
    fn test_generate_linux_script_with_custom_command() {
        let script = generate_linux_launch_script("/usr/local/bin/opencode", Path::new("/tmp/proj"), false);
        assert!(script.contains("/usr/local/bin/opencode"));
    }

    #[test]
    fn test_terminal_launch_args_gnome_terminal() {
        let script = Path::new("/tmp/test/script.sh");
        let (bin, args) = terminal_launch_args("gnome-terminal", script).unwrap();
        assert_eq!(bin, "gnome-terminal");
        assert_eq!(args[0], "--");
        assert_eq!(args[1], "bash");
        assert!(args[2].contains("script.sh"));
    }

    #[test]
    fn test_terminal_launch_args_konsole() {
        let script = Path::new("/tmp/test/script.sh");
        let (bin, args) = terminal_launch_args("konsole", script).unwrap();
        assert_eq!(bin, "konsole");
        assert_eq!(args[0], "-e");
        assert_eq!(args[1], "bash");
    }

    #[test]
    fn test_terminal_launch_args_xterm() {
        let script = Path::new("/tmp/test/script.sh");
        let (bin, args) = terminal_launch_args("xterm", script).unwrap();
        assert_eq!(bin, "xterm");
        assert_eq!(args[0], "-e");
        assert_eq!(args[1], "bash");
    }

    #[test]
    fn test_terminal_launch_args_unknown() {
        let script = Path::new("/tmp/test/script.sh");
        assert!(terminal_launch_args("not-a-terminal", script).is_none());
    }

    #[test]
    fn test_terminal_launch_args_kitty() {
        let script = Path::new("/tmp/test/script.sh");
        let (bin, args) = terminal_launch_args("kitty", script).unwrap();
        assert_eq!(bin, "kitty");
        assert_eq!(args[0], "bash");
    }

    #[test]
    fn test_terminal_launch_args_xfce4_terminal() {
        let script = Path::new("/tmp/test/script.sh");
        let (bin, args) = terminal_launch_args("xfce4-terminal", script).unwrap();
        assert_eq!(bin, "xfce4-terminal");
        assert_eq!(args[0], "--command");
        assert!(args[1].contains("bash '"));
        assert!(args[1].contains("script.sh'"));
    }

    // --- Windows launcher helper tests ---

    #[test]
    fn test_escape_powershell_single_quoted_plain() {
        let result = escape_powershell_single_quoted(r"C:\Users\test");
        assert_eq!(result, "'C:\\Users\\test'");
    }

    #[test]
    fn test_escape_powershell_single_quoted_with_spaces() {
        let result = escape_powershell_single_quoted(r"C:\Users\Pablo Carrasco\Test Project");
        assert_eq!(result, "'C:\\Users\\Pablo Carrasco\\Test Project'");
    }

    #[test]
    fn test_escape_powershell_single_quoted_with_single_quote() {
        let result = escape_powershell_single_quoted(r"C:\it's here");
        assert_eq!(result, "'C:\\it''s here'");
    }

    #[test]
    fn test_escape_powershell_single_quoted_empty() {
        let result = escape_powershell_single_quoted("");
        assert_eq!(result, "''");
    }

    #[test]
    fn test_generate_windows_script_with_prompt() {
        let script = generate_windows_launch_script("opencode", Path::new(r"C:\Users\test"), true);
        assert!(script.contains("Set-Location -LiteralPath"), "Script should contain Set-Location");
        assert!(script.contains(r".context-bridge\launch-prompt.md"), "Script should reference launch-prompt.md");
        assert!(script.contains("Get-Content -LiteralPath"), "Script should use Get-Content");
        assert!(script.contains("-Raw"), "Script should use -Raw flag");
        assert!(script.contains("$prompt"), "Script should use $prompt variable");
        assert!(script.contains("--prompt $prompt"), "Script should pass --prompt with $prompt, got: {}", script);
        assert!(script.contains("Read-Host"), "Script should have Read-Host to keep terminal open");
    }

    #[test]
    fn test_generate_windows_script_without_prompt() {
        let script = generate_windows_launch_script("opencode", Path::new(r"C:\Users\test"), false);
        assert!(script.contains("Set-Location -LiteralPath"));
        assert!(!script.contains("--prompt"), "Script should NOT use --prompt when no prompt exists, got: {}", script);
        assert!(!script.contains("$prompt"), "Script should not reference $prompt when no prompt");
        assert!(script.contains("Read-Host"), "Script should have Read-Host");
    }

    #[test]
    fn test_generate_windows_script_no_positional_prompt_path() {
        let script = generate_windows_launch_script("opencode", Path::new(r"C:\Users\test"), true);
        let lines: Vec<&str> = script.lines().collect();
        for line in &lines {
            if line.contains("opencode") {
                assert!(
                    !line.contains(r"launch-prompt.md"),
                    "OpenCode invocation should not receive launch-prompt.md as positional arg. Got: {}",
                    line
                );
            }
        }
    }

    #[test]
    fn test_generate_windows_script_with_opencode_cmd() {
        let script = generate_windows_launch_script("opencode.cmd", Path::new(r"C:\test"), true);
        assert!(script.contains("opencode.cmd --prompt $prompt"), "Should support opencode.cmd");
    }

    #[test]
    fn test_generate_windows_script_with_full_path() {
        let script = generate_windows_launch_script(
            r"C:\tools\opencode.exe",
            Path::new(r"C:\test"),
            false,
        );
        assert!(script.contains(r"C:\tools\opencode.exe"), "Should support full paths");
    }

    #[test]
    fn test_generate_windows_script_path_inside_context_bridge() {
        let script = generate_windows_launch_script("opencode", Path::new(r"C:\Users\test"), true);
        assert!(
            !script.contains("..\\") && !script.contains("../"),
            "Script should not contain path traversal patterns"
        );
    }
}