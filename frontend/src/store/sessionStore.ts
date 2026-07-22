import { create } from "zustand";
import {
  deleteSession,
  getVersion,
  listVersions,
} from "@/services/session";
import type {
  GetVersionResponse,
  VersionMeta,
} from "@/types/session";

interface SessionState {
  projectId?: string;
  versions: VersionMeta[];
  current?: GetVersionResponse;
  loading: boolean;
  errorMessage?: string;

  load: (projectId: string) => Promise<void>;
  select: (version: number) => Promise<void>;
  drop: (projectId: string) => Promise<void>;
  reset: () => void;
}

export const useSessionStore = create<SessionState>((set, get) => ({
  projectId: undefined,
  versions: [],
  current: undefined,
  loading: false,
  errorMessage: undefined,

  reset: () =>
    set({
      projectId: undefined,
      versions: [],
      current: undefined,
      loading: false,
      errorMessage: undefined,
    }),

  load: async (projectId) => {
    set({ loading: true, errorMessage: undefined, projectId });
    try {
      const res = await listVersions(projectId);
      set({ versions: res.versions, loading: false });
    } catch (err) {
      set({
        loading: false,
        errorMessage: err instanceof Error ? err.message : String(err),
      });
    }
  },

  select: async (version) => {
    const projectId = get().projectId;
    if (!projectId) return;
    set({ loading: true, errorMessage: undefined });
    try {
      const snap = await getVersion(projectId, version);
      set({ current: snap, loading: false });
    } catch (err) {
      set({
        loading: false,
        errorMessage: err instanceof Error ? err.message : String(err),
      });
    }
  },

  drop: async (projectId) => {
    await deleteSession(projectId);
    get().reset();
  },
}));
