# 🔌 SSH Runtime 契约

这份文档专门定义 OneShell 当前阶段的 SSH 前后端契约，方便后续 agent 直接接力，不需要重新猜接口。

## 🎯 当前目标

在新的 `Tauri + Rust + React` 主线上，先打通一条最小可用 SSH 链路：

- 前端可以发起连接
- 后端可以建立 SSH 会话
- 后端可以把 stdout / stderr 推回前端终端
- 前端可以发送按键输入
- 支持 PTY resize
- 支持 SOCKS5 代理

## 📡 事件名约定

当前统一使用两个事件：

- `ssh-output`
- `ssh-lifecycle`

### `ssh-output`

用途：

- 推送 stdout
- 推送 stderr
- 推送其他扩展流

结构：

```ts
type SshOutputEvent = {
  sessionId: string;
  stream: "stdout" | "stderr" | "extended";
  text: string;
  dataBase64: string;
};
```

说明：

- `text` 给终端直接写入
- `dataBase64` 预留给后面做更严格的原始字节处理

### `ssh-lifecycle`

用途：

- 连接建立
- 断开
- 错误
- 远端退出

结构：

```ts
type SshLifecycleEvent = {
  sessionId: string;
  state: string;
  message?: string;
};
```

建议状态值：

- `connected`
- `disconnected`
- `closed`
- `error`
- `exit-status`
- `exit-signal`
- `eof`

## 🧾 Tauri Command 约定

### `connect_ssh_session`

输入：

```ts
type SshConnectInput = {
  host: string;
  port: number;
  username: string;
  password: string;
  proxy?: {
    host: string;
    port: number;
    username?: string;
    password?: string;
  };
  termType?: string;
  cols?: number;
  rows?: number;
  pixelWidth?: number;
  pixelHeight?: number;
};
```

输出：

```ts
type SshSessionSummary = {
  id: string;
  host: string;
  port: number;
  username: string;
  proxyHost?: string;
  connectedAt: number;
  cols: number;
  rows: number;
};
```

### `send_ssh_input`

```ts
type SshInputPacket = {
  sessionId: string;
  data: string;
};
```

### `resize_ssh_session`

```ts
type SshResizeInput = {
  sessionId: string;
  cols: number;
  rows: number;
  pixelWidth?: number;
  pixelHeight?: number;
};
```

### `disconnect_ssh_session`

输入：

- `sessionId: string`

### `list_ssh_sessions`

输出：

- `SshSessionSummary[]`

### `get_ssh_runtime_capabilities`

输出：

```ts
type SshRuntimeCapabilities = {
  transportMode: string;
  supportsPasswordAuth: boolean;
  supportsSocks5Proxy: boolean;
  supportsProxyAuth: boolean;
  supportsKeepAlive: boolean;
  supportsResize: boolean;
  supportsLifecycleEvents: boolean;
};
```

## 🧭 当前前端已接入的文件

- `src/types/ssh.ts`
- `src/lib/tauri/ssh.ts`
- `src/hooks/useSshTerminalSession.ts`
- `src/components/terminal/MacTerminal.tsx`
- `src-tauri/src/commands/ssh.rs`
- `src-tauri/src/modules/ssh.rs`

这些文件已经完成：

- 类型定义
- Tauri invoke 包装
- 事件监听包装
- 终端组件的连接面板
- 终端组件对 `ssh-output` / `ssh-lifecycle` 的监听
- Rust 侧会话注册表与事件发射骨架
- `connect / send_input / resize / disconnect / list_sessions` command handler
- mock keepalive lifecycle tick
- transport 已开始按独立枚举分层
- `connect` 阶段已增加真实网络预检：直连 / SOCKS5 都会先做 socket 级连通性验证
- `connect` 成功后会继续进入 `russh` 真正握手、密码认证、PTY 请求和交互式 shell 建立
- `stdout / stderr / exit-status / eof / closed` 已开始走真实 channel 读循环回推前端
- keepalive 已开始按 `russh` handle 周期性发出
- host key 已支持三态策略：
  - `strict`
  - `accept-new`
  - `off`

## ✅ 当前已经接通的真实链路

当前已经不是单纯 mock：

- 直连 TCP 预检
- SOCKS5 预检
- `russh` 握手
- 密码认证
- `request_pty`
- `request_shell`
- channel 读循环
- keepalive

也就是说，前端现在收到的 `ssh-output / ssh-lifecycle`，已经开始可以来自真实 SSH channel，而不只是 mock 回显。

## 🌍 当前 connect 已经不是纯假动作

`connect` 现在会先做一轮真实网络预检，再继续进入真正的 `russh` 建链：

- 直连模式：`TcpStream::connect`
- SOCKS5 模式：`tokio-socks::Socks5Stream`

这意味着：

- host / port 填错会直接失败
- 代理地址填错会直接失败
- 代理认证参数会参与预检

当前仍然缺的部分是：

- 更完整的认证模式，例如密钥认证
- 连接中断后的自动重连与更细粒度错误分类

## 🔐 Host Key 策略说明

当前 SSH 表单和 Rust runtime 都已经支持三态策略：

- `strict`
  已知主机里必须存在完全匹配的 host key，否则拒绝连接。
- `accept-new`
  如果目标 host 还没有记录，就自动写入并接受；如果已经存在但不匹配，则拒绝连接。
- `off`
  直接放行，不校验 host key。

当前 `known_hosts` 采用 OneShell 自己的极简持久化格式：

- 每行一条
- 结构：`host<TAB>openssh-public-key`
- 前端传 `~/.ssh/known_hosts` 时，Rust runtime 会先展开 `~`

这样做的目的是先把 Tauri 重建主线里的 host key 安全基线接通，后面再按需要升级成更完整的 OpenSSH known_hosts 兼容解析。

## 🧩 当前 Rust runtime 的分层方向

`src-tauri/src/modules/ssh.rs` 现在不再把所有行为硬编码在一个大函数里，而是按 transport 分层：

- `SessionTransport::MockBridge`
- `SessionTransport::Russh`

这个结构的目的很明确：

- 保留前后端契约稳定
- 让真实 `russh` transport 成为默认路径，同时保留 mock bridge 结构便于开发期兜底
- 避免后面继续扩认证、脚本注入、SFTP 时把整个会话层重新打碎

所以下一个 agent 的最高优先级，不是改前端视觉，而是：

1. 收紧 host key 校验策略
2. 补齐密钥认证
3. 把脚本工作站的命令注入接到真实 SSH session
4. 在此 transport 之上接 SFTP 子系统
