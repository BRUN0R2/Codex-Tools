mod commands;
mod contracts;
mod platform;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    if let Some(exit_code) = commands::automation::run_cli_request()
        .or_else(commands::persistent_priority::run_cli_request)
    {
        std::process::exit(exit_code);
    }

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::automation::get_automation_status,
            commands::automation::install_codex_automation,
            commands::automation::remove_codex_automation,
            commands::codex::get_codex_status,
            commands::codex::open_codex,
            commands::persistent_priority::get_persistent_priority_status,
            commands::persistent_priority::install_persistent_high_priority,
            commands::persistent_priority::remove_persistent_high_priority
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
