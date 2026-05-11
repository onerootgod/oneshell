export type SaveServerProfileInput = {
  name?: string;
  host: string;
  port: number;
  username: string;
  password: string;
};

export type ServerProfileSummary = {
  id: string;
  name?: string;
  host: string;
  port: number;
  username: string;
  createdAt: number;
  updatedAt: number;
};
