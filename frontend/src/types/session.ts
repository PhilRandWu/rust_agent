// 对齐 backend/src/session/model.rs 与 routes/session/dto.rs

export type SessionOperation = "create" | "edit";

export interface VersionMeta {
  version: number;
  timestampMs: number;
  operation: SessionOperation;
  prompt: string;
  fileCount: number;
}

export interface VersionSnapshot extends VersionMeta {
  files: Record<string, string>;
}

export interface ListVersionsResponse {
  projectId: string;
  versions: VersionMeta[];
}

export interface GetVersionResponse extends VersionSnapshot {
  projectId: string;
}
