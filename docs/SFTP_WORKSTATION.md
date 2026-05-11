# 📂 SFTP Workstation 契约

这份文档定义 OneShell 当前阶段的 SFTP 工作台约定，方便后续 agent 直接接力。

## 🎯 当前目标

先把 SFTP 工作台最容易返工的基础面固定下来：

- 统一文件树 JSON 结构
- 保证 UTF-8 / 中文 / Emoji 文件名链路稳定
- 提供目录浏览命令和前端面板
- 为后续挂到真实 SSH transport 做准备

## ✅ 当前已接通

- `get_sftp_root`
- `list_sftp_directory`
- React 文件工作台第一版
- 中文 / Emoji 文件名展示链路
- 文件大小、权限、修改时间等基础元数据

## ⚠️ 当前实现边界

这轮还不是“真正的远端 SFTP 传输”：

- 当前读取源是本地工作区根目录
- 目的是先固定文件树 DTO、浏览交互和 UTF-8 文件名链路
- 下一步再把同一套 DTO 直接挂到真实 SSH transport 上

## 🧾 Tauri Command 约定

### `get_sftp_root`

输出：

- `string`

### `list_sftp_directory`

输入：

```ts
type ListSftpDirectoryInput = {
  path?: string;
};
```

输出：

```ts
type SftpEntryNode = {
  name: string;
  path: string;
  kind: "directory" | "file";
  sizeBytes: number;
  permissions: string;
  modifiedAt: number;
};

type SftpDirectorySnapshot = {
  rootPath: string;
  currentPath: string;
  entries: SftpEntryNode[];
  totalEntries: number;
};
```

## 🧩 当前接入文件

- `src-tauri/src/modules/sftp.rs`
- `src-tauri/src/commands/sftp.rs`
- `src/types/sftp.ts`
- `src/lib/tauri/sftp.ts`
- `src/components/sftp/SftpWorkbench.tsx`

## ⏭️ 下一步最高优先级

1. 把 `SftpDirectorySnapshot` 挂到真实 SSH transport
2. 接上传 / 下载 / 删除
3. 把脚本工作站和 SFTP 目录联动
4. 为大文件传输补异步队列和进度状态
