use crate::modules::models::{
    SshConnectInput, SshLifecycleEvent, SshOutputEvent, SshRuntimeCapabilities,
    SshSessionSummary,
};
use anyhow::{anyhow, bail, Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use russh::{
    client,
    keys::ssh_key,
    ChannelMsg, ChannelReadHalf, ChannelWriteHalf, Disconnect,
};
use std::{
    collections::HashMap,
    io::Cursor,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Emitter};
use tokio::{
    net::TcpStream,
    sync::{Mutex, RwLock},
    task::JoinHandle,
    time::{sleep, timeout, Duration},
};
use tokio_socks::tcp::Socks5Stream;
use uuid::Uuid;

pub const SSH_OUTPUT_EVENT: &str = "ssh-output";
pub const SSH_LIFECYCLE_EVENT: &str = "ssh-lifecycle";
const TRANSPORT_MODE_MOCK_RUSSH_BRIDGE: &str = "mock-russh-bridge+preflight";
const TRANSPORT_MODE_RUSSH_PASSWORD_PTY: &str = "russh-password+pty-shell";
const RUNTIME_TRANSPORT_MODE: &str = "russh-password+pty-shell";
const KEEP_ALIVE_SECONDS: u64 = 20;
const CONNECT_TIMEOUT_SECONDS: u64 = 8;

#[derive(Clone)]
pub struct SshSessionManager {
    app: AppHandle,
    sessions: Arc<RwLock<HashMap<String, SessionRecord>>>,
}

struct SessionRecord {
    summary: SshSessionSummary,
    term_type: String,
    pixel_width: u32,
    pixel_height: u32,
    transport: SessionTransport,
    keepalive_task: Option<JoinHandle<()>>,
}

#[derive(Clone)]
enum SessionTransport {
    MockBridge,
    Russh(RusshTransportState),
}

#[derive(Clone)]
struct RusshTransportState {
    handle: Arc<Mutex<RusshHandle>>,
    writer: Arc<Mutex<RusshChannelWriter>>,
}

type RusshHandle = client::Handle<RusshClientHandler>;
type RusshChannelWriter = ChannelWriteHalf<client::Msg>;

#[derive(Clone)]
enum KeepaliveTarget {
    MockBridge,
    Russh(Arc<Mutex<RusshHandle>>),
}

#[derive(Clone, Default)]
struct RusshClientHandler;

impl client::Handler for RusshClientHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &ssh_key::PublicKey,
    ) -> std::result::Result<bool, Self::Error> {
        Ok(true)
    }
}

