export type ScriptEntrySummary = {
  id: string;
  name: string;
  path: string;
  kind: "python" | "shell" | "unknown";
  relativePath: string;
  sizeBytes: number;
  modifiedAt: number;
};

export type ScriptEntryDetail = {
  summary: ScriptEntrySummary;
  content: string;
  suggestedRemoteCommand: string;
  localRunner: string;
};

export type RunLocalScriptInput = {
  path: string;
  args?: string[];
};

export type BuildRemoteScriptCommandInput = {
  path: string;
  args?: string[];
};

export type ScriptExecutionResult = {
  command: string;
  exitCode: number;
  stdout: string;
  stderr: string;
};
