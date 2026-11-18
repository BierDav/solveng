// A Typst interpreter for solveng documents: it evaluates the markup, code and math a solveng worksheet uses and runs
// the equations through solveng's own engine, in document order, as the package's show rules do in Typst.
import { Float } from "./cbor";
import { type BindingMeta, type C, EMPTY, Err, isErr, numStr, plain, seq, src, sym, text, unitSrc } from "./content";
import { MathML } from "./mathml";
import { ACCENTS, COLORS, OPERATORS, SPACES, SYMBOLS } from "./symbols";
import { type Expr, type Markup, type MArg, type MNode, Parser, type Stmt, SyntaxError } from "./syntax";
import {
  Args, AUTO, Auto, Color, Content, Dict, Equation, Fn, Label, Length, MathV, Module, Regex, Scope, Selector, Sym, TypstError, type V, typeName,
} from "./values";
import type { Plugin } from "./wasm";

// ---------- output ----------
export type GraphView = {
  x0: number; x1: number; y0: number; y1: number;
  xticks: number[]; yticks: number[]; xlabels: string[]; ylabels: string[]; xmarked: boolean[]; ymarked: boolean[];
  xlabel: string; ylabel: string; xname: string; yname: string; xunit: string; yunit: string;
  xAxisZero: boolean; yAxisZero: boolean; ax: number; ay: number;
  series: { points: [number, number][]; curve: number; label: string | null; dots: boolean; data: boolean; beyond: boolean }[];
  points: { x: number; y: number; guides: string }[];
  styles: { color: string; dash: string | null; dot: string | null; joined: boolean }[];
  width?: string; height?: string;
};
export type Block =
  | { t: "p"; html: string }
  | { t: "h"; level: number; html: string }
  | { t: "eq"; html: string }
  | { t: "graph"; graph: GraphView }
  | { t: "defs"; style: string; rows: { name: string; value: string }[] }
  | { t: "err"; html: string }
  | { t: "raw"; code: string; lang?: string }
  | { t: "list"; html: string };
export type Problem = { message: string; source?: string };
export type RunResult = { blocks: Block[]; problems: Problem[]; definitions: { name: string; kind: string; value: string }[] };

// ---------- options ----------
const DEF: Record<string, any> = {
  digits: 4, "trailing-zeros": false, "decimal-separator": AUTO, grouping: null, "notation-thresholds": [-3, 6], notation: "auto", multiplication: "dot",
  "number-form": "auto", "max-denominator": 1000, "fraction-style": "stacked",
  prefixes: "all", "show-si": false, "preferred-units": [],
  "show-approximation": true, block: true, "max-solutions": 6,
  "defined-style": "grid", errors: "document", "show-types": "chip",
  words: new Dict(new Map([["any", "any"]])), chips: AUTO, "unit-style": AUTO, "unit-separator": AUTO, "constant-displays": new Dict(),
  "graph-colors": ["#3f90da", "#ffa90e", "#bd1f01", "#94a4a2", "#832db6", "#a96b59", "#e76300", "#b9ac70", "#717581", "#92dadd"].map(h => new Color(h)),
  "graph-lines": ["solid", "dashed", "dotted", "dash-dotted"],
  "graph-dots": ["circle", "square", "triangle", "diamond"],
  "graph-joined": [false],
};
const CHOICES: Record<string, string[]> = {
  notation: ["auto", "plain", "scientific", "engineering"], "number-form": ["auto", "decimal", "fraction", "input"], multiplication: ["dot", "times"],
  "fraction-style": ["stacked", "slash"], errors: ["document", "panic"], "show-types": ["none", "plain", "chip"], "defined-style": ["grid", "column", "inline", "list"],
};
const LANGS = new Set(["de", "fr", "it", "es", "pt", "nl", "ru", "pl", "cs", "sk", "sl", "hr", "hu", "sv", "da", "nb", "fi", "tr", "ro", "uk"]);
const GRAPH = ["graph-colors", "graph-lines", "graph-dots", "graph-joined"];
const DOTS = ["circle", "square", "triangle", "triangle-down", "diamond", "pentagon", "hexagon", "star", "cross", "plus"];
const DASHES = ["solid", "dotted", "densely-dotted", "loosely-dotted", "dashed", "densely-dashed", "loosely-dashed", "dash-dotted", "densely-dash-dotted", "loosely-dash-dotted"];
type Cfg = Record<string, any>;

const CONSTANTS: Record<string, C> = {
  pi: sym("π"), euler: sym("e"), imaginary: sym("i"), "speed-of-light": sym("c"),
};

class Panic extends Error {}

/** The engine's state of a variable scope: its definitions, where it started, the scope around it. */
type State = { env: Uint8Array; start: Uint8Array; parent?: State };

export type Sources = { unitWords: string; constants: string };

export class Interp {
  // what bindings show, by id: the binding-displays state of the package
  displays = new Map<string, C>();
  std = new Scope();
  private units!: Module;
  private constantsMod!: Module;
  private operations!: Module;
  private modifiers!: Module;
  private pkg!: Module;
  private mathModule!: Module;
  private bindingCache = new Map<string, MathV>();

  // realization
  blocks: Block[] = [];
  private inline: string[] = [];
  private problems: Problem[] = [];
  private st: State = { env: new Uint8Array(), start: new Uint8Array() };
  cf: Cfg = { ...DEF };
  private rules = 0;
  private lang = "en";
  private budget = 0;
  // what the last math block ended with, as Typst's block note records it
  private note: { cf: Cfg; env: Uint8Array } | null = null;

  constructor(public plugin: Plugin, private sources: Sources) {
    this.buildStd();
  }

  // ======================================================================================================
  // running a document
  run(source: string): RunResult {
    this.blocks = [];
    this.inline = [];
    this.problems = [];
    this.st = { env: new Uint8Array(), start: new Uint8Array() };
    this.cf = { ...DEF };
    this.rules = 0;
    this.lang = "en";
    this.budget = Date.now() + 8000;
    this.note = null;
    const scope = new Scope(this.std);
    try {
      const nodes = new Parser(source).markup();
      this.realizeMarkup(nodes, scope);
    } catch (e) {
      this.fail(e);
    }
    this.flush();
    let definitions: RunResult["definitions"] = [];
    try {
      const note = this.note ?? { cf: this.cf, env: this.st.env };
      if (note.env.length) {
        definitions = (this.plugin.call("defined", note.env, { cfg: this.reqCfg(note.cf) }) as any[])
          .filter(r => r.listed).map(r => ({ name: r.name, kind: r.kind, value: r.plain }));
      }
    } catch { /* the list is a convenience */ }
    return { blocks: this.blocks, problems: this.problems, definitions };
  }

  private fail(e: unknown) {
    const msg = e instanceof SyntaxError ? `syntax: ${e.message}` : e instanceof Panic ? e.message : e instanceof TypstError ? e.message : `${(e as Error).message ?? e}`;
    this.flush();
    this.blocks.push({ t: "err", html: escHtml(msg.startsWith("solveng:") ? "⚠ " + msg : "⚠ " + msg) });
    this.problems.push({ message: msg });
  }

  // ---------- output ----------
  private flush() {
    const html = this.inline.join("").trim();
    if (html) this.blocks.push({ t: "p", html });
    this.inline = [];
  }
  private emitInline(html: string) { this.inline.push(html); }
  private emitBlock(b: Block) { this.flush(); this.blocks.push(b); }

  private errorHtml(msg: string, wide: boolean) {
    this.problems.push({ message: msg });
    if (this.cf.errors === "panic") throw new Panic("solveng: " + msg);
    const html = `<span class="sv-error">⚠ solveng: ${escHtml(msg)}</span>`;
    if (wide) this.emitBlock({ t: "err", html: `⚠ solveng: ${escHtml(msg)}` });
    else this.emitInline(html);
  }

  // ======================================================================================================
  // markup
  realizeMarkup(nodes: Markup[], scope: Scope) {
    for (let i = 0; i < nodes.length; i++) {
      const n = nodes[i];
      if (Date.now() > this.budget) throw new TypstError("the document took too long to calculate");
      switch (n.k) {
        case "text": this.emitInline(escHtml(n.s)); break;
        case "space": this.emitInline(" "); break;
        case "parbreak": this.flush(); break;
        case "linebreak": this.emitInline("<br>"); break;
        case "strong": this.emitInline("<strong>"); this.realizeMarkup(n.body, scope); this.emitInline("</strong>"); break;
        case "emph": this.emitInline("<em>"); this.realizeMarkup(n.body, scope); this.emitInline("</em>"); break;
        case "raw":
          if (n.block) this.emitBlock({ t: "raw", code: n.s, lang: n.lang });
          else this.emitInline(`<code>${escHtml(n.s)}</code>`);
          break;
        case "heading": {
          this.flush();
          const saved = this.inline;
          this.inline = [];
          this.realizeMarkup(n.body, scope);
          const html = this.inline.join("").trim();
          this.inline = saved;
          this.blocks.push({ t: "h", level: n.level, html });
          this.clearAt(n.level);
          break;
        }
        case "list": {
          this.flush();
          this.realizeMarkup(n.body, scope);
          const html = this.inline.join("").trim();
          this.inline = [];
          this.blocks.push({ t: "list", html });
          break;
        }
        case "code": {
          const s = n.stmt;
          if (s.k === "show" && s.selector === null) {
            // a show rule for everything: the rest of this block goes through it
            let f: V;
            try { f = this.ev(s.e, scope); } catch (e) { this.fail(e); return; }
            const rest = new Content([{ k: "markup", nodes: nodes.slice(i + 1), scope }]);
            try { this.place(this.apply(f, rest)); } catch (e) { this.fail(e); }
            return;
          }
          try {
            const v = this.exec(s, scope);
            this.place(v);
          } catch (e) {
            this.fail(e);
          }
          break;
        }
      }
    }
  }

  private apply(f: V, body: Content): V {
    if (f instanceof Fn) return f.call(new Args([body]));
    if (f instanceof Content) return f;
    throw new TypstError(`a show rule takes a function, got ${typeName(f)}`);
  }

