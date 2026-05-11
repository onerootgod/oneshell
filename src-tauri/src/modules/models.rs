use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveServerProfileInput {
    pub name: Option<String>,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerProfileSummary {
    pub id: String,
    pub name: Option<String>,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SshProxyInput {
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SshConnectInput {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub proxy: Option<SshProxyInput>,
    pub host_key_policy: Option<String>,
    pub known_hosts_path: Option<String>,
    pub term_type: Option<String>,
    pub cols: Option<u32>,
    pub rows: Option<u32>,
    pub pixel_width: Option<u32>,
    pub pixel_height: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SshSessionSummary {
    pub id: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub proxy_host: Option<String>,
    pub proxy_auth_enabled: bool,
    pub host_key_policy: String,
    pub known_hosts_path: Option<String>,
    pub connected_at: i64,
    pub keep_alive_seconds: u64,
    pub transport_mode: String,
    pub cols: u32,
    pub rows: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SshRuntimeCapabilities {
    pub transport_mode: String,
    pub supports_password_auth: bool,
    pub supports_socks5_proxy: bool,
    pub supports_proxy_auth: bool,
    pub supports_keep_alive: bool,
    pub supports_resize: bool,
    pub supports_lifecycle_events: bool,
    pub supports_host_key_policy: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SshInputPacket {
    pub session_id: String,
    pub data: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SshResizeInput {
    pub session_id: String,
    pub cols: u32,
    pub rows: u32,
    pub pixel_width: Option<u32>,
    pub pixel_height: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SshOutputEvent {
    pub session_id: String,
    pub stream: String,
    pub text: String,
    pub data_base64: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SshLifecycleEvent {
    pub session_id: String,
    pub state: String,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptEntrySummary {
    pub id: String,
    pub name: String,
    pub path: String,
    pub kind: String,
    pub relative_path: String,
    pub size_bytes: u64,
    pub modified_at: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptEntryDetail {
    pub summary: ScriptEntrySummary,
    pub content: String,
    pub suggested_remote_command: String,
    pub local_runner: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunLocalScriptInput {
    pub path: String,
    pub args: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptExecutionResult {
    pub command: String,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}
