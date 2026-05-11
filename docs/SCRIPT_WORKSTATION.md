# 🧰 Script Workstation 契约

这份文档定义 OneShell 当前阶段的脚本工作站约定，方便后续 agent 直接接力。

## 🎯 当前目标

让 `~/NexusScripts` 成为 OneShell 的本地脚本资产目录：

- 自动扫描 `.py / .sh`
- 前端可直接浏览脚本内容
- 可本地执行并把 stdout / stderr 回传工作台
- 可一键把脚本执行命令注入当前 SSH 会话

## 📁 目录约定

当前固定根目录：

- `~/NexusScripts`

Rust runtime 启动时会把它收进 `AppState.scripts`，如果目录不存在，会在第一次扫描时自动创建。

## 🧾 Tauri Command 约定

### `list_script_entries`

输出：

```ts
type ScriptEntrySummary = {
  id: string;
  name: string;
  path: string;
  kind: "python" | "shell" | "unknown";
  relativePath: string;
  sizeBytes: number;
  modifiedAt: number;
};
```

### `get_script_entry_detail`

输入：

- `path: string`

输出：

```ts
type ScriptEntryDetail = {
  summary: ScriptEntrySummary;
  content: string;
  suggestedRemoteCommand: string;
  localRunner: string;
};
```

### `run_local_script`

输入：

```ts
type RunLocalScriptInput = {
  path: string;
  args?: string[];
};
```

输出：

```ts
type ScriptExecutionResult = {
  command: string;
  exitCode: number;
  stdout: string;
  stderr: string;
};
```

### `build_remote_script_command`

输入：

```ts
type BuildRemoteScriptCommandInput = {
  path: string;
  args?: string[];
};
```

输出：

- `string`

### `get_script_workspace_root`

输出：

- `string`

## ⚙️ 当前执行模式

### Python

- 本地执行：`python3 <script.py>`
- 远端注入：`python3 - <<'PY' ... PY`

### Shell

- 本地执行：`bash <script.sh>`
- 远端注入：`bash -s <<'SH' ... SH`

## 🧪 参数支持

当前脚本工作站已经支持参数输入：

- 本地执行参数：通过 `run_local_script.args`
- 远端注入参数：通过 `build_remote_script_command.args`

当前参数解析规则：

- 默认按空格分词
- 支持单引号
- 支持双引号
- 暂不支持复杂转义语法

## 🔐 当前安全边界

- 只允许 `~/NexusScripts` 根目录内的脚本
- 会对输入路径做 `canonicalize`
- 不允许逃逸到根目录之外
- 当前只接受 `.py` 和 `.sh`

## 🧩 当前前端接入文件

- `src/lib/tauri/scripts.ts`
- `src/types/scripts.ts`
- `src/components/scripts/ScriptWorkbench.tsx`
- `src/components/terminal/MacTerminal.tsx`

## ⏭️ 下一步最高优先级

1. 支持保存常用脚本运行模板
2. 把脚本工作站的远端注入升级成“上传后执行 / 内联执行 + 参数模板”
3. 在真实 SSH transport 之上接 SFTP 文件分发能力
4. 为脚本执行结果补历史与收藏视图
