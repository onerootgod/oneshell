use crate::{
    app_state::AppState,
    modules::models::{
        RunLocalScriptInput, ScriptEntryDetail, ScriptEntrySummary, ScriptExecutionResult,
    },
};
use tauri::State;

#[tauri::command]
pub async fn list_script_entries(
    state: State<'_, AppState>,
) -> Result<Vec<ScriptEntrySummary>, String> {
    state
        .scripts
        .list_scripts()
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn get_script_entry_detail(
    path: String,
    state: State<'_, AppState>,
) -> Result<ScriptEntryDetail, String> {
    state
        .scripts
        .read_script(&path)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn run_local_script(
    input: RunLocalScriptInput,
    state: State<'_, AppState>,
) -> Result<ScriptExecutionResult, String> {
    state
        .scripts
        .run_local_script(input)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn get_script_workspace_root(
    state: State<'_, AppState>,
) -> Result<String, String> {
    Ok(state.scripts.root_dir().to_string_lossy().into_owned())
}
