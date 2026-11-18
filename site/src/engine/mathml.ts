// Math content as MathML Core, set as Typst sets it: letters italic, text and numbers upright, operators spaced.
import type { C } from "./content";

const esc = (s: string) => s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");

const RELATIONS = new Set("=≔≕⇒→←↔⟹⟸⟺⟶⟵↦≠<>≤≥≈≡∼∝∈∉⊂⊆⊃⊇≪≫⩴∣".split(""));
const BINARY = new Set("+−-±∓×÷⋅·∗∘∧∨∩∪⊕⊗∖".split(""));
const PUNCT = new Set(",;:!".split(""));
const FENCES = new Set("()[]{}|‖⌈⌉⌊⌋⟨⟩".split(""));
const LARGE = new Set("∑∏∫∬∭∮⋃⋂".split(""));
const LIMITS_OPS = new Set(["lim", "liminf", "limsup", "max", "min", "sup", "inf", "det", "gcd", "lcm", "Pr"]);

type Style = { variant?: "normal" | "bold" | "italic" | "cal" | "bb" | "frak" | "sans" | "mono"; color?: string };

// Mathematical alphanumerics for the styles MathML Core can't apply with mathvariant
const ALPHA: Record<string, [number, number, Record<string, number>?]> = {
  bold: [0x1d400, 0x1d41a],
  cal: [0x1d49c, 0x1d4b6, { B: 0x212c, E: 0x2130, F: 0x2131, H: 0x210b, I: 0x2110, L: 0x2112, M: 0x2133, R: 0x211b, e: 0x212f, g: 0x210a, o: 0x2134 }],
  frak: [0x1d504, 0x1d51e, { C: 0x212d, H: 0x210c, I: 0x2111, R: 0x211c, Z: 0x2128 }],
  bb: [0x1d538, 0x1d552, { C: 0x2102, H: 0x210d, N: 0x2115, P: 0x2119, Q: 0x211a, R: 0x211d, Z: 0x2124 }],
  sans: [0x1d5a0, 0x1d5ba],
  mono: [0x1d670, 0x1d68a],
};
function styled(s: string, variant: string | undefined): string {
  const map = variant ? ALPHA[variant] : undefined;
  if (!map) return s;
  return [...s].map(ch => {
    if (map[2] && map[2][ch]) return String.fromCodePoint(map[2][ch]);
    if (ch >= "A" && ch <= "Z") return String.fromCodePoint(map[0] + ch.charCodeAt(0) - 65);
    if (ch >= "a" && ch <= "z") return String.fromCodePoint(map[1] + ch.charCodeAt(0) - 97);
    return ch;
  }).join("");
}

function attrs(st: Style, extra = ""): string {
  let a = extra;
  if (st.color) a += ` style="color:${esc(st.color)}"`;
  return a;
}

export type RenderHooks = {
  /** what a binding shows: its display content */
  binding?: (c: Extract<C, { t: "binding" }>) => C;
};

export class MathML {
  constructor(private hooks: RenderHooks = {}) {}

  equation(c: C, block: boolean): string {
    const lines = splitLines(c);
    if (lines.length > 1) {
      const rows = lines.map(l => `<mtr><mtd>${this.node(l, {})}</mtd></mtr>`).join("");
      return `<math display="${block ? "block" : "inline"}"><mtable columnalign="center" rowspacing="0.5em">${rows}</mtable></math>`;
    }
    return `<math display="${block ? "block" : "inline"}">${this.row(c, {})}</math>`;
  }

  row(c: C, st: Style): string {
    const out = this.node(c, st);
    return out.startsWith("<mrow") ? out : `<mrow>${out}</mrow>`;
  }

  private seq(ch: C[], st: Style): string {
    // spaces matter between atoms; next to operators, fences and punctuation the operator's spacing is enough
    const parts: string[] = [];
    const kinds: string[] = [];
    for (let i = 0; i < ch.length; i++) {
      const x = ch[i];
      if (x.t === "space") {
        const prev = kinds[kinds.length - 1];
        const next = ch.slice(i + 1).find(y => y.t !== "space");
        const nk = next ? kindOf(next) : "op";
        if (prev && prev !== "op" && nk !== "op" && prev !== "open" && nk !== "close" && nk !== "punct") {
          parts.push(`<mspace width="0.25em"></mspace>`);
          kinds.push("space");
        }
        continue;
      }
      parts.push(this.node(x, st));
      kinds.push(kindOf(x));
    }
    return parts.join("");
  }

