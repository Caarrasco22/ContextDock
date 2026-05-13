mod commands;

use commands::{get_settings, get_init_preview, init_context, save_settings, scan_projects, get_context_files, write_context_file, get_git_info, is_git_repo, generate_opencode_launch_prompt, read_launch_prompt, launch_opencode};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            scan_projects,
            get_init_preview,
            init_context,
            get_context_files,
            write_context_file,
            get_settings,
            save_settings,
            get_git_info,
            is_git_repo,
            generate_opencode_launch_prompt,
            read_launch_prompt,
            launch_opencode
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}