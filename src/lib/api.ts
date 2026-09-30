import { invoke } from "@tauri-apps/api/core";
import type {
  AppInfo,
  CachedRelease,
  CompatEntry,
  Game,
  GameStatus,
  GpuInfo,
  LogEntry,
  ReleaseFeed,
  RepairReport,
  Settings,
  UpdateAllResult,
} from "./types";

/** Wrappers tipados para os comandos Tauri. */
export const api = {
  // Aplicativo / ambiente
  getAppInfo: () => invoke<AppInfo>("get_app_info"),
  getSettings: () => invoke<Settings>("get_settings"),
  saveSettings: (settings: Settings) => invoke<Settings>("save_settings", { settings }),
  detectGpu: () => invoke<GpuInfo>("detect_gpu"),
  isElevated: () => invoke<boolean>("is_elevated"),
  restartElevated: () => invoke<void>("restart_elevated"),
  setStartWithWindows: (enabled: boolean) =>
    invoke<void>("set_start_with_windows", { enabled }),

  // Biblioteca
  listGames: () => invoke<Game[]>("list_games"),
  scanLibrary: () => invoke<Game[]>("scan_library"),
  refreshCompat: () => invoke<Game[]>("refresh_compat"),
  addGameManual: (path: string, name?: string) =>
    invoke<Game>("add_game_manual", { path, name: name ?? null }),
  removeGame: (id: string) => invoke<void>("remove_game", { id }),
  updateGame: (game: Game) => invoke<Game>("update_game", { game }),
  findExecutable: (installDir: string, name: string) =>
    invoke<string | null>("find_executable", { installDir, name }),

  // Instalacao
  applyMod: (id: string, tag?: string | null) =>
    invoke<Game>("apply_mod", { id, tag: tag ?? null }),
  removeMod: (id: string) => invoke<Game>("remove_mod", { id }),
  repairMod: (id: string) => invoke<RepairReport>("repair_mod", { id }),
  getGameStatus: (id: string) => invoke<GameStatus>("get_game_status", { id }),
  findModConfig: (id: string) => invoke<string | null>("find_mod_config", { id }),
  revealInExplorer: (path: string) => invoke<void>("reveal_in_explorer", { path }),

  // Versoes / GitHub
  listReleases: (force = false) => invoke<ReleaseFeed>("list_releases", { force }),
  listCachedReleases: () => invoke<CachedRelease[]>("list_cached_releases"),
  downloadRelease: (tag: string) => invoke<CachedRelease>("download_release", { tag }),
  importCustomSetup: (path: string) =>
    invoke<CachedRelease>("import_custom_setup", { path }),
  removeCachedRelease: (tag: string) =>
    invoke<void>("remove_cached_release", { tag }),
  updateAll: (ids: string[], tag?: string | null) =>
    invoke<UpdateAllResult>("update_all", { ids, tag: tag ?? null }),

  // Compatibilidade
  getCompatRaw: () => invoke<string>("get_compat_raw"),
  getCompatEntries: () => invoke<CompatEntry[]>("get_compat_entries"),
  saveCompat: (content: string) => invoke<void>("save_compat", { content }),

  // Logs
  getLogs: () => invoke<LogEntry[]>("get_logs"),
  clearLogs: () => invoke<void>("clear_logs"),
};

// Reexport util
export type { AppInfo };
