import { useMemo, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { FolderPlus, RefreshCw, TriangleAlert, Cpu } from "lucide-react";
import { useStore } from "../lib/store";
import type { Game } from "../lib/types";
import { GameCard } from "../components/GameCard";
import { Modal } from "../components/Modal";

type Filter = "all" | "compat" | "active";

const filters: { id: Filter; label: string }[] = [
  { id: "all", label: "Todos" },
  { id: "compat", label: "Compatíveis" },
  { id: "active", label: "Com mod ativo" },
];

export function GamesPage() {
  const games = useStore((s) => s.games);
  const gpu = useStore((s) => s.gpu);
  const busy = useStore((s) => s.busy);
  const scan = useStore((s) => s.scan);
  const openGame = useStore((s) => s.openGame);
  const applyMod = useStore((s) => s.applyMod);
  const removeMod = useStore((s) => s.removeMod);
  const toast = useStore((s) => s.toast);

  const [filter, setFilter] = useState<Filter>("all");
  const [confirm, setConfirm] = useState<{ game: Game; action: "apply" | "remove" } | null>(
    null
  );

  const visible = useMemo(() => {
    let list = [...games];
    if (filter === "compat") {
      list = list.filter((g) => g.compat === "compatible" || g.compat === "probable");
    } else if (filter === "active") {
      list = list.filter((g) => g.modInstalled);
    }
    return list.sort((a, b) => a.name.localeCompare(b.name, "pt-BR"));
  }, [games, filter]);

  const addManual = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Escolha a pasta do jogo (ou use o modo arquivo para o .exe)",
      });
      if (typeof selected === "string") {
        const { api } = await import("../lib/api");
        await api.addGameManual(selected);
        await useStore.getState().reloadGames();
        toast("Jogo adicionado à biblioteca.", "success");
      }
    } catch (e) {
      toast(String(e), "error");
    }
  };

  const addManualExe = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: "Executável", extensions: ["exe"] }],
        title: "Escolha o executável principal do jogo",
      });
      if (typeof selected === "string") {
        const { api } = await import("../lib/api");
        await api.addGameManual(selected);
        await useStore.getState().reloadGames();
        toast("Jogo adicionado à biblioteca.", "success");
      }
    } catch (e) {
      toast(String(e), "error");
    }
  };

  const doToggle = async () => {
    if (!confirm) return;
    const { game, action } = confirm;
    setConfirm(null);
    if (action === "apply") {
      await applyMod(game.id);
    } else {
      await removeMod(game.id);
    }
  };

  return (
    <div className="flex h-full flex-col">
      <header className="flex items-center justify-between gap-4 px-6 pt-5">
        <div>
          <h1 className="text-xl font-bold text-white">Biblioteca de Jogos</h1>
          <p className="text-xs text-slate-400">
            {games.length} jogo(s) · {games.filter((g) => g.modInstalled).length} com mod ativo
          </p>
        </div>
        <div className="flex items-center gap-2">
          <div className="flex rounded-lg bg-graphite-800/80 p-1 ring-1 ring-white/5">
            {filters.map((f) => (
              <button
                key={f.id}
                onClick={() => setFilter(f.id)}
                className={`rounded-md px-3 py-1.5 text-xs font-medium transition-colors ${
                  filter === f.id ? "bg-amd/90 text-white" : "text-slate-300 hover:text-white"
                }`}
              >
                {f.label}
              </button>
            ))}
          </div>
          <button className="btn-ghost" onClick={addManual} title="Adicionar pasta de jogo">
            <FolderPlus size={16} /> Pasta
          </button>
          <button className="btn-ghost" onClick={addManualExe} title="Adicionar por .exe">
            <FolderPlus size={16} /> .exe
          </button>
          <button className="btn-primary" onClick={scan} disabled={!!busy}>
            <RefreshCw size={16} className={busy ? "animate-spin" : ""} />
            Detectar jogos
          </button>
        </div>
      </header>

      {gpu && !gpu.supported && (
        <div className="mx-6 mt-4 flex items-start gap-3 rounded-xl2 border border-status-warn/30 bg-status-warn/10 p-3 text-xs text-status-warn">
          <Cpu size={18} className="mt-0.5 shrink-0" />
          <div>
            <p className="font-semibold">GPU possivelmente incompatível</p>
            <p className="text-status-warn/80">{gpu.message}</p>
          </div>
        </div>
      )}
      {gpu?.supported && (
        <div className="mx-6 mt-4 flex items-center gap-2 text-[11px] text-slate-500">
          <Cpu size={13} /> {gpu.name} · driver {gpu.driverVersion}
        </div>
      )}

      <div className="flex-1 overflow-y-auto px-6 py-4">
        {visible.length === 0 ? (
          <div className="mt-20 flex flex-col items-center text-center text-slate-500">
            <TriangleAlert size={42} className="mb-3 opacity-50" />
            <p className="text-sm">Nenhum jogo encontrado.</p>
            <p className="mt-1 text-xs">
              Clique em <span className="text-slate-300">Detectar jogos</span> ou adicione uma
              pasta manualmente.
            </p>
          </div>
        ) : (
          <div className="grid grid-cols-2 gap-4 pb-6 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5">
            {visible.map((g) => (
              <GameCard
                key={g.id}
                game={g}
                onOpen={(game) => openGame(game.id)}
                onToggle={(game) =>
                  setConfirm({ game, action: game.modInstalled ? "remove" : "apply" })
                }
              />
            ))}
          </div>
        )}
      </div>

      <Modal
        open={!!confirm}
        title={confirm?.action === "remove" ? "Remover mod" : "Aplicar mod"}
        onClose={() => setConfirm(null)}
        footer={
          <>
            <button className="btn-ghost" onClick={() => setConfirm(null)}>
              Cancelar
            </button>
            <button
              className={confirm?.action === "remove" ? "btn-danger" : "btn-primary"}
              onClick={doToggle}
            >
              {confirm?.action === "remove" ? "Remover" : "Aplicar"}
            </button>
          </>
        }
      >
        {confirm && (
          <div className="space-y-2">
            <p>
              <span className="font-semibold text-white">{confirm.game.name}</span>
            </p>
            {confirm.action === "apply" ? (
              <>
                <p className="text-slate-300">
                  O mod será instalado em:
                </p>
                <p className="break-all rounded-lg bg-graphite-950/70 p-2 text-xs text-slate-400">
                  {confirm.game.modDir || confirm.game.installDir}
                </p>
                <p className="text-xs text-slate-400">
                  O instalador oficial do mod será executado. Siga as instruções na janela que
                  abrir (UAC pode ser solicitado).
                </p>
              </>
            ) : (
              <p className="text-slate-300">
                Todos os arquivos adicionados pelo mod serão removidos e os originais restaurados
                a partir do backup.
              </p>
            )}
          </div>
        )}
      </Modal>
    </div>
  );
}
