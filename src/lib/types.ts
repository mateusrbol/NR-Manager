export type CompatStatus = "compatible" | "probable" | "untested" | "incompatible";
export type GraphicsApi = "dx12" | "vulkan" | "dx11" | "unknown";
export type GameSource = "steam" | "epic" | "gog" | "xbox" | "manual";

export interface Game {
  id: string;
  name: string;
  source: GameSource | string;
  appId?: string | null;
  installDir: string;
  exePath?: string | null;
  modDir?: string | null;
  coverUrl?: string | null;
  graphicsApi: GraphicsApi | string;
  compat: CompatStatus | string;
  compatNote?: string | null;
  antiCheat?: string | null;
  fsrDetected: boolean;
  modInstalled: boolean;
  modVersion?: string | null;
  lastChecked?: string | null;
  statusMessage?: string | null;
  divergence: boolean;
  addedManually: boolean;
}

export interface ReleaseAsset {
  name: string;
  size: number;
  digest?: string | null;
  downloadUrl: string;
  downloadCount: number;
}

export interface Release {
  tag: string;
  name: string;
  body: string;
  publishedAt: string;
  prerelease: boolean;
  draft: boolean;
  htmlUrl: string;
  assets: ReleaseAsset[];
}

export interface CachedRelease {
  tag: string;
  name: string;
  path: string;
  size: number;
  sha256?: string | null;
  source: string;
  downloadedAt: string;
  body?: string | null;
  publishedAt?: string | null;
}

export interface ReleaseFeed {
  releases: Release[];
  latest?: Release | null;
  origin: string;
  message?: string | null;
}

export interface Settings {
  repoUrl: string;
  cacheDir: string;
  dlssnrDllPath: string;
  checkUpdatesOnStart: boolean;
  startWithWindows: boolean;
  theme: string;
  pinnedVersion?: string | null;
  customSetupPath: string;
  setupSilentArgs: string;
  largeFileCapMb: number;
  language: string;
}

export interface GpuInfo {
  name: string;
  driverVersion: string;
  family: string;
  supported: boolean;
  message: string;
}

export interface LogEntry {
  time: string;
  level: string;
  message: string;
}

export interface Progress {
  gameId?: string | null;
  stage: string;
  message: string;
  percent?: number | null;
}

export interface GameStatus {
  gameId: string;
  installed: boolean;
  version?: string | null;
  installedAt?: string | null;
  filesCount: number;
  divergence: boolean;
  gameRunning: boolean;
  manifestPath?: string | null;
  backupDir?: string | null;
  message: string;
}

export interface RepairReport {
  ok: boolean;
  missing: string[];
  modified: string[];
  checked: number;
  message: string;
}

export interface AppInfo {
  appData: string;
  cacheDir: string;
  logFile: string;
  elevated: boolean;
  version: string;
}

export interface UpdateFailure {
  gameId: string;
  name: string;
  message: string;
}

export interface UpdateAllResult {
  updated: Game[];
  failed: UpdateFailure[];
}

export interface CompatEntry {
  name: string;
  aliases: string[];
  appIds: Record<string, string>;
  graphicsApi: string;
  exeSubdir: string;
  exeName: string;
  status: CompatStatus;
  fsr: boolean;
  antiCheat?: string | null;
  notes?: string | null;
}
