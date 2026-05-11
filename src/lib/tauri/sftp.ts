import { invoke } from "@tauri-apps/api/core";
import type { ListSftpDirectoryInput, SftpDirectorySnapshot } from "../../types/sftp";

export async function getSftpRoot(): Promise<string> {
  return invoke<string>("get_sftp_root");
}

export async function listSftpDirectory(
  input: ListSftpDirectoryInput
): Promise<SftpDirectorySnapshot> {
  return invoke<SftpDirectorySnapshot>("list_sftp_directory", { input });
}
