import { create } from "zustand";
import { generateAppStream } from "@/services/chat";
import type { ChatMessage } from "@/types/chat";
import type { StreamEvent } from "@/types/sse";

type Status = "idle" | "streaming" | "done" | "error";

interface ChatState {
  projectId?: string;
  messages: ChatMessage[];
  events: StreamEvent[];
  status: Status;
  errorMessage?: string;
  abort?: () => void;

  setProjectId: (id?: string) => void;
  appendMessage: (msg: ChatMessage) => void;
  send: (userText: string) => Promise<void>;
  cancel: () => void;
  reset: () => void;
}

export const useChatStore = create<ChatState>((set, get) => ({
  projectId: undefined,
  messages: [],
  events: [],
  status: "idle",
  errorMessage: undefined,
  abort: undefined,

  setProjectId: (id) => set({ projectId: id }),
  appendMessage: (msg) =>
    set((s) => ({ messages: [...s.messages, msg] })),

  reset: () =>
    set({
      messages: [],
      events: [],
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
    const userMsg: ChatMessage = { role: "user", content: userText };

    set((s) => ({
      messages: [...s.messages, userMsg],
      events: [],
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
          set((s) => ({ events: [...s.events, event] }));
          if (event.type === "error") {
            set({ status: "error", errorMessage: event.message });
          } else if (event.type === "done") {
            set({ status: "done" });
          }
        },
      });
      if (get().status === "streaming") set({ status: "done" });
    } catch (err) {
      if (controller.signal.aborted) {
        set({ status: "idle" });
        return;
      }
      set({
        status: "error",
        errorMessage: err instanceof Error ? err.message : String(err),
      });
    } finally {
      set({ abort: undefined });
    }
  },
}));
