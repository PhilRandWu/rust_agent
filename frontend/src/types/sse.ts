// 对齐 backend/src/sse/event.rs 与 sse/phase.rs
// 后端 FrontendEventEnum / TraditionalPhase 使用 serde rename，
// 与下面的字符串字面量完全一一对应。

export type TraditionalPhase =
  | "planning"
  | "foundation"
  | "logic"
  | "view"
  | "assembly";

export type Phase = TraditionalPhase;

export type StreamEventKind =
  | "analysis"
  | "intent"
  | "capabilities"
  | "ui"
  | "components"
  | "structure"
  | "dependency"
  | "types"
  | "utils"
  | "mockData"
  | "service"
  | "hooks"
  | "componentsCode"
  | "pagesCode"
  | "componentsCodePartial"
  | "layouts"
  | "styles"
  | "app"
  | "plan"
  | "files"
  | "session"
  | "done"
  | "error";

interface BaseEvent<T extends StreamEventKind> {
  type: T;
  phase?: Phase;
  data?: unknown;
  message?: string;
}

export type StreamEvent =
  | BaseEvent<Exclude<StreamEventKind, "done" | "error">>
  | BaseEvent<"done">
  | (BaseEvent<"error"> & { message: string });

export function isStreamEvent(v: unknown): v is StreamEvent {
  return (
    typeof v === "object" &&
    v !== null &&
    typeof (v as { type?: unknown }).type === "string"
  );
}