  node(c: C, st: Style): string {
    switch (c.t) {
      case "seq": return `<mrow>${this.seq(c.ch, st)}</mrow>`;
      case "sym": return this.sym(c.s, st);
      case "text": return this.text(c.s, st);
      case "num": return `<mn${attrs(st)}>${esc(fmtNum(c.v))}</mn>`;
      case "space": return `<mspace width="0.25em"></mspace>`;
      case "h": return `<mspace width="${c.em}em"></mspace>`;
      case "lr": {
        const ch = c.body.t === "seq" ? c.body.ch : [c.body];
        const first = ch[0], last = ch[ch.length - 1];
        const isFence = (x: C | undefined) => !!x && (x.t === "sym" || x.t === "text") && FENCES.has(x.s);
        if (ch.length >= 2 && isFence(first) && isFence(last)) {
          const inner = this.seq(ch.slice(1, -1), st);
          return `<mrow><mo fence="true" stretchy="true"${attrs(st)}>${esc((first as any).s)}</mo>${inner}<mo fence="true" stretchy="true"${attrs(st)}>${esc((last as any).s)}</mo></mrow>`;
        }
        if (ch.length >= 1 && isFence(first)) {
          return `<mrow><mo fence="true" stretchy="true"${attrs(st)}>${esc((first as any).s)}</mo>${this.seq(ch.slice(1), st)}</mrow>`;
        }
        return `<mrow>${this.seq(ch, st)}</mrow>`;
      }
      case "attach": {
        const base = this.node(c.base, st);
        const sub = c.b ? this.row(c.b, st) : "";
        let sup = c.tt ? this.row(c.tt, st) : "";
        if (c.primes) {
          const p = `<mo>${"′".repeat(c.primes)}</mo>`;
          sup = sup ? `<mrow>${p}${sup}</mrow>` : p;
        }
        const big = (c.base.t === "sym" && LARGE.has(c.base.s) && c.base.s !== "∫" && c.base.s !== "∬" && c.base.s !== "∭" && c.base.s !== "∮")
          || (c.base.t === "op" && (c.base.limits || LIMITS_OPS.has(c.base.text)));
        if (big) {
          if (sub && sup) return `<munderover>${base}${sub}${sup}</munderover>`;
          if (sub) return `<munder>${base}${sub}</munder>`;
          return `<mover>${base}${sup}</mover>`;
        }
        if (sub && sup) return `<msubsup>${base}${sub}${sup}</msubsup>`;
        if (sub) return `<msub>${base}${sub}</msub>`;
        return `<msup>${base}${sup}</msup>`;
      }
      case "frac": return `<mfrac>${this.row(c.num, st)}${this.row(c.den, st)}</mfrac>`;
      case "root": return c.index ? `<mroot>${this.row(c.rad, st)}${this.row(c.index, st)}</mroot>` : `<msqrt>${this.row(c.rad, st)}</msqrt>`;
      case "vec": {
        const [l, r] = delims(c.delim ?? "(");
        const rows = c.ch.map(x => `<mtr><mtd>${this.row(x, st)}</mtd></mtr>`).join("");
        return `<mrow><mo fence="true" stretchy="true">${l}</mo><mtable rowspacing="0.3em">${rows}</mtable><mo fence="true" stretchy="true">${r}</mo></mrow>`;
      }
      case "mat": {
        const [l, r] = delims(c.delim ?? "(");
        const rows = c.rows.map(row => `<mtr>${row.map(x => `<mtd>${this.row(x, st)}</mtd>`).join("")}</mtr>`).join("");
        return `<mrow><mo fence="true" stretchy="true">${l}</mo><mtable rowspacing="0.3em" columnspacing="0.8em">${rows}</mtable><mo fence="true" stretchy="true">${r}</mo></mrow>`;
      }
      case "cases": {
        const rows = c.ch.map(row => {
          const ch = row.t === "seq" ? row.ch : [row];
          const at = ch.findIndex(x => x.t === "align");
          const cells = at < 0 ? [ch] : [ch.slice(0, at), ch.slice(at + 1)];
          return `<mtr>${cells.map(cell => `<mtd style="text-align:left;padding-right:1em">${this.row({ t: "seq", ch: cell }, st)}</mtd>`).join("")}</mtr>`;
        }).join("");
        return `<mrow><mo fence="true" stretchy="true">{</mo><mtable columnalign="left left" rowspacing="0.25em">${rows}</mtable></mrow>`;
      }
      case "op": {
        if (c.limits || LIMITS_OPS.has(c.text)) return `<mo movablelimits="true" lspace="0.1667em" rspace="0.1667em"${attrs(st)}>${esc(c.text)}</mo>`;
        return `<mi mathvariant="normal"${attrs(st)}>${esc(c.text)}</mi>`;
      }
      case "builtin": return `<mi mathvariant="normal"${attrs(st)}>${esc(c.name)}</mi>`;
      case "style": {
        if (c.style.startsWith("color:")) return `<mrow style="color:${esc(c.style.slice(6))}">${this.node(c.body, { ...st, color: undefined })}</mrow>`;
        if (["display", "inline", "script", "sscript"].includes(c.style)) return this.node(c.body, st);
        const variant = c.style === "upright" ? "normal" : (c.style as Style["variant"]);
        return this.node(c.body, { ...st, variant });
      }
      case "class": {
        if (c.cls === "normal" || c.cls === "punctuation") {
          const s = plainSym(c.body);
          if (s !== null) return `<mo lspace="0" rspace="${c.cls === "punctuation" ? "0.1667em" : "0"}">${esc(s)}</mo>`;
        }
        if (c.cls === "binary" || c.cls === "relation") {
          const s = plainSym(c.body);
          if (s !== null) return `<mo>${esc(s)}</mo>`;
        }
        return this.node(c.body, st);
      }
      case "accent": return `<mover accent="true">${this.row(c.base, st)}<mo stretchy="true">${esc(c.accent)}</mo></mover>`;
      case "dif": return `<mrow><mspace width="0.1667em"></mspace><mi mathvariant="normal">d</mi></mrow>`;
      case "linebreak": return "";
      case "align": return "";
      case "binding": return this.node(this.hooks.binding ? this.hooks.binding(c) : c.shown, st);
      case "operation": return this.node(c.shown, st);
      case "key": return "";
      case "chip": return `<mrow class="sv-chip"><mrow class="sv-chip-mark">${this.node(c.mark, {})}</mrow><mspace width="0.1667em"></mspace>${this.node(c.unit, {})}</mrow>`;
      case "styled-unit": return this.node(c.body, st);
      case "error": return `<mtext class="sv-err">${esc(c.message)}</mtext>`;
    }
  }

