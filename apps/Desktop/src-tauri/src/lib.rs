mod commands;
mod contracts;
mod platform;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    if platform::windows_process::relaunch_current_process_as_administrator_if_needed()
        .expect("Codex Tools must be able to request administrator elevation")
    {
        return;
    }

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::codex::get_codex_status,
            commands::codex::open_codex
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