  /** Puts a value into the document. */
  place(v: V) {
    if (v === null || v === undefined) return;
    if (v instanceof Content) {
      for (const it of v.items) {
        if (it.k === "markup") this.realizeMarkup(it.nodes, it.scope);
        else if (it.k === "text") this.emitInline(escHtml(it.s));
        else if (it.k === "equation") this.equation(it.eq);
        else it.run(this);
      }
      return;
    }
    if (v instanceof Equation) return this.equation(v);
    if (v instanceof MathV) return this.emitInline(new MathML(this.hooks()).equation(v.c, false));
    if (typeof v === "string") return this.emitInline(escHtml(v));
    if (typeof v === "number") return this.emitInline(escHtml(reprNum(v)));
    if (typeof v === "boolean") return this.emitInline(String(v));
    if (v instanceof Auto) return this.emitInline("auto");
    if (v instanceof Sym) return this.emitInline(escHtml(v.s));
    if (Array.isArray(v)) return this.emitInline(escHtml(repr(v)));
    if (v instanceof Fn) return;
    this.emitInline(escHtml(repr(v)));
  }

  // ======================================================================================================
  // code
  exec(s: Stmt, scope: Scope): V {
    switch (s.k) {
      case "expr": return this.ev(s.e, scope);
      case "let": {
        if (s.params) {
          const params = s.params;
          const body = s.e;
          scope.set(s.name, this.closure(s.name, params, body, scope));
        } else scope.set(s.name, this.ev(s.e, scope));
        return null;
      }
      case "import": return this.import(s, scope);
      case "show": return null;
      case "set": {
        if (s.e.k === "call" && s.e.callee.k === "ident" && s.e.callee.name === "text") {
          for (const a of s.e.args) if (a.name === "lang") this.lang = String(this.ev(a.value, scope));
        }
        return null;
      }
      case "context": return this.ev(s.e, scope);
    }
  }

  private import(s: Extract<Stmt, { k: "import" }>, scope: Scope): V {
    const source = s.source.k === "str" ? this.module(s.source.v) : this.ev(s.source, scope);
    if (!(source instanceof Module)) throw new TypstError(`can't import from ${typeName(source)}`);
    if (s.items === null) { scope.set(s.alias ?? source.name, source); return null; }
    if (s.items === "*") {
      for (const name of source.names()) scope.set(name, () => source.get(name) ?? null);
      return null;
    }
    for (const it of s.items) {
      const v = source.get(it.name);
      if (v === undefined) throw new TypstError(`unresolved import: ${it.name} isn't in ${source.name}`);
      scope.set(it.as ?? it.name, v);
    }
    return null;
  }

  private module(path: string): Module {
    if (path.includes("solveng") || path === "/lib.typ" || path.endsWith("lib.typ")) return this.pkg;
    throw new TypstError(`can't import ${JSON.stringify(path)} here – only solveng: #import "@preview/solveng:3.0.0": *`);
  }

  private closure(name: string, params: { name: string; def?: Expr; sink?: boolean }[], body: Expr, scope: Scope): Fn {
    return new Fn(name, args => {
      const inner = new Scope(scope);
      let i = 0;
      for (const p of params) {
        if (p.sink) { inner.set(p.name, args.pos.slice(i)); i = args.pos.length; continue; }
        if (p.def !== undefined) inner.set(p.name, args.named.has(p.name) ? args.named.get(p.name)! : this.ev(p.def, scope));
        else inner.set(p.name, args.pos[i++] ?? null);
      }
      return this.ev(body, inner);
    });
  }

  ev(e: Expr, scope: Scope): V {
    switch (e.k) {
      case "none": return null;
      case "auto": return AUTO;
      case "bool": return e.v;
      case "int": case "float": return e.v;
      case "length": return new Length(e.v, e.unit);
      case "str": return e.v;
      case "label": return new Label(e.v);
      case "ident": {
        const v = scope.get(e.name);
        if (v === undefined) throw new TypstError(`unknown variable: ${e.name}`, e.pos);
        return v;
      }
      case "field": return this.field(this.ev(e.target, scope), e.name);
      case "call": {
        const f = this.ev(e.callee, scope);
        const args = this.args(e.args, scope);
        return this.call(f, args);
      }
      case "array": {
        const out: V[] = [];
        for (const a of e.items) { const v = this.ev(a.value, scope); if (a.spread && Array.isArray(v)) out.push(...v); else out.push(v); }
        return out;
      }
      case "dict": return new Dict(new Map(e.items.map(([k, v]) => [k, this.ev(v, scope)])));
      case "content": return new Content([{ k: "markup", nodes: e.body, scope }]);
      case "math": return new Equation(this.evMath(e.body, scope), e.block, e.src);
      case "closure": return this.closure("closure", e.params, e.body, scope);
      case "block": {
        const inner = new Scope(scope);
        const parts: V[] = [];
        for (const s of e.stmts) { const v = this.exec(s, inner); if (v !== null) parts.push(v); }
        if (parts.length === 0) return null;
        if (parts.length === 1) return parts[0];
        if (parts.every(p => typeof p === "number")) return parts.reduce((a, b) => (a as number) + (b as number), 0);
        return Content.join(parts);
      }
      case "unary": {
        const x = this.ev(e.x, scope);
        if (e.op === "-") { if (typeof x === "number") return -x; if (x instanceof Length) return new Length(-x.v, x.unit); }
        if (e.op === "not" && typeof x === "boolean") return !x;
        throw new TypstError(`can't apply ${e.op} to ${typeName(x)}`);
      }
      case "binary": return this.binary(e.op, this.ev(e.a, scope), () => this.ev(e.b, scope));
    }
  }

  private binary(op: string, a: V, bf: () => V): V {
    if (op === "and") return a === true && bf() === true;
    if (op === "or") return a === true || bf() === true;
    const b = bf();
    if (typeof a === "number" && typeof b === "number") {
      switch (op) {
        case "+": return a + b; case "-": return a - b; case "*": return a * b; case "/": return a / b;
        case "<": return a < b; case "<=": return a <= b; case ">": return a > b; case ">=": return a >= b;
      }
    }
    if (op === "==") return repr(a) === repr(b);
    if (op === "!=") return repr(a) !== repr(b);
    if (op === "+" && typeof a === "string" && typeof b === "string") return a + b;
    if (op === "+" && Array.isArray(a) && Array.isArray(b)) return [...a, ...b];
    if (op === "+" && (a instanceof Content || a instanceof Equation || typeof a === "string") && (b instanceof Content || b instanceof Equation || typeof b === "string")) return Content.join([a, b]);
    if (op === "*" && typeof a === "number" && b instanceof Length) return new Length(a * b.v, b.unit);
    if (op === "*" && b instanceof Length === false && a instanceof Length && typeof b === "number") return new Length(a.v * b, a.unit);
    if (op === "in" && typeof a === "string") {
      if (Array.isArray(b)) return b.includes(a);
      if (b instanceof Dict) return b.m.has(a);
      if (typeof b === "string") return b.includes(a);
    }
    throw new TypstError(`can't apply ${op} to ${typeName(a)} and ${typeName(b)}`);
  }

  args(list: { name?: string; value: Expr; spread?: boolean }[], scope: Scope): Args {
    const a = new Args();
    for (const x of list) {
      const v = this.ev(x.value, scope);
      if (x.spread) {
        if (Array.isArray(v)) a.pos.push(...v);
        else if (v instanceof Dict) for (const [k, y] of v.m) a.named.set(k, y);
      } else if (x.name) a.named.set(x.name, v);
      else a.pos.push(v);
    }
    return a;
  }

  call(f: V, args: Args): V {
    if (f instanceof Fn) return f.call(args);
    throw new TypstError(`expected function, found ${typeName(f)}`);
  }

  field(v: V, name: string): V {
    if (v instanceof Module) {
      const r = v.get(name);
      if (r === undefined) throw new TypstError(`module ${v.name} doesn't contain ${name}`);
      return r;
    }
    if (v instanceof Fn) {
      if (name === "with") return new Fn("with", args => v.with(args));
      if (name in v.fields) return v.fields[name];
      throw new TypstError(`function ${v.name} has no field ${name}`);
    }
    if (v instanceof Dict) {
      const r = v.m.get(name);
      if (r === undefined) throw new TypstError(`dictionary doesn't contain key ${JSON.stringify(name)}`);
      return r;
    }
    if (v instanceof Sym) {
      const s = SYMBOLS.get(v.path + "." + name) ?? (v.path === "space" ? " " : undefined);
      if (s === undefined) throw new TypstError(`unknown symbol modifier ${name}`);
      return new Sym(s, v.path + "." + name);
    }
    if (typeof v === "string" && name === "len") return new Fn("len", () => v.length);
    if (Array.isArray(v)) {
      if (name === "len") return new Fn("len", () => v.length);
      if (name === "map") return new Fn("map", a => v.map(x => this.call(a.pos[0], new Args([x]))));
      if (name === "join") return new Fn("join", a => {
        if (v.every(x => typeof x === "string")) return (v as string[]).join(typeof a.pos[0] === "string" ? a.pos[0] : "");
        return Content.join(v.flatMap((x, i) => (i && a.pos[0] != null ? [a.pos[0], x] : [x])));
      });
    }
    throw new TypstError(`${typeName(v)} has no field ${name}`);
  }

  // ======================================================================================================
  // math
  /** Math as Typst evaluates it: names looked up in the scope, then in the math module. */
  evMath(nodes: MNode[], scope: Scope): C {
    return seq(nodes.map(n => this.evM(n, scope)));
  }

  private lookupMath(name: string, scope: Scope): V | undefined {
    for (let s: Scope | undefined = scope; s && s !== this.std; s = s.parent) {
      if (s.vars.has(name)) return s.get(name);
    }
    return this.mathModule.get(name) ?? this.std.get(name);
  }

