pub mod projects;
pub mod settings;
pub mod git;
pub mod opencode;

#[cfg(test)]
pub mod git_tests;

use tauri::command;

#[command]
pub fn get_settings() -> Result<settings::AppSettings, String> {
    settings::load_settings()
}

#[command]
pub fn save_settings(settings: settings::AppSettings) -> Result<(), String> {
    settings::save_settings(&settings)
}

pub use projects::{
    scan_projects, get_init_preview, init_context, get_context_files, write_context_file
};

pub use git::{get_git_info, is_git_repo};

pub use opencode::{generate_opencode_launch_prompt, read_launch_prompt, launch_opencode};