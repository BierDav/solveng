// Math content as Typst builds it from an equation, and solveng's reading of it for the engine (src/engine.typ's _src).

export type BindingMeta = {
  token: string;
  id: string;
  kind: "unit" | "base-unit" | "constant";
  identity?: string;
  /** displays of the bindings it refers to, by id (for #svref in results) */
  displays: Record<string, C>;
  unit: boolean;
  numeric: boolean;
  compound: boolean;
};

export type C =
  | { t: "sym"; s: string }                 // a letter or symbol: x, π, +, ≔
  | { t: "text"; s: string }                // text: numbers, quoted strings
  | { t: "num"; v: number }                 // a number from code: #calc.pi
  | { t: "space" }
  | { t: "h"; em: number }                  // spacing: thin, quad, h(…)
  | { t: "seq"; ch: C[] }
  | { t: "lr"; body: C }                    // body: the delimiters and what's between them
  | { t: "attach"; base: C; b?: C; tt?: C; primes?: number }
  | { t: "frac"; num: C; den: C }
  | { t: "root"; index?: C; rad: C }
  | { t: "vec"; ch: C[]; delim?: string }
  | { t: "mat"; rows: C[][]; delim?: string }
  | { t: "cases"; ch: C[] }
  | { t: "op"; text: string; limits?: boolean }
  | { t: "style"; style: string; body: C }   // upright, bold, italic, cal, bb, frak, sans, mono, display…
  | { t: "class"; cls: string; body: C }
  | { t: "accent"; base: C; accent: string }
  | { t: "dif" }
  | { t: "linebreak" }
  | { t: "align" }
  | { t: "binding"; meta: BindingMeta; shown: C }
  | { t: "operation"; name: string; shown: C }       // <solveng-operation>: an operation() or convolve(…)
  | { t: "builtin"; name: string }                   // <solveng-builtin-operation>: #import operations: tri
  | { t: "key"; key: string; args: C[] }             // a modifier: solve(x), digits(2)
  | { t: "chip"; mark: C; unit: C }                  // a parameter type, f(I: mA)
  | { t: "styled-unit"; body: C }                    // unit-style
  | { t: "error"; message: string };

export const seq = (ch: C[]): C => (ch.length === 1 ? ch[0] : { t: "seq", ch });
export const sym = (s: string): C => ({ t: "sym", s });
export const text = (s: string): C => ({ t: "text", s });
export const EMPTY: C = { t: "seq", ch: [] };

export class Err { constructor(public message: string) {} }
const err = (m: string) => new Err(m);
const E = (x: unknown): x is Err => x instanceof Err;

/** Plain text of content (a unit's display), as _plain. */
export function plain(c: C | string | number): string {
  if (typeof c === "string") return c;
  if (typeof c === "number") return String(c);
  switch (c.t) {
    case "sym": case "text": return c.s;
    case "num": return String(c.v);
    case "seq": return c.ch.map(plain).join("");
    case "lr": return plain(c.body);
    case "style": case "class": return plain(c.body);
    case "op": return c.text;
    case "attach": return plain(c.base) + (c.b ? "_" + plain(c.b) : "") + (c.tt ? "^" + plain(c.tt) : "");
    case "frac": return plain(c.num) + "/" + plain(c.den);
    case "binding": return plain(c.meta.kind === "constant" ? c.shown : c.shown);
    case "builtin": return c.name;
    case "operation": return plain(c.shown);
    case "dif": return "d";
    default: return " ";
  }
}