  private sym(s: string, st: Style): string {
    if (/^[0-9.]+$/.test(s)) return `<mn${attrs(st)}>${s}</mn>`;
    if (RELATIONS.has(s) || BINARY.has(s)) return `<mo${attrs(st)}>${esc(s)}</mo>`;
    if (PUNCT.has(s)) return `<mo${attrs(st)} separator="true">${esc(s)}</mo>`;
    if (FENCES.has(s)) return `<mo stretchy="false"${attrs(st)}>${esc(s)}</mo>`;
    if (LARGE.has(s)) return `<mo largeop="true"${attrs(st)}>${esc(s)}</mo>`;
    if (s === "∂" || s === "∇" || s === "∞") return `<mi mathvariant="normal"${attrs(st)}>${esc(s)}</mi>`;
    if (st.variant && st.variant !== "normal" && st.variant !== "italic") return `<mi mathvariant="normal"${attrs(st)}>${esc(styled(s, st.variant))}</mi>`;
    if ([...s].length === 1) {
      // Typst sets Latin and Greek letters italic, uppercase Greek too
      return st.variant === "normal" ? `<mi mathvariant="normal"${attrs(st)}>${esc(s)}</mi>` : `<mi${attrs(st)}>${esc(s)}</mi>`;
    }
    return `<mi mathvariant="normal"${attrs(st)}>${esc(s)}</mi>`;
  }

  private text(s: string, st: Style): string {
    if (s === "") return "";
    if (/^[0-9]+([.,][0-9]+)?$/.test(s)) return `<mn${attrs(st)}>${esc(s)}</mn>`;
    const m = s.match(/^([−-])([0-9]+([.,][0-9]+)?)$/);
    if (m) return `<mrow><mo form="prefix"${attrs(st)}>−</mo><mn${attrs(st)}>${esc(m[2])}</mn></mrow>`;
    if (RELATIONS.has(s) || BINARY.has(s)) return `<mo${attrs(st)}>${esc(s)}</mo>`;
    if (PUNCT.has(s)) return `<mo${attrs(st)} separator="true">${esc(s)}</mo>`;
    if (FENCES.has(s)) return `<mo stretchy="false"${attrs(st)}>${esc(s)}</mo>`;
    if (st.variant && st.variant !== "normal" && st.variant !== "italic") return `<mtext${attrs(st)}>${esc(styled(s, st.variant))}</mtext>`;
    if (st.variant === "italic") return `<mi${attrs(st)}>${esc(s)}</mi>`;
    // a single letter of text is upright (quoted text in Typst math)
    return `<mtext${attrs(st)}>${esc(s.replace(/ /g, " "))}</mtext>`;
  }
}

function plainSym(c: C): string | null {
  if (c.t === "sym" || c.t === "text") return c.s;
  if (c.t === "seq" && c.ch.length === 1) return plainSym(c.ch[0]);
  return null;
}

function kindOf(c: C): string {
  if (c.t === "sym" || c.t === "text") {
    if (RELATIONS.has(c.s) || BINARY.has(c.s)) return "op";
    if (PUNCT.has(c.s)) return "punct";
    if ("([{⌈⌊⟨".includes(c.s)) return "open";
    if (")]}⌉⌋⟩".includes(c.s)) return "close";
  }
  if (c.t === "class") return "op";
  if (c.t === "h") return "op";
  if (c.t === "key") return "op";
  return "atom";
}

function delims(d: string): [string, string] {
  const M: Record<string, [string, string]> = { "(": ["(", ")"], "[": ["[", "]"], "{": ["{", "}"], "|": ["|", "|"], "||": ["‖", "‖"], "‖": ["‖", "‖"], none: ["", ""] };
  return M[d] ?? ["(", ")"];
}

function splitLines(c: C): C[] {
  const ch = c.t === "seq" ? c.ch : [c];
  if (!ch.some(x => x.t === "linebreak")) return [c];
  const lines: C[][] = [[]];
  for (const x of ch) { if (x.t === "linebreak") lines.push([]); else lines[lines.length - 1].push(x); }
  return lines.map(l => ({ t: "seq", ch: l }));
}

function fmtNum(v: number): string {
  if (Number.isInteger(v)) return String(v);
  return String(v);
}
