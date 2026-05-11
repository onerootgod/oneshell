export type ListSftpDirectoryInput = {
  path?: string;
};

export type SftpEntryNode = {
  name: string;
  path: string;
  kind: "directory" | "file";
  sizeBytes: number;
  permissions: string;
  modifiedAt: number;
};

export type SftpDirectorySnapshot = {
  rootPath: string;
  currentPath: string;
  entries: SftpEntryNode[];
  totalEntries: number;
};
