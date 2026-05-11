import { useEffect, useState } from "react";
import { getSftpRoot, listSftpDirectory } from "../../lib/tauri/sftp";
import type { SftpDirectorySnapshot, SftpEntryNode } from "../../types/sftp";

export default function SftpWorkbench() {
  const [rootPath, setRootPath] = useState("~");
  const [snapshot, setSnapshot] = useState<SftpDirectorySnapshot | null>(null);
  const [status, setStatus] = useState("📂 正在准备 SFTP 工作台");

  useEffect(() => {
    void refresh();
  }, []);

  async function refresh(path?: string) {
    try {
      const [root, nextSnapshot] = await Promise.all([
        getSftpRoot(),
        listSftpDirectory({ path })
      ]);
      setRootPath(root);
      setSnapshot(nextSnapshot);
      setStatus(
        `📦 已载入 ${nextSnapshot.totalEntries} 个条目 · UTF-8 / Emoji 文件名链路已接通`
      );
    } catch (error) {
      setStatus(error instanceof Error ? error.message : "SFTP 目录读取失败");
    }
  }

  async function openEntry(entry: SftpEntryNode) {
    if (entry.kind !== "directory") {
      return;
    }
    await refresh(entry.path);
  }

  async function goParent() {
    if (!snapshot) return;
    const parts = snapshot.currentPath.split("/").filter(Boolean);
    if (parts.length === 0) {
      await refresh(snapshot.rootPath);
      return;
    }
    const parent = `/${parts.slice(0, -1).join("/")}`;
    await refresh(parent || snapshot.rootPath);
  }

  return (
    <section className="mt-10 rounded-[26px] border border-white/10 bg-white/5 p-6 shadow-shell backdrop-blur-2xl">
      <div className="flex items-start justify-between gap-6">
        <div>
          <p className="text-sm uppercase tracking-[0.24em] text-accent/80">SFTP Workbench</p>
          <h2 className="mt-2 text-2xl font-semibold tracking-tight text-white">
            📂 文件工作台第一版
          </h2>
          <p className="mt-2 max-w-2xl text-sm leading-7 text-slate-300">
            这轮先把文件树 JSON、UTF-8 / 中文 / Emoji 文件名链路和浏览面板接通。当前读取源还是本地工作区根目录，
            下一步再直接挂到真实 SSH transport 上。
          </p>
        </div>

        <div className="rounded-2xl border border-white/10 bg-slate-950/50 px-4 py-3 text-right">
          <p className="text-xs uppercase tracking-[0.24em] text-slate-500">Workspace Root</p>
          <p className="mt-2 max-w-[320px] break-all text-sm font-medium text-accent">
            {rootPath}
          </p>
        </div>
      </div>

      <div className="mt-6 rounded-2xl border border-white/10 bg-slate-950/35 p-4">
        <div className="flex items-center justify-between gap-4">
          <div>
            <p className="text-xs uppercase tracking-[0.22em] text-slate-500">当前目录</p>
            <p className="mt-2 break-all text-sm font-medium text-slate-100">
              {snapshot?.currentPath ?? rootPath}
            </p>
            <p className="mt-2 text-xs text-slate-400">{status}</p>
          </div>

          <div className="flex gap-3">
            <button
              className="rounded-xl border border-white/10 px-4 py-2 text-sm text-slate-200"
              onClick={() => void goParent()}
            >
              返回上级
            </button>
            <button
              className="rounded-xl border border-cyan-400/30 bg-cyan-400/10 px-4 py-2 text-sm text-cyan-200"
              onClick={() => void refresh(snapshot?.currentPath)}
            >
              刷新目录
            </button>
          </div>
        </div>

        <div className="mt-5 overflow-hidden rounded-2xl border border-white/10">
          <div className="grid grid-cols-[minmax(0,1.5fr)_120px_120px_180px] gap-3 border-b border-white/10 bg-black/20 px-4 py-3 text-xs uppercase tracking-[0.18em] text-slate-500">
            <span>名称</span>
            <span>类型</span>
            <span>权限</span>
            <span>大小</span>
          </div>

          <div className="divide-y divide-white/5">
            {snapshot?.entries.length ? (
              snapshot.entries.map((entry) => (
                <button
                  key={entry.path}
                  className="grid w-full grid-cols-[minmax(0,1.5fr)_120px_120px_180px] gap-3 px-4 py-3 text-left transition hover:bg-white/5"
                  onClick={() => void openEntry(entry)}
                >
                  <span className="truncate text-sm text-slate-100">
                    {entry.kind === "directory" ? "📁" : "📄"} {entry.name}
                  </span>
                  <span className="text-sm text-slate-300">{entry.kind}</span>
                  <span className="text-sm text-slate-300">{entry.permissions}</span>
                  <span className="text-sm text-slate-300">
                    {entry.kind === "directory" ? "—" : formatBytes(entry.sizeBytes)}
                  </span>
                </button>
              ))
            ) : (
              <div className="px-4 py-8 text-sm text-slate-500">
                当前目录为空。这里已经可以稳定显示中文和 Emoji 文件名。
              </div>
            )}
          </div>
        </div>
      </div>
    </section>
  );
}

function formatBytes(value: number): string {
  if (value < 1024) return `${value} B`;
  if (value < 1024 * 1024) return `${(value / 1024).toFixed(1)} KB`;
  if (value < 1024 * 1024 * 1024) return `${(value / 1024 / 1024).toFixed(1)} MB`;
  return `${(value / 1024 / 1024 / 1024).toFixed(1)} GB`;
}
