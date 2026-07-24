"use client";

import { useChatStore } from "@/store/chatStore";
import { useEffect, useRef, useState } from "react";
import { MessageBubble } from "../chat/MessageBubble";

export function ChatPanel() {
  const messages = useChatStore((s) => s.messages);
  const eventsByMessage = useChatStore((s) => s.eventsByMessage);
  const streamingMessageId = useChatStore((s) => s.streamingMessageId);
  const status = useChatStore((s) => s.status);
  const send = useChatStore((s) => s.send);
  const cancel = useChatStore((s) => s.cancel);

  const [input, setInput] = useState("");
  const scrollRef = useRef<HTMLDivElement>(null);
  const isLoading = status === "streaming";

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTo({
        top: scrollRef.current.scrollHeight,
        behavior: "smooth",
      });
    }
  }, [messages, eventsByMessage]);

  const handleSend = () => {
    const text = input.trim();
    if (!text || isLoading) return;
    send(text);
    setInput("");
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  return (
    <div className="flex h-full flex-col">
      {/* 消息列表 */}
      <div
        ref={scrollRef}
        className="flex-1 overflow-y-auto px-4 py-4 scroll-smooth"
      >
        {messages.length === 0 && (
          <div className="flex h-full flex-col items-center justify-center gap-2 text-center">
            <div className="flex h-10 w-10 items-center justify-center rounded-xl bg-accent-light text-lg">
              🤖
            </div>
            <p className="text-sm text-muted">输入需求，开始生成 React 应用</p>
            <p className="max-w-[240px] text-[12px] leading-relaxed text-muted/60">
              描述你想要的应用，AI 会自动完成需求分析、组件设计、代码生成
            </p>
          </div>
        )}
        <div className="space-y-4">
          {messages.map((msg) => (
            <div key={msg.id} className="message-enter">
              <MessageBubble
                message={msg}
                events={msg.id ? eventsByMessage[msg.id] : undefined}
                isStreaming={msg.id === streamingMessageId}
              />
            </div>
          ))}
        </div>
      </div>

      {/* 输入框 */}
      <div className="shrink-0 border-t border-border bg-surface px-3 py-3">
        <div className="flex items-end gap-2">
          <textarea
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="今天你想构建什么样的应用？"
            rows={2}
            disabled={isLoading}
            className="flex-1 resize-none rounded-xl border border-border bg-background px-3.5 py-2.5 text-sm leading-relaxed text-foreground placeholder:text-muted/50 transition-colors focus:border-accent/40 focus:outline-none focus:ring-2 focus:ring-accent/10 disabled:opacity-50"
          />
          {isLoading ? (
            <button
              onClick={cancel}
              className="shrink-0 rounded-xl bg-error px-4 py-2.5 text-sm font-medium text-white transition hover:bg-error/90 active:scale-[0.97]"
            >
              取消
            </button>
          ) : (
            <button
              onClick={handleSend}
              disabled={!input.trim()}
              className="shrink-0 rounded-xl bg-accent px-4 py-2.5 text-sm font-medium text-white transition hover:bg-accent/90 active:scale-[0.97] disabled:opacity-40 disabled:active:scale-100"
            >
              发送
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