impl SessionTransport {
    fn mode(&self) -> &'static str {
        match self {
            SessionTransport::MockBridge => TRANSPORT_MODE_MOCK_RUSSH_BRIDGE,
            SessionTransport::Russh(_) => TRANSPORT_MODE_RUSSH_PASSWORD_PTY,
        }
    }

    fn bootstrap_message(
        &self,
        proxy_message: &str,
        term_type: &str,
        cols: u32,
        rows: u32,
    ) -> String {
        match self {
            SessionTransport::MockBridge => format!(
                "[oneshell:ssh-bootstrap] {}\r\n[oneshell:ssh-bootstrap] term={} size={}x{}\r\n[oneshell:ssh-bootstrap] 当前为 mock transport，下一步将替换成 russh 真正传输层。\r\n",
                proxy_message, term_type, cols, rows
            ),
            SessionTransport::Russh(_) => format!(
                "[oneshell:ssh-bootstrap] {}\r\n[oneshell:ssh-bootstrap] term={} size={}x{}\r\n[oneshell:ssh-bootstrap] 已切到 russh 真 SSH transport，PTY 与 shell 已建立。\r\n",
                proxy_message, term_type, cols, rows
            ),
        }
    }

    fn keepalive_message(&self) -> String {
        match self {
            SessionTransport::MockBridge => format!(
                "mock keepalive tick ({KEEP_ALIVE_SECONDS}s) 已发出，等待切换到真实 russh transport"
            ),
            SessionTransport::Russh(_) => {
                format!("russh keepalive ({KEEP_ALIVE_SECONDS}s) 已发送")
            }
        }
    }

    fn keepalive_target(&self) -> KeepaliveTarget {
        match self {
            SessionTransport::MockBridge => KeepaliveTarget::MockBridge,
            SessionTransport::Russh(state) => KeepaliveTarget::Russh(state.handle.clone()),
        }
    }
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
        self.emit_lifecycle(SshLifecycleEvent {
            session_id: "pending".into(),
            state: "preflight".into(),
            message: Some(match &sanitized.proxy {
                Some(proxy) => format!(
                    "正在通过 SOCKS5 预检 {}:{} -> {}:{}",
                    proxy.host, proxy.port, sanitized.host, sanitized.port
                ),
                None => format!("正在预检直连 {}:{}", sanitized.host, sanitized.port),
            }),
        })?;
        self.run_preflight(&sanitized).await?;

        let session_id = Uuid::new_v4().to_string();
        self.emit_lifecycle(SshLifecycleEvent {
            session_id: session_id.clone(),
            state: "handshake".into(),
            message: Some("网络预检已通过，开始进行 russh 握手与密码认证".into()),
        })?;

        let transport = self
            .connect_transport(&session_id, &sanitized)
            .await
            .with_context(|| format!("SSH 握手失败：{}@{}:{}", sanitized.username, sanitized.host, sanitized.port))?;

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
            transport_mode: transport.mode().into(),
            cols: sanitized.cols,
            rows: sanitized.rows,
        };

        let keepalive_task =
            self.spawn_keepalive_loop(session_id.clone(), transport.keepalive_target(), transport.clone());

        let record = SessionRecord {
            summary: summary.clone(),
            term_type: sanitized.term_type.clone(),
            pixel_width: sanitized.pixel_width,
            pixel_height: sanitized.pixel_height,
            transport: transport.clone(),
            keepalive_task: Some(keepalive_task),
        };

        self.sessions
            .write()
            .await
            .insert(session_id.clone(), record);

        self.emit_lifecycle(SshLifecycleEvent {
            session_id: session_id.clone(),
            state: "connected".into(),
            message: Some(format!(
                "SSH 会话已建立：{}@{}:{}",
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
            &transport.bootstrap_message(
                &proxy_message,
                &sanitized.term_type,
                sanitized.cols,
                sanitized.rows,
            ),
        )?;

        Ok(summary)
    }

    pub async fn send_input(&self, session_id: &str, data: &str) -> Result<()> {
        let transport = {
            let sessions = self.sessions.read().await;
            let Some(record) = sessions.get(session_id) else {
                bail!("SSH session not found: {session_id}");
            };
            record.transport.clone()
        };

        match transport {
            SessionTransport::MockBridge => {
                let sessions = self.sessions.read().await;
                let record = sessions
                    .get(session_id)
                    .ok_or_else(|| anyhow!("SSH session not found: {session_id}"))?;
                let escaped = data
                    .replace('\r', "\\r")
                    .replace('\n', "\\n")
                    .replace('\u{3}', "^C");
                self.emit_output(
                    session_id,
                    "stdout",
                    &format!(
                        "[oneshell:ssh-stdin] {}\r\n[oneshell:ssh-target] {}@{}:{} ({})\r\n",
                        escaped,
                        record.summary.username,
                        record.summary.host,
                        record.summary.port,
                        record.term_type
                    ),
                )?;
            }
            SessionTransport::Russh(state) => {
                let mut writer = state.writer.lock().await;
                writer
                    .data(Cursor::new(data.as_bytes().to_vec()))
                    .await
                    .context("failed to send ssh stdin to russh channel")?;
            }
        }

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
        let transport = {
            let mut sessions = self.sessions.write().await;
            let Some(record) = sessions.get_mut(session_id) else {
                bail!("SSH session not found: {session_id}");
            };

            record.summary.cols = cols.max(1);
            record.summary.rows = rows.max(1);
            record.pixel_width = pixel_width;
            record.pixel_height = pixel_height;

            record.transport.clone()
        };

        if let SessionTransport::Russh(state) = transport {
            let mut writer = state.writer.lock().await;
            writer
                .window_change(cols.max(1), rows.max(1), pixel_width, pixel_height)
                .await
                .context("failed to send window_change to russh channel")?;
        }

        self.emit_lifecycle(SshLifecycleEvent {
            session_id: session_id.to_owned(),
            state: "resized".into(),
            message: Some(format!(
                "PTY resize -> {}x{} ({}x{} px)",
                cols.max(1),
                rows.max(1),
                pixel_width,
                pixel_height
            )),
        })?;

        Ok(())
    }

    pub async fn disconnect(&self, session_id: &str) -> Result<()> {
        let removed = self.sessions.write().await.remove(session_id);
        let Some(mut removed) = removed else {
            bail!("SSH session not found: {session_id}");
        };

        if let Some(task) = removed.keepalive_task.take() {
            task.abort();
        }

        if let SessionTransport::Russh(state) = &removed.transport {
            {
                let mut writer = state.writer.lock().await;
                let _ = writer.eof().await;
                let _ = writer.close().await;
            }
            let mut handle = state.handle.lock().await;
            let _ = handle
                .disconnect(Disconnect::ByApplication, "OneShell disconnect", "zh-CN")
                .await;
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
            transport_mode: RUNTIME_TRANSPORT_MODE.into(),
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

    async fn connect_transport(
        &self,
        session_id: &str,
        input: &SanitizedConnectInput,
    ) -> Result<SessionTransport> {
        let config = Arc::new(client::Config {
            inactivity_timeout: Some(Duration::from_secs(KEEP_ALIVE_SECONDS * 3)),
            ..Default::default()
        });

        let mut handle = match &input.proxy {
            Some(proxy) => {
                let stream = self.connect_proxy_stream(proxy, input).await?;
                client::connect_stream(config, stream, RusshClientHandler)
                    .await
                    .context("russh proxy transport connect_stream failed")?
            }
            None => client::connect(
                config,
                (input.host.as_str(), input.port),
                RusshClientHandler,
            )
            .await
            .context("russh direct transport connect failed")?,
        };

        let auth_result = handle
            .authenticate_password(input.username.clone(), input.password.clone())
            .await
            .context("russh password authentication failed to execute")?;
        if !auth_result.success() {
            bail!("SSH 密码认证失败");
        }

        let mut channel = handle
            .channel_open_session()
            .await
            .context("failed to open ssh session channel")?;
        channel
            .request_pty(
                false,
                input.term_type.as_str(),
                input.cols,
                input.rows,
                input.pixel_width,
                input.pixel_height,
                &[],
            )
            .await
            .context("failed to request ssh pty")?;
        channel
            .request_shell(false)
            .await
            .context("failed to request interactive shell")?;

        let (reader, writer) = channel.split();
        self.spawn_russh_reader_loop(session_id.to_owned(), reader);

        Ok(SessionTransport::Russh(RusshTransportState {
            handle: Arc::new(Mutex::new(handle)),
            writer: Arc::new(Mutex::new(writer)),
        }))
    }

    async fn connect_proxy_stream(
        &self,
        proxy: &SanitizedProxyConfig,
        input: &SanitizedConnectInput,
    ) -> Result<Socks5Stream<TcpStream>> {
        let proxy_connect = async {
            match (&proxy.username, &proxy.password) {
                (Some(username), Some(password)) => {
                    Socks5Stream::connect_with_password(
                        (proxy.host.as_str(), proxy.port),
                        (input.host.as_str(), input.port),
                        username,
                        password,
                    )
                    .await
                }
                _ => Socks5Stream::connect(
                    (proxy.host.as_str(), proxy.port),
                    (input.host.as_str(), input.port),
                )
                .await,
            }
        };

        timeout(Duration::from_secs(CONNECT_TIMEOUT_SECONDS), proxy_connect)
            .await
            .context("SOCKS5 代理 SSH transport 连接超时")?
            .with_context(|| {
                format!(
                    "无法通过 SOCKS5 {}:{} 建立 SSH transport 到 {}:{}",
                    proxy.host, proxy.port, input.host, input.port
                )
            })
    }

    fn spawn_russh_reader_loop(&self, session_id: String, mut reader: ChannelReadHalf) {
        let app = self.app.clone();
        let sessions = self.sessions.clone();

        tauri::async_runtime::spawn(async move {
            let mut saw_close = false;

            while let Some(message) = reader.wait().await {
                match message {
                    ChannelMsg::Data { data } => {
                        let text = String::from_utf8_lossy(data.as_ref()).into_owned();
                        let _ = app.emit(
                            SSH_OUTPUT_EVENT,
                            SshOutputEvent {
                                session_id: session_id.clone(),
                                stream: "stdout".into(),
                                data_base64: STANDARD.encode(data.as_ref()),
                                text,
                            },
                        );
                    }
                    ChannelMsg::ExtendedData { data, ext } => {
                        let text = String::from_utf8_lossy(data.as_ref()).into_owned();
                        let stream = if ext == 1 { "stderr" } else { "extended" };
                        let _ = app.emit(
                            SSH_OUTPUT_EVENT,
                            SshOutputEvent {
                                session_id: session_id.clone(),
                                stream: stream.into(),
                                data_base64: STANDARD.encode(data.as_ref()),
                                text,
                            },
                        );
                    }
                    ChannelMsg::ExitStatus { exit_status } => {
                        let _ = app.emit(
                            SSH_LIFECYCLE_EVENT,
                            SshLifecycleEvent {
                                session_id: session_id.clone(),
                                state: "exit-status".into(),
                                message: Some(format!("远端进程退出码：{exit_status}")),
                            },
                        );
                    }
                    ChannelMsg::ExitSignal { signal_name, .. } => {
                        let _ = app.emit(
                            SSH_LIFECYCLE_EVENT,
                            SshLifecycleEvent {
                                session_id: session_id.clone(),
                                state: "exit-signal".into(),
                                message: Some(format!("远端收到退出信号：{signal_name:?}")),
                            },
                        );
                    }
                    ChannelMsg::Eof => {
                        let _ = app.emit(
                            SSH_LIFECYCLE_EVENT,
                            SshLifecycleEvent {
                                session_id: session_id.clone(),
                                state: "eof".into(),
                                message: Some("SSH channel EOF".into()),
                            },
                        );
                    }
                    ChannelMsg::Close => {
                        saw_close = true;
                        let _ = app.emit(
                            SSH_LIFECYCLE_EVENT,
                            SshLifecycleEvent {
                                session_id: session_id.clone(),
                                state: "closed".into(),
                                message: Some("SSH channel 已关闭".into()),
                            },
                        );
                        break;
                    }
                    ChannelMsg::OpenFailure(reason) => {
                        let _ = app.emit(
                            SSH_LIFECYCLE_EVENT,
                            SshLifecycleEvent {
                                session_id: session_id.clone(),
                                state: "error".into(),
                                message: Some(format!("SSH channel open failure: {reason:?}")),
                            },
                        );
                    }
                    _ => {}
                }
            }

            if !saw_close {
                let _ = app.emit(
                    SSH_LIFECYCLE_EVENT,
                    SshLifecycleEvent {
                        session_id: session_id.clone(),
                        state: "closed".into(),
                        message: Some("SSH channel 读取循环结束".into()),
                    },
                );
            }

            if let Some(mut removed) = sessions.write().await.remove(&session_id) {
                if let Some(task) = removed.keepalive_task.take() {
                    task.abort();
                }
            }
        });
    }

    async fn run_preflight(&self, input: &SanitizedConnectInput) -> Result<()> {
        let preflight = match &input.proxy {
            Some(proxy) => self.run_proxy_preflight(proxy, input).await,
            None => self.run_direct_preflight(input).await,
        };

        preflight.map_err(|error| anyhow!("SSH 网络预检失败: {error}"))
    }

    async fn run_direct_preflight(&self, input: &SanitizedConnectInput) -> Result<()> {
        timeout(
            Duration::from_secs(CONNECT_TIMEOUT_SECONDS),
            TcpStream::connect((input.host.as_str(), input.port)),
        )
        .await
        .context("直连超时")?
        .with_context(|| format!("无法连接到 {}:{}", input.host, input.port))?;

        Ok(())
    }

    async fn run_proxy_preflight(
        &self,
        proxy: &SanitizedProxyConfig,
        input: &SanitizedConnectInput,
    ) -> Result<()> {
        let proxy_connect = async {
            match (&proxy.username, &proxy.password) {
                (Some(username), Some(password)) => {
                    Socks5Stream::connect_with_password(
                        (proxy.host.as_str(), proxy.port),
                        (input.host.as_str(), input.port),
                        username,
                        password,
                    )
                    .await
                    .map(|_| ())
                }
                _ => Socks5Stream::connect(
                    (proxy.host.as_str(), proxy.port),
                    (input.host.as_str(), input.port),
                )
                .await
                .map(|_| ()),
            }
        };

        timeout(Duration::from_secs(CONNECT_TIMEOUT_SECONDS), proxy_connect)
            .await
            .context("SOCKS5 代理预检超时")?
            .with_context(|| {
                format!(
                    "无法通过 SOCKS5 {}:{} 连到 {}:{}",
                    proxy.host, proxy.port, input.host, input.port
                )
            })?;

        Ok(())
    }

    fn spawn_keepalive_loop(
        &self,
        session_id: String,
        target: KeepaliveTarget,
        transport: SessionTransport,
    ) -> JoinHandle<()> {
        let app = self.app.clone();
        let sessions = self.sessions.clone();

        tauri::async_runtime::spawn(async move {
            loop {
                sleep(Duration::from_secs(KEEP_ALIVE_SECONDS)).await;

                let session_exists = sessions.read().await.contains_key(&session_id);
                if !session_exists {
                    break;
                }

                match &target {
                    KeepaliveTarget::MockBridge => {}
                    KeepaliveTarget::Russh(handle) => {
                        let mut handle = handle.lock().await;
                        if let Err(error) = handle.send_keepalive(true).await {
                            let _ = app.emit(
                                SSH_LIFECYCLE_EVENT,
                                SshLifecycleEvent {
                                    session_id: session_id.clone(),
                                    state: "keepalive-error".into(),
                                    message: Some(format!("russh keepalive 失败：{error}")),
                                },
                            );
                            break;
                        }
                    }
                }

                let _ = app.emit(
                    SSH_LIFECYCLE_EVENT,
                    SshLifecycleEvent {
                        session_id: session_id.clone(),
                        state: "keepalive".into(),
                        message: Some(transport.keepalive_message()),
                    },
                );
            }
        })
    }
}

#[derive(Debug, Clone)]
struct SanitizedProxyConfig {
    host: String,
    port: u16,
    username: Option<String>,
    password: Option<String>,
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
            username: proxy
                .username
                .map(|value| value.trim().to_owned())
                .filter(|value| !value.is_empty()),
            password: proxy.password.filter(|value| !value.is_empty()),
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
