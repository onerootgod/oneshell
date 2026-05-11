mod app_state;
mod commands;
mod modules;

use app_state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let state = AppState::bootstrap(app.handle())?;
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::server_profiles::list_server_profiles,
            commands::server_profiles::save_server_profile
        ])
        .run(tauri::generate_context!())
        .expect("failed to run OneShell Tauri application");
}
