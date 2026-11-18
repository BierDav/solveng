// Typst's syntax, as far as solveng documents use it: markup, code and math. The trees keep what solveng reads
// (math as Typst structures it: attachments, fractions, groups, calls), not every detail of Typst.

export class SyntaxError extends Error {
  constructor(message: string, public pos: number) { super(message); }
}

// ---------- trees ----------
export type Expr =
  | { k: "none" } | { k: "auto" }
  | { k: "bool"; v: boolean }
  | { k: "int"; v: number } | { k: "float"; v: number }
  | { k: "length"; v: number; unit: string }
  | { k: "str"; v: string }
  | { k: "label"; v: string }
  | { k: "ident"; name: string; pos: number }
  | { k: "field"; target: Expr; name: string }
  | { k: "call"; callee: Expr; args: Arg[]; pos: number }
  | { k: "array"; items: Arg[] }
  | { k: "dict"; items: [string, Expr][] }
  | { k: "content"; body: Markup[] }
  | { k: "math"; body: MNode[]; block: boolean; src: string; pos: number }
  | { k: "closure"; params: Param[]; body: Expr }
  | { k: "block"; stmts: Stmt[] }
  | { k: "unary"; op: string; x: Expr }
  | { k: "binary"; op: string; a: Expr; b: Expr };
export type Arg = { name?: string; value: Expr; spread?: boolean };
export type Param = { name: string; def?: Expr; sink?: boolean };
export type Stmt =
  | { k: "expr"; e: Expr; pos: number }
  | { k: "let"; name: string; params?: Param[]; e: Expr; pos: number }
  | { k: "import"; source: Expr; items: "*" | { name: string; as?: string }[] | null; alias?: string; pos: number }
  | { k: "show"; selector: Expr | null; e: Expr; pos: number }
  | { k: "set"; e: Expr; pos: number }
  | { k: "context"; e: Expr; pos: number };

export type Markup =
  | { k: "text"; s: string }
  | { k: "space" }
  | { k: "parbreak" }
  | { k: "linebreak" }
  | { k: "heading"; level: number; body: Markup[] }
  | { k: "strong"; body: Markup[] }
  | { k: "emph"; body: Markup[] }
  | { k: "raw"; s: string; block: boolean; lang?: string }
  | { k: "list"; body: Markup[] }
  | { k: "code"; stmt: Stmt };

export type MNode =
  | { k: "text"; s: string; num?: boolean; quoted?: boolean }
  | { k: "space" }
  | { k: "ident"; name: string; fields: string[]; pos: number }
  | { k: "code"; e: Expr }
  | { k: "call"; callee: MNode; args: MArg[]; pos: number }
  | { k: "seq"; ch: MNode[] }
  | { k: "lr"; open: string; body: MNode[]; close: string }
  | { k: "attach"; base: MNode; b?: MNode; t?: MNode; primes: number }
  | { k: "frac"; num: MNode; den: MNode }
  | { k: "align" }
  | { k: "linebreak" };
// a math call's arguments: content, or (before a `;`) an array of it
export type MArg = { name?: string; value: MNode | MNode[] };

// ---------- characters ----------
const isIdStart = (c: string) => /[\p{L}_]/u.test(c);
const isIdContinue = (c: string) => /[\p{L}\p{N}_-]/u.test(c);
const isMathIdStart = (c: string) => /\p{L}/u.test(c);
const isMathIdContinue = (c: string) => /[\p{L}\p{N}]/u.test(c);
const isDigit = (c: string) => c >= "0" && c <= "9";
const isSpace = (c: string) => c === " " || c === "\t" || c === "\n" || c === "\r";

// Typst's math shorthands, longest first
const SHORTHANDS: [string, string][] = ([
  ["<==>", "⟺"], ["-->", "⟶"], ["<--", "⟵"], ["==>", "⟹"], ["<==", "⟸"], ["|->", "↦"], ["<->", "↔"], ["::=", "⩴"],
  [":=", "≔"], ["=:", "≕"], ["=>", "⇒"], ["->", "→"], ["<-", "←"], ["!=", "≠"], ["<=", "≤"], [">=", "≥"],
  ["...", "…"], ["||", "‖"], ["[|", "⟦"], ["|]", "⟧"], ["<<", "≪"], [">>", "≫"], ["~~", "≈"], ["-", "−"], ["*", "∗"],
] as [string, string][]).sort((a, b) => b[0].length - a[0].length);

