import { useEffect, useState, type ElementType, type ReactNode } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { openPath, revealItemInDir } from "@tauri-apps/plugin-opener";
import {
  AlertTriangle,
  Cpu,
  FileJson,
  FolderOpen,
  HardDrive,
  MonitorCog,
  RefreshCw,
  Save,
  ShieldCheck,
  Upload,
} from "lucide-react";
import { api } from "../lib/api";
import { useStore } from "../lib/store";
import type { Settings } from "../lib/types";
import { Toggle } from "../components/Toggle";
import { Modal } from "../components/Modal";

export function SettingsPage() {
  const settings = useStore((s) => s.settings);
  const appInfo = useStore((s) => s.appInfo);
  const gpu = useStore((s) => s.gpu);
  const feed = useStore((s) => s.feed);
  const saveSettings = useStore((s) => s.saveSettings);
  const toast = useStore((s) => s.toast);
  const reloadCached = useStore((s) => s.reloadCached);

  const [draft, setDraft] = useState<Settings | null>(settings);
  const [compatOpen, setCompatOpen] = useState(false);
  const [compatText, setCompatText] = useState("");

  useEffect(() => {
    setDraft(settings);
  }, [settings]);

  if (!draft) {
    return <div className="p-6 text-slate-400">Carregando configurações...</div>;
  }

  const update = <K extends keyof Settings>(key: K, value: Settings[K]) =>
    setDraft({ ...draft, [key]: value });

  const pickFolder = async () => {
    const dir = await open({ directory: true, title: "Escolha a pasta de cache" });
    if (typeof dir === "string") update("cacheDir", dir);
  };

  const pickDll = async () => {
    const file = await open({
      multiple: false,
      filters: [{ name: "DLL", extensions: ["dll"] }],
      title: "Selecione o nvngx_dlssnr.dll (build 310.8.0.0)",
    });
    if (typeof file === "string") update("dlssnrDllPath", file);
  };

  const pickCustomSetup = async () => {
    const file = await open({
      multiple: false,
      filters: [{ name: "Instalador", extensions: ["exe", "zip", "7z", "rar"] }],
      title: "Selecione o setup local",
    });
    if (typeof file === "string") {
      update("customSetupPath", file);
      try {
        await api.importCustomSetup(file);
        await reloadCached();
        toast("Setup local importado para o cache.", "success");
      } catch (e) {
        toast(String(e), "warn");
      }
    }
  };

  const openCompatEditor = async () => {
    const text = await api.getCompatRaw();
    setCompatText(text);
    setCompatOpen(true);
  };

  const saveCompatEditor = async () => {
    try {
      await api.saveCompat(compatText);
      setCompatOpen(false);
      toast("compat.json salvo.", "success");
      await useStore.getState().refreshCompat();
    } catch (e) {
      toast(String(e), "error");
    }
  };

  return (
    <div className="flex h-full flex-col overflow-y-auto px-6 py-5">
      <header className="mb-5 flex items-center justify-between">
        <div>
          <h1 className="text-xl font-bold text-white">Configurações</h1>
          <p className="text-xs text-slate-400">
            Versão {appInfo?.version} · {appInfo?.elevated ? "executando como administrador" : "sem elevação"}
          </p>
        </div>
        <button className="btn-primary" onClick={() => saveSettings(draft)}>
          <Save size={16} /> Salvar
        </button>
      </header>

      <div className="grid gap-5 lg:grid-cols-2">
        <Section title="Mod e atualizações" icon={RefreshCw}>
          <Field label="URL do repositório (GitHub)">
            <input
              className="input"
              value={draft.repoUrl}
              onChange={(e) => update("repoUrl", e.target.value)}
            />
          </Field>
          <Field label="Versão fixada (rollback)">
            <select
              className="input"
              value={draft.pinnedVersion || ""}
              onChange={(e) => update("pinnedVersion", e.target.value || null)}
            >
              <option value="">Automática (última release)</option>
              {(feed?.releases || []).map((r) => (
                <option key={r.tag} value={r.tag}>
                  {r.tag} — {r.name}
                </option>
              ))}
            </select>
          </Field>
          <Field label="Setup local (build do Discord)">
            <div className="flex gap-2">
              <input
                className="input"
                placeholder="Nenhum"
                value={draft.customSetupPath}
                onChange={(e) => update("customSetupPath", e.target.value)}
              />
              <button className="btn-ghost shrink-0" onClick={pickCustomSetup}>
                <Upload size={14} />
              </button>
            </div>
          </Field>
          <Field label="Argumentos silenciosos do setup (opcional)">
            <input
              className="input"
              placeholder="Ex.: /S (deixe vazio para modo interativo)"
              value={draft.setupSilentArgs}
              onChange={(e) => update("setupSilentArgs", e.target.value)}
            />
          </Field>
          <ToggleRow
            label="Verificar atualizações ao iniciar"
            checked={draft.checkUpdatesOnStart}
            onChange={(v) => update("checkUpdatesOnStart", v)}
          />
          <ToggleRow
            label="Iniciar com o Windows"
            checked={draft.startWithWindows}
            onChange={(v) => {
              update("startWithWindows", v);
              api.setStartWithWindows(v).catch((e) => toast(String(e), "error"));
            }}
          />
        </Section>

        <Section title="Arquivos e cache" icon={HardDrive}>
          <Field label="nvngx_dlssnr.dll (seu arquivo)">
            <div className="flex gap-2">
              <input
                className="input"
                placeholder="Selecione o DLL (build 310.8.0.0)"
                value={draft.dlssnrDllPath}
                onChange={(e) => update("dlssnrDllPath", e.target.value)}
              />
              <button className="btn-ghost shrink-0" onClick={pickDll}>
                <FolderOpen size={14} />
              </button>
            </div>
            <p className="mt-1 text-[11px] text-slate-500">
              O NR Manager nunca baixa nem redistribui este arquivo. Você o obtém de um jogo com
              DLSS 5.
            </p>
          </Field>
          <Field label="Pasta de cache">
            <div className="flex gap-2">
              <input
                className="input"
                placeholder={appInfo?.cacheDir}
                value={draft.cacheDir}
                onChange={(e) => update("cacheDir", e.target.value)}
              />
              <button className="btn-ghost shrink-0" onClick={pickFolder}>
                <FolderOpen size={14} />
              </button>
            </div>
          </Field>
          <Field label={`Limite de arquivo para hash/backup (${draft.largeFileCapMb} MB)`}>
            <input
              type="range"
              min={64}
              max={2048}
              step={64}
              value={draft.largeFileCapMb}
              onChange={(e) => update("largeFileCapMb", Number(e.target.value))}
              className="w-full accent-amd"
            />
            <p className="mt-1 text-[11px] text-slate-500">
              Arquivos acima deste tamanho não recebem hash nem backup (evita copiar arquivos de
              dados gigantes, como em GTA V).
            </p>
          </Field>
          <div className="rounded-lg bg-graphite-950/50 p-3 text-[11px] text-slate-400">
            <div className="mb-1 font-medium text-slate-300">Dados do app</div>
            <div className="break-all">AppData: {appInfo?.appData}</div>
            <div className="break-all">Cache: {appInfo?.cacheDir}</div>
            <div className="break-all">Log: {appInfo?.logFile}</div>
            <div className="mt-2 flex gap-2">
              <button
                className="btn-ghost btn-sm"
                onClick={() => appInfo && openPath(appInfo.appData)}
              >
                Abrir AppData
              </button>
              <button
                className="btn-ghost btn-sm"
                onClick={() => appInfo && revealItemInDir(appInfo.logFile)}
              >
                Ver log
              </button>
            </div>
          </div>
        </Section>

        <Section title="GPU e ambiente" icon={MonitorCog}>
          <div className="rounded-lg bg-graphite-950/50 p-3 text-xs">
            <div className="flex items-center gap-2">
              <Cpu size={16} className={gpu?.supported ? "text-status-ok" : "text-status-warn"} />
              <span className="font-semibold text-slate-100">{gpu?.name}</span>
            </div>
            <div className="mt-1 text-slate-400">Driver: {gpu?.driverVersion || "—"}</div>
            <div className="mt-1 text-slate-400">{gpu?.message}</div>
          </div>
          {!appInfo?.elevated && (
            <div className="rounded-lg border border-status-warn/30 bg-status-warn/10 p-3 text-xs text-status-warn">
              <div className="flex items-center gap-2 font-semibold">
                <AlertTriangle size={14} /> Sem privilégios de administrador
              </div>
              <p className="mt-1 text-status-warn/80">
                Alguns jogos ficam em pastas protegidas. Reinicie o app elevado se a instalação
                falhar.
              </p>
              <button
                className="btn-ghost btn-sm mt-2"
                onClick={() => api.restartElevated().catch((e) => toast(String(e), "error"))}
              >
                <ShieldCheck size={13} /> Reiniciar como administrador
              </button>
            </div>
          )}
          <button className="btn-ghost" onClick={openCompatEditor}>
            <FileJson size={15} /> Editar compat.json
          </button>
        </Section>
      </div>

      <Modal
        open={compatOpen}
        title="compat.json"
        width="max-w-3xl"
        onClose={() => setCompatOpen(false)}
        footer={
          <>
            <button className="btn-ghost" onClick={() => setCompatOpen(false)}>
              Cancelar
            </button>
            <button className="btn-primary" onClick={saveCompatEditor}>
              Salvar compat.json
            </button>
          </>
        }
      >
        <p className="mb-2 text-xs text-slate-400">
          Lista editável de compatibilidade. Campos: name, aliases, appIds, graphicsApi, exeSubdir,
          exeName, status, fsr, antiCheat, notes.
        </p>
        <textarea
          className="input h-80 font-mono text-xs"
          value={compatText}
          onChange={(e) => setCompatText(e.target.value)}
          spellCheck={false}
        />
      </Modal>
    </div>
  );
}

function Section({
  title,
  icon: Icon,
  children,
}: {
  title: string;
  icon: ElementType;
  children: ReactNode;
}) {
  return (
    <div className="card p-5">
      <h2 className="mb-4 flex items-center gap-2 text-sm font-semibold text-white">
        <Icon size={16} className="text-amd" /> {title}
      </h2>
      <div className="space-y-4">{children}</div>
    </div>
  );
}

function Field({ label, children }: { label: string; children: ReactNode }) {
  return (
    <div>
      <div className="label mb-1.5">{label}</div>
      {children}
    </div>
  );
}

function ToggleRow({
  label,
  checked,
  onChange,
}: {
  label: string;
  checked: boolean;
  onChange: (v: boolean) => void;
}) {
  return (
    <div className="flex items-center justify-between rounded-lg bg-graphite-950/40 px-3 py-2">
      <span className="text-sm text-slate-200">{label}</span>
      <Toggle checked={checked} onChange={onChange} />
    </div>
  );
}
