export const API_BASE = "/api";

export const API_ENDPOINTS = {
  health: `${API_BASE}/health`,
  chat: `${API_BASE}/chat`,
  sessionVersions: (projectId: string) =>
    `${API_BASE}/session/${encodeURIComponent(projectId)}/versions`,
  sessionVersion: (projectId: string, version: number) =>
    `${API_BASE}/session/${encodeURIComponent(projectId)}/versions/${version}`,
  session: (projectId: string) =>
    `${API_BASE}/session/${encodeURIComponent(projectId)}`,
} as const;