export class Parser {
  p = 0;
  constructor(public s: string) {}
  get c() { return this.s[this.p] ?? ""; }
  at(t: string) { return this.s.startsWith(t, this.p); }
  eof() { return this.p >= this.s.length; }
  eat(t: string) { if (this.at(t)) { this.p += t.length; return true; } return false; }
  expect(t: string) { if (!this.eat(t)) throw new SyntaxError(`expected ${JSON.stringify(t)}${this.eof() ? " at the end" : `, found ${JSON.stringify(this.c)}`}`, this.p); }
  fail(m: string): never { throw new SyntaxError(m, this.p); }

  // ---------- trivia ----------
  comment(): boolean {
    if (this.at("//")) { while (!this.eof() && this.c !== "\n") this.p++; return true; }
    if (this.at("/*")) {
      let depth = 0;
      do {
        if (this.at("/*")) { depth++; this.p += 2; } else if (this.at("*/")) { depth--; this.p += 2; } else this.p++;
      } while (depth > 0 && !this.eof());
      return true;
    }
    return false;
  }
  /** Code-mode whitespace and comments; newlines too unless `inline`. */
  ws(inline = false) {
    for (;;) {
      if (this.c === " " || this.c === "\t" || this.c === "\r" || (!inline && this.c === "\n")) this.p++;
      else if (!this.comment()) return;
    }
  }

  // ---------- markup ----------
  markup(stop: (p: Parser) => boolean = () => false, atLineStart = true): Markup[] {
    const out: Markup[] = [];
    let text = "";
    let lineStart = atLineStart;
    const flush = () => { if (text) { out.push({ k: "text", s: text }); text = ""; } };
    const space = () => { flush(); if (out.length && out[out.length - 1].k !== "space" && out[out.length - 1].k !== "parbreak") out.push({ k: "space" }); };
    while (!this.eof() && !stop(this)) {
      const c = this.c;
      if (this.at("//") || this.at("/*")) {
        // a comment alone on its line takes the line with it
        this.comment();
        continue;
      }
      if (c === "\n" || c === " " || c === "\t" || c === "\r") {
        let newlines = 0;
        while (!this.eof() && isSpace(this.c)) { if (this.c === "\n") newlines++; this.p++; }
        if (newlines >= 2) { flush(); while (out.length && out[out.length - 1].k === "space") out.pop(); out.push({ k: "parbreak" }); }
        else space();
        lineStart = newlines > 0;
        continue;
      }
      if (lineStart && c === "=") {
        let level = 0;
        const save = this.p;
        while (this.c === "=") { level++; this.p++; }
        if (this.c === " ") {
          this.p++;
          flush();
          const body = this.markup(p => p.c === "\n", false);
          out.push({ k: "heading", level, body: trimMarkup(body) });
          lineStart = false;
          continue;
        }
        this.p = save;
      }
      if (lineStart && (c === "-" || c === "+") && this.s[this.p + 1] === " ") {
        this.p += 2;
        flush();
        out.push({ k: "list", body: trimMarkup(this.markup(p => p.c === "\n", false)) });
        lineStart = false;
        continue;
      }
      lineStart = false;
      if (c === "\\") {
        const n = this.s[this.p + 1];
        if (n === undefined || isSpace(n)) { this.p++; flush(); out.push({ k: "linebreak" }); continue; }
        this.p += 2;
        text += n;
        continue;
      }
      if (c === "$") { flush(); out.push({ k: "code", stmt: { k: "expr", e: this.equation(), pos: this.p } }); continue; }
      if (c === "#") {
        const n = this.s[this.p + 1] ?? "";
        if (isIdStart(n) || n === "(" || n === "{" || n === "[" || n === "\"") {
          this.p++;
          flush();
          const pos = this.p;
          out.push({ k: "code", stmt: this.embedded(pos) });
          // an optional semicolon ends the expression
          this.eat(";");
          continue;
        }
      }
      if (c === "*") { this.p++; flush(); out.push({ k: "strong", body: this.markup(p => p.c === "*", false) }); this.eat("*"); continue; }
      if (c === "_" && !/[\p{L}\p{N}]/u.test(this.s[this.p - 1] ?? "")) { this.p++; flush(); out.push({ k: "emph", body: this.markup(p => p.c === "_", false) }); this.eat("_"); continue; }
      if (c === "`") {
        flush();
        if (this.at("```")) {
          this.p += 3;
          const end = this.s.indexOf("```", this.p);
          const body = this.s.slice(this.p, end < 0 ? undefined : end);
          this.p = end < 0 ? this.s.length : end + 3;
          const m = body.match(/^(\w*)\n?/);
          out.push({ k: "raw", s: body.slice(m![0].length).replace(/\n$/, ""), block: true, lang: m![1] || undefined });
        } else {
          this.p++;
          const end = this.s.indexOf("`", this.p);
          out.push({ k: "raw", s: this.s.slice(this.p, end < 0 ? undefined : end), block: false });
          this.p = end < 0 ? this.s.length : end + 1;
        }
        continue;
      }
      if (c === "<" && /^<[\p{L}_][\p{L}\p{N}_:.-]*>/u.test(this.s.slice(this.p))) { this.p = this.s.indexOf(">", this.p) + 1; continue; }
      if (c === "~") { text += " "; this.p++; continue; }
      if (this.at("---")) { text += "—"; this.p += 3; continue; }
      if (this.at("--")) { text += "–"; this.p += 2; continue; }
      text += c;
      this.p++;
    }
    flush();
    return out;
  }

