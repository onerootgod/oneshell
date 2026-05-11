import { invoke } from "@tauri-apps/api/core";
import type {
  SaveServerProfileInput,
  ServerProfileSummary
} from "../../types/serverProfiles";

export async function listServerProfiles(): Promise<ServerProfileSummary[]> {
  return invoke<ServerProfileSummary[]>("list_server_profiles");
}

export async function saveServerProfile(
  input: SaveServerProfileInput
): Promise<ServerProfileSummary> {
  return invoke<ServerProfileSummary>("save_server_profile", { input });
}
