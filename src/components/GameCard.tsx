import { useState } from "react";
import { Gamepad2, TriangleAlert } from "lucide-react";
import type { Game } from "../lib/types";
import { CompatBadge } from "./Badge";
import { Toggle } from "./Toggle";

function Cover({ game }: { game: Game }) {
  const [failed, setFailed] = useState(false);
  const show = game.coverUrl && !failed;
  return (
    <div className="relative aspect-[3/4] w-full overflow-hidden bg-graphite-950">
      {show ? (
        <img
          src={game.coverUrl as string}
          alt={game.name}
          className="h-full w-full object-cover transition-transform duration-300 group-hover:scale-105"
          onError={() => setFailed(true)}
        />
      ) : (
        <div className="flex h-full w-full flex-col items-center justify-center gap-2 bg-gradient-to-br from-graphite-700 to-graphite-950 text-slate-500">
          <Gamepad2 size={40} />
          <span className="px-3 text-center text-xs">{game.name}</span>
        </div>
      )}
      <div className="absolute inset-0 bg-gradient-to-t from-graphite-950 via-transparent to-transparent" />
      {game.modInstalled && (
        <div className="absolute right-2 top-2 rounded-full bg-status-ok/90 px-2 py-0.5 text-[10px] font-bold text-black shadow">
          MOD ATIVO
        </div>
      )}
      {game.antiCheat && (
        <div
          title={`Anti-cheat: ${game.antiCheat}`}
          className="absolute left-2 top-2 rounded-full bg-status-bad/90 p-1 text-black shadow"
        >
          <TriangleAlert size={14} />
        </div>
      )}
    </div>
  );
}

export function GameCard({
  game,
  onOpen,
  onToggle,
}: {
  game: Game;
  onOpen: (g: Game) => void;
  onToggle: (g: Game) => void;
}) {
  return (
    <div className="game-card group" onClick={() => onOpen(game)}>
      <Cover game={game} />
      <div className="p-3">
        <div className="mb-2 min-h-[2.5rem]">
          <h3 className="line-clamp-2 text-sm font-semibold text-white" title={game.name}>
            {game.name}
          </h3>
        </div>
        <div className="mb-3 flex items-center justify-between">
          <CompatBadge status={game.compat} title={game.compatNote || undefined} />
          <Toggle
            checked={game.modInstalled}
            onChange={() => onToggle(game)}
            title={game.modInstalled ? "Remover mod" : "Aplicar mod"}
          />
        </div>
        <div className="flex items-center justify-between text-[11px] text-slate-400">
          <span className="truncate">
            {game.modVersion ? `v${game.modVersion.replace(/^v/, "")}` : "—"}
          </span>
          <span className="truncate">{game.graphicsApi.toUpperCase()}</span>
        </div>
        {game.divergence && (
          <p className="mt-2 text-[11px] font-medium text-status-warn">
            Mod alterado por atualização do jogo
          </p>
        )}
      </div>
    </div>
  );
}
