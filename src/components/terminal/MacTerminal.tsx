import { useEffect, useMemo, useRef, useState } from "react";
import { Terminal } from "xterm";
import { FitAddon } from "xterm-addon-fit";
import { Unicode11Addon } from "xterm-addon-unicode11";
import { WebglAddon } from "xterm-addon-webgl";
import "xterm/css/xterm.css";
import { useSshTerminalSession } from "../../hooks/useSshTerminalSession";
import type { SshConnectInput, SshLifecycleEvent, SshOutputEvent } from "../../types/ssh";
import {
  listServerProfiles,
  saveServerProfile
} from "../../lib/tauri/serverProfiles";
import type { ServerProfileSummary } from "../../types/serverProfiles";

const BOOT_LINES = [
  "\u001b[1;36mOneShell\u001b[0m terminal bootstrap",
  "\u001b[38;5;114mUnicode11 emoji width test:\u001b[0m 😀 🚀 🧠 🛠️ 📦 👨‍💻",
  "\u001b[38;5;180mChinese / emoji filename test:\u001b[0m ls ~/桌面/发布🚀",
  "\u001b[38;5;81mSSH runtime\u001b[0m waiting for backend session manager...",
  "",
  "Last login: Sun May 11 09:28:00 on ttys001",
  "oneshell@local % "
];

type RendererMode = "webgl" | "canvas";

