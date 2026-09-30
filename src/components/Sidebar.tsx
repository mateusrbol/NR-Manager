import type { ElementType } from "react";
import { Gamepad2, PackageSearch, Settings, ScrollText, Zap } from "lucide-react";
import { useStore, type Page } from "../lib/store";

const items: { id: Page; label: string; icon: ElementType }[] = [
  { id: "games", label: "Jogos", icon: Gamepad2 },
  { id: "mod", label: "Mod", icon: PackageSearch },
  { id: "settings", label: "Configurações", icon: Settings },
  { id: "logs", label: "Logs", icon: ScrollText },
];

export function Sidebar() {
  const page = useStore((s) => s.page);
  const setPage = useStore((s) => s.setPage);
  const games = useStore((s) => s.games);
  const feed = useStore((s) => s.feed);

  const activeMods = games.filter((g) => g.modInstalled).length;
  const latest = feed?.latest;

  return (
    <aside className="flex w-60 shrink-0 flex-col border-r border-white/5 bg-graphite-900/60 p-3">
      <div className="mb-4 flex items-center gap-3 rounded-xl2 bg-gradient-to-br from-amd/20 to-transparent p-3 ring-1 ring-white/5">
        <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-gradient-to-br from-amd to-amd-dark shadow-glow">
          <Zap size={20} className="text-white" />
        </div>
        <div className="min-w-0">
          <div className="truncate text-sm font-semibold text-white">NR Manager</div>
          <div className="truncate text-[11px] text-slate-400">
            {latest ? `Última: ${latest.tag}` : "Verificando versões..."}
          </div>
        </div>
      </div>

      <nav className="flex flex-col gap-1">
        {items.map((item) => {
          const Icon = item.icon;
          const active = page === item.id || (page === "detail" && item.id === "games");
          return (
            <div
              key={item.id}
              onClick={() => setPage(item.id)}
              className={`nav-item ${active ? "nav-item-active" : ""}`}
            >
              <Icon size={18} />
              <span className="flex-1">{item.label}</span>
              {item.id === "games" && games.length > 0 && (
                <span className="rounded-full bg-white/10 px-2 py-0.5 text-[10px] text-slate-300">
                  {games.length}
                </span>
              )}
              {item.id === "mod" && activeMods > 0 && (
                <span className="rounded-full bg-status-ok/20 px-2 py-0.5 text-[10px] text-status-ok">
                  {activeMods}
                </span>
              )}
            </div>
          );
        })}
      </nav>

      <div className="mt-auto px-2 py-3 text-[11px] leading-relaxed text-slate-500">
        Projeto não oficial. Não redistribui arquivos do mod nem o{" "}
        <span className="text-slate-400">nvngx_dlssnr.dll</span>.
      </div>
    </aside>
  );
}
