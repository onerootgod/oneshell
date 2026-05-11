import MacTerminal from "./components/terminal/MacTerminal";
import SftpWorkbench from "./components/sftp/SftpWorkbench";

const architectureTracks = [
  "Tauri command bridge for SSH stdin / stdout streaming",
  "Encrypted local storage (SQLCipher + AES-GCM)",
  "xterm.js + WebGL + Unicode11 emoji rendering",
  "Script workstation and future automation hooks"
];

export default function App() {
  return (
    <main className="min-h-screen bg-[radial-gradient(circle_at_top,_rgba(93,212,255,0.12),_transparent_45%),linear-gradient(180deg,_#08111b_0%,_#02060c_100%)] px-8 py-10 text-ink">
      <section className="mx-auto max-w-5xl rounded-[28px] border border-white/10 bg-white/5 p-8 shadow-shell backdrop-blur-2xl">
        <div className="flex items-start justify-between gap-6">
          <div className="space-y-4">
            <p className="text-sm uppercase tracking-[0.24em] text-accent/80">
              OneShell Rebuild / Phase 2
            </p>
            <h1 className="text-4xl font-semibold tracking-tight">
              Terminal rendering is now wired for macOS emoji correctness.
            </h1>
            <p className="max-w-2xl text-sm leading-7 text-slate-300">
              The rebuild now has a dedicated terminal surface using xterm.js,
              WebGL acceleration, and the Unicode11 addon so emoji width stays
              stable on macOS. This is the shell layer we can connect to the
              upcoming Rust SSH runtime.
            </p>
          </div>

          <div className="rounded-2xl border border-white/10 bg-slate-950/50 px-4 py-3 text-right">
            <p className="text-xs uppercase tracking-[0.24em] text-slate-500">
              Stack
            </p>
            <p className="mt-2 text-sm font-medium text-accent">
              React 18 + Tailwind + Tauri 2 + Rust
            </p>
          </div>
        </div>

        <div className="mt-10 grid gap-4 md:grid-cols-2">
          {architectureTracks.map((track) => (
            <article
              key={track}
              className="rounded-2xl border border-white/10 bg-slate-900/45 p-5"
            >
              <p className="text-sm text-slate-200">{track}</p>
            </article>
          ))}
        </div>

        <div className="mt-10">
          <MacTerminal />
        </div>

        <div className="mt-10">
          <SftpWorkbench />
        </div>
      </section>
    </main>
  );
}
