"use client";

interface Props {
  children?: React.ReactNode;
}

export function PreviewPanel({ children }: Props) {
  return (
    <section className="relative h-full w-full overflow-hidden rounded-xl border border-border bg-surface shadow-sm">
      {children ?? (
        <div className="flex h-full flex-col items-center justify-center gap-3 text-center">
          <div className="flex h-16 w-16 items-center justify-center rounded-2xl bg-accent-light text-2xl">
            🎨
          </div>
          <div className="space-y-1">
            <p className="text-sm font-medium text-foreground/60">预览面板</p>
            <p className="max-w-[200px] text-[12px] leading-relaxed text-muted/60">
              生成完成后，你的应用将在这里呈现
            </p>
          </div>
        </div>
      )}
    </section>
  );
}
