import { useMemo, useState } from "react";
import { openPath } from "@tauri-apps/plugin-opener";
import { RefreshCw, ScrollText, Trash2 } from "lucide-react";
import { api } from "../lib/api";
import { useStore } from "../lib/store";

const levelColor: Record<string, string> = {
  INFO: "text-status-info",
  WARN: "text-status-warn",
  ERROR: "text-status-bad",
};

type Filter = "all" | "INFO" | "WARN" | "ERROR";

export function LogsPage() {
  const logs = useStore((s) => s.logs);
  const appInfo = useStore((s) => s.appInfo);
  const reloadLogs = useStore((s) => s.reloadLogs);
  const toast = useStore((s) => s.toast);
  const [filter, setFilter] = useState<Filter>("all");

  const visible = useMemo(() => {
    const list = filter === "all" ? logs : logs.filter((l) => l.level === filter);
    return [...list].reverse();
  }, [logs, filter]);

  const clear = async () => {
    await api.clearLogs();
    await reloadLogs();
    toast("Logs limpos.", "success");
  };

  return (
    <div className="flex h-full flex-col">
      <header className="flex items-center justify-between px-6 pt-5">
        <div>
          <h1 className="flex items-center gap-2 text-xl font-bold text-white">
            <ScrollText size={20} /> Logs
          </h1>
          <p className="text-xs text-slate-400">
            {logs.length} entrada(s) · {appInfo?.logFile}
          </p>
        </div>
        <div className="flex items-center gap-2">
          <div className="flex rounded-lg bg-graphite-800/80 p-1 ring-1 ring-white/5">
            {(["all", "INFO", "WARN", "ERROR"] as Filter[]).map((f) => (
              <button
                key={f}
                onClick={() => setFilter(f)}
                className={`rounded-md px-3 py-1.5 text-xs font-medium transition-colors ${
                  filter === f ? "bg-amd/90 text-white" : "text-slate-300 hover:text-white"
                }`}
              >
                {f === "all" ? "Todos" : f}
              </button>
            ))}
          </div>
          <button className="btn-ghost" onClick={reloadLogs}>
            <RefreshCw size={15} /> Atualizar
          </button>
          <button
            className="btn-ghost"
            onClick={() => appInfo && openPath(appInfo.logFile)}
            disabled={!appInfo}
          >
            Abrir arquivo
          </button>
          <button className="btn-danger" onClick={clear}>
            <Trash2 size={15} /> Limpar
          </button>
        </div>
      </header>

      <div className="m-6 flex-1 overflow-y-auto rounded-xl2 border border-white/5 bg-graphite-950/60 p-3 font-mono text-xs">
        {visible.length === 0 ? (
          <p className="p-4 text-center text-slate-500">Sem entradas de log.</p>
        ) : (
          visible.map((l, i) => (
            <div key={i} className="flex gap-3 border-b border-white/5 py-1 last:border-0">
              <span className="shrink-0 text-slate-500">{l.time}</span>
              <span className={`w-12 shrink-0 font-semibold ${levelColor[l.level] || "text-slate-400"}`}>
                {l.level}
              </span>
              <span className="whitespace-pre-wrap break-all text-slate-300">{l.message}</span>
            </div>
          ))
        )}
      </div>
    </div>
  );
}
