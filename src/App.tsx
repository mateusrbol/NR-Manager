import { useEffect } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useStore } from "./lib/store";
import type { LogEntry, Progress } from "./lib/types";
import { TitleBar } from "./components/TitleBar";
import { Sidebar } from "./components/Sidebar";
import { Toasts } from "./components/Toasts";
import { ProgressBar } from "./components/ProgressBar";
import { GamesPage } from "./pages/GamesPage";
import { GameDetail } from "./pages/GameDetail";
import { ModPage } from "./pages/ModPage";
import { SettingsPage } from "./pages/SettingsPage";
import { LogsPage } from "./pages/LogsPage";

export default function App() {
  const page = useStore((s) => s.page);
  const loading = useStore((s) => s.loading);
  const init = useStore((s) => s.init);
  const setProgress = useStore((s) => s.setProgress);
  const busy = useStore((s) => s.busy);
  const progress = useStore((s) => s.progress);

  useEffect(() => {
    init();
  }, [init]);

  useEffect(() => {
    const unsubs: UnlistenFn[] = [];
    (async () => {
      unsubs.push(
        await listen<Progress>("nr-progress", (e) => setProgress(e.payload))
      );
      unsubs.push(
        await listen<LogEntry>("nr-log", (e) =>
          useStore.setState((s) => ({
            logs: [...s.logs.slice(-799), e.payload],
          }))
        )
      );
      unsubs.push(
        await listen("nr-library-updated", () => {
          useStore.getState().reloadGames();
        })
      );
    })();
    return () => unsubs.forEach((u) => u());
  }, [setProgress]);

  return (
    <div className="h-screen w-screen p-1.5">
      <div className="app-shell relative">
        <div className="flex min-w-0 flex-1 flex-col">
          <TitleBar />
          <div className="flex min-h-0 flex-1">
            <Sidebar />
            <main className="min-w-0 flex-1 overflow-hidden">
              {loading ? (
                <div className="flex h-full flex-col items-center justify-center gap-3 text-slate-400">
                  <div className="h-8 w-8 animate-spin rounded-full border-2 border-white/10 border-t-amd" />
                  <span className="text-sm">Carregando NR Manager...</span>
                </div>
              ) : page === "games" ? (
                <GamesPage />
              ) : page === "detail" ? (
                <GameDetail />
              ) : page === "mod" ? (
                <ModPage />
              ) : page === "settings" ? (
                <SettingsPage />
              ) : (
                <LogsPage />
              )}
            </main>
          </div>
        </div>

        {busy && progress && (
          <div className="absolute bottom-0 left-0 right-0 border-t border-amd/30 bg-graphite-900/95 px-4 py-2">
            <ProgressBar percent={progress.percent} />
            <p className="mt-1 truncate text-[11px] text-slate-300">{progress.message}</p>
          </div>
        )}
      </div>
      <Toasts />
    </div>
  );
}
