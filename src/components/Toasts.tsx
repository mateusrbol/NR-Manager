import { AlertTriangle, CheckCircle2, Info, XCircle, X } from "lucide-react";
import { useStore } from "../lib/store";

const config = {
  info: { icon: Info, color: "text-status-info", ring: "ring-status-info/30" },
  success: { icon: CheckCircle2, color: "text-status-ok", ring: "ring-status-ok/30" },
  warn: { icon: AlertTriangle, color: "text-status-warn", ring: "ring-status-warn/30" },
  error: { icon: XCircle, color: "text-status-bad", ring: "ring-status-bad/30" },
};

export function Toasts() {
  const toasts = useStore((s) => s.toasts);
  const dismiss = useStore((s) => s.dismissToast);

  return (
    <div className="pointer-events-none fixed bottom-4 right-4 z-[60] flex w-80 flex-col gap-2">
      {toasts.map((t) => {
        const c = config[t.type];
        const Icon = c.icon;
        return (
          <div
            key={t.id}
            className={`pointer-events-auto flex items-start gap-3 rounded-xl2 bg-graphite-800/95 p-3 shadow-panel ring-1 ${c.ring} animate-fade-in`}
          >
            <Icon size={18} className={`mt-0.5 shrink-0 ${c.color}`} />
            <p className="flex-1 text-xs leading-relaxed text-slate-200">{t.message}</p>
            <button className="text-slate-500 hover:text-white" onClick={() => dismiss(t.id)}>
              <X size={14} />
            </button>
          </div>
        );
      })}
    </div>
  );
}