  private evM(n: MNode, scope: Scope): C {
    switch (n.k) {
      case "text":
        if (n.quoted) return text(n.s);
        if (n.num) return text(n.s);
        return sym(n.s);
      case "space": return { t: "space" };
      case "align": return { t: "align" };
      case "linebreak": return { t: "linebreak" };
      case "seq": return seq(n.ch.map(x => this.evM(x, scope)));
      case "lr": return { t: "lr", body: seq([sym(n.open), ...n.body.map(x => this.evM(x, scope)), ...(n.close ? [sym(n.close)] : [])]) };
      case "frac": return { t: "frac", num: this.evM(n.num, scope), den: this.evM(n.den, scope) };
      case "attach": return { t: "attach", base: this.evM(n.base, scope), b: n.b && this.evM(n.b, scope), tt: n.t && this.evM(n.t, scope), primes: n.primes || undefined };
      case "code": return this.toC(this.ev(n.e, scope));
      case "ident": {
        const full = [n.name, ...n.fields].join(".");
        let v = this.lookupMath(n.name, scope);
        if (v === undefined) throw new TypstError(`unknown variable: ${n.name}`, n.pos);
        for (const f of n.fields) v = this.field(v, f);
        if (v instanceof Fn && !(v.name in MATH_VALUE_FNS)) {
          // a function written without a call: Typst shows it as an error
          throw new TypstError(`${full} is a function – call it, ${full}(…)`, n.pos);
        }
        return this.toC(v);
      }
      case "call": {
        const callee = n.callee;
        let f: V | undefined;
        if (callee.k === "text") f = new MathV(this.evM(callee, scope));
        else if (callee.k === "ident") {
          f = this.lookupMath(callee.name, scope);
          if (f === undefined) throw new TypstError(`unknown variable: ${callee.name}`, callee.pos);
          for (const fl of callee.fields) f = this.field(f, fl);
        } else f = this.toV(callee, scope);
        const args = n.args.map(a => ({ name: a.name, value: Array.isArray(a.value) ? a.value.map(x => new MathV(this.evM(x, scope))) : new MathV(this.evM(a.value, scope)) }));
        if (f instanceof Sym && ACCENTS[(callee as any).name]) return { t: "accent", base: (args[0]?.value as MathV).c, accent: ACCENTS[(callee as any).name] };
        if (!(f instanceof Fn)) {
          if (callee.k === "ident" && n.args.some(a => a.name)) throw new TypstError("named-argument syntax can only be used with functions", n.pos);
          // not a function: shown as written, its arguments in parentheses
          const parts: C[] = [];
          n.args.forEach((a, i) => {
            if (i) parts.push(sym(","), { t: "space" });
            if (a.name) parts.push(text(a.name), sym(":"), { t: "space" });
            parts.push(Array.isArray(a.value) ? seq(a.value.map(x => this.evM(x, scope))) : this.evM(a.value, scope));
          });
          return seq([this.toC(f), { t: "lr", body: seq([sym("("), ...parts, sym(")")]) }]);
        }
        const a = new Args();
        for (const x of args) {
          const v: V = Array.isArray(x.value) ? x.value : x.value;
          if (x.name) a.named.set(x.name, v); else a.pos.push(v);
        }
        return this.toC(f.call(a));
      }
    }
  }

  private toV(n: MNode, scope: Scope): V {
    if (n.k === "code") return this.ev(n.e, scope);
    return new MathV(this.evM(n, scope));
  }

  /** A value in math. */
  toC(v: V): C {
    if (v === null || v === undefined) return EMPTY;
    if (v instanceof MathV) return v.c;
    if (v instanceof Equation) return v.body;
    if (typeof v === "string") return text(v);
    if (typeof v === "number") return { t: "num", v };
    if (typeof v === "boolean") return text(String(v));
    if (v instanceof Sym) return sym(v.s);
    if (v instanceof Content) return text(this.plainContent(v));
    if (v instanceof Auto) return text("auto");
    if (v instanceof Length) return text(`${v.v}${v.unit}`);
    if (Array.isArray(v)) return seq(v.map(x => this.toC(x)));
    throw new TypstError(`expected content, found ${typeName(v)}`);
  }

  plainContent(v: Content): string {
    return v.items.map(it => {
      if (it.k === "text") return it.s;
      if (it.k === "markup") return it.nodes.map(m => (m.k === "text" ? m.s : m.k === "space" ? " " : "")).join("");
      if (it.k === "equation") return plain(it.eq.body);
      return "";
    }).join("");
  }

  // ======================================================================================================
  // the engine's options and markup
  reqCfg(cf: Cfg) {
    const g = cf.grouping;
    const sep = cf["decimal-separator"] instanceof Auto ? (LANGS.has(this.lang) ? "," : ".") : cf["decimal-separator"];
    const thresholds = cf["notation-thresholds"];
    return {
      digits: cf.digits, "trailing-zeros": cf["trailing-zeros"], separator: sep,
      grouping: g === null ? null : typeof g === "string" ? g : g instanceof Sym ? g.s : "#",
      scientific: cf.notation === "plain" ? null : ["scientific", "engineering"].includes(cf.notation) ? [0, 1] : thresholds,
      engineering: cf.notation === "engineering", multiplication: cf.multiplication, form: cf["number-form"],
      "max-denominator": cf["max-denominator"], fractions: cf["fraction-style"],
      prefixes: cf.prefixes === null ? "None" : cf.prefixes === "all" ? "All" : { Some: cf.prefixes },
      "show-si": cf["show-si"],
      "preferred-units": (cf["preferred-units"] as V[]).map(u => { const s = src((u as Equation).body); return s instanceof Err ? "" : s; }),
      approximate: cf["show-approximation"], block: cf.block, "max-solutions": cf["max-solutions"],
      "show-types": cf["show-types"],
      words: { any: (cf.words as Dict).get("any") ?? "any" },
      "unit-style": cf["unit-style"] instanceof Fn,
    };
  }

  private markupScope(cf: Cfg): Scope {
    const s = new Scope(this.std);
    s.set("svconst", new Fn("svconst", a => new MathV(this.constantDisplay(String(a.pos[0]), a.pos.length > 1 ? this.toC(a.pos[1]) : { t: "style", style: "upright", body: text(String(a.pos[0])) }, cf))));
    s.set("svref", new Fn("svref", a => new MathV(this.displays.get(String(a.pos[0])) ?? { t: "style", style: "upright", body: text(String(a.pos[1] ?? a.pos[0])) })));
    s.set("svop", new Fn("svop", a => new MathV(this.displays.get("operation:" + String(a.pos[0])) ?? { t: "op", text: String(a.pos[0]) })));
    const sepOpt = cf["unit-separator"];
    s.set("svsep", sepOpt instanceof Auto ? new MathV(seq([{ t: "h", em: 0.06 }, { t: "class", cls: "normal", body: sym("⋅") }, { t: "h", em: 0.06 }])) : typeof sepOpt === "string" ? new MathV({ t: "style", style: "upright", body: text(sepOpt) }) : new MathV(this.toC(sepOpt)));
    s.set("svgroup", cf.grouping === null ? null : new MathV(this.toC(cf.grouping)));
    s.set("svu", new Fn("svu", a => (cf["unit-style"] instanceof Fn ? new MathV(this.toC(cf["unit-style"].call(new Args([a.pos[0]])))) : a.pos[0])));
    s.set("svchip", new Fn("svchip", a => new MathV({ t: "chip", mark: this.toC(a.pos[1]), unit: this.toC(a.pos[2]) })));
    s.set("at", new MathV({ t: "op", text: "at" }));
    return s;
  }

  /** Markup from the engine as math content (_m). */
  m(markup: string, cf: Cfg = this.cf): C {
    const p = new Parser(markup);
    const nodes = p.mathSeq(() => false);
    return this.evMath(nodes, this.markupScope(cf));
  }

  constantDisplay(identity: string, fallback: C, cf: Cfg = this.cf): C {
    const own = (cf["constant-displays"] as Dict).get(identity);
    if (own !== undefined) return this.toC(own);
    return CONSTANTS[identity] ?? fallback;
  }

  hooks() {
    const cf = this.cf;
    return {
      binding: (c: Extract<C, { t: "binding" }>): C => {
        if (c.meta.kind === "constant") return this.constantDisplay(c.meta.identity!, c.shown, cf);
        return c.shown;
      },
    };
  }

  // ======================================================================================================
  // equations (_math-def)
  private equation(eq: Equation) {
    const active = this.rules > 0 && this.evaluates();
    const ml = new MathML(this.hooks());
    if (!active) {
      const html = ml.equation(eq.body, eq.block);
      return eq.block ? this.emitBlock({ t: "eq", html }) : this.emitInline(html);
    }
    const ch = eq.body.t === "seq" ? eq.body.ch : [eq.body];
    if (ch.some(c => c.t === "linebreak")) {
      const groups: C[][] = [[]];
      for (const c of ch) { if (c.t === "linebreak") groups.push([]); else groups[groups.length - 1].push(c); }
      for (const g of groups) this.equation(new Equation(seq(g), eq.block, eq.src));
      return;
    }
    const cf = { ...this.cf, block: eq.block };
    const s = src(eq.body);
    if (s instanceof Err || s.trim() === "") {
      const html = ml.equation(this.restyle(eq.body, cf), eq.block);
      return eq.block ? this.emitBlock({ t: "eq", html }) : this.emitInline(html);
    }
    let out: any;
    try {
      const r = this.plugin.call("line", this.st.env, { src: s, cfg: this.reqCfg(cf), listed: true });
      this.st.env = r.env;
      out = r.out;
    } catch (e) {
      out = { errors: [(e as Error).message], mark: null, result: null, prose: false };
    }
    const hidden = this.cf["#hidden"] === true;
    let shown: C | null = null;
    if (out.prose) shown = this.restyle(eq.body, cf);
    else if (!hidden) {
      if (out.mark === null || out.mark === undefined) shown = this.restyle(eq.body, cf);
      else {
        const [head] = splitEq(ch, out.mark);
        const res = out.result == null ? EMPTY : this.m(out.result, cf);
        shown = seq([this.restyle(seq(head), cf), sym(out.mark === "->" ? "→" : "⇒"), res]);
      }
    }
    if (shown) {
      const html = ml.equation(shown, eq.block);
      if (eq.block) this.emitBlock({ t: "eq", html }); else this.emitInline(html);
    }
    for (const e of out.errors as string[]) { this.problems.push({ message: e, source: s }); this.errorOut(e, eq.block, false); }
  }

