import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  SshConnectInput,
  SshInputPacket,
  SshLifecycleEvent,
  SshOutputEvent,
  SshResizeInput,
  SshSessionSummary
} from "../../types/ssh";

export const SSH_OUTPUT_EVENT = "ssh-output";
export const SSH_LIFECYCLE_EVENT = "ssh-lifecycle";

export async function connectSshSession(
  input: SshConnectInput
): Promise<SshSessionSummary> {
  return invoke<SshSessionSummary>("connect_ssh_session", { input });
}

export async function sendSshInput(packet: SshInputPacket): Promise<void> {
  return invoke("send_ssh_input", { packet });
}

export async function resizeSshSession(input: SshResizeInput): Promise<void> {
  return invoke("resize_ssh_session", { input });
}

export async function disconnectSshSession(sessionId: string): Promise<void> {
  return invoke("disconnect_ssh_session", { sessionId });
}

export async function listSshSessions(): Promise<SshSessionSummary[]> {
  return invoke<SshSessionSummary[]>("list_ssh_sessions");
}

export async function listenSshOutput(
  handler: (event: SshOutputEvent) => void
): Promise<UnlistenFn> {
  return listen<SshOutputEvent>(SSH_OUTPUT_EVENT, ({ payload }) => handler(payload));
}

export async function listenSshLifecycle(
  handler: (event: SshLifecycleEvent) => void
): Promise<UnlistenFn> {
  return listen<SshLifecycleEvent>(SSH_LIFECYCLE_EVENT, ({ payload }) =>
    handler(payload)
  );
}
