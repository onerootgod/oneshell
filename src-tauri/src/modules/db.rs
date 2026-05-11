use crate::modules::{
    crypto,
    models::{SaveServerProfileInput, ServerProfileSummary},
};
use anyhow::{anyhow, Result};
use rusqlite::{params, Connection};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use uuid::Uuid;

#[derive(Clone)]
pub struct Database {
    db_path: Arc<PathBuf>,
    master_key: Arc<String>,
}

impl Database {
    pub fn bootstrap(app_data_dir: &Path) -> Result<Self> {
        fs::create_dir_all(app_data_dir)?;

        let master_key = crypto::load_or_create_master_key(app_data_dir)?;
        let database = Self {
            db_path: Arc::new(app_data_dir.join("oneshell.secure.db")),
            master_key: Arc::new(master_key),
        };

        database.initialize_schema()?;
        Ok(database)
    }

    pub fn save_server_profile(
        &self,
        input: SaveServerProfileInput,
    ) -> Result<ServerProfileSummary> {
        let sanitized = sanitize_input(input)?;
        let SanitizedServerProfileInput {
            name,
            host,
            port,
            username,
            password,
        } = sanitized;
        let encrypted_password =
            crypto::encrypt_secret(&self.master_key, &password)?;
        let now = current_timestamp();
        let id = Uuid::new_v4().to_string();
        let conn = self.open_connection()?;

        conn.execute(
            r#"
            INSERT INTO server_profiles (
                id,
                name,
                host,
                port,
                username,
                password_ciphertext,
                created_at,
                updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
            params![
                &id,
                name.as_deref(),
                &host,
                port,
                &username,
                encrypted_password,
                now,
                now
            ],
        )?;

        Ok(ServerProfileSummary {
            id,
            name,
            host,
            port,
            username,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn list_server_profiles(&self) -> Result<Vec<ServerProfileSummary>> {
        let conn = self.open_connection()?;
        let mut statement = conn.prepare(
            r#"
            SELECT id, name, host, port, username, created_at, updated_at
            FROM server_profiles
            ORDER BY updated_at DESC, created_at DESC
            "#,
        )?;

        let rows = statement.query_map([], |row| {
            Ok(ServerProfileSummary {
                id: row.get(0)?,
                name: row.get(1)?,
                host: row.get(2)?,
                port: row.get(3)?,
                username: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|error| error.into())
    }

    fn initialize_schema(&self) -> Result<()> {
        let conn = self.open_connection()?;
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS server_profiles (
                id TEXT PRIMARY KEY,
                name TEXT,
                host TEXT NOT NULL,
                port INTEGER NOT NULL CHECK(port > 0 AND port <= 65535),
                username TEXT NOT NULL,
                password_ciphertext TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_server_profiles_host
                ON server_profiles(host);

            CREATE INDEX IF NOT EXISTS idx_server_profiles_updated_at
                ON server_profiles(updated_at DESC);
            "#,
        )?;

        Ok(())
    }

    fn open_connection(&self) -> Result<Connection> {
        let conn = Connection::open(&*self.db_path)?;
        let database_key = crypto::derive_database_key(&self.master_key);
        let escaped_key = database_key.replace('\'', "''");

        conn.execute_batch(&format!(
            "PRAGMA key = '{}'; PRAGMA cipher_page_size = 4096; PRAGMA foreign_keys = ON;",
            escaped_key
        ))?;

        Ok(conn)
    }
}

struct SanitizedServerProfileInput {
    name: Option<String>,
    host: String,
    port: u16,
    username: String,
    password: String,
}

fn sanitize_input(input: SaveServerProfileInput) -> Result<SanitizedServerProfileInput> {
    let host = input.host.trim().to_owned();
    let username = input.username.trim().to_owned();
    let password = input.password;
    let name = input
        .name
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty());

    if host.is_empty() {
        return Err(anyhow!("server host cannot be empty"));
    }

    if username.is_empty() {
        return Err(anyhow!("username cannot be empty"));
    }

    if password.is_empty() {
        return Err(anyhow!("password cannot be empty"));
    }

    Ok(SanitizedServerProfileInput {
        name,
        host,
        port: input.port,
        username,
        password,
    })
}

fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or_default()
}