  /** A problem shown where it happens; `record`: the message reported for it (Typst's metadata note), or false. */
  private errorOut(msg: string, wide: boolean, record: string | false = msg) {
    if (record !== false) this.problems.push({ message: record });
    if (this.cf.errors === "panic") throw new Panic("solveng: " + msg);
    if (wide) this.emitBlock({ t: "err", html: `⚠ solveng: ${escHtml(msg)}` });
    else this.emitInline(` <span class="sv-error">⚠ solveng: ${escHtml(msg)}</span>`);
  }

  private evaluates(): boolean {
    const list = this.cf["#evaluate"] as boolean[] | undefined;
    return list && list.length ? list[0] : true;
  }

  // ---------- the equation as written (_restyle) ----------
  private name(s: string): string | null {
    try { return this.plugin.call("name", s); } catch { return null; }
  }

  restyle(c: C, cf: Cfg): C {
    switch (c.t) {
      case "binding": case "operation": case "builtin": case "dif": return c;
      case "text": {
        const n = this.name(c.s);
        return n !== null ? this.m(n, cf) : c;
      }
      case "vec": return { ...c, ch: c.ch.map(x => this.restyle(x, cf)) };
      case "mat": return { ...c, rows: c.rows.map(r => r.map(x => this.restyle(x, cf))) };
      case "cases": return { ...c, ch: c.ch.map(x => this.restyle(x, cf)) };
      case "attach": {
        if (c.b && c.b.t === "seq") {
          const b = c.b.ch, group = b[b.length - 1];
          if (b.length > 1 && group.t === "lr" && plain(group).startsWith("(") && b.slice(0, -1).some(x => x.t !== "space")) {
            const head: C = { t: "attach", base: c.base, b: seq(b.slice(0, -1)) };
            const hs = src(head);
            const n = hs instanceof Err ? null : this.name(hs);
            const g = this.restyle(group, cf);
            return seq([n !== null ? this.m(n, cf) : head, c.tt ? { t: "attach", base: g, tt: this.restyle(c.tt, cf) } : g]);
          }
        }
        if (c.tt && c.base.t === "binding" && c.base.meta.compound) {
          return { ...c, base: { t: "lr", body: seq([sym("("), c.base, sym(")")]) } };
        }
        return c;
      }
      case "lr": return { t: "lr", body: this.restyle(c.body, cf) };
      case "frac": return { t: "frac", num: this.restyle(c.num, cf), den: this.restyle(c.den, cf) };
      case "root": return { t: "root", index: c.index, rad: this.restyle(c.rad, cf) };
      case "seq": return this.restyleSeq(c.ch, cf);
      default: return c;
    }
  }

  private restyleSeq(ch: C[], cf: Cfg): C {
    const out: C[] = [];
    let i = 0;
    const isSpace = (x: C | undefined) => !x || x.t === "space";
    while (i < ch.length) {
      const x = ch[i];
      if (x.t === "dif") { out.push(x); i++; continue; }
      const notation = this.notation(x, ch[i + 1], cf);
      if (notation) { out.push(notation); i += 2; continue; }
      if ((x.t === "sym" || x.t === "text") && x.s === ":") {
        let k = i + 1, end = -1, um: C | null = null;
        while (k < ch.length) {
          const w = ch[k];
          const ws = w.t === "sym" || w.t === "text" ? w.s : null;
          if (!isSpace(w) && !["∗", "*", "·", "⋅", "/"].includes(ws ?? "")) {
            const parts = ch.slice(i + 1, k + 1).map(src);
            if (parts.some(isErr)) break;
            let r: string | null = null;
            try { r = this.plugin.call("unit", { cfg: this.reqCfg(cf), src: (parts as string[]).join(" ") }); } catch { r = null; }
            if (r === null) { if (end >= 0) break; } else { end = k + 1; um = this.m(r, cf); }
          }
          k++;
        }
        if (end >= 0 && um) {
          while (out.length && isSpace(out[out.length - 1])) out.pop();
          const how = cf["show-types"];
          const mark: C = { t: "class", cls: "normal", body: sym(":") };
          if (how === "chip") out.push({ t: "h", em: 0.15 }, { t: "chip", mark, unit: um });
          else if (how === "plain") out.push({ t: "h", em: 2 / 9 }, mark, { t: "h", em: 2 / 9 }, um);
          i = end;
          continue;
        }
      }
      let end = -1, normalized: string | null = null, source = "";
      for (let k = i; k < ch.length && !isSpace(ch[k]); k++) {
        const s = src(ch[k]);
        if (s instanceof Err) break;
        source += s;
        const n = this.name(source);
        if (n !== null) { end = k + 1; normalized = n; } else if (s !== "\"\"") break;
      }
      if (end >= 0 && normalized !== null) { out.push(this.m(normalized, cf)); i = end; }
      else { out.push(this.restyle(x, cf)); i++; }
    }
    return dotted(out);
  }

  // diff, integrate, laplace and invlaplace called in math, shown as mathematics writes them
  private notation(op: C, next: C | undefined, cf: Cfg): C | null {
    if (op.t !== "builtin" || !next || !["diff", "integrate", "laplace", "invlaplace"].includes(op.name)) return null;
    const args = callArgs(next);
    if (!args || args.some(a => a.length === 0)) return null;
    const shown = args.map(a => this.restyle(seq(a), cf));
    const isSum = (a: C[]) => a.slice(1).some(x => (x.t === "sym" || x.t === "text") && ["+", "−", "-", "±", "∓"].includes(x.s));
    const operand = isSum(args[0]) ? { t: "lr", body: seq([sym("("), shown[0], sym(")")]) } as C : shown[0];
    const n = args.length;
    const dx = (v: C): C => seq([{ t: "dif" }, v]);
    if (op.name === "diff" && n === 2) return seq([{ t: "frac", num: { t: "dif" }, den: dx(shown[1]) }, operand]);
    if (op.name === "integrate" && n === 2) return seq([sym("∫"), operand, dx(shown[1])]);
    if (op.name === "integrate" && n === 4) return seq([{ t: "attach", base: sym("∫"), b: shown[2], tt: shown[3] }, operand, dx(shown[1])]);
    const L: C = op.name === "laplace" ? { t: "style", style: "cal", body: sym("L") } : { t: "attach", base: { t: "style", style: "cal", body: sym("L") }, tt: seq([sym("−"), text("1")]) };
    const braced: C = { t: "lr", body: seq([sym("{"), shown[0], sym("}")]) };
    if (n === 1) return seq([L, braced]);
    if (n === 3) return seq([L, braced, { t: "lr", body: seq([sym("("), shown[2], sym(")")]) }]);
    return null;
  }

  // ======================================================================================================
  // solveng's API
  private optionArgs(args: Args, allowed: Set<string>): Cfg {
    const lc: Cfg = {};
    for (const [k, v] of [...args.named]) {
      if (!allowed.has(k)) continue;
      args.named.delete(k);
      if (v instanceof Auto) continue;
      lc[k] = v;
    }
    return this.check(lc);
  }

  private check(o: Cfg): Cfg {
    for (const [k, v] of Object.entries(o)) {
      if (!(k in DEF)) throw new TypstError(`unknown option '${k}' (options: ${Object.keys(DEF).join(", ")})`);
      if (CHOICES[k] && !CHOICES[k].includes(v as string)) throw new TypstError(`${k}: ${repr(v)} – use one of ${CHOICES[k].map(x => JSON.stringify(x)).join(", ")}`);
    }
    if ("digits" in o && (typeof o.digits !== "number" || o.digits < 1 || o.digits > 15)) throw new TypstError("digits must be 1…15");
    if ("max-denominator" in o && o["max-denominator"] < 1) throw new TypstError("max-denominator must be ≥ 1");
    if ("max-solutions" in o && o["max-solutions"] < 1) throw new TypstError("max-solutions must be ≥ 1");
    if ("notation-thresholds" in o && o["notation-thresholds"] !== null && (!Array.isArray(o["notation-thresholds"]) || o["notation-thresholds"].length !== 2)) throw new TypstError("notation-thresholds must be (low, high) with integers, or none");
    if ("prefixes" in o && typeof o.prefixes === "string" && o.prefixes !== "all") throw new TypstError("prefixes: \"all\", none or an array like (\"m\", \"k\")");
    if ("preferred-units" in o) {
      const p = o["preferred-units"];
      o["preferred-units"] = p instanceof Equation ? [p] : p;
      if (!Array.isArray(o["preferred-units"]) || !o["preferred-units"].every((u: V) => u instanceof Equation)) throw new TypstError("preferred-units: math expressions containing unit values");
    }
    for (const k of GRAPH) if (k in o) o[k] = graphStyle(k, o[k]);
    return o;
  }

  private merge(c: Cfg, lc: Cfg): Cfg {
    const out = { ...c, ...lc };
    for (const k of GRAPH.filter(k => k in lc)) {
      const old = c[k] as V[], nw = lc[k] as V[];
      const gcd = (a: number, b: number): number => (b === 0 ? a : gcd(b, a % b));
      const period = (old.length / gcd(old.length, nw.length)) * nw.length;
      out[k] = Array.from({ length: period }, (_, i) => { const v = nw[i % nw.length]; return v instanceof Auto ? old[i % old.length] : v; });
    }
    if ("constant-displays" in lc) out["constant-displays"] = new Dict(new Map([...(c["constant-displays"] as Dict).m, ...(lc["constant-displays"] as Dict).m]));
    return out;
  }

