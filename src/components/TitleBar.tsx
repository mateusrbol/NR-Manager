import { getCurrentWindow } from "@tauri-apps/api/window";
import { Minus, Square, X } from "lucide-react";

export function TitleBar() {
  const win = getCurrentWindow();
  return (
    <div
      data-tauri-drag-region
      className="flex h-10 shrink-0 select-none items-center justify-between border-b border-white/5 bg-graphite-900/80 px-3"
    >
      <div data-tauri-drag-region className="flex items-center gap-2">
        <span className="h-4 w-4 rounded bg-gradient-to-br from-amd to-amd-dark" />
        <span className="text-sm font-semibold tracking-wide text-slate-200">
          NR Manager
        </span>
        <span className="badge ml-1 bg-white/5 text-[10px] text-slate-400">
          DLSS-NR on AMD
        </span>
      </div>
      <div className="flex items-center">
        <button className="win-btn" title="Minimizar" onClick={() => win.minimize()}>
          <Minus size={15} />
        </button>
        <button className="win-btn" title="Maximizar" onClick={() => win.toggleMaximize()}>
          <Square size={12} />
        </button>
        <button
          className="win-btn win-btn-close"
          title="Fechar"
          onClick={() => win.close()}
        >
          <X size={15} />
        </button>
      </div>
    </div>
  );
}
