import type { ElementType, ReactNode } from "react";
import {
  AlertTriangle,
  BookOpen,
  CheckCircle2,
  Cpu,
  FolderSearch,
  Gamepad2,
  Lightbulb,
  PlayCircle,
  ShieldAlert,
} from "lucide-react";
import { useStore } from "../lib/store";

export function TutorialPage() {
  const setPage = useStore((s) => s.setPage);

  return (
    <div className="flex h-full flex-col overflow-y-auto px-6 py-5">
      <header className="mb-5">
        <h1 className="flex items-center gap-2 text-xl font-bold text-white">
          <BookOpen size={22} className="text-amd" /> Como usar o mod
        </h1>
        <p className="text-xs text-slate-400">
          Guia rápido e simples. Em poucos minutos você aplica o DLSS 5 Neural Rendering nos seus
          jogos pela placa AMD.
        </p>
      </header>

      {/* O que você precisa */}
      <div className="card mb-5 p-5">
        <h2 className="mb-4 flex items-center gap-2 text-sm font-semibold text-white">
          <CheckCircle2 size={16} className="text-amd" /> Antes de começar (o que você precisa)
        </h2>
        <div className="grid gap-3 sm:grid-cols-2">
          <Requirement
            icon={Cpu}
            title="Placa AMD RDNA3 ou RDNA4"
            text="Ex.: RX 7000 / RX 9000 + driver Adrenalin 26.1.1 ou mais novo. O jogo precisa ser DirectX 12 e ter opção de FSR."
          />
          <Requirement
            icon={FolderSearch}
            title="O arquivo nvngx_dlssnr.dll"
            text="Você o obtém de um jogo com DLSS 5. O app não baixa nem redistribui esse arquivo."
          />
          <Requirement
            icon={Gamepad2}
            title="Jogo instalado e fechado"
            text="O jogo precisa estar fechado durante a instalação (senão os arquivos ficam em uso)."
          />
          <Requirement
            icon={ShieldAlert}
            title="Apenas jogos offline / single-player"
            text="Anti-cheat (EAC, BattlEye, Vanguard) pode bloquear DLLs modificadas."
          />
        </div>
      </div>

      {/* Passo a passo */}
      <div className="card mb-5 p-5">
        <h2 className="mb-4 flex items-center gap-2 text-sm font-semibold text-white">
          <PlayCircle size={16} className="text-amd" /> Passo a passo
        </h2>
        <div className="space-y-4">
          <Step
            n={1}
            title="Detecte seus jogos"
            text={
              <>
                Abra a aba{" "}
                <PageLink label="Jogos" onClick={() => setPage("games")} /> e clique em{" "}
                <b>Detectar jogos</b>. O app encontra automaticamente Steam, Epic, GOG e Xbox. Também
                dá para adicionar um jogo manualmente por pasta ou <code>.exe</code>.
              </>
            }
          />
          <Step
            n={2}
            title="Informe o arquivo nvngx_dlssnr.dll"
            text={
              <>
                Vá em{" "}
                <PageLink label="Configurações" onClick={() => setPage("settings")} />, no campo{" "}
                <b>nvngx_dlssnr.dll (seu arquivo)</b>, clique na pasta e selecione o arquivo. Você só
                precisa fazer isso uma vez.
              </>
            }
          />
          <Step
            n={3}
            title="Escolha o jogo e aplique o mod"
            text={
              <>
                Na aba{" "}
                <PageLink label="Jogos" onClick={() => setPage("games")} />, clique no jogo e depois
                em <b>Aplicar mod</b>. O app faz backup dos arquivos, roda o instalador oficial
                (pedindo permissão de administrador) e registra tudo. Aguarde terminar.
              </>
            }
          />
          <Step
            n={4}
            title="Verifique a integridade (opcional)"
            text={
              <>
                Se o jogo for atualizado pela Steam/Epic, o app mostra <b>divergência</b>. Nesse caso
                use <b>Verificar</b> e depois <b>Reaplicar</b> no detalhe do jogo.
              </>
            }
          />
          <Step
            n={5}
            title="Atualize para a versão mais nova"
            text={
              <>
                Na aba{" "}
                <PageLink label="Mod" onClick={() => setPage("mod")} /> você vê a última versão e
                atualiza todos os jogos de uma vez, ou fixa uma versão antiga (rollback).
              </>
            }
          />
          <Step
            n={6}
            title="Remover o mod / restaurar original"
            text={
              <>
                No detalhe do jogo, clique em <b>Remover mod</b>. O app apaga só o que ele instalou e
                restaura os arquivos originais do backup. Nada fora disso é mexido.
              </>
            }
          />
        </div>
      </div>

      {/* Ativar no jogo */}
      <div className="card mb-5 p-5">
        <h2 className="mb-4 flex items-center gap-2 text-sm font-semibold text-white">
          <Gamepad2 size={16} className="text-amd" /> Ativar no jogo (a parte que importa)
        </h2>
        <p className="mb-4 text-xs text-slate-400">
          O mod já foi aplicado pelo NR Manager. Agora é dentro do jogo que ele liga:
        </p>
        <div className="space-y-4">
          <Step
            n={1}
            title="Abra o jogo normalmente"
            text={
              <>
                Abra pelo Steam / Epic / GOG / Xbox como você sempre faz. O mod já está nos arquivos
                do jogo.
              </>
            }
          />
          <Step
            n={2}
            title="Nas opções de gráficos, ATIVE o FSR"
            text={
              <>
                Vá em <b>Configurações → Gráficos / Vídeo</b> e ligue o <b>FSR</b> (FSR 3 ou FSR 4,
                qualquer modo de qualidade ou <b>FSRAA</b>). É o FSR que "alimenta" o mod — se ficar
                desligado, o Neural Rendering não roda. Não procure uma opção "DLSS": o jogo usa o
                FSR dele.
              </>
            }
          />
          <Step
            n={3}
            title="Aperte END para abrir o overlay do mod"
            text={
              <>
                Com o jogo aberto, pressione <Key>End</Key>. Aparece o menu do mod por cima do jogo.
                Aperte <Key>End</Key> de novo para fechar.
              </>
            }
          />
          <Step
            n={4}
            title="Ajuste do jeito que gostar"
            text={
              <>
                Use as <b>setas ← ↑ → ↓</b> para navegar e <b>Enter</b> para alternar as opções.
              </>
            }
          />
        </div>

        <div className="mt-4 rounded-lg bg-graphite-950/50 p-3">
          <div className="mb-2 text-xs font-semibold text-slate-200">
            O que dá pra ajustar no overlay (<Key>End</Key>)
          </div>
          <ul className="space-y-1 text-xs text-slate-400">
            <li>
              <b className="text-slate-300">Mode:</b> inline ou async — usado apenas para{" "}
              <b>photo mode</b>.
            </li>
            <li>
              <b className="text-slate-300">Tone intensity:</b> intensidade de tonalidade.
            </li>
            <li>
              <b className="text-slate-300">Structure intensity:</b> intensidade de estrutura.
            </li>
            <li>
              <b className="text-slate-300">Skin structure:</b> estrutura de pele.
            </li>
          </ul>
          <p className="mt-2 text-[11px] text-slate-500">
            Seleção de modelo está nos planos dos autores do mod.
          </p>
        </div>
      </div>

      {/* Dicas */}
      <div className="card mb-5 p-5">
        <h2 className="mb-4 flex items-center gap-2 text-sm font-semibold text-white">
          <Lightbulb size={16} className="text-amd" /> Dicas para dar certo
        </h2>
        <ul className="space-y-2 text-sm text-slate-300">
          <Bullet>
            Sempre <b>feche o jogo</b> antes de aplicar, remover ou atualizar.
          </Bullet>
          <Bullet>
            Se a instalação falhar, rode o NR Manager <b>como administrador</b> (há um botão em
            Configurações).
          </Bullet>
          <Bullet>
            O app só toca em arquivos que ele registrou no <b>manifesto</b> — o resto fica intocado.
          </Bullet>
          <Bullet>
            Usa um build do Discord? Importe o setup em <b>Mod → Importar setup local</b>.
          </Bullet>
          <Bullet>
            Dúvidas de versão? O selo <b>online</b> ou <b>cache/offline</b> mostra de onde vieram os
            dados.
          </Bullet>
        </ul>
      </div>

      {/* Aviso */}
      <div className="rounded-xl2 border border-status-warn/30 bg-status-warn/10 p-4 text-sm text-status-warn">
        <div className="flex items-center gap-2 font-semibold">
          <AlertTriangle size={16} /> Aviso de segurança
        </div>
        <p className="mt-1 text-status-warn/80">
          DLLs modificadas podem ser bloqueadas por anti-cheat. Use somente em jogos offline /
          single-player e por sua conta e risco. Este é um projeto não oficial, sem vínculo com a
          NVIDIA, AMD ou o autor do mod.
        </p>
      </div>
    </div>
  );
}

