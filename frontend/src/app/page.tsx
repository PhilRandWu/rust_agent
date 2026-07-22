import { HealthCheckButton } from "@/components/health/HealthCheckButton";

export default function Page() {
  return (
    <main className="mx-auto flex min-h-screen max-w-3xl flex-col items-start gap-6 px-6 py-16">
      <div className="space-y-2">
        <h1 className="text-2xl font-semibold">Rust Agent Frontend</h1>
        <p className="text-sm text-neutral-500">
          Next.js 16 · React 19 · Tailwind v4 · Zustand
        </p>
      </div>

      <section className="w-full rounded-lg border border-neutral-200 p-4 dark:border-neutral-800">
        <h2 className="mb-3 text-sm font-medium text-neutral-700 dark:text-neutral-300">
          后端探活
        </h2>
        <HealthCheckButton />
      </section>
    </main>
  );
}
