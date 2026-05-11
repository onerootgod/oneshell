use crate::{
    app_state::AppState,
    modules::models::{
        SshConnectInput, SshInputPacket, SshResizeInput, SshRuntimeCapabilities,
        SshSessionSummary,
    },
};
use tauri::State;

#[tauri::command]
pub async fn connect_ssh_session(
    input: SshConnectInput,
    state: State<'_, AppState>,
) -> Result<SshSessionSummary, String> {
    state
        .ssh
        .connect(input)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn send_ssh_input(
    packet: SshInputPacket,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .ssh
        .send_input(&packet.session_id, &packet.data)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn resize_ssh_session(
    input: SshResizeInput,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .ssh
        .resize(
            &input.session_id,
            input.cols,
            input.rows,
            input.pixel_width.unwrap_or_default(),
            input.pixel_height.unwrap_or_default(),
        )
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn disconnect_ssh_session(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .ssh
        .disconnect(&session_id)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn list_ssh_sessions(
    state: State<'_, AppState>,
) -> Result<Vec<SshSessionSummary>, String> {
    state
        .ssh
        .list_sessions()
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn get_ssh_runtime_capabilities(
    state: State<'_, AppState>,
) -> Result<SshRuntimeCapabilities, String> {
    state
        .ssh
        .runtime_capabilities()
        .await
        .map_err(|error| error.to_string())
}
