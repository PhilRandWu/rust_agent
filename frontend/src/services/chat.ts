import { API_ENDPOINTS } from "@/constants/api";
import { postSse } from "@/services/sse";
import type { ChatRequestPayload } from "@/types/chat";
import type { StreamEvent } from "@/types/sse";

export interface GenerateAppStreamOptions {
  payload: ChatRequestPayload;
  onEvent: (event: StreamEvent) => void;
  signal?: AbortSignal;
}

export function generateAppStream(
  options: GenerateAppStreamOptions,
): Promise<void> {
  return postSse({
    url: API_ENDPOINTS.chat,
    body: options.payload,
    onEvent: options.onEvent,
    signal: options.signal,
  });
}
