import { create } from "zustand";
import { generateAppStream } from "@/services/chat";
import type { ChatMessage } from "@/types/chat";
import type { StreamEvent } from "@/types/sse";

type Status = "idle" | "streaming" | "done" | "error";

interface ChatState {
  projectId?: string;
  messages: ChatMessage[];
  eventsByMessage: Record<string, StreamEvent[]>;
  streamingMessageId?: string;
  status: Status;
  errorMessage?: string;
  abort?: () => void;

  setProjectId: (id?: string) => void;
  send: (userText: string) => Promise<void>;
  cancel: () => void;
  reset: () => void;
}

export const useChatStore = create<ChatState>((set, get) => ({
  projectId: undefined,
  messages: [],
  eventsByMessage: {},
  streamingMessageId: undefined,
  status: "idle",
  errorMessage: undefined,
  abort: undefined,

  setProjectId: (id) => set({ projectId: id }),

  reset: () =>
    set({
      messages: [],
      eventsByMessage: {},
      streamingMessageId: undefined,
      status: "idle",
      errorMessage: undefined,
      abort: undefined,
    }),

  cancel: () => {
    const abort = get().abort;
    if (abort) abort();
  },

  send: async (userText) => {
    if (get().status === "streaming") return;

    const controller = new AbortController();
    const userMsg: ChatMessage = {
      id: crypto.randomUUID(),
      role: "user",
      content: userText,
    };
    const assistantMsg: ChatMessage = {
      id: crypto.randomUUID(),
      role: "assistant",
      content: "",
    };

    set((s) => ({
      messages: [...s.messages, userMsg, assistantMsg],
      eventsByMessage: { ...s.eventsByMessage, [assistantMsg.id!]: [] },
      streamingMessageId: assistantMsg.id,
      status: "streaming",
      errorMessage: undefined,
      abort: () => controller.abort(),
    }));

    try {
      await generateAppStream({
        payload: {
          messages: [...get().messages],
          projectId: get().projectId,
        },
        signal: controller.signal,
        onEvent: (event) => {
          const sid = get().streamingMessageId;
          if (sid) {
            set((s) => ({
              eventsByMessage: {
                ...s.eventsByMessage,
                [sid]: [...(s.eventsByMessage[sid] ?? []), event],
              },
            }));
          }
          if (event.type === "error") {
            set({ status: "error", errorMessage: event.message });
          } else if (event.type === "done") {
            set({ status: "done", streamingMessageId: undefined });
          }
        },
      });
      if (get().status === "streaming") {
        set({ status: "done", streamingMessageId: undefined });
      }
    } catch (err) {
      if (controller.signal.aborted) {
        set({ status: "idle", streamingMessageId: undefined });
        return;
      }
      set({
        status: "error",
        streamingMessageId: undefined,
        errorMessage: err instanceof Error ? err.message : String(err),
      });
    } finally {
      set({ abort: undefined });
    }
  },
}));
