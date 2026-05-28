mod commands;
mod contracts;
mod platform;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![commands::codex::get_codex_status])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
