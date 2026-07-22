import { create } from "zustand";
import { fetchHealth } from "@/services/health";

type Status = "idle" | "loading" | "ok" | "error";

interface HealthState {
  status: Status;
  message: string;
  check: () => Promise<void>;
  reset: () => void;
}

export const useHealthStore = create<HealthState>((set) => ({
  status: "idle",
  message: "",
  reset: () => set({ status: "idle", message: "" }),
  check: async () => {
    set({ status: "loading", message: "" });
    try {
      const data = await fetchHealth();
      set({ status: "ok", message: `status: ${data.status}` });
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      set({ status: "error", message: msg });
    }
  },
}));
