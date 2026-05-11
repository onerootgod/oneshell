use crate::{
    app_state::AppState,
    modules::models::{ListSftpDirectoryInput, SftpDirectorySnapshot},
};
use tauri::State;

#[tauri::command]
pub async fn get_sftp_root(
    state: State<'_, AppState>,
) -> Result<String, String> {
    Ok(state.sftp.root_dir().to_string_lossy().into_owned())
}

#[tauri::command]
pub async fn list_sftp_directory(
    input: ListSftpDirectoryInput,
    state: State<'_, AppState>,
) -> Result<SftpDirectorySnapshot, String> {
    state
        .sftp
        .list_directory(input)
        .map_err(|error| error.to_string())
}