  private mathFn(): Fn {
    const allowed = new Set(Object.keys(DEF));
    return new Fn("math", args => {
      const clearAt = args.take("clear-at"), hidden = args.take("hidden"), evaluate = args.take("evaluate");
      const lc = this.optionArgs(args, allowed);
      if (args.named.size) throw new TypstError(`unknown argument '${[...args.named.keys()][0]}' for math; use mathscope(scope: …) to create a variable scope`);
      if (!args.pos.length) throw new TypstError("math takes content, #math[…] or #math($x =>$)");
      const body = Content.join(args.pos);
      return new Content([{ k: "run", run: () => {
        const outer = this.cf;
        const next = this.merge(outer, lc);
        next["#evaluate"] = [evaluate instanceof Auto ? this.evaluates() : evaluate, ...(outer["#evaluate"] ?? [])];
        if (!(hidden instanceof Auto)) next["#hidden"] = hidden;
        if (!(clearAt instanceof Auto)) next["#clear-at"] = clearAt;
        this.cf = next;
        this.rules++;
        try { this.place(body); this.note = { cf: this.cf, env: this.st.env }; } finally { this.rules--; this.cf = outer; }
      } }]);
    });
  }

  private clearAt(level: number) {
    const sel = this.cf["#clear-at"];
    if (!sel) return;
    const match = sel instanceof Selector ? (sel.level === undefined || sel.level === level) : sel instanceof Fn && sel.name === "heading";
    if (match) { this.st.env = this.st.start; }
  }

  private mathscopeFn(): Fn {
    return new Fn("mathscope", args => {
      const scope = args.take("scope", "isolated");
      if (args.named.size) throw new TypstError(`unknown argument '${[...args.named.keys()][0]}' for mathscope; use math(…) to configure calculations`);
      if (!["isolated", "inherit", "shared"].includes(scope as string)) throw new TypstError(`scope: "isolated", "inherit" or "shared", got ${repr(scope)}`);
      const body = Content.join(args.pos);
      return new Content([{ k: "run", run: () => {
        const outer = this.st;
        const env = scope === "isolated" ? new Uint8Array() : outer.env;
        this.st = { env, start: env, parent: outer };
        try {
          this.place((this.mathFn().call(new Args([body]))));
        } finally {
          const inner = this.st;
          this.st = outer;
          if (scope === "shared") outer.env = inner.env;
        }
      } }]);
    });
  }

  private eqSrc(v: V, what: string): string {
    if (!(v instanceof Equation)) throw new TypstError(`${what}: an expression as math, got ${typeName(v)}`);
    const s = src(v.body);
    if (s instanceof Err) throw new TypstError(`${what}: can't read the expression`);
    return s;
  }

  private valueFn(): Fn {
    return new Fn("value", args => {
      const expr = args.pos[0];
      return new Content([{ k: "run", run: () => {
        const cf = { ...this.cf, block: false };
        if (!(expr instanceof Equation)) return this.errorOut(`value: an expression as math, got ${typeName(expr)}`, false, false);
        const s = src(expr.body);
        if (s instanceof Err) return this.errorOut("value: can't read the expression", false, false);
        const r = this.plugin.call("value", this.st.env, { src: s, cfg: this.reqCfg(cf), listed: false });
        if (r.err) return this.errorOut("value: " + r.err, false, r.err);
        this.emitInline(new MathML(this.hooks()).equation(this.m(r.value, cf), false));
      } }]);
    });
  }

  private clearFn(): Fn {
    return new Fn("clear", args => {
      const all = args.take("all", false);
      if (args.named.size) throw new TypstError(`unknown argument '${[...args.named.keys()][0]}' for clear (arguments: the names, all)`);
      const names = args.pos;
      return new Content([{ k: "run", run: () => {
        if (all === true) {
          if (names.length) return this.errorOut("clear: either names or all: true, not both", true);
          this.st.env = this.st.start;
          return;
        }
        if (!names.length) return this.errorOut("clear: which names – clear(\"x\", \"f\") – or every one, clear(all: true)", true);
        const req = names.map(n => (typeof n === "string" ? n : n instanceof Equation ? (s => (s instanceof Err ? plain(n.body) : s))(src(n.body)) : (() => { throw new TypstError(`clear: names as strings or math, clear("x", $f$), got ${typeName(n)}`); })()));
        const r = this.plugin.call("clear", this.st.env, { names: req });
        this.st.env = r.env;
        for (const e of r.errors as string[]) this.errorOut(e, true);
      } }]);
    });
  }

  private definedFn(): Fn {
    const allowed = new Set(Object.keys(DEF));
    return new Fn("defined", args => {
      const names = args.take("names", "all"), kindsV = args.take("kinds", ["given", "computed", "function"]), where = args.take("where", null), style = args.take("style");
      const lc = this.optionArgs(args, allowed);
      if (!(style instanceof Auto)) lc["defined-style"] = style;
      this.check(lc);
      if (args.pos.length) throw new TypstError("defined takes named arguments");
      if (args.named.size) throw new TypstError(`unknown argument '${[...args.named.keys()][0]}' for defined`);
      const kinds = typeof kindsV === "string" ? [kindsV] : (kindsV as string[]);
      return new Content([{ k: "run", run: () => {
        const cf = this.merge(this.cf, lc);
        if (!this.st.env.length) return;
        const named = (n: string, p: V): boolean => typeof p === "string" ? p === "all" || n === p : Array.isArray(p) ? p.some(x => named(n, x)) : p instanceof Regex ? p.re.test(n) : p === null ? false : this.call(p, new Args([n])) === true;
        const rows = (this.plugin.call("defined", this.st.env, { cfg: this.reqCfg(cf) }) as any[])
          .filter(e => e.listed && kinds.includes(e.kind) && named(e.name, names) && (where === null || this.call(where, new Args([e.name])) === true));
        if (!rows.length) return;
        const ml = new MathML(this.hooks());
        const out = rows.map(e => {
          const rel: C = e.rel === "=" ? sym("⇒") : this.m(e.rel, cf);
          const vals: C[] = (e.values as string[]).map(v => this.m(v, cf));
          const value = seq(vals.flatMap((v, i) => (i === 0 ? [rel, v] : [sym("="), v])));
          return { name: ml.equation(this.m(e.lhs, cf), false), value: ml.equation(value, false) };
        });
        const st = cf["defined-style"];
        if (st === "inline") return this.emitInline(out.map(r => r.name + " " + r.value).join(", "));
        this.emitBlock({ t: "defs", style: st, rows: out });
      } }]);
    });
  }

  private compileFn(): Fn {
    return new Fn("compile", args => {
      const expr = args.pos[0];
      const parameters = args.take("parameters", new Dict());
      const resultUnit = args.take("result-unit", null);
      const fail = (m: string) => { throw new Panic("solveng: compile: " + m); };
      if (!(parameters instanceof Dict)) fail("parameters must be an ordered dictionary");
      if (!(expr instanceof Equation)) fail("the expression must be math");
      const s = src((expr as Equation).body);
      if (s instanceof Err) fail("can't read the expression");
      const units: Record<string, string> = {};
      for (const [k, v] of (parameters as Dict).m) if (!(v instanceof Auto)) units[k] = unitSrc((v as Equation).body);
      const r = this.plugin.call("compile", this.st.env, {
        cfg: this.reqCfg({ ...this.cf, "decimal-separator": "." }), src: s,
        params: [...(parameters as Dict).m.keys()],
        unit: resultUnit === null ? null : unitSrc((resultUnit as Equation).body),
        units,
      });
      if (r.err) fail(r.err);
      const compiled = r.compiled as Uint8Array;
      return new Fn("compiled", a => {
        const xs = a.pos.map(v => (typeof v === "number" ? new Float(v) : v instanceof Equation ? unitSrc(v.body) : fail("an argument must be a number or a math quantity")));
        const y = this.plugin.call("compiled_eval", compiled, xs);
        if (y && typeof y === "object" && "err" in y) fail(y.err);
        return y;
      });
    });
  }

  // ---------- graph ----------
  private graphFn(): Fn {
    const allowed = new Set(Object.keys(DEF).filter(k => !GRAPH.includes(k)));
    return new Fn("graph", args => {
      const equation0 = args.take("equation", null);
      const take = (k: string, d: V = AUTO) => args.take(k, d);
      const xRange = take("x-range"), yRange = take("y-range"), xInclude = take("x-include", []), variable = take("variable"), xUnit = take("x-unit"), yUnit = take("y-unit");
      const curves = take("curves", []) as V[], mark = take("mark", null), xmark = take("xmark", []), ymark = take("ymark", []), samples = take("samples", 400);
      const width = take("width"), height = take("height");
      const styleArgs: Cfg = {};
      for (const [k, g] of [["colors", "graph-colors"], ["lines", "graph-lines"], ["dots", "graph-dots"], ["joined", "graph-joined"]]) {
        const v = take(k);
        if (!(v instanceof Auto)) styleArgs[g] = v;
      }
      const lc = this.optionArgs(args, allowed);
      for (const [k, v] of Object.entries(styleArgs)) lc[k] = graphStyle(k, v);
      for (const k of args.named.keys()) throw new TypstError(`graph: unknown argument '${k}' for graph`);
      const positional = args.pos;
      const equation = positional.length === 0 ? equation0 : positional.length === 1 && equation0 === null ? positional[0] : (() => { throw new TypstError("graph: pass one equation, or curves in one equation separated by a backslash"); })();
      const advanced = curves.length > 0 && curves.some(c => c instanceof Dict && c.m.has("expression"));
      const eqs: V[] = advanced ? curves.map(c => (c as Dict).get("expression")!) : [equation];
      if (eqs.some(e => !(e instanceof Equation))) throw new TypstError("graph: pass curves as math equations");
      return new Content([{ k: "run", run: () => this.drawGraph({ eqs: eqs as Equation[], curves, lc, xRange, yRange, xInclude, variable, xUnit, yUnit, mark, xmark, ymark, samples, width, height }) }]);
    });
  }

