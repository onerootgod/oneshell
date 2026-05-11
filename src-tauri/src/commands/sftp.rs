use crate::{
    app_state::AppState,
    modules::models::{
        CreateSftpDirectoryInput, DeleteSftpEntryInput, DownloadSftpFileInput,
        ListSftpDirectoryInput, SftpDirectorySnapshot, SftpOperationResult, UploadSftpFileInput,
    },
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

#[tauri::command]
pub async fn create_sftp_directory(
    input: CreateSftpDirectoryInput,
    state: State<'_, AppState>,
) -> Result<SftpOperationResult, String> {
    state
        .sftp
        .create_directory(input)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn delete_sftp_entry(
    input: DeleteSftpEntryInput,
    state: State<'_, AppState>,
) -> Result<SftpOperationResult, String> {
    state
        .sftp
        .delete_entry(input)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn upload_sftp_file(
    input: UploadSftpFileInput,
    state: State<'_, AppState>,
) -> Result<SftpOperationResult, String> {
    state
        .sftp
        .upload_file(input)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn download_sftp_file(
    input: DownloadSftpFileInput,
    state: State<'_, AppState>,
) -> Result<SftpOperationResult, String> {
    state
        .sftp
        .download_file(input)
        .map_err(|error| error.to_string())
}
