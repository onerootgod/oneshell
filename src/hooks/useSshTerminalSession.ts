import { useEffect, useState } from "react";
import {
  connectSshSession,
  disconnectSshSession,
  listSshSessions,
  listenSshLifecycle,
  listenSshOutput,
  resizeSshSession,
  sendSshInput
} from "../lib/tauri/ssh";
import type {
  SshConnectInput,
  SshLifecycleEvent,
  SshOutputEvent,
  SshSessionSummary
} from "../types/ssh";

type UseSshTerminalSessionOptions = {
  onOutput: (event: SshOutputEvent) => void;
  onLifecycle?: (event: SshLifecycleEvent) => void;
};

export function useSshTerminalSession({
  onOutput,
  onLifecycle
}: UseSshTerminalSessionOptions) {
  const [session, setSession] = useState<SshSessionSummary | null>(null);
  const [knownSessions, setKnownSessions] = useState<SshSessionSummary[]>([]);
  const [status, setStatus] = useState("🚧 等待连接");
  const [lastError, setLastError] = useState<string | null>(null);

  useEffect(() => {
    let mounted = true;
    let unlistenOutput: (() => void) | undefined;
    let unlistenLifecycle: (() => void) | undefined;

    void listSshSessions()
      .then((sessions) => {
        if (!mounted) return;
        setKnownSessions(sessions);
        if (sessions.length > 0) {
          setSession(sessions[0]);
          setStatus(`🟢 已发现 ${sessions.length} 个 SSH 会话`);
        }
      })
      .catch(() => {
        if (!mounted) return;
        setStatus("🧪 后端 SSH 命令尚未连通");
      });

    void listenSshOutput((event) => {
      onOutput(event);
    }).then((unlisten) => {
      unlistenOutput = unlisten;
    });

    void listenSshLifecycle((event) => {
      onLifecycle?.(event);
      setStatus(lifecycleMessage(event));
      if (event.state === "connected" || event.state === "reconnected") {
        void listSshSessions()
          .then((sessions) => {
            setKnownSessions(sessions);
            const matched = sessions.find((item) => item.id === event.sessionId);
            if (matched) {
              setSession(matched);
            }
          })
          .catch(() => undefined);
      }
      if (event.state === "closed" || event.state === "disconnected") {
        setSession((current) =>
          current?.id === event.sessionId ? null : current
        );
      }
      if (event.state === "error") {
        setLastError(event.message ?? "SSH runtime error");
      }
    }).then((unlisten) => {
      unlistenLifecycle = unlisten;
    });

    return () => {
      mounted = false;
      unlistenOutput?.();
      unlistenLifecycle?.();
    };
  }, [onLifecycle, onOutput]);

  async function connect(input: SshConnectInput) {
    setLastError(null);
    setStatus("🛰️ 正在发起 SSH 连接");
    try {
      const nextSession = await connectSshSession(input);
      setSession(nextSession);
      setKnownSessions((current) => {
        const filtered = current.filter((item) => item.id !== nextSession.id);
        return [nextSession, ...filtered];
      });
      setStatus(
        input.proxy
          ? `🌍 代理连接已建立：${nextSession.username}@${nextSession.host}:${nextSession.port}`
          : `🟢 已连接：${nextSession.username}@${nextSession.host}:${nextSession.port}`
      );
      return nextSession;
    } catch (error) {
      const message =
        error instanceof Error ? error.message : "SSH 连接启动失败";
      setLastError(message);
      setStatus("🔴 SSH 连接失败");
      throw error;
    }
  }

  async function send(data: string) {
    if (!session) return;
    await sendSshInput({ sessionId: session.id, data });
  }

  async function resize(cols: number, rows: number, pixelWidth = 0, pixelHeight = 0) {
    if (!session) return;
    await resizeSshSession({
      sessionId: session.id,
      cols,
      rows,
      pixelWidth,
      pixelHeight
    });
  }

  async function disconnect() {
    if (!session) return;
    const currentId = session.id;
    await disconnectSshSession(currentId);
    setSession(null);
    setStatus("🧼 已主动断开 SSH 会话");
    setKnownSessions((current) => current.filter((item) => item.id !== currentId));
  }

  return {
    session,
    knownSessions,
    status,
    lastError,
    connect,
    send,
    resize,
    disconnect
  };
}

function lifecycleMessage(event: SshLifecycleEvent) {
  switch (event.state) {
    case "connected":
      return `🟢 ${event.message ?? "SSH 已连接"}`;
    case "disconnected":
      return `🧼 ${event.message ?? "SSH 已断开"}`;
    case "closed":
      return `📪 ${event.message ?? "远端会话已关闭"}`;
    case "error":
      return `🔴 ${event.message ?? "SSH 运行时错误"}`;
    case "exit-status":
      return `📦 ${event.message ?? "远端进程已退出"}`;
    default:
      return `🛰️ ${event.message ?? event.state}`;
  }
}