  private drawGraph(o: any) {
    const outer = this.cf;
    const cf = this.merge(outer, o.lc);
    const fail = (m: string) => this.errorOut("graph: " + m, true);
    const list = (v: V): V[] => (v instanceof Auto || v === null ? [] : Array.isArray(v) ? v : [v]);
    const isValue = (v: V) => typeof v === "number" || v instanceof Equation;
    const rng = list(o.xRange), yrng = list(o.yRange);
    for (const [n, b] of [["x-range", rng], ["y-range", yrng]] as const) {
      if (b.length !== 0 && b.length !== 2) return fail(`${n}: exactly two bounds; use x-include to widen an automatic x range`);
      if (b.some(v => !isValue(v))) return fail(`${n}: values to show, like (1, 2), ($10 #Ω$, $1 kΩ$) or (0, $2 R$)`);
    }
    const KW = ["crossings", "roots", "extrema"];
    const ms = (v: V): V[] => (v === null || v === false ? [] : v === true ? [AUTO] : Array.isArray(v) ? v : [v]);
    const eqSrc = (q: Equation) => { const m = src(q.body); return { src: m instanceof Err ? "?" : m, label: m instanceof Err ? null : m }; };
    const v = (a: V) => (typeof a === "number" ? new Float(a) : eqSrc(a as Equation));
    const mk = (m: V) => (m instanceof Auto ? "auto" : typeof m === "string" && KW.includes(m) ? m : v(m));
    for (const [m] of [...ms(o.mark), ...ms(o.xmark)].map(m => [m])) {
      if (!(m instanceof Auto) && !isValue(m) && !(typeof m === "string" && KW.includes(m))) return fail("mark / xmark: auto, x values like $R$ or 2, \"crossings\", \"roots\", \"extrema\", or an array of these; () for none");
    }
    const curvesReq = o.eqs.map((q: Equation) => eqSrc(q)).flatMap((q: { src: string; label: string | null }) => {
      const labels: string[] = q.label === null ? [] : this.plugin.call("statements", q.label);
      return (this.plugin.call("statements", q.src) as string[]).map((s, i) => ({ src: s, label: labels[i] ?? null }));
    });
    const req = {
      axes: {
        variable: o.variable instanceof Auto ? null : typeof o.variable === "string" ? o.variable : (s => (s instanceof Err ? null : s))(src((o.variable as Equation).body)),
        xunit: o.xUnit instanceof Auto ? null : unitSrc((o.xUnit as Equation).body),
        yunit: o.yUnit instanceof Auto ? null : unitSrc((o.yUnit as Equation).body),
        include: list(o.xInclude).map(v),
      },
      cfg: this.reqCfg(cf), curves: curvesReq, xrange: rng.map(v), yrange: yrng.map(v),
      marks: [...ms(o.mark).map(m => [mk(m), true]), ...ms(o.xmark).map(m => [mk(m), false])], ymark: list(o.ymark).map(v), samples: o.samples,
    };
    let g: any;
    try { g = this.plugin.call("graph", this.st.env, req); } catch (e) { return fail((e as Error).message); }
    if (g.err) return fail(g.err);
    const ml = new MathML(this.hooks());
    const M = (s: string) => ml.equation(this.m(s, cf), false);
    const ncurves = Math.max(curvesReq.length, ...g.series.map((s: any) => s.curve + 1));
    const styles = Array.from({ length: ncurves }, (_, i) => {
      const at = (a: V[]) => a[i % a.length];
      const own = (key: string, kind: string, fallback: V) => {
        const c = o.curves[i] instanceof Dict ? (o.curves[i] as Dict).get(key) : undefined;
        return c === undefined || c instanceof Auto ? fallback : graphStyle(kind, c)[0];
      };
      const color = own("color", "graph-colors", at(cf["graph-colors"]));
      return {
        color: color instanceof Color ? color.hex : "#3f90da",
        dash: own("line", "graph-lines", at(cf["graph-lines"])) as string | null,
        dot: own("dot", "graph-dots", at(cf["graph-dots"])) as string | null,
        joined: own("joined", "graph-joined", at(cf["graph-joined"])) === true,
      };
    });
    const view: GraphView = {
      x0: g.x0, x1: g.x1, y0: g.y0, y1: g.y1, xticks: g.xticks, yticks: g.yticks,
      xlabels: g.xlabels.map(M), ylabels: g.ylabels.map(M), xmarked: g.xmarked ?? [], ymarked: g.ymarked ?? [],
      xlabel: M(g.xlabel), ylabel: M(g.ylabel), xname: g.xname, yname: g.yname, xunit: g.xunit ? M(g.xunit) : "", yunit: g.yunit ? M(g.yunit) : "",
      xAxisZero: g["x-axis-zero"], yAxisZero: g["y-axis-zero"], ax: g.ax, ay: g.ay,
      series: g.series.map((s: any) => ({ points: s.points, curve: s.curve, label: s.label ? M(s.label) : null, dots: !!s.dots, data: !!s.data, beyond: !!s.beyond })),
      points: g.points, styles,
      width: o.width instanceof Length ? `${o.width.v}${o.width.unit}` : undefined,
      height: o.height instanceof Length ? `${o.height.v}${o.height.unit}` : undefined,
    };
    this.emitBlock({ t: "graph", graph: view });
    try {
      const r = this.plugin.call("graph_define", this.st.env, req);
      this.st.env = r.env;
    } catch { /* defining curves is a convenience */ }
  }

  // ---------- bindings ----------
  private binding(kind: "unit" | "base-unit" | "constant", expression: V, identity: string | null, display: V): MathV {
    if (identity !== null && (typeof identity !== "string" || !identity.trim())) throw new TypstError(`${kind}: identity must be a nonempty string`);
    const isBase = kind === "base-unit";
    let body: C = EMPTY;
    if (!isBase) {
      if (!(expression instanceof Equation)) throw new TypstError(`${kind}: expected a math expression`);
      body = expression.body;
    }
    const s = isBase ? "" : src(body);
    if (s instanceof Err) throw new TypstError(`${kind}: cannot read the expression`);
    const written = kind === "unit" && display instanceof Auto;
    let shown: C;
    if (kind === "constant") shown = body;
    else if (written) shown = this.m(this.plugin.call("written", { cfg: this.reqCfg(this.cf), src: s }), this.cf);
    else if (!(display instanceof Auto)) shown = this.displayOf(kind, display);
    else shown = { t: "style", style: "upright", body: text(identity ?? "") };
    // part of the binding's identity, as Typst's repr of the display: the same unit gets the same id in both
    const presentation = kind === "constant" || written ? "" : display instanceof Equation ? `equation(block: ${display.block}, body: ${reprC(display.body)})` : reprC(shown);
    const description = kind === "constant" || written ? "" : plain(shown);
    const key = JSON.stringify([kind, s, identity, presentation, written]);
    const cached = this.bindingCache.get(key);
    if (cached) return cached;
    let b: any;
    try {
      b = this.plugin.call("binding", { kind, src: s, identity, presentation, description, auto: written });
    } catch (e) {
      throw new TypstError((e as Error).message);
    }
    if (kind !== "constant") this.displays.set(b.id, shown);
    const meta: BindingMeta = { token: b.token, id: b.id, kind, identity: identity ?? undefined, displays: {}, unit: kind !== "constant", numeric: !!b.numeric, compound: !!b.compound };
    const v = new MathV({ t: "binding", meta, shown });
    this.bindingCache.set(key, v);
    return v;
  }

  private displayOf(kind: string, display: V): C {
    if (display instanceof Fn) return this.toC(display.call(new Args()));
    if (typeof display === "string") return { t: "style", style: "upright", body: text(display) };
    if (display instanceof Equation || display instanceof MathV || display instanceof Content || display instanceof Sym) return this.toC(display);
    throw new TypstError(`${kind}: display must be content or a function returning it`);
  }

  private opCall(name: string, args: Args, display: V = AUTO): MathV {
    const parts: C[] = [...args.pos.map(a => this.toC(a)), ...[...args.named].map(([k, v]) => seq([text(k), { t: "class", cls: "normal", body: sym(":") }, { t: "h", em: 0.25 }, this.toC(v)]))];
    const shown: C = display instanceof Auto ? { t: "op", text: name } : display instanceof Fn ? this.toC(display.call(new Args())) : typeof display === "string" ? { t: "op", text: display } : this.toC(display);
    if (!(display instanceof Auto)) this.displays.set("operation:" + name, shown);
    const body: C[] = [];
    parts.forEach((p, i) => { if (i) body.push({ t: "class", cls: "punctuation", body: sym(",") }, { t: "h", em: 0.17 }); body.push(p); });
    return new MathV(seq([{ t: "operation", name, shown }, { t: "space" }, { t: "lr", body: seq([sym("("), ...body, sym(")")]) }]));
  }

