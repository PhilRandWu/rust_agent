// 后端直连地址（next.config.ts 的 rewrites 代理不转发 SSE 流式响应，
// 所以 SSE 请求必须直连后端，参考实现同样处理）
export const BACKEND_URL =
  process.env.NEXT_PUBLIC_BACKEND_URL ?? "http://localhost:7001";

export const API_BASE = "/api";

export const API_ENDPOINTS = {
  health: `${API_BASE}/health`,
  chat: `${BACKEND_URL}/api/chat`,
  sessionVersions: (projectId: string) =>
    `${BACKEND_URL}/api/session/${encodeURIComponent(projectId)}/versions`,
  sessionVersion: (projectId: string, version: number) =>
    `${BACKEND_URL}/api/session/${encodeURIComponent(projectId)}/versions/${version}`,
  session: (projectId: string) =>
    `${BACKEND_URL}/api/session/${encodeURIComponent(projectId)}`,
} as const;
