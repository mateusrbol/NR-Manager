export function ProgressBar({
  percent,
  indeterminate,
}: {
  percent?: number | null;
  indeterminate?: boolean;
}) {
  return (
    <div className="h-2 w-full overflow-hidden rounded-full bg-graphite-950/80 ring-1 ring-white/5">
      {indeterminate || percent == null ? (
        <div className="h-full w-1/3 animate-[fade-in_0.3s_ease-out] rounded-full bg-gradient-to-r from-amd to-amd-light"
          style={{ animation: "slide 1.2s ease-in-out infinite" }}
        />
      ) : (
        <div
          className="h-full rounded-full bg-gradient-to-r from-amd to-amd-light transition-all duration-300"
          style={{ width: `${Math.min(100, Math.max(0, percent))}%` }}
        />
      )}
      <style>{`@keyframes slide { 0% { transform: translateX(-120%); } 100% { transform: translateX(320%); } }`}</style>
    </div>
  );
}
