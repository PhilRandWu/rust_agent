"use client";

import type { StreamEvent } from "@/types/sse";

interface Props {
  events: StreamEvent[];
  isStreaming: boolean;
}

interface DisplayItem {
  type: "phase" | "step";
  label: string;
  isError: boolean;
  isLast: boolean;
  isStreaming: boolean;
}

const PHASE_LABELS: Record<string, string> = {
  planning: "规划阶段",
  foundation: "基础建设",
  logic: "逻辑构建",
  view: "视图构建",
  assembly: "应用组装",
};

const EVENT_LABELS: Record<string, string> = {
  analysis: "需求分析",
  intent: "意图识别",
  capabilities: "能力评估",
  ui: "UI 分析",
  components: "组件识别",
  structure: "结构分析",
  dependency: "依赖分析",
  types: "类型生成",
  utils: "工具函数",
  mockData: "Mock 数据",
  service: "服务层",
  hooks: "Hooks",
  componentsCode: "组件代码",
  pagesCode: "页面代码",
  componentsCodePartial: "部分组件代码",
  layouts: "布局",
  styles: "样式",
  app: "应用组装",
  plan: "计划",
  files: "文件生成",
  session: "会话保存",
};

function buildDisplayItems(
  events: StreamEvent[],
  isStreaming: boolean,
): DisplayItem[] {
  const items: DisplayItem[] = [];
  let lastPhase = "";

  for (let i = 0; i < events.length; i++) {
    const event = events[i];
    if (event.type === "done") continue;

    const showPhase = event.phase && event.phase !== lastPhase;
    if (event.phase) lastPhase = event.phase;

    if (showPhase) {
      items.push({
        type: "phase",
        label: PHASE_LABELS[event.phase!] ?? event.phase!,
        isError: false,
        isLast: false,
        isStreaming: false,
      });
    }

    items.push({
      type: "step",
      label:
        event.type === "error"
          ? (event.message ?? "未知错误")
          : (EVENT_LABELS[event.type] ?? event.type),
      isError: event.type === "error",
      isLast: i === events.length - 1,
      isStreaming,
    });
  }

  return items;
}

export function EventTimeline({ events, isStreaming }: Props) {
  const items = buildDisplayItems(events, isStreaming);

  if (items.length === 0) return null;

  return (
    <div className="space-y-1.5 rounded-xl border border-border bg-background p-3">
      {items.map((item, i) =>
        item.type === "phase" ? (
          <div key={`p-${i}`} className="mb-2 mt-1 flex items-center gap-2">
            <div className="h-px flex-1 bg-border" />
            <span className="text-[10px] font-semibold uppercase tracking-widest text-muted">
              {item.label}
            </span>
            <div className="h-px flex-1 bg-border" />
          </div>
        ) : (
          <div key={`s-${i}`} className="flex items-start gap-2.5 px-1">
            <div className="relative mt-1">
              {item.isError ? (
                <span className="block h-2 w-2 rounded-full bg-error" />
              ) : item.isLast && item.isStreaming ? (
                <span className="block h-2 w-2 animate-pulse rounded-full bg-accent" />
              ) : (
                <span className="block h-2 w-2 rounded-full bg-success" />
              )}
            </div>
            <span
              className={`text-sm leading-tight ${
                item.isError ? "font-medium text-error" : "text-muted"
              }`}
            >
              {item.label}
            </span>
          </div>
        ),
      )}
    </div>
  );
}
