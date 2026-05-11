use crate::modules::models::{
    SshConnectInput, SshLifecycleEvent, SshOutputEvent, SshRuntimeCapabilities,
    SshSessionSummary,
};
use anyhow::{anyhow, bail, Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Emitter};
use tokio::sync::RwLock;
use uuid::Uuid;

pub const SSH_OUTPUT_EVENT: &str = "ssh-output";
pub const SSH_LIFECYCLE_EVENT: &str = "ssh-lifecycle";
const MOCK_TRANSPORT_MODE: &str = "mock-russh-bridge";
const KEEP_ALIVE_SECONDS: u64 = 20;

#[derive(Clone)]
pub struct SshSessionManager {
    app: AppHandle,
    sessions: Arc<RwLock<HashMap<String, SessionRecord>>>,
}

#[derive(Debug, Clone)]
struct SessionRecord {
    summary: SshSessionSummary,
    term_type: String,
    pixel_width: u32,
    pixel_height: u32,
}

impl SshSessionManager {
    pub fn new(app: AppHandle) -> Self {
        Self {
            app,
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn connect(&self, input: SshConnectInput) -> Result<SshSessionSummary> {
        let sanitized = sanitize_connect_input(input)?;
        let session_id = Uuid::new_v4().to_string();

        let summary = SshSessionSummary {
            id: session_id.clone(),
            host: sanitized.host.clone(),
            port: sanitized.port,
            username: sanitized.username.clone(),
            proxy_host: sanitized
                .proxy
                .as_ref()
                .map(|proxy| format!("{}:{}", proxy.host, proxy.port)),
            proxy_auth_enabled: sanitized.proxy_auth_enabled,
            connected_at: current_timestamp(),
            keep_alive_seconds: KEEP_ALIVE_SECONDS,
            transport_mode: MOCK_TRANSPORT_MODE.into(),
            cols: sanitized.cols,
            rows: sanitized.rows,
        };

        let record = SessionRecord {
            summary: summary.clone(),
            term_type: sanitized.term_type.clone(),
            pixel_width: sanitized.pixel_width,
            pixel_height: sanitized.pixel_height,
        };

        self.sessions
            .write()
            .await
            .insert(session_id.clone(), record);

        self.emit_lifecycle(SshLifecycleEvent {
            session_id: session_id.clone(),
            state: "connected".into(),
            message: Some(format!(
                "SSH 会话已注册：{}@{}:{}",
                summary.username, summary.host, summary.port
            )),
        })?;

        let proxy_message = match &sanitized.proxy {
            Some(proxy) => format!(
                "SOCKS5 已配置：{}:{} -> {}:{}",
                proxy.host, proxy.port, sanitized.host, sanitized.port
            ),
            None => format!("直连模式：{}:{}", sanitized.host, sanitized.port),
        };
        self.emit_output(
            &session_id,
            "stdout",
            &format!(
                "[oneshell:ssh-bootstrap] {}\r\n[oneshell:ssh-bootstrap] term={} size={}x{}\r\n[oneshell:ssh-bootstrap] 下一步将把这里替换成 russh 真正传输层。\r\n",
                proxy_message, sanitized.term_type, sanitized.cols, sanitized.rows
            ),
        )?;

        Ok(summary)
    }

    pub async fn send_input(&self, session_id: &str, data: &str) -> Result<()> {
        let sessions = self.sessions.read().await;
        let Some(record) = sessions.get(session_id) else {
            bail!("SSH session not found: {session_id}");
        };

        let escaped = data
            .replace('\r', "\\r")
            .replace('\n', "\\n")
            .replace('\u{3}', "^C");

        self.emit_output(
            session_id,
            "stdout",
            &format!(
                "[oneshell:ssh-stdin] {}\r\n[oneshell:ssh-target] {}@{}:{} ({})\r\n",
                escaped, record.summary.username, record.summary.host, record.summary.port, record.term_type
            ),
        )?;

        Ok(())
    }

    pub async fn resize(
        &self,
        session_id: &str,
        cols: u32,
        rows: u32,
        pixel_width: u32,
        pixel_height: u32,
    ) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        let Some(record) = sessions.get_mut(session_id) else {
            bail!("SSH session not found: {session_id}");
        };

        record.summary.cols = cols.max(1);
        record.summary.rows = rows.max(1);
        record.pixel_width = pixel_width;
        record.pixel_height = pixel_height;

        self.emit_lifecycle(SshLifecycleEvent {
            session_id: session_id.to_owned(),
            state: "resized".into(),
            message: Some(format!(
                "PTY resize -> {}x{} ({}x{} px)",
                record.summary.cols, record.summary.rows, pixel_width, pixel_height
            )),
        })?;

        Ok(())
    }

    pub async fn disconnect(&self, session_id: &str) -> Result<()> {
        let removed = self.sessions.write().await.remove(session_id);
        if removed.is_none() {
            bail!("SSH session not found: {session_id}");
        }

        self.emit_lifecycle(SshLifecycleEvent {
            session_id: session_id.to_owned(),
            state: "disconnected".into(),
            message: Some("SSH 会话已从当前 runtime 注销".into()),
        })?;

        Ok(())
    }

    pub async fn list_sessions(&self) -> Result<Vec<SshSessionSummary>> {
        let sessions = self.sessions.read().await;
        Ok(sessions
            .values()
            .map(|record| record.summary.clone())
            .collect())
    }

    pub async fn runtime_capabilities(&self) -> Result<SshRuntimeCapabilities> {
        Ok(SshRuntimeCapabilities {
            transport_mode: MOCK_TRANSPORT_MODE.into(),
            supports_password_auth: true,
            supports_socks5_proxy: true,
            supports_proxy_auth: true,
            supports_keep_alive: true,
            supports_resize: true,
            supports_lifecycle_events: true,
        })
    }

    fn emit_output(&self, session_id: &str, stream: &str, text: &str) -> Result<()> {
        self.app
            .emit(
                SSH_OUTPUT_EVENT,
                SshOutputEvent {
                    session_id: session_id.to_owned(),
                    stream: stream.to_owned(),
                    text: text.to_owned(),
                    data_base64: STANDARD.encode(text.as_bytes()),
                },
            )
            .context("failed to emit ssh output event")
    }

    fn emit_lifecycle(&self, event: SshLifecycleEvent) -> Result<()> {
        self.app
            .emit(SSH_LIFECYCLE_EVENT, event)
            .context("failed to emit ssh lifecycle event")
    }
}

#[derive(Debug, Clone)]
struct SanitizedProxyConfig {
    host: String,
    port: u16,
}

#[derive(Debug, Clone)]
struct SanitizedConnectInput {
    host: String,
    port: u16,
    username: String,
    password: String,
    proxy: Option<SanitizedProxyConfig>,
    proxy_auth_enabled: bool,
    term_type: String,
    cols: u32,
    rows: u32,
    pixel_width: u32,
    pixel_height: u32,
}

fn sanitize_connect_input(input: SshConnectInput) -> Result<SanitizedConnectInput> {
    let host = input.host.trim().to_owned();
    let username = input.username.trim().to_owned();
    let password = input.password;
    let term_type = input
        .term_type
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "xterm-256color".into());

    if host.is_empty() {
        bail!("SSH host cannot be empty");
    }
    if username.is_empty() {
        bail!("SSH username cannot be empty");
    }
    if password.is_empty() {
        bail!("SSH password cannot be empty");
    }

    let proxy = input
        .proxy
        .map(|proxy| SanitizedProxyConfig {
            host: proxy.host.trim().to_owned(),
            port: proxy.port,
        })
        .filter(|proxy| !proxy.host.is_empty());
    let proxy_auth_enabled = input
        .proxy
        .as_ref()
        .map(|proxy| proxy.username.as_ref().is_some() || proxy.password.as_ref().is_some())
        .unwrap_or(false);

    Ok(SanitizedConnectInput {
        host,
        port: input.port,
        username,
        password,
        proxy,
        proxy_auth_enabled,
        term_type,
        cols: input.cols.unwrap_or(120).max(40),
        rows: input.rows.unwrap_or(32).max(12),
        pixel_width: input.pixel_width.unwrap_or_default(),
        pixel_height: input.pixel_height.unwrap_or_default(),
    })
}

fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or_default()
}
