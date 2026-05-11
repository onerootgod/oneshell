import { invoke } from "@tauri-apps/api/core";
import type {
  BuildRemoteScriptCommandInput,
  RunLocalScriptInput,
  ScriptEntryDetail,
  ScriptEntrySummary,
  ScriptExecutionResult
} from "../../types/scripts";

export async function listScriptEntries(): Promise<ScriptEntrySummary[]> {
  return invoke<ScriptEntrySummary[]>("list_script_entries");
}

export async function getScriptEntryDetail(path: string): Promise<ScriptEntryDetail> {
  return invoke<ScriptEntryDetail>("get_script_entry_detail", { path });
}

export async function runLocalScript(
  input: RunLocalScriptInput
): Promise<ScriptExecutionResult> {
  return invoke<ScriptExecutionResult>("run_local_script", { input });
}

export async function buildRemoteScriptCommand(
  input: BuildRemoteScriptCommandInput
): Promise<string> {
  return invoke<string>("build_remote_script_command", { input });
}

export async function getScriptWorkspaceRoot(): Promise<string> {
  return invoke<string>("get_script_workspace_root");
}
