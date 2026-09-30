import { create } from "zustand";
import { api } from "./api";
import type {
  AppInfo,
  CachedRelease,
  Game,
  GpuInfo,
  LogEntry,
  Progress,
  ReleaseFeed,
  Settings,
} from "./types";

export type Page = "games" | "detail" | "mod" | "tutorial" | "settings" | "logs";

export interface Toast {
  id: number;
  message: string;
  type: "info" | "success" | "error" | "warn";
}

interface AppStore {
  page: Page;
  selectedGameId: string | null;
  games: Game[];
  settings: Settings | null;
  gpu: GpuInfo | null;
  appInfo: AppInfo | null;
  feed: ReleaseFeed | null;
  cached: CachedRelease[];
  logs: LogEntry[];
  progress: Progress | null;
  busy: string | null;
  toasts: Toast[];
  loading: boolean;

  init: () => Promise<void>;
  setPage: (p: Page) => void;
  openGame: (id: string) => void;
  selectedGame: () => Game | undefined;
  reloadGames: () => Promise<void>;
  scan: () => Promise<void>;
  loadFeed: (force?: boolean) => Promise<void>;
  reloadCached: () => Promise<void>;
  reloadLogs: () => Promise<void>;
  refreshCompat: () => Promise<void>;
  saveSettings: (s: Settings, startWithWindows?: boolean) => Promise<void>;
  applyMod: (id: string, tag?: string | null) => Promise<boolean>;
  removeMod: (id: string) => Promise<boolean>;
  updateAll: (ids: string[], tag?: string | null) => Promise<void>;
  setProgress: (p: Progress | null) => void;
  toast: (message: string, type?: Toast["type"]) => void;
  dismissToast: (id: number) => void;
}

let toastSeq = 1;

export const useStore = create<AppStore>((set, get) => ({
  page: "games",
  selectedGameId: null,
  games: [],
  settings: null,
  gpu: null,
  appInfo: null,
  feed: null,
  cached: [],
  logs: [],
  progress: null,
  busy: null,
  toasts: [],
  loading: true,

  async init() {
    set({ loading: true });
    try {
      const [settings, gpu, appInfo, games, logs, cached] = await Promise.all([
        api.getSettings(),
        api.detectGpu(),
        api.getAppInfo(),
        api.listGames(),
        api.getLogs(),
        api.listCachedReleases(),
      ]);
      set({ settings, gpu, appInfo, games, logs, cached, loading: false });
      // Reavalia a compatibilidade (compat.json) para os cards mostrarem o status real.
      void get().refreshCompat();
      // Verificacao automatica de atualizacoes no GitHub (configuravel).
      if (settings.checkUpdatesOnStart) {
        try {
          const feed = await api.listReleases(false);
          set({ feed });
          if (feed.message) get().toast(feed.message, "warn");
        } catch (e) {
          get().toast(String(e), "warn");
        }
      }
    } catch (e) {
      set({ loading: false });
      get().toast(String(e), "error");
    }
  },

  setPage: (page) => set({ page }),
  openGame: (id) => set({ selectedGameId: id, page: "detail" }),
  selectedGame: () => {
    const { games, selectedGameId } = get();
    return games.find((g) => g.id === selectedGameId);
  },

  async reloadGames() {
    const games = await api.listGames();
    set({ games });
  },

  async scan() {
    set({ busy: "Detectando jogos..." });
    try {
      const games = await api.scanLibrary();
      set({ games });
      get().toast(`${games.length} jogo(s) na biblioteca.`, "success");
    } catch (e) {
      get().toast(String(e), "error");
    } finally {
      set({ busy: null });
    }
  },

  async loadFeed(force = false) {
    try {
      const feed = await api.listReleases(force);
      set({ feed });
      if (feed.message) get().toast(feed.message, "warn");
    } catch (e) {
      get().toast(String(e), "error");
    }
  },

  async reloadCached() {
    const cached = await api.listCachedReleases();
    set({ cached });
  },

  async reloadLogs() {
    const logs = await api.getLogs();
    set({ logs });
  },

  async refreshCompat() {
    const games = await api.refreshCompat();
    set({ games });
  },

  async saveSettings(s, startWithWindows) {
    try {
      const saved = await api.saveSettings(s);
      set({ settings: saved });
      if (startWithWindows !== undefined) {
        await api.setStartWithWindows(startWithWindows);
      }
      get().toast("Configuracoes salvas.", "success");
      await get().loadFeed(true);
    } catch (e) {
      get().toast(String(e), "error");
    }
  },

  async applyMod(id, tag) {
    set({ busy: "Aplicando mod..." });
    try {
      const game = await api.applyMod(id, tag ?? null);
      await get().reloadGames();
      get().toast(`Mod aplicado em ${game.name}.`, "success");
      return true;
    } catch (e) {
      get().toast(String(e), "error");
      return false;
    } finally {
      set({ busy: null, progress: null });
    }
  },

  async removeMod(id) {
    set({ busy: "Removendo mod..." });
    try {
      const game = await api.removeMod(id);
      await get().reloadGames();
      get().toast(`Mod removido de ${game.name}.`, "success");
      return true;
    } catch (e) {
      get().toast(String(e), "error");
      return false;
    } finally {
      set({ busy: null, progress: null });
    }
  },

  async updateAll(ids, tag) {
    if (ids.length === 0) return;
    set({ busy: "Atualizando jogos..." });
    try {
      const result = await api.updateAll(ids, tag ?? null);
      await get().reloadGames();
      for (const f of result.failed) {
        get().toast(`${f.name}: ${f.message}`, "error");
      }
      get().toast(
        `${result.updated.length} jogo(s) atualizado(s), ${result.failed.length} falha(s).`,
        result.failed.length ? "warn" : "success"
      );
    } catch (e) {
      get().toast(String(e), "error");
    } finally {
      set({ busy: null, progress: null });
    }
  },

  setProgress: (progress) => set({ progress }),

  toast(message, type = "info") {
    const id = toastSeq++;
    set((s) => ({ toasts: [...s.toasts, { id, message, type }] }));
    setTimeout(() => get().dismissToast(id), 6000);
  },

  dismissToast: (id) => set((s) => ({ toasts: s.toasts.filter((t) => t.id !== id) })),
}));