  /** `#…` in markup: a keyword statement, or one expression without operators. */
  embedded(pos: number): Stmt {
    const word = this.s.slice(this.p).match(/^[\p{L}_][\p{L}\p{N}_-]*/u)?.[0];
    if (word && ["let", "import", "show", "set", "context", "include"].includes(word)) return this.statement(true);
    if (word && ["if", "for", "while"].includes(word)) this.fail(`#${word} isn't supported here`);
    return { k: "expr", e: this.postfix(true), pos };
  }

  // ---------- code ----------
  statement(embedded = false): Stmt {
    const pos = this.p;
    if (this.eat("let") ) {
      this.ws(true);
      const name = this.ident();
      let params: Param[] | undefined;
      if (this.c === "(") params = this.params();
      this.ws(true);
      if (!this.eat("=")) return { k: "let", name, params, e: { k: "none" }, pos };
      this.ws(true);
      return { k: "let", name, params, e: this.expr(), pos };
    }
    if (this.eat("import")) {
      this.ws(true);
      const source = this.postfix(false);
      this.ws(true);
      let alias: string | undefined;
      if (this.eat("as")) { this.ws(true); alias = this.ident(); this.ws(true); }
      if (!this.eat(":")) return { k: "import", source, items: null, alias, pos };
      this.ws(true);
      if (this.eat("*")) return { k: "import", source, items: "*", pos };
      const items: { name: string; as?: string }[] = [];
      const paren = this.eat("(");
      for (;;) {
        this.ws(!paren);
        if (paren && this.eat(")")) break;
        const name = this.ident();
        this.ws(!paren);
        let as: string | undefined;
        if (this.eat("as")) { this.ws(!paren); as = this.ident(); }
        items.push({ name, as });
        this.ws(!paren);
        if (!this.eat(",")) { if (paren) this.expect(")"); break; }
        this.ws(!paren);
        if (!paren && (this.c === "\n" || this.eof())) break;
      }
      return { k: "import", source, items, pos };
    }
    if (this.eat("show")) {
      this.ws(true);
      let selector: Expr | null = null;
      if (this.c !== ":") selector = this.expr();
      this.ws(true);
      this.expect(":");
      this.ws(true);
      return { k: "show", selector, e: this.expr(), pos };
    }
    if (this.eat("set")) { this.ws(true); return { k: "set", e: this.postfix(false), pos }; }
    if (this.eat("context")) { this.ws(true); return { k: "context", e: embedded ? this.postfix(true) : this.expr(), pos }; }
    if (this.eat("include")) this.fail("#include isn't supported here");
    return { k: "expr", e: this.expr(), pos };
  }

