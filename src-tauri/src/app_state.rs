use crate::modules::db::Database;
use anyhow::Result;
use tauri::{AppHandle, Manager};

pub struct AppState {
    pub database: Database,
}

impl AppState {
    pub fn bootstrap(app: &AppHandle) -> Result<Self> {
        let app_data_dir = app.path().app_data_dir()?;
        let database = Database::bootstrap(&app_data_dir)?;
        Ok(Self { database })
    }
}
