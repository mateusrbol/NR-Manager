import type { CompatStatus, GraphicsApi } from "./types";

export function compatLabel(status: string): string {
  switch (status as CompatStatus) {
    case "compatible":
      return "Compatível";
    case "probable":
      return "Provável";
    case "incompatible":
      return "Incompatível";
    default:
      return "Não testado";
  }
}

export function compatClasses(status: string): string {
  switch (status as CompatStatus) {
    case "compatible":
      return "bg-status-ok/15 text-status-ok ring-1 ring-status-ok/30";
    case "probable":
      return "bg-status-warn/15 text-status-warn ring-1 ring-status-warn/30";
    case "incompatible":
      return "bg-status-bad/15 text-status-bad ring-1 ring-status-bad/30";
    default:
      return "bg-white/5 text-slate-400 ring-1 ring-white/10";
  }
}

export function apiLabel(api: string): string {
  switch (api as GraphicsApi) {
    case "dx12":
      return "DirectX 12";
    case "vulkan":
      return "Vulkan";
    case "dx11":
      return "DirectX 11";
    default:
      return "API desconhecida";
  }
}

export function sourceLabel(source: string): string {
  switch (source) {
    case "steam":
      return "Steam";
    case "epic":
      return "Epic Games";
    case "gog":
      return "GOG";
    case "xbox":
      return "Xbox / Game Pass";
    case "manual":
      return "Manual";
    default:
      return source;
  }
}

export function formatDate(iso?: string | null): string {
  if (!iso) return "—";
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return iso;
  return d.toLocaleDateString("pt-BR", {
    day: "2-digit",
    month: "short",
    year: "numeric",
  });
}

export function formatBytes(bytes: number): string {
  const units = ["B", "KB", "MB", "GB", "TB"];
  let v = bytes;
  let i = 0;
  while (v >= 1024 && i < units.length - 1) {
    v /= 1024;
    i++;
  }
  return i === 0 ? `${bytes} B` : `${v.toFixed(1)} ${units[i]}`;
}

/** Converte o markdown simples do changelog em HTML seguro. */
export function changelogHtml(markdown: string): string {
  const esc = markdown
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");

  const lines = esc.split(/\r?\n/);
  const html: string[] = [];
  let inList = false;

  const inline = (s: string) =>
    s
      .replace(/\*\*(.+?)\*\*/g, "<strong>$1</strong>")
      .replace(/`([^`]+)`/g, "<code>$1</code>")
      .replace(/\*(.+?)\*/g, "<em>$1</em>");

  for (const raw of lines) {
    const line = raw.trimEnd();
    const heading = /^(#{1,4})\s+(.*)$/.exec(line);
    const bullet = /^[-*]\s+(.*)$/.exec(line);

    if (bullet) {
      if (!inList) {
        html.push('<ul class="list-disc pl-5 space-y-1 my-2">');
        inList = true;
      }
      html.push(`<li>${inline(bullet[1])}</li>`);
      continue;
    }
    if (inList) {
      html.push("</ul>");
      inList = false;
    }
    if (heading) {
      const level = Math.min(heading[1].length + 2, 5);
      html.push(`<h${level} class="font-semibold mt-3 mb-1">${inline(heading[2])}</h${level}>`);
    } else if (line.trim() === "") {
      html.push("");
    } else {
      html.push(`<p class="my-1">${inline(line)}</p>`);
    }
  }
  if (inList) html.push("</ul>");
  return html.join("\n");
}