  ident(): string {
    const m = this.s.slice(this.p).match(/^[\p{L}_][\p{L}\p{N}_-]*/u);
    if (!m) this.fail("expected a name");
    // a trailing hyphen belongs to what follows (`a-` in `a - b`)
    let name = m[0];
    while (name.endsWith("-")) name = name.slice(0, -1);
    this.p += name.length;
    return name;
  }

  params(): Param[] {
    this.expect("(");
    const out: Param[] = [];
    for (;;) {
      this.ws();
      if (this.eat(")")) return out;
      const sink = this.eat("..");
      const name = sink && !isIdStart(this.c) ? "_" : this.ident();
      this.ws();
      let def: Expr | undefined;
      if (this.eat(":")) { this.ws(); def = this.expr(); }
      out.push({ name, def, sink });
      this.ws();
      if (!this.eat(",")) { this.expect(")"); return out; }
    }
  }

  expr(): Expr {
    // closures: x => …, (a, b) => …
    const save = this.p;
    if (isIdStart(this.c)) {
      const name = this.ident();
      this.ws(true);
      if (this.eat("=>")) { this.ws(); return { k: "closure", params: [{ name }], body: this.expr() }; }
      this.p = save;
    } else if (this.c === "(") {
      try {
        const params = this.params();
        this.ws(true);
        if (this.eat("=>")) { this.ws(); return { k: "closure", params, body: this.expr() }; }
      } catch { /* not a parameter list */ }
      this.p = save;
    }
    return this.binary(0);
  }

  binary(min: number): Expr {
    const PREC: Record<string, number> = { or: 1, and: 2, "==": 3, "!=": 3, "<": 3, "<=": 3, ">": 3, ">=": 3, in: 3, "+": 4, "-": 4, "*": 5, "/": 5 };
    let a = this.unary();
    for (;;) {
      const save = this.p;
      this.ws(true);
      const op = ["==", "!=", "<=", ">=", "<", ">", "+", "-", "*", "/"].find(o => this.at(o) && !(o === "<" && /^<[\p{L}]/u.test(this.s.slice(this.p))))
        ?? ["and", "or", "in"].find(o => this.at(o) && !isIdContinue(this.s[this.p + o.length] ?? ""));
      if (!op || PREC[op] < min || (op === "=" )) { this.p = save; return a; }
      this.p += op.length;
      this.ws(true);
      const b = this.binary(PREC[op] + 1);
      a = { k: "binary", op, a, b };
    }
  }

  unary(): Expr {
    if (this.eat("-")) return { k: "unary", op: "-", x: this.unary() };
    if (this.eat("+")) return this.unary();
    if (this.at("not") && !isIdContinue(this.s[this.p + 3] ?? "")) { this.p += 3; this.ws(true); return { k: "unary", op: "not", x: this.unary() }; }
    return this.postfix(false);
  }

  /** A primary expression with field accesses and calls; in markup (`embedded`) without spaces before them. */
  postfix(embedded: boolean): Expr {
    let e = this.primary(embedded);
    for (;;) {
      if (this.c === "." && isIdStart(this.s[this.p + 1] ?? "")) { this.p++; e = { k: "field", target: e, name: this.ident() }; continue; }
      if (this.c === "(" ) { const pos = this.p; e = { k: "call", callee: e, args: this.args(), pos }; continue; }
      if (this.c === "[" && (e.k === "call" || e.k === "ident" || e.k === "field")) {
        const pos = this.p;
        const body = this.contentBlock();
        if (e.k === "call") e.args.push({ value: body }); else e = { k: "call", callee: e, args: [{ value: body }], pos };
        continue;
      }
      return e;
    }
  }

  args(): Arg[] {
    this.expect("(");
    const out: Arg[] = [];
    for (;;) {
      this.ws();
      if (this.eat(")")) return out;
      if (this.eat("..")) { out.push({ value: this.expr(), spread: true }); }
      else {
        const m = this.s.slice(this.p).match(/^([\p{L}_][\p{L}\p{N}_-]*)\s*:(?!=)/u);
        if (m) { this.p += m[0].length; this.ws(); out.push({ name: m[1], value: this.expr() }); }
        else out.push({ value: this.expr() });
      }
      this.ws();
      if (!this.eat(",")) { this.expect(")"); return out; }
    }
  }

