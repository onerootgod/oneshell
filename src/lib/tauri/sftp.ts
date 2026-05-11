import { invoke } from "@tauri-apps/api/core";
import type {
  CreateSftpDirectoryInput,
  DeleteSftpEntryInput,
  DownloadSftpFileInput,
  ListSftpDirectoryInput,
  SftpDirectorySnapshot,
  SftpOperationResult,
  UploadSftpFileInput
} from "../../types/sftp";

export async function getSftpRoot(): Promise<string> {
  return invoke<string>("get_sftp_root");
}

export async function listSftpDirectory(
  input: ListSftpDirectoryInput
): Promise<SftpDirectorySnapshot> {
  return invoke<SftpDirectorySnapshot>("list_sftp_directory", { input });
}

export async function createSftpDirectory(
  input: CreateSftpDirectoryInput
): Promise<SftpOperationResult> {
  return invoke<SftpOperationResult>("create_sftp_directory", { input });
}

export async function deleteSftpEntry(
  input: DeleteSftpEntryInput
): Promise<SftpOperationResult> {
  return invoke<SftpOperationResult>("delete_sftp_entry", { input });
}

export async function uploadSftpFile(
  input: UploadSftpFileInput
): Promise<SftpOperationResult> {
  return invoke<SftpOperationResult>("upload_sftp_file", { input });
}

export async function downloadSftpFile(
  input: DownloadSftpFileInput
): Promise<SftpOperationResult> {
  return invoke<SftpOperationResult>("download_sftp_file", { input });
}