  // ======================================================================================================
  // the standard library, the math module, solveng's modules
  private buildStd() {
    const std = this.std;
    const fn = (name: string, f: (a: Args) => V, fields: Record<string, V> = {}) => new Fn(name, f, undefined, fields);
    const mathC = (c: C) => new MathV(c);
    const arg = (a: Args, i: number) => this.toC(a.pos[i] ?? null);

    // ---- Typst's math module ----
    const style = (st: string) => fn(st, a => mathC({ t: "style", style: st, body: arg(a, 0) }));
    const delimited = (l: string, r: string) => (a: Args) => mathC({ t: "lr", body: seq([sym(l), arg(a, 0), sym(r)]) });
    const mathFns: Record<string, V> = {
      frac: fn("frac", a => mathC({ t: "frac", num: arg(a, 0), den: arg(a, 1) })),
      binom: fn("binom", a => mathC({ t: "lr", body: seq([sym("("), { t: "frac", num: arg(a, 0), den: arg(a, 1) }, sym(")")]) })),
      sqrt: fn("sqrt", a => mathC({ t: "root", rad: arg(a, 0) })),
      root: fn("root", a => mathC({ t: "root", index: arg(a, 0), rad: arg(a, 1) })),
      vec: fn("vec", a => mathC({ t: "vec", ch: a.pos.map(x => this.toC(x)), delim: delimName(a.named.get("delim")) })),
      // rows are arrays (written with `;`); without them the arguments are one row
      mat: fn("mat", a => mathC({ t: "mat", rows: a.pos.some(Array.isArray) ? a.pos.map(r => (Array.isArray(r) ? r.map(x => this.toC(x)) : [this.toC(r)])) : [a.pos.map(x => this.toC(x))], delim: delimName(a.named.get("delim")) })),
      cases: fn("cases", a => mathC({ t: "cases", ch: a.pos.map(x => this.toC(x)) })),
      abs: fn("abs", delimited("|", "|")), norm: fn("norm", delimited("‖", "‖")),
      floor: fn("floor", delimited("⌊", "⌋")), ceil: fn("ceil", delimited("⌈", "⌉")), round: fn("round", delimited("⌊", "⌉")),
      lr: fn("lr", a => mathC({ t: "lr", body: arg(a, 0) })),
      mid: fn("mid", a => arg(a, 0) as any && mathC(arg(a, 0))),
      upright: style("upright"), italic: style("italic"), bold: style("bold"), cal: style("cal"), scr: style("cal"), frak: style("frak"),
      bb: style("bb"), sans: style("sans"), mono: style("mono"), serif: style("upright"),
      display: style("display"), inline: style("inline"), script: style("script"), sscript: style("sscript"),
      op: fn("op", a => mathC({ t: "op", text: typeof a.pos[0] === "string" ? a.pos[0] : plain(arg(a, 0)), limits: a.named.get("limits") === true })),
      class: fn("class", a => mathC({ t: "class", cls: String(a.pos[0]), body: arg(a, 1) })),
      overline: fn("overline", a => mathC({ t: "accent", base: arg(a, 0), accent: "‾" })),
      underline: fn("underline", a => mathC(arg(a, 0))),
      accent: fn("accent", a => mathC({ t: "accent", base: arg(a, 0), accent: a.pos[1] instanceof Sym ? (a.pos[1] as Sym).s : plain(arg(a, 1)) })),
      attach: fn("attach", a => mathC({ t: "attach", base: arg(a, 0), b: a.named.has("b") ? this.toC(a.named.get("b")!) : undefined, tt: a.named.has("t") ? this.toC(a.named.get("t")!) : undefined })),
      limits: fn("limits", a => mathC(arg(a, 0))), scripts: fn("scripts", a => mathC(arg(a, 0))), stretch: fn("stretch", a => mathC(arg(a, 0))),
      cancel: fn("cancel", a => mathC(arg(a, 0))),
      equation: fn("equation", a => new Equation(arg(a, 0), a.named.get("block") === true, "")),
      text: fn("text", a => this.textFn(a)),
      dif: mathC({ t: "dif" }), Dif: mathC({ t: "style", style: "upright", body: sym("D") }),
    };
    // accents that are symbols too (dot, hat, tilde) stay symbols; calling them sets the accent
    for (const [k, v] of Object.entries(ACCENTS)) if (!(k in mathFns) && !SYMBOLS.has(k)) mathFns[k] = fn(k, a => mathC({ t: "accent", base: arg(a, 0), accent: v }));
    for (const op of OPERATORS) mathFns[op] = mathC({ t: "op", text: op, limits: ["lim", "max", "min", "sup", "inf", "liminf", "limsup", "det", "gcd", "Pr"].includes(op) });
    for (const [k, v] of Object.entries(SPACES)) if (!k.includes(".")) mathFns[k] = mathC({ t: "h", em: v });
    this.mathModule = new Module("math", name => {
      if (name in mathFns) return mathFns[name];
      const s = SYMBOLS.get(name);
      if (s !== undefined) return new Sym(s, name);
      return undefined;
    }, () => Object.keys(mathFns));

    // ---- symbols ----
    const symModule = new Module("sym", name => {
      if (name === "space") return new Sym(" ", "space");
      const s = SYMBOLS.get(name);
      return s === undefined ? undefined : new Sym(s, name);
    }, () => [...SYMBOLS.keys()]);

    // ---- std ----
    std.set("math", this.mathModule);
    std.set("sym", symModule);
    std.set("calc", new Module("calc", name => ({
      pi: Math.PI, e: Math.E, tau: 2 * Math.PI, inf: Infinity,
      round: fn("round", a => { const d = (a.named.get("digits") as number) ?? 0; const f = 10 ** d; return Math.round((a.pos[0] as number) * f) / f; }),
      sqrt: fn("sqrt", a => Math.sqrt(a.pos[0] as number)), abs: fn("abs", a => Math.abs(a.pos[0] as number)),
      pow: fn("pow", a => (a.pos[0] as number) ** (a.pos[1] as number)), exp: fn("exp", a => Math.exp(a.pos[0] as number)),
      ln: fn("ln", a => Math.log(a.pos[0] as number)), log: fn("log", a => Math.log10(a.pos[0] as number)),
      sin: fn("sin", a => Math.sin(a.pos[0] as number)), cos: fn("cos", a => Math.cos(a.pos[0] as number)), tan: fn("tan", a => Math.tan(a.pos[0] as number)),
      min: fn("min", a => Math.min(...(a.pos as number[]))), max: fn("max", a => Math.max(...(a.pos as number[]))),
      floor: fn("floor", a => Math.floor(a.pos[0] as number)), ceil: fn("ceil", a => Math.ceil(a.pos[0] as number)),
    } as Record<string, V>)[name], () => []));
    std.set("regex", fn("regex", a => new Regex(new RegExp(String(a.pos[0]), "u"))));
    std.set("rgb", fn("rgb", a => new Color(typeof a.pos[0] === "string" ? a.pos[0] : "#" + a.pos.slice(0, 3).map(x => Math.round(x as number).toString(16).padStart(2, "0")).join(""))));
    std.set("luma", fn("luma", a => { const h = Math.round(a.pos[0] as number).toString(16).padStart(2, "0"); return new Color(`#${h}${h}${h}`); }));
    for (const [k, v] of Object.entries(COLORS)) std.set(k, new Color(v));
    std.set("heading", fn("heading", a => Content.join(a.pos), { where: fn("where", a => new Selector("heading", a.named.get("level") as number)) }));
    std.set("str", fn("str", a => (typeof a.pos[0] === "number" ? reprNum(a.pos[0]) : String(a.pos[0]))));
    std.set("repr", fn("repr", a => repr(a.pos[0])));
    std.set("type", fn("type", a => typeName(a.pos[0])));
    std.set("range", fn("range", a => {
      const [s, e] = a.pos.length > 1 ? [a.pos[0] as number, a.pos[1] as number] : [0, a.pos[0] as number];
      const step = (a.named.get("step") as number) ?? 1;
      const out: number[] = [];
      for (let x = s; step > 0 ? x < e : x > e; x += step) out.push(x);
      return out;
    }));
    std.set("h", fn("h", a => { const l = a.pos[0]; return new MathV({ t: "h", em: l instanceof Length ? (l.unit === "em" ? l.v : l.v / 11) : 0.25 }); }));
    std.set("v", fn("v", () => null));
    std.set("text", fn("text", a => this.textFn(a)));
    for (const f of ["align", "block", "box", "pad", "strong", "emph", "underline", "highlight", "smallcaps", "par", "figure", "place"]) {
      std.set(f, fn(f, a => {
        const body = a.pos.find(p => p instanceof Content || p instanceof Equation || typeof p === "string") ?? null;
        if (f === "strong") return Content.join([new Content([{ k: "run", run: () => this.emitInline("<strong>") }]), body, new Content([{ k: "run", run: () => this.emitInline("</strong>") }])]);
        if (f === "emph") return Content.join([new Content([{ k: "run", run: () => this.emitInline("<em>") }]), body, new Content([{ k: "run", run: () => this.emitInline("</em>") }])]);
        return body;
      }));
    }
    for (const a of ["left", "right", "center", "top", "bottom", "horizon"]) std.set(a, a);
    std.set("lorem", fn("lorem", a => "Lorem ipsum dolor sit amet, consectetur adipiscing elit. ".repeat(Math.ceil((a.pos[0] as number) / 8)).split(" ").slice(0, a.pos[0] as number).join(" ")));
    std.set("std", new Module("std", name => (name === "math" ? this.mathModule : std.get(name)), () => []));

    // ---- solveng ----
    const baseUnit = fn("base-unit", a => this.binding("base-unit", null, a.pos[0] as string, a.named.get("display") ?? AUTO));
    const unit = fn("unit", a => this.binding("unit", a.pos[0], null, a.named.get("display") ?? AUTO));
    const constant = fn("constant", a => this.binding("constant", a.pos[1], a.pos[0] as string, AUTO));
    const operation = fn("operation", a => {
      const name = a.pos[0], display = a.named.get("display") ?? AUTO;
      if (typeof name !== "string") throw new TypstError("operation: expected an operation identity");
      return fn(name, args => this.opCall(name, args, display));
    });

    // the units and constants modules: solveng's own Typst files, evaluated as they're used
    const unitScope = new Scope(this.std);
    unitScope.set("base-unit", baseUnit);
    unitScope.set("unit", unit);
    this.units = this.lazyModule("units", this.sources.unitWords, unitScope);
    const constScope = new Scope(this.std);
    constScope.set("constant", constant);
    constScope.set("m", () => this.units.get("m")!);
    constScope.set("s", () => this.units.get("s")!);
    this.constantsMod = this.lazyModule("constants", this.sources.constants, constScope);

    // operations: rect is math.op("rect") marked as solveng's; convolve, range and the linear ones are called
    const builtin = (name: string) => mathC({ t: "builtin", name });
    const OPS = ["rect", "tri", "sinc", "cosc", "tanc", "dirac", "floor", "ceil", "factorial", "sgn", "Re", "Im", "conj", "number", "diff", "integrate",
      "at", "asin", "acos", "atan", "laplace", "invlaplace", "rank", "identity", "rows", "cols", "atan2", "min", "max", "mean", "prod", "lsolve",
      "eigenvals", "singularvals", "cond", "median", "variance", "stdev", "pnorm", "qnorm", "linterp", "linfit", "dft", "idft", "gamma", "erf",
      "besselj", "minimize", "odesolve", "celsius", "fahrenheit", "polyroots", "jacobian", "hessian", "taylor"];
    const ops: Record<string, V> = {};
    for (const o of OPS) ops[o] = builtin(o);
    ops.convolve = fn("convolve", a => this.opCall("convolve", a));
    ops.range = fn("range", a => (a.pos.every(x => typeof x === "number") && [...a.named.values()].every(x => typeof x === "number") ? (this.std.get("range") as Fn).call(a) : this.opCall("range", a)));
    ops["linear-solve"] = fn("linear-solve", a => this.opCall("lsolve", a));
    ops["linear-interpolate"] = fn("linear-interpolate", a => this.opCall("linterp", a));
    ops["linear-fit"] = fn("linear-fit", a => this.opCall("linfit", a));
    this.operations = new Module("operations", n => ops[n], () => Object.keys(ops));

    const key = (name: string, args: V[] = []) => mathC({ t: "key", key: name, args: args.map(x => this.toC(x)) });
    const modifier = (name: string) => fn(name, a => (a.named.size ? key("invalid named arguments for " + name + ": " + [...a.named.keys()].join(", ")) : key(name, a.pos)));
    const mods: Record<string, V> = {
      solve: modifier("solve"), substitute: modifier("substitute"), assume: modifier("assume"),
      simplify: key("simplify"), expand: key("expand"), factor: key("factor"), steps: key("steps"),
      digits: fn("digits", a => key("digits", [a.pos[0]])), sci: key("sci"), eng: key("eng"), fraction: key("frac"),
    };
    this.modifiers = new Module("modifiers", n => mods[n], () => Object.keys(mods));

    const pkg: Record<string, V> = {
      math: this.mathFn(), mathscope: this.mathscopeFn(), clear: this.clearFn(), any: key("any"), defined: this.definedFn(), graph: this.graphFn(),
      compile: this.compileFn(), value: this.valueFn(), "base-unit": baseUnit, unit, constant, operation,
      units: this.units, operations: this.operations, constants: this.constantsMod, modifiers: this.modifiers,
    };
    this.pkg = new Module("solveng", n => pkg[n], () => Object.keys(pkg));
  }

