// The documentation's Markdown dialect (see content/types.ts) as HTML.
import { href } from "./router";

const esc = (s: string) => s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");

export function inline(s: string): string {
  const codes: string[] = [];
  // code spans first: nothing inside them is Markdown
  let out = s.replace(/(`+)([\s\S]*?[^`])\1(?!`)/g, (_, _t, c) => { codes.push(`<code>${esc(c.trim() === "" ? c : c.replace(/^ (.*) $/, "$1"))}</code>`); return `\u0000${codes.length - 1}\u0000`; });
  out = esc(out);
  out = out.replace(/\[([^\]]+)\]\(([^)\s]+)\)/g, (_, t, url) => {
    const external = /^https?:/.test(url);
    const target = external ? url : href(url);
    return `<a href="${target}"${external ? ' target="_blank" rel="noopener"' : ""}>${t}</a>`;
  });
  out = out.replace(/\*\*([^*]+)\*\*/g, "<strong>$1</strong>");
  out = out.replace(/(^|[^*\w])\*([^*\s][^*]*?)\*(?!\w)/g, "$1<em>$2</em>");
  out = out.replace(/ → /g, " → ");
  return out.replace(/\u0000(\d+)\u0000/g, (_, i) => codes[+i]);
}

export function md(src: string): string {
  const lines = src.replace(/\r/g, "").split("\n");
  const out: string[] = [];
  let i = 0;
  while (i < lines.length) {
    const line = lines[i];
    if (!line.trim()) { i++; continue; }
    const h = line.match(/^(#{2,4})\s+(.*)$/);
    if (h) { const lvl = Math.min(h[1].length + 0, 4); out.push(`<h${lvl} id="${slugify(h[2])}">${inline(h[2])}</h${lvl}>`); i++; continue; }
    if (/^\s*[-*] /.test(line) || /^\s*\d+\. /.test(line)) {
      const ordered = /^\s*\d+\. /.test(line);
      const items: string[] = [];
      while (i < lines.length && (/^\s*[-*] /.test(lines[i]) || /^\s*\d+\. /.test(lines[i]) || (/^\s{2,}\S/.test(lines[i]) && items.length))) {
        if (/^\s{2,}\S/.test(lines[i]) && !/^\s*([-*]|\d+\.) /.test(lines[i])) items[items.length - 1] += " " + lines[i].trim();
        else items.push(lines[i].replace(/^\s*([-*]|\d+\.) /, ""));
        i++;
      }
      out.push(`<${ordered ? "ol" : "ul"}>${items.map(x => `<li>${inline(x)}</li>`).join("")}</${ordered ? "ol" : "ul"}>`);
      continue;
    }
    const para: string[] = [];
    while (i < lines.length && lines[i].trim() && !/^(#{2,4})\s/.test(lines[i]) && !/^\s*([-*]|\d+\.) /.test(lines[i])) para.push(lines[i++]);
    out.push(`<p>${inline(para.join(" "))}</p>`);
  }
  return out.join("\n");
}

export function slugify(s: string): string {
  return s.toLowerCase().replace(/<[^>]+>/g, "").replace(/`/g, "").replace(/[^\p{L}\p{N}]+/gu, "-").replace(/^-|-$/g, "");
}

/** Plain text of a md string, for search and descriptions. */
export function plainText(s: string): string {
  return s.replace(/`([^`]*)`/g, "$1").replace(/\[([^\]]+)\]\([^)]+\)/g, "$1").replace(/[*_#]/g, "");
}
