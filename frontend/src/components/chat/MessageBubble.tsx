"use client";

import type { ChatMessage } from "@/types/chat";
import type { StreamEvent } from "@/types/sse";
import { EventTimeline } from "../shell/EventTimeline";

interface Props {
  message: ChatMessage;
  events?: StreamEvent[];
  isStreaming: boolean;
}

function formatContent(content: unknown): string {
  if (typeof content === "string") return content;
  if (content === null || content === undefined) return "";
  try {
    return JSON.stringify(content);
  } catch {
    return String(content);
  }
}

export function MessageBubble({ message, events, isStreaming }: Props) {
  const isUser = message.role === "user";
  const text = formatContent(message.content);

  return (
    <div className={`flex ${isUser ? "justify-end" : "justify-start"}`}>
      <div className="max-w-[85%] space-y-2">
        {isUser && text && (
          <div className="rounded-2xl bg-accent px-4 py-2.5 text-sm leading-relaxed text-white">
            {text}
          </div>
        )}

        {!isUser && (
          <>
            {text ? (
              <div className="rounded-2xl bg-neutral-100 px-4 py-2.5 text-sm leading-relaxed text-foreground dark:bg-neutral-800">
                {text}
              </div>
            ) : isStreaming && !events?.length ? (
              <div className="flex items-center gap-1.5 rounded-2xl bg-neutral-100 px-4 py-3 text-sm text-muted dark:bg-neutral-800">
                <span>思考中</span>
                <span className="thinking-dots flex items-center gap-1">
                  <span />
                  <span />
                  <span />
                </span>
              </div>
            ) : null}

            {events && events.length > 0 && (
              <EventTimeline events={events} isStreaming={isStreaming} />
            )}
          </>
        )}
      </div>
    </div>
  );
}