  contentBlock(): Expr {
    this.expect("[");
    let depth = 0;
    const body = this.markup(p => {
      if (p.c === "[") depth++;
      if (p.c === "]") { if (depth === 0) return true; depth--; }
      return false;
    }, false);
    this.expect("]");
    return { k: "content", body };
  }

  primary(embedded: boolean): Expr {
    const c = this.c;
    const pos = this.p;
    if (c === "$") return this.equation();
    if (c === "[") return this.contentBlock();
    if (c === "\"") return { k: "str", v: this.string() };
    if (c === "<") {
      const m = this.s.slice(this.p).match(/^<([\p{L}_][\p{L}\p{N}_:.-]*)>/u);
      if (m) { this.p += m[0].length; return { k: "label", v: m[1] }; }
    }
    if (isDigit(c) || (c === "." && isDigit(this.s[this.p + 1] ?? ""))) {
      const m = this.s.slice(this.p).match(/^(\d*\.?\d+(?:e[+-]?\d+)?)(pt|mm|cm|in|em|fr|deg|rad|%)?/)!;
      this.p += m[0].length;
      const v = Number(m[1]);
      if (m[2]) return { k: "length", v, unit: m[2] };
      return /[.e]/.test(m[1]) ? { k: "float", v } : { k: "int", v };
    }
    if (c === "{") {
      this.p++;
      const stmts: Stmt[] = [];
      for (;;) {
        this.ws();
        if (this.eat("}")) break;
        stmts.push(this.statement());
        this.ws(true);
        this.eat(";");
      }
      return { k: "block", stmts };
    }
    if (c === "(") {
      this.p++;
      this.ws();
      if (this.eat(")")) return { k: "array", items: [] };
      if (this.eat(":")) { this.ws(); this.expect(")"); return { k: "dict", items: [] }; }
      const items: Arg[] = [];
      let dict = false, comma = false;
      for (;;) {
        this.ws();
        if (this.eat(")")) break;
        const m = this.s.slice(this.p).match(/^(?:([\p{L}_][\p{L}\p{N}_-]*)|"((?:[^"\\]|\\.)*)")\s*:(?!=)/u);
        if (m) {
          this.p += m[0].length;
          this.ws();
          items.push({ name: m[1] ?? JSON.parse(`"${m[2]}"`), value: this.expr() });
          dict = true;
        } else if (this.eat("..")) items.push({ value: this.expr(), spread: true });
        else items.push({ value: this.expr() });
        this.ws();
        if (this.eat(",")) { comma = true; continue; }
        this.expect(")");
        break;
      }
      if (dict) return { k: "dict", items: items.map(a => [a.name!, a.value]) };
      if (items.length === 1 && !comma && !items[0].spread) return items[0].value;
      return { k: "array", items };
    }
    if (isIdStart(c)) {
      const name = this.ident();
      if (name === "none") return { k: "none" };
      if (name === "auto") return { k: "auto" };
      if (name === "true" || name === "false") return { k: "bool", v: name === "true" };
      return { k: "ident", name, pos };
    }
    if (embedded) this.fail("expected an expression after #");
    this.fail(this.eof() ? "unexpected end of the code" : `unexpected ${JSON.stringify(c)}`);
  }

  string(): string {
    this.expect("\"");
    let out = "";
    while (!this.eof() && this.c !== "\"") {
      if (this.c === "\\") {
        const n = this.s[this.p + 1];
        this.p += 2;
        if (n === "n") out += "\n"; else if (n === "t") out += "\t";
        else if (n === "u" && (this.c as string) === "{") { const end = this.s.indexOf("}", this.p); out += String.fromCodePoint(parseInt(this.s.slice(this.p + 1, end), 16)); this.p = end + 1; }
        else out += n;
      } else { out += this.c; this.p++; }
    }
    this.expect("\"");
    return out;
  }

  // ---------- math ----------
  equation(): Expr {
    const pos = this.p;
    this.expect("$");
    const start = this.p;
    const body = this.mathSeq(p => p.c === "$");
    const src = this.s.slice(start, this.p);
    this.expect("$");
    // Typst sets an equation as a block when it starts and ends with whitespace
    const block = src.length > 1 && isSpace(src[0]) && isSpace(src[src.length - 1]);
    return { k: "math", body, block, src, pos };
  }

  mathSeq(stop: (p: Parser) => boolean): MNode[] {
    const out: MNode[] = [];
    while (!this.eof() && !stop(this)) {
      if (this.at("//") || this.at("/*")) { this.comment(); continue; }
      if (isSpace(this.c)) {
        while (!this.eof() && isSpace(this.c)) this.p++;
        if (out.length && out[out.length - 1].k !== "space") out.push({ k: "space" });
        continue;
      }
      if (this.c === "/" ) {
        this.p++;
        while (out.length && out[out.length - 1].k === "space") out.pop();
        const num = out.pop() ?? { k: "seq", ch: [] };
        while (isSpace(this.c)) this.p++;
        const den = this.mathAttached(stop);
        out.push({ k: "frac", num: unparen(num), den: unparen(den) });
        continue;
      }
      out.push(this.mathAttached(stop));
    }
    while (out.length && out[out.length - 1].k === "space") out.pop();
    if (out.length && out[0].k === "space") out.shift();
    return out;
  }

  mathAttached(stop: (p: Parser) => boolean): MNode {
    let base = this.mathPrimary(stop, false);
    let b: MNode | undefined, t: MNode | undefined, primes = 0;
    for (;;) {
      if (this.c === "'" ) { this.p++; primes++; continue; }
      if (this.c === "_" && !b) { this.p++; b = unparen(this.mathScript(stop)); continue; }
      if (this.c === "^" && !t) { this.p++; t = unparen(this.mathScript(stop)); continue; }
      break;
    }
    if (!b && !t && !primes) return base;
    return { k: "attach", base, b, t, primes };
  }

  // an attachment: one primary; a call-like group right after a letter goes along (y_A(x))
  mathScript(stop: (p: Parser) => boolean): MNode {
    const node = this.mathPrimary(stop, true);
    if (node.k === "text" && !node.num && this.c === "(") {
      const group = this.mathPrimary(stop, true);
      return { k: "seq", ch: [node, group] };
    }
    return node;
  }

  mathPrimary(stop: (p: Parser) => boolean, script: boolean): MNode {
    const c = this.c;
    const pos = this.p;
    const OPEN: Record<string, string> = { "(": ")", "[": "]", "{": "}", "⌈": "⌉", "⌊": "⌋", "⟨": "⟩" };
    if (OPEN[c]) {
      this.p++;
      const close = OPEN[c];
      const body = this.mathSeq(p => p.c === close || stop(p));
      if (this.c === close) { this.p++; return { k: "lr", open: c, body, close }; }
      return { k: "lr", open: c, body, close: "" };
    }
    if (c === "\"") { const s = this.string(); return { k: "text", s, quoted: true }; }
    if (c === "#") {
      this.p++;
      const e = this.postfixMath();
      // as in markup, a semicolon ends the embedded expression and goes with it: `0 #s; 1` is one row
      this.eat(";");
      return { k: "code", e };
    }
    if (c === "\\") {
      const n = this.s[this.p + 1];
      if (n === undefined || isSpace(n)) { this.p++; return { k: "linebreak" }; }
      if (n === "u" && this.s[this.p + 2] === "{") { const end = this.s.indexOf("}", this.p); const ch = String.fromCodePoint(parseInt(this.s.slice(this.p + 3, end), 16)); this.p = end + 1; return { k: "text", s: ch }; }
      this.p += 2;
      return { k: "text", s: n };
    }
    if (c === "&") { this.p++; return { k: "align" }; }
    if (isDigit(c)) {
      const m = this.s.slice(this.p).match(/^\d+(\.\d+)?/)!;
      this.p += m[0].length;
      return { k: "text", s: m[0], num: true };
    }
    if (isMathIdStart(c)) {
      let n = 0;
      const rest = this.s.slice(this.p);
      const m = rest.match(/^[\p{L}][\p{L}\p{N}]*/u)!;
      n = m[0].length;
      if ([...m[0]].length === 1) {
        this.p += n;
        // f(x): a letter called like a function stays one atom, the numerator of f(x)/2
        if (this.c === "(" && !script) return { k: "call", callee: { k: "text", s: m[0] }, args: this.mathArgs(false), pos };
        return { k: "text", s: m[0] };
      }
      this.p += n;
      const fields: string[] = [];
      while (this.c === "." && isMathIdStart(this.s[this.p + 1] ?? "")) {
        const f = this.s.slice(this.p + 1).match(/^[\p{L}][\p{L}\p{N}]*/u)!;
        fields.push(f[0]);
        this.p += 1 + f[0].length;
      }
      const ident: MNode = { k: "ident", name: m[0], fields, pos };
      if (this.c === "(") return { k: "call", callee: ident, args: this.mathArgs(), pos };
      return ident;
    }
    for (const [from, to] of SHORTHANDS) {
      if (this.at(from)) { this.p += from.length; return { k: "text", s: to }; }
    }
    this.p += c.length > 0 ? [...this.s.slice(this.p, this.p + 2)][0].length : 1;
    return { k: "text", s: [...c][0] ?? c };
  }

  /** `#…` in math: a name with fields and calls, `#(…)`, `#{…}` or a string. */
  postfixMath(): Expr {
    let e = this.primary(true);
    for (;;) {
      if (this.c === "." && isIdStart(this.s[this.p + 1] ?? "")) { this.p++; e = { k: "field", target: e, name: this.identMath() }; continue; }
      if (this.c === "(") { const pos = this.p; e = { k: "call", callee: e, args: this.args(), pos }; continue; }
      if (this.c === "[" && e.k === "call") { e.args.push({ value: this.contentBlock() }); continue; }
      return e;
    }
  }
  identMath(): string { return this.ident(); }

  /** A call's arguments; `named`: `name: value` is a named argument (a call of a name, not of a letter). */
  mathArgs(named = true): MArg[] {
    this.expect("(");
    // the arguments of the current row; a `;` ends a row, and then every row is an array argument
    let row: MArg[] = [];
    const rows: MArg[][] = [];
    for (;;) {
      while (isSpace(this.c)) this.p++;
      if (this.eat(")")) break;
      const m = this.s.slice(this.p).match(/^([\p{L}][\p{L}\p{N}_-]*)\s*:(?![=:])/u);
      let name: string | undefined;
      if (m && named) { name = m[1]; this.p += m[0].length; }
      const body = this.mathSeq(p => p.c === "," || p.c === ";" || p.c === ")" || p.c === "$");
      const value: MNode = body.length === 1 ? body[0] : { k: "seq", ch: body };
      row.push({ name, value });
      if (this.eat(";")) { rows.push(row); row = []; continue; }
      if (this.eat(",")) continue;
      if (this.c === "$" || this.eof()) this.fail("unclosed call in math");
    }
    // a trailing comma leaves an empty argument out
    const empty = (a: MArg) => { const v = a.value as MNode; return v.k === "seq" && v.ch.length === 0 && !a.name; };
    row = row.filter(a => !empty(a));
    if (rows.length) {
      if (row.length) rows.push(row);
      return rows.map(r => ({ value: r.filter(a => !empty(a)).map(a => a.value as MNode) }));
    }
    return row;
  }
}

/** A fraction's parts lose the parentheses they're written with: (a + b)/c. */
export function unparen(n: MNode): MNode {
  if (n.k === "lr" && n.open === "(" && n.close === ")") return n.body.length === 1 ? n.body[0] : { k: "seq", ch: n.body };
  return n;
}

function trimMarkup(m: Markup[]): Markup[] {
  while (m.length && m[0].k === "space") m.shift();
  while (m.length && m[m.length - 1].k === "space") m.pop();
  return m;
}

export function parseDocument(src: string): Markup[] {
  return new Parser(src).markup();
}

export function parseMath(src: string): MNode[] {
  const p = new Parser(src);
  return p.mathSeq(() => false);
}
