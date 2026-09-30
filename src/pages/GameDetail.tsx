import { useEffect, useState, type ReactNode } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { openPath, revealItemInDir } from "@tauri-apps/plugin-opener";
import {
  ArrowLeft,
  FileCog,
  FolderOpen,
  Pencil,
  Play,
  RefreshCw,
  ShieldAlert,
  Trash2,
  Wrench,
} from "lucide-react";
import { api } from "../lib/api";
import { useStore } from "../lib/store";
import { apiLabel, sourceLabel } from "../lib/format";
import { CompatBadge, Pill } from "../components/Badge";
import { Modal } from "../components/Modal";

export function GameDetail() {
  const game = useStore((s) => s.selectedGame());
  const setPage = useStore((s) => s.setPage);
  const applyMod = useStore((s) => s.applyMod);
  const removeMod = useStore((s) => s.removeMod);
  const reloadGames = useStore((s) => s.reloadGames);
  const toast = useStore((s) => s.toast);
  const feed = useStore((s) => s.feed);
  const busy = useStore((s) => s.busy);

  const [confirm, setConfirm] = useState<"remove" | null>(null);
  const [repairMsg, setRepairMsg] = useState<string | null>(null);

  useEffect(() => {
    // Reavalia o estado ao abrir a tela.
    reloadGames();
  }, [reloadGames]);

  if (!game) {
    return (
      <div className="flex h-full items-center justify-center text-slate-400">
        Jogo não encontrado.
      </div>
    );
  }

  const latest = feed?.latest;
  const updateAvailable =
    game.modInstalled &&
    latest != null &&
    game.modVersion != null &&
    latest.tag !== game.modVersion;

  const pickExe = async () => {
    const selected = await open({
      multiple: false,
      filters: [{ name: "Executável", extensions: ["exe"] }],
      defaultPath: game.installDir,
      title: "Selecione o executável principal do jogo",
    });
    if (typeof selected === "string") {
      const parent = selected.replace(/[\\/][^\\/]+$/, "");
      const updated = { ...game, exePath: selected, modDir: parent || game.modDir };
      const saved = await api.updateGame(updated);
      await reloadGames();
      toast(`Executável atualizado: ${saved.modDir}`, "success");
    }
  };

  const pickModDir = async () => {
    const selected = await open({
      directory: true,
      multiple: false,
      defaultPath: game.modDir || game.installDir,
      title: "Selecione a pasta onde o mod deve ser instalado",
    });
    if (typeof selected === "string") {
      const updated = { ...game, modDir: selected };
      await api.updateGame(updated);
      await reloadGames();
      toast("Pasta do mod atualizada.", "success");
    }
  };

  const doRepair = async () => {
    try {
      const report = await api.repairMod(game.id);
      setRepairMsg(report.message);
      toast(report.message, report.ok ? "success" : "warn");
    } catch (e) {
      toast(String(e), "error");
    }
  };

  const openFolder = async () => {
    await revealItemInDir(game.exePath || game.modDir || game.installDir);
  };

  const openConfig = async () => {
    try {
      const path = await api.findModConfig(game.id);
      if (path) {
        await openPath(path);
      } else {
        toast("Arquivo de configuração do mod não encontrado. Abrindo a pasta.", "warn");
        await openPath(game.modDir || game.installDir);
      }
    } catch (e) {
      toast(String(e), "error");
    }
  };

  const removeGame = async () => {
    await api.removeGame(game.id);
    await reloadGames();
    setPage("games");
    toast("Jogo removido da biblioteca.", "success");
  };

  return (
    <div className="flex h-full flex-col overflow-y-auto">
      <div className="relative">
        <div className="absolute inset-0 h-40 overflow-hidden">
          {game.coverUrl && (
            <img
              src={game.coverUrl}
              alt=""
              className="h-full w-full scale-110 object-cover opacity-20 blur-sm"
            />
          )}
          <div className="absolute inset-0 bg-gradient-to-b from-graphite-900/40 to-graphite-900" />
        </div>
        <div className="relative px-6 pt-5">
          <button className="btn-ghost btn-sm mb-4" onClick={() => setPage("games")}>
            <ArrowLeft size={14} /> Voltar
          </button>
          <div className="flex gap-5">
            <div className="hidden h-48 w-36 shrink-0 overflow-hidden rounded-xl2 bg-graphite-950 shadow-card sm:block">
              {game.coverUrl ? (
                <img src={game.coverUrl} alt={game.name} className="h-full w-full object-cover" />
              ) : (
                <div className="flex h-full items-center justify-center text-slate-600">
                  Sem capa
                </div>
              )}
            </div>
            <div className="min-w-0 flex-1">
              <h1 className="text-2xl font-bold text-white">{game.name}</h1>
              <div className="mt-2 flex flex-wrap items-center gap-2">
                <CompatBadge status={game.compat} />
                <Pill>{sourceLabel(game.source)}</Pill>
                <Pill>{apiLabel(game.graphicsApi)}</Pill>
                {game.fsrDetected && <Pill className="!text-status-ok">FSR detectado</Pill>}
                {game.modInstalled && (
                  <Pill className="!text-status-ok">
                    Mod ativo · {game.modVersion?.replace(/^v/, "v")}
                  </Pill>
                )}
              </div>
              {game.compatNote && (
                <p className="mt-3 max-w-2xl text-xs leading-relaxed text-slate-400">
                  {game.compatNote}
                </p>
              )}
            </div>
          </div>
        </div>
      </div>

      {game.antiCheat && (
        <div className="mx-6 mt-5 flex items-start gap-3 rounded-xl2 border border-status-bad/30 bg-status-bad/10 p-3 text-xs text-status-bad">
          <ShieldAlert size={18} className="mt-0.5 shrink-0" />
          <div>
            <p className="font-semibold">Anti-cheat detectado: {game.antiCheat}</p>
            <p className="text-status-bad/80">
              DLLs modificadas podem causar bloqueio/ban. Use o mod apenas em modos offline /
              single-player.
            </p>
          </div>
        </div>
      )}

      {updateAvailable && (
        <div className="mx-6 mt-4 flex items-center justify-between rounded-xl2 border border-amd/40 bg-amd/10 p-3 text-xs text-white">
          <span>
            Nova versão disponível: <b>{latest?.tag}</b> (instalada {game.modVersion})
          </span>
          <button className="btn-primary btn-sm" onClick={() => applyMod(game.id, latest?.tag)}>
            <Play size={13} /> Atualizar
          </button>
        </div>
      )}

      <div className="grid gap-4 px-6 py-5 lg:grid-cols-2">
        <InfoCard title="Instalação">
          <Row label="Pasta do jogo" value={game.installDir} mono onEdit={pickExe} editTitle="Corrigir executável" />
          <Row
            label="Executável"
            value={game.exePath || "Não localizado"}
            mono
            onEdit={pickExe}
            editTitle="Escolher executável"
          />
          <Row
            label="Pasta do mod"
            value={game.modDir || "—"}
            mono
            onEdit={pickModDir}
            editTitle="Escolher pasta do mod"
          />
          <Row label="Versão aplicada" value={game.modVersion || "—"} />
          <Row label="Última verificação" value={game.lastChecked || "—"} />
          {game.statusMessage && (
            <p className={`mt-2 text-xs ${game.divergence ? "text-status-warn" : "text-slate-400"}`}>
              {game.statusMessage}
            </p>
          )}
        </InfoCard>

        <InfoCard title="Ações">
          <div className="flex flex-wrap gap-2">
            {game.modInstalled ? (
              <>
                <button className="btn-primary" onClick={() => applyMod(game.id)} disabled={!!busy}>
                  <RefreshCw size={15} /> Reaplicar
                </button>
                <button className="btn-danger" onClick={() => setConfirm("remove")}>
                  <Trash2 size={15} /> Remover mod
                </button>
              </>
            ) : (
              <button className="btn-primary" onClick={() => applyMod(game.id)} disabled={!!busy}>
                <Play size={15} /> Aplicar mod
              </button>
            )}
            <button className="btn-ghost" onClick={doRepair}>
              <Wrench size={15} /> Reparar/Verificar
            </button>
            <button className="btn-ghost" onClick={openFolder}>
              <FolderOpen size={15} /> Abrir pasta
            </button>
            <button className="btn-ghost" onClick={openConfig}>
              <FileCog size={15} /> Config do mod
            </button>
            <button className="btn-ghost" onClick={removeGame}>
              <Trash2 size={15} /> Remover da biblioteca
            </button>
          </div>
          <div className="mt-4 rounded-lg bg-graphite-950/60 p-3 text-[11px] leading-relaxed text-slate-400">
            Fluxo de instalação: snapshot antes → backup dos originais → cópia do setup +{" "}
            <code>nvngx_dlssnr.dll</code> → execução do instalador oficial → snapshot depois →
            manifesto. O desfazer usa apenas o manifesto.
          </div>
        </InfoCard>
      </div>

      <Modal
        open={!!confirm}
        title={`Remover mod de ${game.name}`}
        onClose={() => setConfirm(null)}
        footer={
          <>
            <button className="btn-ghost" onClick={() => setConfirm(null)}>
              Cancelar
            </button>
            <button
              className="btn-danger"
              onClick={async () => {
                setConfirm(null);
                await removeMod(game.id);
              }}
            >
              Remover
            </button>
          </>
        }
      >
        <p>
          Todos os arquivos criados pelo mod serão removidos e os arquivos originais restaurados
          a partir do backup. Nenhum arquivo fora do manifesto será apagado.
        </p>
      </Modal>

      <Modal
        open={!!repairMsg}
        title="Resultado da verificação"
        onClose={() => setRepairMsg(null)}
        footer={
          <button className="btn-primary" onClick={() => setRepairMsg(null)}>
            OK
          </button>
        }
      >
        <p>{repairMsg}</p>
      </Modal>
    </div>
  );
}

function InfoCard({ title, children }: { title: string; children: ReactNode }) {
  return (
    <div className="card p-4">
      <h2 className="mb-3 text-sm font-semibold text-white">{title}</h2>
      <div className="space-y-3">{children}</div>
    </div>
  );
}

function Row({
  label,
  value,
  mono,
  onEdit,
  editTitle,
}: {
  label: string;
  value: string;
  mono?: boolean;
  onEdit?: () => void;
  editTitle?: string;
}) {
  return (
    <div className="flex items-start justify-between gap-3">
      <div className="min-w-0">
        <div className="label mb-0.5">{label}</div>
        <div className={`break-all text-xs text-slate-300 ${mono ? "font-mono" : ""}`}>
          {value}
        </div>
      </div>
      {onEdit && (
        <button className="btn-ghost btn-sm shrink-0" title={editTitle} onClick={onEdit}>
          <Pencil size={13} />
        </button>
      )}
    </div>
  );
}
