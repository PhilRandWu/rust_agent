"use client";

import { ChatPanel } from "./ChatPanel";
import { PreviewPanel } from "./PreviewPanel";

export function AppShell() {
  return (
    <div className="flex h-screen w-screen flex-col overflow-hidden bg-background">
      {/* Header */}
      <header className="relative flex h-13 shrink-0 items-center justify-between border-b border-border bg-surface px-5">
        <div className="flex items-center gap-2.5">
          <div className="flex h-7 w-7 items-center justify-center rounded-lg bg-accent text-[11px] font-bold text-white">
            RA
          </div>
          <h1 className="text-sm font-semibold text-foreground">Rust Agent</h1>
          <span className="rounded-full bg-accent-light px-2 py-0.5 text-[11px] font-medium text-accent">
            Beta
          </span>
        </div>
      </header>

      {/* Body */}
      <main className="flex flex-1 gap-4 overflow-hidden p-4">
        {/* Left: Chat */}
        <div className="flex w-[420px] shrink-0 flex-col overflow-hidden rounded-xl border border-border bg-surface shadow-sm">
          <ChatPanel />
        </div>

        {/* Right: Preview */}
        <div className="relative flex-1">
          <PreviewPanel />
        </div>
      </main>
    </div>
  );
}
