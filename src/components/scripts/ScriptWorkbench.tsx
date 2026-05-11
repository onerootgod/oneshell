import { useEffect, useMemo, useState } from "react";
import {
  buildRemoteScriptCommand,
  getScriptEntryDetail,
  getScriptWorkspaceRoot,
  listScriptEntries,
  runLocalScript
} from "../../lib/tauri/scripts";
import type {
  ScriptEntryDetail,
  ScriptEntrySummary,
  ScriptExecutionResult
} from "../../types/scripts";

type ScriptWorkbenchProps = {
  canInjectRemote: boolean;
  onInjectRemote: (command: string) => Promise<void>;
};

export default function ScriptWorkbench({
  canInjectRemote,
  onInjectRemote
}: ScriptWorkbenchProps) {
  const [workspaceRoot, setWorkspaceRoot] = useState("~/NexusScripts");
  const [entries, setEntries] = useState<ScriptEntrySummary[]>([]);
  const [selectedPath, setSelectedPath] = useState<string | null>(null);
  const [detail, setDetail] = useState<ScriptEntryDetail | null>(null);
  const [status, setStatus] = useState("🧭 正在检查脚本工作站");
  const [execution, setExecution] = useState<ScriptExecutionResult | null>(null);
  const [argsText, setArgsText] = useState("");

  const selectedEntry = useMemo(
    () => entries.find((entry) => entry.path === selectedPath) ?? null,
    [entries, selectedPath]
  );

  useEffect(() => {
    void refreshWorkspace();
  }, []);

  useEffect(() => {
    if (!selectedPath) {
      setDetail(null);
      return;
    }
    void loadDetail(selectedPath);
  }, [selectedPath]);

  async function refreshWorkspace() {
    try {
      const [root, nextEntries] = await Promise.all([
        getScriptWorkspaceRoot(),
        listScriptEntries()
      ]);
      setWorkspaceRoot(root);
      setEntries(nextEntries);
      if (nextEntries.length === 0) {
        setSelectedPath(null);
        setDetail(null);
        setStatus("📭 脚本目录为空，先往 ~/NexusScripts 放入 .py / .sh");
        return;
      }
      const nextSelected = selectedPath && nextEntries.some((entry) => entry.path === selectedPath)
        ? selectedPath
        : nextEntries[0].path;
      setSelectedPath(nextSelected);
      setStatus(`📚 已发现 ${nextEntries.length} 个脚本条目`);
    } catch (error) {
      setStatus(error instanceof Error ? error.message : "脚本工作站初始化失败");
    }
  }

  async function loadDetail(path: string) {
    try {
      const nextDetail = await getScriptEntryDetail(path);
      setDetail(nextDetail);
      setExecution(null);
      setStatus(`🧾 已载入脚本：${nextDetail.summary.relativePath}`);
    } catch (error) {
      setStatus(error instanceof Error ? error.message : "脚本详情读取失败");
    }
  }

  async function handleRunLocal() {
    if (!detail) return;
    setStatus(`🧪 正在本地执行：${detail.summary.name}`);
    try {
      const result = await runLocalScript({
        path: detail.summary.path,
        args: splitArgs(argsText)
      });
      setExecution(result);
      setStatus(
        result.exitCode === 0
          ? `✅ 本地执行完成：exit ${result.exitCode}`
          : `⚠️ 本地执行结束：exit ${result.exitCode}`
      );
    } catch (error) {
      setStatus(error instanceof Error ? error.message : "本地脚本执行失败");
    }
  }

  async function handleInjectRemote() {
    if (!detail) return;
    const command = await buildRemoteScriptCommand({
      path: detail.summary.path,
      args: splitArgs(argsText)
    });
    await onInjectRemote(`${command}\r`);
    setStatus(`🚀 已注入远端执行命令：${detail.summary.name}`);
  }

  return (
    <section className="mt-10 rounded-[26px] border border-white/10 bg-white/5 p-6 shadow-shell backdrop-blur-2xl">
      <div className="flex items-start justify-between gap-6">
        <div>
          <p className="text-sm uppercase tracking-[0.24em] text-accent/80">
            Script Workstation
          </p>
          <h2 className="mt-2 text-2xl font-semibold tracking-tight text-white">
            🧰 本地脚本工作站
          </h2>
          <p className="mt-2 max-w-2xl text-sm leading-7 text-slate-300">
            固定扫描 <span className="font-medium text-cyan-200">{workspaceRoot}</span>，
            列出 `.py / .sh`，支持本地执行结果回传，也支持一键注入当前 SSH 会话。
          </p>
        </div>

        <button
          className="rounded-xl border border-white/10 px-4 py-2 text-sm text-slate-200"
          onClick={() => void refreshWorkspace()}
        >
          刷新脚本目录
        </button>
      </div>

      <div className="mt-6 grid gap-6 xl:grid-cols-[320px_minmax(0,1fr)]">
        <aside className="rounded-2xl border border-white/10 bg-slate-950/35 p-4">
          <p className="text-xs uppercase tracking-[0.22em] text-slate-500">脚本列表</p>
          <p className="mt-2 text-xs leading-6 text-slate-400">{status}</p>

          <div className="mt-4 space-y-3">
            {entries.length === 0 ? (
              <div className="rounded-xl border border-dashed border-white/10 px-3 py-4 text-xs leading-6 text-slate-500">
                目录为空。把 Python 或 Shell 脚本放进 `~/NexusScripts` 后，这里会自动收口成工作站入口。
              </div>
            ) : (
              entries.map((entry) => {
                const selected = entry.path === selectedPath;
                return (
                  <button
                    key={entry.id}
                    className={`block w-full rounded-xl border px-3 py-3 text-left transition ${
                      selected
                        ? "border-cyan-400/40 bg-cyan-400/10"
                        : "border-white/10 bg-black/10 hover:border-white/20"
                    }`}
                    onClick={() => setSelectedPath(entry.path)}
                  >
                    <div className="flex items-center justify-between gap-3">
                      <p className="text-sm font-medium text-slate-100">{entry.name}</p>
                      <span className="text-[11px] uppercase tracking-[0.18em] text-slate-500">
                        {entry.kind}
                      </span>
                    </div>
                    <p className="mt-1 text-xs text-slate-400">{entry.relativePath}</p>
                  </button>
                );
              })
            )}
          </div>
        </aside>

        <div className="space-y-4">
          <div className="rounded-2xl border border-white/10 bg-slate-950/35 p-4">
            <div className="flex items-start justify-between gap-4">
              <div>
                <p className="text-xs uppercase tracking-[0.22em] text-slate-500">
                  脚本详情
                </p>
                <p className="mt-2 text-lg font-medium text-slate-100">
                  {selectedEntry?.name ?? "未选择脚本"}
                </p>
                <p className="mt-1 text-xs text-slate-400">
                  {detail?.summary.relativePath ?? "从左侧选择一个脚本条目"}
                </p>
              </div>

              <div className="flex flex-wrap gap-3">
                <button
                  className="rounded-xl bg-cyan-400 px-4 py-2 text-sm font-semibold text-slate-950 disabled:cursor-not-allowed disabled:bg-slate-700 disabled:text-slate-300"
                  onClick={() => void handleRunLocal()}
                  disabled={!detail}
                >
                  本地执行
                </button>
                <button
                  className="rounded-xl border border-cyan-400/30 bg-cyan-400/10 px-4 py-2 text-sm text-cyan-200 disabled:cursor-not-allowed disabled:border-white/10 disabled:bg-transparent disabled:text-slate-500"
                  onClick={() => void handleInjectRemote()}
                  disabled={!detail || !canInjectRemote}
                >
                  注入远端
                </button>
              </div>
            </div>

            {detail ? (
              <>
                <label className="mt-4 block">
                  <span className="mb-1 block text-xs uppercase tracking-[0.18em] text-slate-500">
                    参数模板
                  </span>
                  <input
                    className="w-full rounded-xl border border-white/10 bg-black/10 px-3 py-3 text-sm text-slate-100 outline-none"
                    value={argsText}
                    onChange={(event) => setArgsText(event.target.value)}
                    placeholder="例如：--region us-phoenix-1 --tag 发布🚀"
                  />
                  <span className="mt-2 block text-xs text-slate-500">
                    当前采用轻量空格分词，支持单引号和双引号包裹参数。
                  </span>
                </label>

                <div className="mt-4 grid gap-3 md:grid-cols-2">
                  <div className="rounded-xl border border-white/10 bg-black/10 px-3 py-3 text-sm text-slate-300">
                    <p className="text-xs uppercase tracking-[0.18em] text-slate-500">
                      Local Runner
                    </p>
                    <p className="mt-2 break-all font-medium text-slate-100">
                      {detail.localRunner}
                    </p>
                  </div>
                  <div className="rounded-xl border border-white/10 bg-black/10 px-3 py-3 text-sm text-slate-300">
                    <p className="text-xs uppercase tracking-[0.18em] text-slate-500">
                      Remote Inject
                    </p>
                    <p className="mt-2 break-all font-medium text-slate-100">
                      {detail.suggestedRemoteCommand}
                    </p>
                  </div>
                </div>

                <pre className="mt-4 max-h-[340px] overflow-auto rounded-2xl border border-white/10 bg-black/20 p-4 text-xs leading-6 text-slate-200">
                  {detail.content}
                </pre>
              </>
            ) : null}
          </div>

          <div className="rounded-2xl border border-white/10 bg-slate-950/35 p-4">
            <p className="text-xs uppercase tracking-[0.22em] text-slate-500">执行结果</p>
            {execution ? (
              <>
                <p className="mt-3 text-sm text-slate-300">
                  命令：<span className="font-medium text-slate-100">{execution.command}</span>
                </p>
                <p className="mt-1 text-sm text-slate-300">
                  退出码：<span className="font-medium text-slate-100">{execution.exitCode}</span>
                </p>
                <div className="mt-4 grid gap-4 xl:grid-cols-2">
                  <pre className="max-h-[240px] overflow-auto rounded-2xl border border-white/10 bg-black/20 p-4 text-xs leading-6 text-slate-200">
                    {execution.stdout || "stdout 为空"}
                  </pre>
                  <pre className="max-h-[240px] overflow-auto rounded-2xl border border-white/10 bg-black/20 p-4 text-xs leading-6 text-rose-200">
                    {execution.stderr || "stderr 为空"}
                  </pre>
                </div>
              </>
            ) : (
              <p className="mt-3 text-sm text-slate-400">
                还没有执行记录。本地执行后，stdout / stderr 会直接落在这里。
              </p>
            )}
          </div>
        </div>
      </div>
    </section>
  );
}

function splitArgs(value: string): string[] {
  const trimmed = value.trim();
  if (!trimmed) {
    return [];
  }

  const args: string[] = [];
  let current = "";
  let quote: "'" | '"' | null = null;

  for (const char of trimmed) {
    if (quote) {
      if (char === quote) {
        quote = null;
      } else {
        current += char;
      }
      continue;
    }

    if (char === "'" || char === '"') {
      quote = char;
      continue;
    }

    if (/\s/.test(char)) {
      if (current) {
        args.push(current);
        current = "";
      }
      continue;
    }

    current += char;
  }

  if (current) {
    args.push(current);
  }

  return args;
}