function Requirement({
  icon: Icon,
  title,
  text,
}: {
  icon: ElementType;
  title: string;
  text: string;
}) {
  return (
    <div className="flex gap-3 rounded-lg bg-graphite-950/50 p-3">
      <Icon size={18} className="mt-0.5 shrink-0 text-amd" />
      <div>
        <div className="text-sm font-medium text-slate-100">{title}</div>
        <div className="text-xs text-slate-400">{text}</div>
      </div>
    </div>
  );
}

function Step({
  n,
  title,
  text,
}: {
  n: number;
  title: string;
  text: ReactNode;
}) {
  return (
    <div className="flex gap-3">
      <div className="flex h-7 w-7 shrink-0 items-center justify-center rounded-full bg-gradient-to-br from-amd to-amd-dark text-sm font-bold text-white shadow-glow">
        {n}
      </div>
      <div className="min-w-0">
        <div className="text-sm font-semibold text-white">{title}</div>
        <p className="mt-0.5 text-sm leading-relaxed text-slate-400">{text}</p>
      </div>
    </div>
  );
}

function PageLink({ label, onClick }: { label: string; onClick: () => void }) {
  return (
    <button
      className="font-medium text-amd hover:underline"
      onClick={onClick}
    >
      {label}
    </button>
  );
}

function Bullet({ children }: { children: ReactNode }) {
  return (
    <li className="flex gap-2">
      <CheckCircle2 size={15} className="mt-0.5 shrink-0 text-status-ok" />
      <span>{children}</span>
    </li>
  );
}

function Key({ children }: { children: ReactNode }) {
  return (
    <kbd className="rounded border border-white/15 bg-white/10 px-1.5 py-0.5 font-mono text-[11px] text-slate-200">
      {children}
    </kbd>
  );
}
