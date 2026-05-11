use crate::modules::{
    db::Database, scripts::ScriptWorkspace, sftp::SftpWorkspace, ssh::SshSessionManager,
};
use anyhow::Result;
use tauri::{AppHandle, Manager};

pub struct AppState {
    pub database: Database,
    pub scripts: ScriptWorkspace,
    pub sftp: SftpWorkspace,
    pub ssh: SshSessionManager,
}

impl AppState {
    pub fn bootstrap(app: &AppHandle) -> Result<Self> {
        let app_data_dir = app.path().app_data_dir()?;
        let home_dir = app.path().home_dir()?;
        let database = Database::bootstrap(&app_data_dir)?;
        let scripts = ScriptWorkspace::new(home_dir.join("NexusScripts"));
        let sftp = SftpWorkspace::new(home_dir.clone());
        let ssh = SshSessionManager::new(app.clone());
        Ok(Self {
            database,
            scripts,
            sftp,
            ssh,
        })
    }
}
