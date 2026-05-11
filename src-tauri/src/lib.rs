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
            commands::scripts::get_script_entry_detail,
            commands::scripts::get_script_workspace_root,
            commands::scripts::list_script_entries,
            commands::scripts::run_local_script,
            commands::ssh::connect_ssh_session,
            commands::ssh::disconnect_ssh_session,
            commands::ssh::get_ssh_runtime_capabilities,
            commands::ssh::list_ssh_sessions,
            commands::ssh::resize_ssh_session,
            commands::ssh::send_ssh_input,
            commands::server_profiles::list_server_profiles,
            commands::server_profiles::save_server_profile
        ])
        .run(tauri::generate_context!())
        .expect("failed to run OneShell Tauri application");
}
