import type { ReactNode } from "react";
import { ShieldCheck, ShieldAlert, ShieldQuestion, ShieldX } from "lucide-react";
import { compatClasses, compatLabel } from "../lib/format";

export function CompatBadge({ status, title }: { status: string; title?: string }) {
  const Icon =
    status === "compatible"
      ? ShieldCheck
      : status === "probable"
      ? ShieldQuestion
      : status === "incompatible"
      ? ShieldX
      : ShieldAlert;
  return (
    <span title={title} className={`badge ${compatClasses(status)}`}>
      <Icon size={12} />
      {compatLabel(status)}
    </span>
  );
}

export function Pill({
  children,
  className = "",
}: {
  children: ReactNode;
  className?: string;
}) {
  return <span className={`badge bg-white/5 text-slate-300 ring-1 ring-white/10 ${className}`}>{children}</span>;
}
