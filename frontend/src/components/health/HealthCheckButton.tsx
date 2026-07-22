"use client";

import { useHealthStore } from "@/store/healthStore";

const STATUS_STYLES: Record<string, string> = {
  idle: "text-neutral-500",
  loading: "text-neutral-500",
  ok: "text-emerald-600",
  error: "text-red-600",
};

export function HealthCheckButton() {
  const status = useHealthStore((s) => s.status);
  const message = useHealthStore((s) => s.message);
  const check = useHealthStore((s) => s.check);

  const isLoading = status === "loading";

  return (
    <div className="flex items-center gap-3">
      <button
        type="button"
        onClick={check}
        disabled={isLoading}
        className="rounded-md bg-neutral-900 px-3 py-1.5 text-sm text-white transition hover:bg-neutral-700 disabled:opacity-50 dark:bg-neutral-100 dark:text-neutral-900 dark:hover:bg-neutral-300"
      >
        {isLoading ? "检查中..." : "检查 /api/health"}
      </button>
      <span className={`text-sm ${STATUS_STYLES[status]}`}>
        {status === "idle" ? "未检查" : message || status}
      </span>
    </div>
  );
}