export default function MacTerminal() {
  const hostRef = useRef<HTMLDivElement | null>(null);
  const terminalRef = useRef<Terminal | null>(null);
  const fitAddonRef = useRef<FitAddon | null>(null);
  const resizeObserverRef = useRef<ResizeObserver | null>(null);
  const sendRef = useRef<(data: string) => Promise<void>>(async () => undefined);
  const resizeRef = useRef<
    (cols: number, rows: number, pixelWidth?: number, pixelHeight?: number) => Promise<void>
  >(async () => undefined);
  const [rendererMode, setRendererMode] = useState<RendererMode>("canvas");
  const [alias, setAlias] = useState("🚀 OCI 生产机");
  const [host, setHost] = useState("127.0.0.1");
  const [port, setPort] = useState("22");
  const [username, setUsername] = useState("root");
  const [password, setPassword] = useState("");
  const [proxyEnabled, setProxyEnabled] = useState(false);
  const [proxyHost, setProxyHost] = useState("127.0.0.1");
  const [proxyPort, setProxyPort] = useState("7891");
  const [savedProfiles, setSavedProfiles] = useState<ServerProfileSummary[]>([]);
  const [profilesStatus, setProfilesStatus] = useState("📁 正在读取连接收藏");

  const {
    session,
    knownSessions,
    capabilities,
    status,
    lastError,
    connect,
    disconnect,
    resize,
    send
  } = useSshTerminalSession({
    onOutput: handleOutputEvent,
    onLifecycle: handleLifecycleEvent
  });

  const connectionSummary = useMemo(() => {
    if (session) {
      return `${session.username}@${session.host}:${session.port}`;
    }
    return `${username || "user"}@${host || "host"}:${port || "22"}`;
  }, [host, port, session, username]);

  useEffect(() => {
    sendRef.current = send;
    resizeRef.current = resize;
  }, [resize, send]);

  useEffect(() => {
    void refreshProfiles();
  }, []);

  useEffect(() => {
    const host = hostRef.current;
    if (!host) {
      return;
    }

    const terminal = new Terminal({
      allowTransparency: true,
      convertEol: true,
      cursorBlink: true,
      cursorStyle: "bar",
      fontFamily:
        "'JetBrainsMono Nerd Font', 'Apple Color Emoji', 'Menlo', monospace",
      fontSize: 15,
      fontWeight: 500,
      lineHeight: 1.2,
      letterSpacing: 0,
      scrollback: 12000,
      theme: {
        background: "rgba(5, 10, 18, 0.18)",
        foreground: "#E6F1FF",
        cursor: "#5DD4FF",
        cursorAccent: "#08111B",
        selectionBackground: "rgba(116, 214, 255, 0.24)",
        black: "#0B1220",
        red: "#FF7B72",
        green: "#8DDB8C",
        yellow: "#F2CC60",
        blue: "#6CB6FF",
        magenta: "#D2A8FF",
        cyan: "#73DACA",
        white: "#C9D1D9",
        brightBlack: "#56657A",
        brightRed: "#FFA198",
        brightGreen: "#A7F3A1",
        brightYellow: "#F9E2AF",
        brightBlue: "#91CBFF",
        brightMagenta: "#E2C5FF",
        brightCyan: "#98F5E1",
        brightWhite: "#F0F6FC"
      }
    });

    const fitAddon = new FitAddon();
    const unicode11Addon = new Unicode11Addon();

    terminal.loadAddon(fitAddon);
    terminal.loadAddon(unicode11Addon);
    terminal.unicode.activeVersion = "11";
    terminal.open(host);
    fitAddon.fit();

    try {
      const webglAddon = new WebglAddon();
      webglAddon.onContextLoss(() => {
        webglAddon.dispose();
        setRendererMode("canvas");
      });
      terminal.loadAddon(webglAddon);
      setRendererMode("webgl");
    } catch {
      setRendererMode("canvas");
    }

    for (const line of BOOT_LINES) {
      terminal.writeln(line);
    }

    terminal.onData((data) => {
      void sendRef.current(data);
    });

    terminal.onResize(({ cols, rows }) => {
      void resizeRef.current(cols, rows);
    });

    terminalRef.current = terminal;
    fitAddonRef.current = fitAddon;

    const resizeObserver = new ResizeObserver(() => {
      fitAddonRef.current?.fit();
    });

    resizeObserver.observe(host);
    resizeObserverRef.current = resizeObserver;

    const handleWindowResize = () => {
      fitAddonRef.current?.fit();
    };

    window.addEventListener("resize", handleWindowResize);

    return () => {
      window.removeEventListener("resize", handleWindowResize);
      resizeObserver.disconnect();
      resizeObserverRef.current = null;
      fitAddonRef.current = null;
      terminalRef.current?.dispose();
      terminalRef.current = null;
    };
  }, []);

  function handleOutputEvent(event: SshOutputEvent) {
    if (session && event.sessionId !== session.id) {
      return;
    }

    const terminal = terminalRef.current;
    if (!terminal) return;

    if (event.stream === "stderr") {
      terminal.write(`\u001b[31m${event.text}\u001b[0m`);
      return;
    }

    terminal.write(event.text);
  }

  function handleLifecycleEvent(event: SshLifecycleEvent) {
    if (session && event.sessionId !== session.id) {
      return;
    }

    const terminal = terminalRef.current;
    if (!terminal) return;

    if (event.message) {
      terminal.writeln(`\r\n\u001b[38;5;81m[SSH]\u001b[0m ${event.message}`);
    }
  }

  async function handleConnect() {
    const payload: SshConnectInput = {
      host,
      port: Number(port) || 22,
      username,
      password,
      termType: "xterm-256color",
      proxy: proxyEnabled
        ? {
            host: proxyHost,
            port: Number(proxyPort) || 1080
          }
        : undefined
    };

    await connect(payload);
  }

  async function refreshProfiles() {
    try {
      const profiles = await listServerProfiles();
      setSavedProfiles(profiles);
      setProfilesStatus(
        profiles.length > 0
          ? `📚 已加载 ${profiles.length} 条连接收藏`
          : "📭 暂无连接收藏"
      );
    } catch {
      setProfilesStatus("🧪 连接收藏接口尚未连通");
    }
  }

  async function handleSaveProfile() {
    if (!host.trim() || !username.trim() || !password) {
      setProfilesStatus("⚠️ 保存前至少要填 Host / User / Password");
      return;
    }

    const saved = await saveServerProfile({
      name: alias.trim() || undefined,
      host: host.trim(),
      port: Number(port) || 22,
      username: username.trim(),
      password
    });

    setSavedProfiles((current) => {
      const filtered = current.filter((item) => item.id !== saved.id);
      return [saved, ...filtered];
    });
    setProfilesStatus(`💾 已保存连接：${saved.name ?? `${saved.username}@${saved.host}`}`);
  }

  function applyProfile(profile: ServerProfileSummary) {
    setAlias(profile.name ?? `🧩 ${profile.host}`);
    setHost(profile.host);
    setPort(String(profile.port));
    setUsername(profile.username);
    setProfilesStatus(`🪄 已套用连接：${profile.name ?? profile.host}，密码需重新输入`);
  }

  return (
    <section className="overflow-hidden rounded-[26px] border border-white/10 bg-slate-950/35 shadow-shell backdrop-blur-[28px]">
      <header className="flex items-center justify-between border-b border-white/10 px-5 py-3">
        <div className="flex items-center gap-3">
          <span className="h-3 w-3 rounded-full bg-[#FF5F57]" />
          <span className="h-3 w-3 rounded-full bg-[#FEBC2E]" />
          <span className="h-3 w-3 rounded-full bg-[#28C840]" />
          <div className="ml-3">
            <p className="text-xs uppercase tracking-[0.22em] text-slate-500">
              Terminal Surface
            </p>
            <p className="text-sm font-medium text-slate-100">
              Unicode11 + {rendererMode.toUpperCase()} renderer
            </p>
          </div>
        </div>

        <div className="rounded-full border border-cyan-400/20 bg-cyan-400/10 px-3 py-1 text-xs font-medium text-cyan-200">
          macOS vibrancy tuned
        </div>
      </header>

      <div className="grid gap-0 xl:grid-cols-[320px_minmax(0,1fr)_300px]">
        <aside className="border-r border-white/10 bg-black/10 p-5">
          <p className="text-xs uppercase tracking-[0.24em] text-slate-500">
            连接工作台
          </p>
          <div className="mt-4 space-y-3">
            <label className="block">
              <span className="mb-1 block text-xs text-slate-400">别名 / Emoji</span>
              <input
                className="w-full rounded-xl border border-white/10 bg-slate-950/60 px-3 py-2 text-sm text-slate-100 outline-none"
                value={alias}
                onChange={(event) => setAlias(event.target.value)}
              />
            </label>
            <label className="block">
              <span className="mb-1 block text-xs text-slate-400">Host</span>
              <input
                className="w-full rounded-xl border border-white/10 bg-slate-950/60 px-3 py-2 text-sm text-slate-100 outline-none"
                value={host}
                onChange={(event) => setHost(event.target.value)}
              />
            </label>
            <div className="grid grid-cols-2 gap-3">
              <label className="block">
                <span className="mb-1 block text-xs text-slate-400">Port</span>
                <input
                  className="w-full rounded-xl border border-white/10 bg-slate-950/60 px-3 py-2 text-sm text-slate-100 outline-none"
                  value={port}
                  onChange={(event) => setPort(event.target.value)}
                />
              </label>
              <label className="block">
                <span className="mb-1 block text-xs text-slate-400">User</span>
                <input
                  className="w-full rounded-xl border border-white/10 bg-slate-950/60 px-3 py-2 text-sm text-slate-100 outline-none"
                  value={username}
                  onChange={(event) => setUsername(event.target.value)}
                />
              </label>
            </div>
            <label className="block">
              <span className="mb-1 block text-xs text-slate-400">Password</span>
              <input
                type="password"
                className="w-full rounded-xl border border-white/10 bg-slate-950/60 px-3 py-2 text-sm text-slate-100 outline-none"
                value={password}
                onChange={(event) => setPassword(event.target.value)}
              />
            </label>

            <label className="flex items-center justify-between rounded-xl border border-white/10 bg-slate-950/40 px-3 py-2 text-sm text-slate-200">
              <span>🌍 启用 SOCKS5 代理</span>
              <input
                type="checkbox"
                checked={proxyEnabled}
                onChange={(event) => setProxyEnabled(event.target.checked)}
              />
            </label>

            {proxyEnabled ? (
              <div className="grid grid-cols-2 gap-3">
                <label className="block">
                  <span className="mb-1 block text-xs text-slate-400">Proxy Host</span>
                  <input
                    className="w-full rounded-xl border border-white/10 bg-slate-950/60 px-3 py-2 text-sm text-slate-100 outline-none"
                    value={proxyHost}
                    onChange={(event) => setProxyHost(event.target.value)}
                  />
                </label>
                <label className="block">
                  <span className="mb-1 block text-xs text-slate-400">Proxy Port</span>
                  <input
                    className="w-full rounded-xl border border-white/10 bg-slate-950/60 px-3 py-2 text-sm text-slate-100 outline-none"
                    value={proxyPort}
                    onChange={(event) => setProxyPort(event.target.value)}
                  />
                </label>
              </div>
            ) : null}

            <div className="flex gap-3">
              <button
                className="rounded-xl bg-cyan-400 px-4 py-2 text-sm font-semibold text-slate-950"
                onClick={() => void handleConnect()}
              >
                连接
              </button>
              <button
                className="rounded-xl border border-cyan-400/30 bg-cyan-400/10 px-4 py-2 text-sm text-cyan-200"
                onClick={() => void handleSaveProfile()}
              >
                保存
              </button>
              <button
                className="rounded-xl border border-white/10 px-4 py-2 text-sm text-slate-200"
                onClick={() => void disconnect()}
              >
                断开
              </button>
            </div>

            <div className="rounded-2xl border border-white/10 bg-slate-950/35 p-4">
              <div className="flex items-center justify-between gap-3">
                <div>
                  <p className="text-xs uppercase tracking-[0.22em] text-slate-500">
                    连接收藏
                  </p>
                  <p className="mt-1 text-xs text-slate-400">{profilesStatus}</p>
                </div>
                <button
                  className="rounded-lg border border-white/10 px-3 py-1 text-xs text-slate-300"
                  onClick={() => void refreshProfiles()}
                >
                  刷新
                </button>
              </div>

              <div className="mt-3 space-y-3">
                {savedProfiles.length === 0 ? (
                  <div className="rounded-xl border border-dashed border-white/10 px-3 py-4 text-xs leading-6 text-slate-500">
                    暂无连接收藏。保存后，这里会沉淀成可复用的连接列表。
                  </div>
                ) : (
                  savedProfiles.map((profile) => (
                    <article
                      key={profile.id}
                      className="rounded-xl border border-white/10 bg-black/10 px-3 py-3"
                    >
                      <div className="flex items-start justify-between gap-3">
                        <div>
                          <p className="text-sm font-medium text-slate-100">
                            {profile.name ?? `🧩 ${profile.host}`}
                          </p>
                          <p className="mt-1 text-xs text-slate-400">
                            {profile.username}@{profile.host}:{profile.port}
                          </p>
                        </div>
                        <button
                          className="rounded-lg border border-cyan-400/30 bg-cyan-400/10 px-3 py-1 text-xs text-cyan-200"
                          onClick={() => applyProfile(profile)}
                        >
                          套用
                        </button>
                      </div>
                    </article>
                  ))
                )}
              </div>
            </div>
          </div>
        </aside>

        <div className="min-h-[440px] bg-[linear-gradient(180deg,rgba(4,10,18,0.55),rgba(4,10,18,0.3))] p-4">
          <div className="mb-3 flex items-center justify-between rounded-2xl border border-white/10 bg-slate-950/30 px-4 py-3">
            <div>
              <p className="text-xs uppercase tracking-[0.22em] text-slate-500">
                当前目标
              </p>
              <p className="mt-1 text-sm font-medium text-slate-100">
                {alias} · {connectionSummary}
              </p>
            </div>
            <div className="text-right">
              <p className="text-xs uppercase tracking-[0.2em] text-slate-500">
                SSH 状态
              </p>
              <p className="mt-1 text-sm text-cyan-200">{status}</p>
            </div>
          </div>
          <div
            ref={hostRef}
            className="h-[420px] rounded-[22px] border border-white/8 bg-[radial-gradient(circle_at_top,rgba(116,214,255,0.08),transparent_35%),rgba(2,6,12,0.18)] px-3 py-3"
          />
        </div>

        <aside className="border-l border-white/10 bg-black/10 p-5">
          <p className="text-xs uppercase tracking-[0.24em] text-slate-500">
            运行时说明
          </p>
          <div className="mt-4 space-y-4 text-sm leading-6 text-slate-300">
            <p>
              😀 Emoji 宽度由{" "}
              <span className="font-medium text-cyan-200">Unicode11Addon</span>{" "}
              负责，并固定为{" "}
              <span className="font-medium text-cyan-200">activeVersion = 11</span>。
            </p>
            <p>
              🎮 默认优先尝试 WebGL 渲染，GPU context 丢失时回退到 canvas。
            </p>
            <p>
              🍎 当前字体回退顺序：
              <br />
              <span className="font-medium text-slate-100">
                JetBrainsMono Nerd Font → Apple Color Emoji → Menlo → monospace
              </span>
            </p>
            <p>
              🔌 当前终端已经接上了 Tauri SSH 事件桥约定，下一步只差 Rust runtime 真正发出
              `ssh-output` / `ssh-lifecycle` 事件。
            </p>
            <p>
              🧠 当前 runtime：
              <span className="font-medium text-slate-100">
                {" "}
                {capabilities?.transportMode ?? "未发现"}
              </span>
            </p>
            <p>
              ❤️ keep-alive：
              <span className="font-medium text-slate-100">
                {" "}
                {capabilities?.supportsKeepAlive ? "已声明支持" : "未声明"}
              </span>
            </p>
            <p>
              📡 已知会话：<span className="font-medium text-slate-100">{knownSessions.length}</span>
            </p>
            {lastError ? (
              <p className="rounded-2xl border border-red-400/20 bg-red-400/10 px-3 py-3 text-red-200">
                {lastError}
              </p>
            ) : null}
          </div>
        </aside>
      </div>
    </section>
  );
}
