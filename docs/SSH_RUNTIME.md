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
- transport 已开始按独立枚举分层，便于把 mock bridge 替换成真实 `russh` transport
- connect 阶段已增加真实网络预检：直连 / SOCKS5 都会先做 socket 级连通性验证

## ⛔ 当前最大缺口

现在前后端契约已经定下来了，Rust 侧也已经有会话注册表和事件发射骨架，但底层还不是 `russh` 真传输层。

当前 mock runtime 已经会周期性发出：

- `state = "keepalive"`

这能帮助前端先把 lifecycle UI 和状态机接完整，后面再替换成真实 SSH keepalive。

## 🌍 当前 connect 已经不是纯假动作

虽然底层还没有切到完整 `russh` 传输层，但 `connect` 现在已经会先做一轮真实网络预检：

- 直连模式：`TcpStream::connect`
- SOCKS5 模式：`tokio-socks::Socks5Stream`

这意味着：

- host / port 填错会直接失败
- 代理地址填错会直接失败
- 代理认证参数会参与预检

当前仍然缺的部分是：

- 真正 SSH 握手
- 真正 PTY / shell 建立
- 真正 stdout / stderr 流

## 🧩 当前 Rust runtime 的分层方向

`src-tauri/src/modules/ssh.rs` 现在不再把所有行为硬编码在一个大函数里，而是开始按 transport 分层：

- `SessionTransport::MockBridge`

这个结构的目的很明确：

- 先保证前后端契约稳定
- 再把 mock bridge 平滑替换成真实 `russh` transport
- 避免后面接真实 SSH 时把整个会话层重新打碎

所以下一个 agent 的最高优先级，不是改前端视觉，而是：

1. 把 `src-tauri/src/modules/ssh.rs` 的 mock session bridge 替换成真正的 `russh` transport
2. 接入 `SOCKS5` 真代理拨号
3. 接入 keep-alive
4. 按本文档里的事件名持续推送真实 stdout / stderr / lifecycle
