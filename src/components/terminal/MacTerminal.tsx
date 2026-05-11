import { useEffect, useRef, useState } from "react";
import { Terminal } from "xterm";
import { FitAddon } from "xterm-addon-fit";
import { Unicode11Addon } from "xterm-addon-unicode11";
import { WebglAddon } from "xterm-addon-webgl";
import "xterm/css/xterm.css";

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
  const [rendererMode, setRendererMode] = useState<RendererMode>("canvas");

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

      <div className="grid gap-0 lg:grid-cols-[minmax(0,1fr)_280px]">
        <div className="min-h-[440px] bg-[linear-gradient(180deg,rgba(4,10,18,0.55),rgba(4,10,18,0.3))] p-4">
          <div
            ref={hostRef}
            className="h-[420px] rounded-[22px] border border-white/8 bg-[radial-gradient(circle_at_top,rgba(116,214,255,0.08),transparent_35%),rgba(2,6,12,0.18)] px-3 py-3"
          />
        </div>

        <aside className="border-l border-white/10 bg-black/10 p-5">
          <p className="text-xs uppercase tracking-[0.24em] text-slate-500">
            Render Notes
          </p>
          <div className="mt-4 space-y-4 text-sm leading-6 text-slate-300">
            <p>
              Emoji width is handled by{" "}
              <span className="font-medium text-cyan-200">Unicode11Addon</span>{" "}
              with <span className="font-medium text-cyan-200">activeVersion = 11</span>.
            </p>
            <p>
              WebGL is attempted first for smoother glyph rendering, then falls
              back to the default canvas path if the GPU context is lost.
            </p>
            <p>
              Font fallback order follows the macOS strategy:
              <br />
              <span className="font-medium text-slate-100">
                JetBrainsMono Nerd Font → Apple Color Emoji → Menlo → monospace
              </span>
            </p>
            <p>
              This surface is ready for the next step: wiring SSH stdin/stdout
              from the Tauri backend.
            </p>
          </div>
        </aside>
      </div>
    </section>
  );
}
