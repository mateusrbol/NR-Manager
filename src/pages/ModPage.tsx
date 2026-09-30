import { useMemo, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import {
  Archive,
  CheckCircle2,
  Download,
  ExternalLink,
  Package,
  Pin,
  PinOff,
  RefreshCw,
  Trash2,
  Upload,
} from "lucide-react";
import { openUrl } from "@tauri-apps/plugin-opener";
import { api } from "../lib/api";
import { useStore } from "../lib/store";
import { changelogHtml, formatBytes, formatDate } from "../lib/format";
import { ProgressBar } from "../components/ProgressBar";
import { Modal } from "../components/Modal";

export function ModPage() {
  const feed = useStore((s) => s.feed);
  const settings = useStore((s) => s.settings);
  const games = useStore((s) => s.games);
  const cached = useStore((s) => s.cached);
  const progress = useStore((s) => s.progress);
  const busy = useStore((s) => s.busy);
  const loadFeed = useStore((s) => s.loadFeed);
  const updateAll = useStore((s) => s.updateAll);
  const saveSettings = useStore((s) => s.saveSettings);
  const reloadCached = useStore((s) => s.reloadCached);
  const toast = useStore((s) => s.toast);

  const [confirmUpdate, setConfirmUpdate] = useState(false);
  const [selectedTag, setSelectedTag] = useState<string | null>(null);

  const latest = feed?.latest ?? null;
  const installed = useMemo(
    () => games.filter((g) => g.modInstalled),
    [games]
  );
  const outdated = useMemo(
    () => installed.filter((g) => latest && g.modVersion !== latest.tag),
    [installed, latest]
  );

  const targetTag = selectedTag || latest?.tag || null;

  const importCustom = async () => {
    const selected = await open({
      multiple: false,
      filters: [{ name: "Instalador / Zip", extensions: ["exe", "zip", "7z", "rar"] }],
      title: "Importar setup local (build antecipada / Discord)",
    });
    if (typeof selected === "string") {
      try {
        await api.importCustomSetup(selected);
        await reloadCached();
        toast("Setup importado para o cache.", "success");
      } catch (e) {
        toast(String(e), "error");
      }
    }
  };

  const pinVersion = async (tag: string | null) => {
    if (!settings) return;
    const updated = { ...settings, pinnedVersion: tag };
    await saveSettings(updated);
  };

  const activeSettings = settings;

  return (
    <div className="flex h-full flex-col overflow-y-auto px-6 py-5">
      <header className="mb-5 flex items-start justify-between">
        <div>
          <h1 className="text-xl font-bold text-white">Mod · DLSS 5 Neural Rendering on AMD</h1>
          <p className="text-xs text-slate-400">
            Fonte oficial: {" "}
            <button
              className="text-amd hover:underline"
              onClick={() => openUrl(settings?.repoUrl || "")}
            >
              {settings?.repoUrl} <ExternalLink size={10} className="inline" />
            </button>
          </p>
        </div>
        <button className="btn-ghost" onClick={() => loadFeed(true)}>
          <RefreshCw size={15} /> Verificar atualizações
        </button>
      </header>

      {busy && (
        <div className="mb-4 card p-4">
          <div className="mb-2 flex items-center gap-2 text-sm text-white">
            <RefreshCw size={15} className="animate-spin text-amd" />
            {busy}
          </div>
          {progress && (
            <>
              <ProgressBar percent={progress.percent} />
              <p className="mt-2 text-xs text-slate-400">{progress.message}</p>
            </>
          )}
        </div>
      )}

      <div className="grid gap-5 lg:grid-cols-3">
        {/* Card de versão */}
        <div className="card p-5 lg:col-span-2">
          <div className="flex items-start justify-between">
            <div className="flex items-center gap-3">
              <div className="flex h-12 w-12 items-center justify-center rounded-xl2 bg-gradient-to-br from-amd to-amd-dark shadow-glow">
                <Package size={24} className="text-white" />
              </div>
              <div>
                <div className="text-xs uppercase tracking-wider text-slate-400">
                  Última versão
                </div>
                <div className="text-2xl font-bold text-white">
                  {latest ? latest.name || latest.tag : "—"}
                </div>
                <div className="text-[11px] text-slate-500">
                  {latest ? `${latest.tag} · ${formatDate(latest.publishedAt)}` : "Sem dados"}
                </div>
              </div>
            </div>
            {latest && feed?.origin === "live" && (
              <span className="badge bg-status-ok/15 text-status-ok">
                <CheckCircle2 size={12} /> online
              </span>
            )}
            {feed?.origin === "cache" && (
              <span className="badge bg-status-warn/15 text-status-warn">cache/offline</span>
            )}
          </div>

          <div className="mt-4 grid grid-cols-2 gap-3 sm:grid-cols-3">
            <Stat label="Jogos com mod" value={String(installed.length)} />
            <Stat label="Desatualizados" value={String(outdated.length)} />
            <Stat
              label="Versão fixada"
              value={settings?.pinnedVersion ? settings.pinnedVersion : "Automática"}
            />
          </div>

          {latest && (
            <div className="mt-4 flex flex-wrap items-center gap-2">
              <button
                className="btn-primary"
                disabled={!!busy || outdated.length === 0}
                onClick={() => setConfirmUpdate(true)}
              >
                <Download size={15} />
                Atualizar {outdated.length} jogo(s) para {targetTag}
              </button>
              <button
                className="btn-ghost"
                onClick={() => latest.htmlUrl && openUrl(latest.htmlUrl)}
              >
                <ExternalLink size={14} /> Abrir release
              </button>
            </div>
          )}
        </div>

        {/* Releases em cache */}
        <div className="card p-5">
          <h2 className="mb-3 flex items-center gap-2 text-sm font-semibold text-white">
            <Archive size={16} /> Setups em cache
          </h2>
          <button className="btn-ghost mb-3 w-full" onClick={importCustom}>
            <Upload size={15} /> Importar setup local
          </button>
          {cached.length === 0 ? (
            <p className="text-xs text-slate-500">Nenhum setup baixado ainda.</p>
          ) : (
            <ul className="space-y-2">
              {cached.map((c) => (
                <li
                  key={c.tag}
                  className="flex items-center justify-between gap-2 rounded-lg bg-graphite-950/50 p-2 text-xs"
                >
                  <div className="min-w-0">
                    <div className="truncate font-medium text-slate-200">{c.tag}</div>
                    <div className="text-[10px] text-slate-500">
                      {formatBytes(c.size)} · {c.source === "custom" ? "personalizada" : "GitHub"}
                    </div>
                  </div>
                  <button
                    className="win-btn h-7 w-7"
                    title="Remover do cache"
                    onClick={async () => {
                      await api.removeCachedRelease(c.tag);
                      await reloadCached();
                    }}
                  >
                    <Trash2 size={13} />
                  </button>
                </li>
              ))}
            </ul>
          )}
        </div>
      </div>

      {/* Changelog */}
      {latest?.body && (
        <div className="card mt-5 p-5">
          <h2 className="mb-2 text-sm font-semibold text-white">
            Changelog · {latest.tag}
          </h2>
          <div
            className="text-xs leading-relaxed text-slate-300"
            dangerouslySetInnerHTML={{ __html: changelogHtml(latest.body) }}
          />
        </div>
      )}

      {/* Histórico */}
      <div className="card mt-5 p-5">
        <h2 className="mb-3 text-sm font-semibold text-white">Histórico de versões</h2>
        <div className="space-y-2">
          {(feed?.releases || []).map((r) => {
            const isPinned = activeSettings?.pinnedVersion === r.tag;
            const isLatest = latest?.tag === r.tag;
            return (
              <div
                key={r.tag}
                className="flex items-center justify-between gap-3 rounded-lg bg-graphite-950/40 p-3"
              >
                <div className="min-w-0">
                  <div className="flex items-center gap-2">
                    <span className="text-sm font-medium text-slate-100">{r.tag}</span>
                    {isLatest && (
                      <span className="badge bg-amd/20 text-amd-light">última</span>
                    )}
                    {r.prerelease && (
                      <span className="badge bg-white/5 text-slate-400">pré-release</span>
                    )}
                    {isPinned && (
                      <span className="badge bg-status-info/15 text-status-info">fixada</span>
                    )}
                  </div>
                  <div className="text-[11px] text-slate-500">{formatDate(r.publishedAt)}</div>
                </div>
                <div className="flex shrink-0 items-center gap-1">
                  <button
                    className="btn-ghost btn-sm"
                    title="Baixar para o cache"
                    onClick={async () => {
                      try {
                        await api.downloadRelease(r.tag);
                        await reloadCached();
                        toast(`${r.tag} baixado.`, "success");
                      } catch (e) {
                        toast(String(e), "error");
                      }
                    }}
                  >
                    <Download size={13} />
                  </button>
                  <button
                    className="btn-ghost btn-sm"
                    title={isPinned ? "Desafixar" : "Fixar esta versão (rollback)"}
                    onClick={() => pinVersion(isPinned ? null : r.tag)}
                  >
                    {isPinned ? <PinOff size={13} /> : <Pin size={13} />}
                  </button>
                  <button
                    className="btn-ghost btn-sm"
                    title="Selecionar como alvo de atualização"
                    onClick={() => setSelectedTag(r.tag)}
                  >
                    <CheckCircle2
                      size={13}
                      className={selectedTag === r.tag ? "text-status-ok" : ""}
                    />
                  </button>
                </div>
              </div>
            );
          })}
        </div>
      </div>

      <Modal
        open={confirmUpdate}
        title="Atualizar jogos com o mod"
        onClose={() => setConfirmUpdate(false)}
        footer={
          <>
            <button className="btn-ghost" onClick={() => setConfirmUpdate(false)}>
              Cancelar
            </button>
            <button
              className="btn-primary"
              onClick={() => {
                setConfirmUpdate(false);
                updateAll(
                  outdated.map((g) => g.id),
                  targetTag
                );
              }}
            >
              Atualizar {outdated.length} jogo(s)
            </button>
          </>
        }
      >
        <p>
          A versão <b>{targetTag}</b> será baixada e reaplicada nos jogos que já usam o mod. Cada
          jogo deve estar fechado. O backup anterior é substituído de forma segura.
        </p>
        <ul className="mt-2 list-disc pl-5 text-xs text-slate-400">
          {outdated.map((g) => (
            <li key={g.id}>
              {g.name} ({g.modVersion} → {targetTag})
            </li>
          ))}
        </ul>
      </Modal>
    </div>
  );
}

function Stat({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-xl2 bg-graphite-950/50 p-3">
      <div className="text-[10px] uppercase tracking-wider text-slate-500">{label}</div>
      <div className="mt-1 truncate text-sm font-semibold text-slate-100">{value}</div>
    </div>
  );
}