const NUMBER = /^[0-9]*\.?[0-9]+$/;
const WORD_END = /[\p{L}\p{N}"⦆⦈)]$/u;
const WORD_START = /^[\p{L}\p{N}"⦅⦇]/u;
const isSpaceC = (c: C) => c.t === "space" || c.t === "h";
const isLabel = (c: C) => c.t === "text" && !NUMBER.test(c.s);
const children = (c: C): C[] => (c.t === "seq" ? c.ch : [c]);

/** Typst's repr of a string. */
export function repr(s: string): string {
  return "\"" + s.replace(/\\/g, "\\\\").replace(/"/g, "\\\"").replace(/\n/g, "\\n") + "\"";
}

/** The equation as text for the engine (_src). */
export function src(c: C): string | Err {
  switch (c.t) {
    case "binding": return c.meta.token;
    case "operation": return "⦇" + c.name + "⦈";
    case "builtin": return "⦇" + c.name + "⦈";
    case "dif": return "dif";
    case "num": return numStr(c.v);
    case "sym": return c.s === "ⅆ" ? "dif" : c.s;
    case "vec": {
      const parts = c.ch.map(src);
      const e = parts.find(E);
      return e ?? "⦇vec⦈(" + parts.join(", ") + ")";
    }
    case "mat": {
      const trim = (row: (string | Err)[]) => { while (row.length > 1 && row[row.length - 1] === "") row = row.slice(0, -1); return row; };
      const rows = c.rows.map(r => trim(r.map(src)));
      const e = rows.flat().find(E);
      return e ?? "mat(" + rows.map(r => r.join(", ")).join("; ") + ")";
    }
    case "cases": {
      const branches: string[] = [];
      for (let i = 0; i < c.ch.length; i++) {
        const row = c.ch[i];
        const parts = children(row);
        const at = parts.map((p, k) => (p.t === "align" ? k : -1)).filter(k => k >= 0);
        if (at.length > 1) return err("cases: one & separates a value from its condition");
        const value = src(at.length === 0 ? row : seq(parts.slice(0, at[0])));
        if (E(value)) return value;
        const condition = at.length === 0 ? [] : parts.slice(at[0] + 1).filter(p => !isSpaceC(p) && !isLabel(p));
        if (condition.length === 0 && i !== c.ch.length - 1) return err("cases: only the last branch can omit its condition");
        const test = condition.length === 0 ? "1" : src(seq(condition));
        if (E(test)) return test;
        branches.push(value + ", " + test);
      }
      return "⦇cases⦈(" + branches.join(", ") + ")";
    }
    case "key": {
      const args = c.args.map(a => { const s = src(a); return E(s) ? "?" : s.replace(/"/g, "").trim(); });
      return "⦅" + c.key + (args.length ? "(" + args.join(",") + ")" : "") + "⦆";
    }
    case "text": {
      const t = c.s;
      if (t === "ⅆ") return "dif";
      return t === "" ? "\"\"" : /\p{L}|\p{So}|[°%‰]/u.test(t) ? repr(t) : t;
    }
    case "align": return "";
    case "linebreak": return "\\";
    case "space": case "h": return " ";
    case "seq": {
      let out = "", prev = "";
      for (const x of c.ch) {
        const s = src(x);
        if (E(s)) return s;
        if (s === "") continue;
        if (prev !== "\"\"" && s !== "\"\"" && WORD_END.test(prev) && WORD_START.test(s)) out += " ";
        out += s;
        prev = s;
      }
      return out;
    }
    case "frac": {
      const a = src(c.num), b = src(c.den);
      if (E(a)) return a;
      if (E(b)) return b;
      return a.trim() === "dif" ? "dif / (" + b.replace(/ⅆ/g, "dif") + ")" : "frac(" + a + ", " + b + ")";
    }
    case "attach": {
      if (c.primes) return err("not a formula");
      const base = src(c.base);
      if (E(base)) return base;
      if ((base === "∫" || base === "∑") && c.b && c.tt) {
        const lower = src(c.b), upper = src(c.tt);
        if (E(lower)) return lower;
        if (E(upper)) return upper;
        return (base === "∑" ? "sum_(" : "integral_(") + lower + ")^(" + upper + ")";
      }
      let out = base;
      // y_A(a p): Typst sets the (a p) in the subscript; it's the call's arguments
      if (c.b && c.b.t === "seq") {
        const b = c.b.ch;
        const group = b[b.length - 1];
        if (b.length > 1 && group.t === "lr" && plain(group).startsWith("(") && b.slice(0, -1).some(x => x.t !== "space")) {
          const head = src({ t: "attach", base: c.base, b: seq(b.slice(0, -1)) });
          const args = src(group);
          if (E(head)) return head;
          if (E(args)) return args;
          if (!c.tt) return head + args;
          const ts = src(c.tt);
          return E(ts) ? ts : head + "(" + args + ")^(" + ts + ")";
        }
      }
      if (c.b) {
        const sub = src(c.b);
        if (E(sub)) return sub;
        out = base.replace(/^"+|"+$/g, "") + "_" + sub.replace(/"/g, "").split(",").map(p => p.replace(/\s/g, "")).join(", ");
      }
      if (c.tt) {
        const t = src(c.tt);
        if (E(t)) return t;
        out += "^(" + t + ")";
      }
      return out;
    }
    case "class": return plain(c.body) === "d" ? "dif" : src(c.body);
    case "op": {
      const name = c.text.trim();
      return name === "d" ? "dif" : "⦇" + name + "⦈";
    }
    case "root": {
      const r = src(c.rad);
      if (E(r)) return r;
      if (c.index) { const i = src(c.index); return E(i) ? i : "(" + r + ")^(1/(" + i + "))"; }
      return "⦇sqrt⦈(" + r + ")";
    }
    case "lr": return src(c.body);
    case "style": return src(c.body);
    case "styled-unit": return src(c.body);
    case "error": return err(c.message);
    default: return err("not a formula");
  }
}

/** Typst's str() of a float or int. */
export function numStr(v: number): string {
  if (Number.isInteger(v) && Math.abs(v) < 1e21) return String(v);
  const s = String(v);
  return s.replace("e+", "e");
}

/** A unit expression as the plugin reads it (_unit-src). */
export function unitSrc(c: C): string {
  if (c.t === "binding") return c.meta.token;
  if (c.t === "attach") {
    if (c.b) throw new Error("solveng: units: use unit for a subscripted unit name");
    return unitSrc(c.base) + (c.tt ? "^(" + unitSrc(c.tt) + ")" : "");
  }
  if (c.t === "lr") return "(" + unitSrc(stripDelims(c.body)) + ")";
  if (c.t === "frac") return "(" + unitSrc(c.num) + ")/(" + unitSrc(c.den) + ")";
  if (c.t === "seq") return c.ch.map(unitSrc).join(" ");
  if (c.t === "style" || c.t === "class") return unitSrc(c.body);
  const s = src(c);
  if (s instanceof Err) throw new Error("solveng: units: not a unit expression");
  return s;
}

function stripDelims(c: C): C {
  const ch = children(c);
  if (ch.length >= 2) return seq(ch.slice(1, -1));
  return c;
}

export { E as isErr };
