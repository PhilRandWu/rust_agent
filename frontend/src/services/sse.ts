import { HttpError } from "@/services/http";
import { isStreamEvent, type StreamEvent } from "@/types/sse";

export interface PostSseOptions<TBody> {
  url: string;
  body: TBody;
  onEvent: (event: StreamEvent) => void;
  signal?: AbortSignal;
}

/**
 * 发送 POST 请求并逐帧解析 SSE。
 * - 帧分隔符: \n\n
 * - 只处理以 "data:" 开头的行；其他行 (如 keep-alive) 忽略
 * - 支持 AbortSignal 取消
 */
export async function postSse<TBody>(
  options: PostSseOptions<TBody>,
): Promise<void> {
  const { url, body, onEvent, signal } = options;

  const response = await fetch(url, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Accept: "text/event-stream",
    },
    body: JSON.stringify(body),
    signal,
  });

  if (!response.ok) {
    throw new HttpError(response.status, `POST ${url} -> ${response.status}`);
  }
  if (!response.body) {
    throw new Error("SSE response has no body");
  }

  const reader = response.body.getReader();
  const decoder = new TextDecoder("utf-8");
  let buffer = "";

  try {
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      buffer += decoder.decode(value, { stream: true });

      const parts = buffer.split("\n\n");
      buffer = parts.pop() ?? "";
      for (const part of parts) {
        dispatchFrame(part, onEvent);
      }
    }
    if (buffer.length > 0) {
      dispatchFrame(buffer, onEvent);
    }
  } finally {
    reader.releaseLock();
  }
}

function dispatchFrame(
  frame: string,
  onEvent: (event: StreamEvent) => void,
): void {
  for (const line of frame.split("\n")) {
    if (!line.startsWith("data:")) continue;
    const payload = line.slice(5).trimStart();
    if (!payload) continue;
    try {
      const parsed: unknown = JSON.parse(payload);
      if (isStreamEvent(parsed)) onEvent(parsed);
    } catch {
      // 忽略非 JSON 数据行 (例如 keep-alive 文本)
    }
  }
}
