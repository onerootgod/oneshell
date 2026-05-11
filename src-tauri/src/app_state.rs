use crate::modules::{db::Database, ssh::SshSessionManager};
use anyhow::Result;
use tauri::{AppHandle, Manager};

pub struct AppState {
    pub database: Database,
    pub ssh: SshSessionManager,
}

impl AppState {
    pub fn bootstrap(app: &AppHandle) -> Result<Self> {
        let app_data_dir = app.path().app_data_dir()?;
        let database = Database::bootstrap(&app_data_dir)?;
        let ssh = SshSessionManager::new(app.clone());
        Ok(Self { database, ssh })
    }
}
