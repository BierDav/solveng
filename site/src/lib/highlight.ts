// Typst source with solveng in mind, as highlighted HTML: markup, code after #, math between $ – where :=, => and ->
// stand out in their colours.
const esc = (s: string) => s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
const span = (cls: string, s: string) => `<span class="tk-${cls}">${esc(s)}</span>`;

const KEYWORDS = new Set(["import", "let", "show", "set", "context", "if", "else", "for", "in", "while", "return", "as", "include", "none", "auto", "true", "false"]);

export function highlight(src: string): string {
  let out = "";
  let i = 0;
  const n = src.length;
  const at = (s: string) => src.startsWith(s, i);
  let lineStart = true;

  const code = (embedded: boolean) => {
    // one code expression after #: a keyword statement runs to the end of the line
    const m = src.slice(i).match(/^[\p{L}_][\p{L}\p{N}_-]*/u);
    if (m && KEYWORDS.has(m[0]) && ["import", "let", "show", "set", "context"].includes(m[0])) {
      const end = src.indexOf("\n", i);
      const stop = end < 0 ? n : end;
      out += codeRun(src.slice(i, stop));
      i = stop;
      return;
    }
    let depth = 0;
    const start = i;
    while (i < n) {
      const c = src[i];
      if (/[\p{L}\p{N}_.-]/u.test(c) && depth === 0) { i++; continue; }
      if (c === "(") { depth++; i++; continue; }
      if (c === ")" && depth > 0) { depth--; i++; continue; }
      if (depth > 0) {
        if (c === "\n" && !embedded) break;
        if (c === "$" || c === "[") break;
        i++;
        continue;
      }
      break;
    }
    out += codeRun(src.slice(start, i));
  };

  while (i < n) {
    const c = src[i];
    if (at("//")) { const e = src.indexOf("\n", i); const stop = e < 0 ? n : e; out += span("comment", src.slice(i, stop)); i = stop; continue; }
    if (at("/*")) { const e = src.indexOf("*/", i); const stop = e < 0 ? n : e + 2; out += span("comment", src.slice(i, stop)); i = stop; continue; }
    if (c === "\n") { out += "\n"; i++; lineStart = true; continue; }
    if (lineStart && c === "=") {
      const e = src.indexOf("\n", i); const stop = e < 0 ? n : e;
      out += span("heading", src.slice(i, stop)); i = stop; continue;
    }
    lineStart = false;
    if (c === "#" && /[\p{L}_(]/u.test(src[i + 1] ?? "")) { out += span("hash", "#"); i++; code(true); continue; }
    if (c === "$") {
      const e = findMathEnd(src, i + 1);
      out += span("dollar", "$") + mathRun(src.slice(i + 1, e)) + (e < n ? span("dollar", "$") : "");
      i = Math.min(n, e + 1);
      continue;
    }
    if (c === "*" || c === "_") { out += span("markup", c); i++; continue; }
    out += esc(c);
    i++;
  }
  return out;
}

function findMathEnd(src: string, from: number): number {
  for (let k = from; k < src.length; k++) {
    if (src[k] === "\\") { k++; continue; }
    if (src[k] === "\"") { const e = src.indexOf("\"", k + 1); if (e < 0) return src.length; k = e; continue; }
    if (src[k] === "$") return k;
  }
  return src.length;
}

function codeRun(s: string): string {
  let out = "";
  let i = 0;
  while (i < s.length) {
    const rest = s.slice(i);
    let m: RegExpMatchArray | null;
    if ((m = rest.match(/^"(?:[^"\\]|\\.)*"?/))) { out += span("string", m[0]); i += m[0].length; continue; }
    if ((m = rest.match(/^\/\/.*/))) { out += span("comment", m[0]); i += m[0].length; continue; }
    if ((m = rest.match(/^\$[^$]*\$?/))) { out += span("dollar", "$") + mathRun(m[0].slice(1, m[0].endsWith("$") && m[0].length > 1 ? -1 : undefined)) + (m[0].length > 1 && m[0].endsWith("$") ? span("dollar", "$") : ""); i += m[0].length; continue; }
    if ((m = rest.match(/^\d+(\.\d+)?(pt|cm|mm|em|fr|%|in)?/))) { out += span("number", m[0]); i += m[0].length; continue; }
    if ((m = rest.match(/^[\p{L}_][\p{L}\p{N}_-]*/u))) {
      const w = m[0];
      const next = rest[w.length];
      if (KEYWORDS.has(w)) out += span("keyword", w);
      else if (next === "(") out += span("func", w);
      else if (next === ":" && rest[w.length + 1] !== "=") out += span("param", w);
      else out += span("ident", w);
      i += w.length;
      continue;
    }
    if ((m = rest.match(/^\[[^\]]*\]?/)) && m[0].includes("$")) {
      out += esc("[");
      const inner = m[0].slice(1, m[0].endsWith("]") ? -1 : undefined);
      out += highlight(inner);
      if (m[0].endsWith("]")) out += esc("]");
      i += m[0].length;
      continue;
    }
    out += esc(s[i]);
    i++;
  }
  return out;
}

function mathRun(s: string): string {
  let out = "";
  let i = 0;
  let afterMarker = false;
  while (i < s.length) {
    const rest = s.slice(i);
    let m: RegExpMatchArray | null;
    if ((m = rest.match(/^(:=|=>|->)/))) {
      out += span(m[0] === ":=" ? "define" : m[0] === "=>" ? "value" : "symbolic", m[0]);
      afterMarker = m[0] !== ":=";
      i += 2;
      continue;
    }
    if ((m = rest.match(/^"(?:[^"\\]|\\.)*"?/))) { out += span("string", m[0]); i += m[0].length; continue; }
    if ((m = rest.match(/^#[\p{L}_][\p{L}\p{N}_.-]*/u))) { out += span("binding", m[0]); i += m[0].length; continue; }
    if ((m = rest.match(/^\d+(\.\d+)?/))) { out += span("number", m[0]); i += m[0].length; continue; }
    if ((m = rest.match(/^\/\/.*/))) { out += span("comment", m[0]); i += m[0].length; continue; }
    if ((m = rest.match(/^[\p{L}][\p{L}\p{N}]+/u))) {
      const w = m[0];
      out += span(afterMarker ? "modifier" : rest[w.length] === "(" ? "mfunc" : "mident", w);
      i += w.length;
      continue;
    }
    if ((m = rest.match(/^\\/))) { out += span("op", "\\"); i++; continue; }
    out += esc(s[i]);
    i++;
  }
  return out;
}
