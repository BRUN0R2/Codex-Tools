mod commands;
mod contracts;
mod platform;
mod priority;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    if platform::windows_shell::relaunch_current_process_as_administrator_if_needed()
        .expect("Codex Tools must be able to request administrator elevation")
    {
        return;
    }

    platform::windows_privilege::enable_process_management_privileges()
        .expect("Codex Tools must enable process management privileges");

    tauri::Builder::default()
        .manage(priority::stabilization::PriorityStabilizationStore::default())
        .invoke_handler(tauri::generate_handler![
            commands::codex::get_codex_status,
            commands::codex::open_codex,
            commands::codex::register_codex_run_as_administrator
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
