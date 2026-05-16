import { invoke } from "@tauri-apps/api/core";
import { AppSettings, GitInfo } from "./types";

export async function getSettings(): Promise<AppSettings> {
  return invoke("get_settings");
}

export async function saveSettings(settings: AppSettings): Promise<void> {
  return invoke("save_settings", { settings });
}

export async function getGitInfo(projectPath: string): Promise<GitInfo> {
  return invoke("get_git_info", { projectPath });
}

export async function isGitRepo(projectPath: string): Promise<boolean> {
  return invoke("is_git_repo", { projectPath });
}

export interface LaunchPromptResult {
  path: string;
  content: string;
}

export async function generateOpenCodeLaunchPrompt(projectPath: string, requestedTask?: string): Promise<LaunchPromptResult> {
  return invoke("generate_opencode_launch_prompt", { projectPath, requestedTask });
}

export async function readLaunchPrompt(projectPath: string): Promise<string> {
  return invoke("read_launch_prompt", { projectPath });
}

export async function launchOpenCode(projectPath: string): Promise<void> {
  return invoke("launch_opencode", { projectPath });
}