  private textFn(a: Args): V {
    const fill = a.named.get("fill");
    const body = a.pos[a.pos.length - 1];
    if (fill instanceof Color) return new MathV({ t: "style", style: "color:" + fill.hex, body: this.toC(body ?? null) });
    return body ?? null;
  }

  /** A module of `#let name = …` lines from a Typst file, each evaluated when it's first used. */
  private lazyModule(name: string, source: string, scope: Scope): Module {
    const defs = new Map<string, Expr>();
    const p = new Parser(source);
    for (const node of p.markup()) {
      if (node.k === "code" && node.stmt.k === "let" && !node.stmt.params) defs.set(node.stmt.name, node.stmt.e);
    }
    const values = new Map<string, V>();
    for (const [n, e] of defs) scope.set(n, () => { if (!values.has(n)) values.set(n, this.ev(e, scope)); return values.get(n)!; });
    return new Module(name, n => (defs.has(n) ? scope.get(n) : undefined), () => [...defs.keys()]);
  }

  /** Names a module offers, for editors: units, operations, … */
  moduleNames(): Record<string, string[]> {
    return { units: this.units.names(), constants: this.constantsMod.names(), operations: this.operations.names(), modifiers: this.modifiers.names() };
  }
}

// functions that are values in math when written without a call
const MATH_VALUE_FNS: Record<string, true> = {};

function delimName(v: V | undefined): string | undefined {
  if (v === undefined || v === null) return v === null ? "none" : undefined;
  if (typeof v === "string") return v;
  if (v instanceof Sym) return v.s;
  return undefined;
}

function graphStyle(name: string, v: V): V[] {
  const kind = name.split("-").pop()!;
  const lens = (a: V) => Array.isArray(a) && a.length > 0 && a.every(l => l instanceof Length || l === "dot");
  const fits = (d: V) => d instanceof Auto || (kind === "colors" ? d instanceof Color : kind === "lines" ? d === null || (typeof d === "string" && DASHES.includes(d)) || lens(d) : kind === "dots" ? d === null || (typeof d === "string" && DOTS.includes(d)) : typeof d === "boolean");
  const one = !Array.isArray(v) || (kind === "lines" && lens(v));
  const arr = one ? [v] : (v as V[]);
  if (!arr.length || !arr.every(fits)) {
    throw new TypstError(name + ": " + ({
      colors: "a color or an array of colors like (gray, blue, rgb(\"#f98eff\"))",
      lines: "a dash pattern or an array of them, like (\"solid\", \"dashed\"); none draws no line; patterns: " + DASHES.join(", "),
      dots: "a dot shape or an array of them, like (\"circle\", \"square\"); none draws no dots; shapes: " + DOTS.join(", "),
      joined: "true, false, or an array of these per curve",
    } as Record<string, string>)[kind]);
  }
  return arr;
}

function splitEq(cs: C[], mark: string): [C[], string | null] {
  const t = mark === "->" ? "→" : "⇒";
  const at = cs.map((c, i) => ((c.t === "sym" || c.t === "text") && (c.s === t || (mark === "=>" && c.s === "=")) ? i : -1)).filter(i => i >= 0);
  if (!at.length) return [cs, null];
  const head = cs.slice(0, at[at.length - 1]);
  while (head.length && head[head.length - 1].t === "space") head.pop();
  return [head, t];
}

function callArgs(c: C): C[][] | null {
  if (c.t !== "lr") return null;
  const cs = c.body.t === "seq" ? c.body.ch : [c.body];
  const s = (x: C) => (x.t === "sym" || x.t === "text" ? x.s : null);
  if (cs.length < 2 || s(cs[0]) !== "(" || s(cs[cs.length - 1]) !== ")") return null;
  const args: C[][] = [];
  let part: C[] = [];
  for (const x of cs.slice(1, -1)) { if (s(x) === ",") { args.push(part); part = []; } else part.push(x); }
  args.push(part);
  return args.map(a => {
    while (a.length && a[0].t === "space") a.shift();
    while (a.length && a[a.length - 1].t === "space") a.pop();
    return a;
  });
}

// factors written next to each other, joined as results are: a dot where digits would meet, a thin space before a unit
function dotted(out: C[]): C {
  const blank = (x: C) => x.t === "space" || x.t === "h";
  const meta = (x: C) => (x.t === "binding" ? x.meta : null);
  const operator = (x: C): boolean => {
    if (x.t === "sym") return ["∫", "∬", "∭", "∮", "∑", "∏"].includes(x.s);
    if (x.t === "builtin" || x.t === "operation" || x.t === "op") return true;
    if (x.t === "attach") return operator(x.base);
    if (x.t === "frac") return x.num.t === "dif" || ["d", "∂"].includes(plain(x.num).trim());
    return false;
  };
  const startsNum = (x: C): boolean => {
    const m = meta(x);
    if (m) return m.numeric;
    if (x.t === "frac") return startsNum(x.num);
    if (x.t === "attach") return startsNum(x.base);
    if (x.t === "seq") { const f = x.ch.find(c => !blank(c)); return !!f && startsNum(f); }
    return /^\p{N}/u.test(plain(x).trim());
  };
  const ends = (x: C) => !operator(x) && (!!meta(x) || /[\p{L}\p{N})\]}′]$/u.test(plain(x).trim()));
  const joined: C[] = [];
  let prev: C | null = null;
  for (const x of out) {
    if (!blank(x)) {
      let join: C | null = null;
      if (prev && ends(prev)) {
        if (startsNum(x)) join = sym("⋅");
        else if (meta(x)?.unit && !meta(prev) && /\p{N}$/u.test(plain(prev).trim())) join = { t: "h", em: 0.1667 };
      }
      if (join) {
        while (joined.length && blank(joined[joined.length - 1])) joined.pop();
        joined.push(join);
      }
      prev = x;
    }
    joined.push(x);
  }
  return seq(joined);
}

/** Typst's repr of math content, for the displays that are part of a binding's identity. */
function reprC(c: C): string {
  switch (c.t) {
    case "sym": case "text": return `[${c.s}]`;
    case "num": return `[${numStr(c.v)}]`;
    case "space": return "[ ]";
    case "style": return c.style === "upright" ? `styled(child: ${reprC(c.body)}, ..)` : `${c.style}(body: ${reprC(c.body)})`;
    case "op": return `op(text: [${c.text}]${c.limits ? ", limits: true" : ""})`;
    case "attach": return `attach(base: ${reprC(c.base)}${c.b ? `, b: ${reprC(c.b)}` : ""}${c.tt ? `, t: ${reprC(c.tt)}` : ""})`;
    case "frac": return `frac(num: ${reprC(c.num)}, denom: ${reprC(c.den)})`;
    case "seq": return `sequence(${c.ch.map(reprC).join(", ")})`;
    case "lr": return `lr(body: ${reprC(c.body)})`;
    default: return `[${plain(c)}]`;
  }
}

export function escHtml(s: string): string {
  return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}

function reprNum(v: number): string {
  return numStr(v);
}

export function repr(v: V): string {
  if (v === null) return "none";
  if (v instanceof Auto) return "auto";
  if (typeof v === "string") return JSON.stringify(v);
  if (typeof v === "number") return reprNum(v);
  if (typeof v === "boolean") return String(v);
  if (Array.isArray(v)) return "(" + v.map(repr).join(", ") + (v.length === 1 ? "," : "") + ")";
  if (v instanceof Dict) return "(" + [...v.m].map(([k, x]) => `${k}: ${repr(x)}`).join(", ") + ")";
  if (v instanceof Length) return `${v.v}${v.unit}`;
  if (v instanceof Color) return `rgb("${v.hex}")`;
  if (v instanceof Label) return `<${v.name}>`;
  if (v instanceof Equation || v instanceof MathV) return "equation";
  if (v instanceof Content) return "[…]";
  if (v instanceof Fn) return v.name;
  return typeName(v);
}

export type { MArg };
