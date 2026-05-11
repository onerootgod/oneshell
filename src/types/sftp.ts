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

export type CreateSftpDirectoryInput = {
  parentPath: string;
  name: string;
};

export type DeleteSftpEntryInput = {
  path: string;
};

export type UploadSftpFileInput = {
  sourcePath: string;
  targetDirectory: string;
  targetName?: string;
};

export type DownloadSftpFileInput = {
  sourcePath: string;
  destinationPath: string;
};

export type SftpOperationResult = {
  action: string;
  sourcePath?: string;
  targetPath: string;
  bytesTransferred: number;
};

export type SftpTransferRecord = {
  id: string;
  action: string;
  sourcePath?: string;
  targetPath: string;
  bytesTransferred: number;
  status: string;
  createdAt: number;
};
