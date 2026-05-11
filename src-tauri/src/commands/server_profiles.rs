use crate::{app_state::AppState, modules::models::{SaveServerProfileInput, ServerProfileSummary}};
use tauri::State;

#[tauri::command]
pub fn save_server_profile(
    input: SaveServerProfileInput,
    state: State<'_, AppState>,
) -> Result<ServerProfileSummary, String> {
    state
        .database
        .save_server_profile(input)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn list_server_profiles(
    state: State<'_, AppState>,
) -> Result<Vec<ServerProfileSummary>, String> {
    state
        .database
        .list_server_profiles()
        .map_err(|error| error.to_string())
}
