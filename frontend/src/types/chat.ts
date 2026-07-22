// 对齐 backend/src/routes/chat/dto.rs

export type ChatRole = "user" | "assistant" | "system";

export interface ChatMessage {
  id?: string;
  role: ChatRole;
  content: unknown;
  attachments?: unknown[];
}

export interface ChatRequestPayload {
  messages: ChatMessage[];
  projectId?: string;
  mockConfig?: Record<string, unknown>;
}
