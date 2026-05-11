export type SshProxyInput = {
  host: string;
  port: number;
  username?: string;
  password?: string;
};

export type SshConnectInput = {
  host: string;
  port: number;
  username: string;
  password: string;
  proxy?: SshProxyInput;
  termType?: string;
  cols?: number;
  rows?: number;
  pixelWidth?: number;
  pixelHeight?: number;
};

export type SshSessionSummary = {
  id: string;
  host: string;
  port: number;
  username: string;
  proxyHost?: string;
  connectedAt: number;
  cols: number;
  rows: number;
};

export type SshInputPacket = {
  sessionId: string;
  data: string;
};

export type SshResizeInput = {
  sessionId: string;
  cols: number;
  rows: number;
  pixelWidth?: number;
  pixelHeight?: number;
};

export type SshOutputEvent = {
  sessionId: string;
  stream: "stdout" | "stderr" | "extended";
  text: string;
  dataBase64: string;
};

export type SshLifecycleEvent = {
  sessionId: string;
  state: string;
  message?: string;
};
