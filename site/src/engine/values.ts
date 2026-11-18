// Typst values, as the interpreter has them.
import type { C } from "./content";
import type { Markup } from "./syntax";

export class Auto { toString() { return "auto"; } }
export const AUTO = new Auto();

export type V =
  | null | Auto | boolean | number | string
  | V[] | Dict | Fn | Content | Equation | MathV | Length | Label | Regex | Color | Selector | Module | Sym | IntV | Bytes;

/** An integer, where Typst tells int from float (2 vs 2.0). Plain JS numbers are ints when whole. */
export class IntV { constructor(public v: number) {} }
export class Bytes { constructor(public b: Uint8Array) {} }

export class Dict {
  constructor(public m: Map<string, V> = new Map()) {}
  get(k: string) { return this.m.get(k); }
}

export class Args {
  constructor(public pos: V[] = [], public named: Map<string, V> = new Map()) {}
  take(name: string, def: V = AUTO): V {
    if (!this.named.has(name)) return def;
    const v = this.named.get(name)!;
    this.named.delete(name);
    return v;
  }
}

export class Fn {
  constructor(public name: string, public impl: (args: Args) => V, public bound?: Args, public fields: Record<string, V> = {}) {}
  call(args: Args): V {
    if (!this.bound) return this.impl(args);
    const named = new Map(this.bound.named);
    for (const [k, v] of args.named) named.set(k, v);
    return this.impl(new Args([...this.bound.pos, ...args.pos], named));
  }
  with(args: Args): Fn {
    const b = this.bound ?? new Args();
    const named = new Map(b.named);
    for (const [k, v] of args.named) named.set(k, v);
    return new Fn(this.name, this.impl, new Args([...b.pos, ...args.pos], named), this.fields);
  }
}

/** What realizing content does: output, state changes – in document order. */
export type Item =
  | { k: "markup"; nodes: Markup[]; scope: Scope }
  | { k: "text"; s: string }
  | { k: "equation"; eq: Equation }
  | { k: "run"; run: (r: any) => void };

export class Content {
  constructor(public items: Item[]) {}
  static join(parts: V[]): Content {
    const items: Item[] = [];
    for (const p of parts) {
      if (p === null) continue;
      if (p instanceof Content) items.push(...p.items);
      else if (p instanceof Equation) items.push({ k: "equation", eq: p });
      else if (typeof p === "string" || typeof p === "number") items.push({ k: "text", s: String(p) });
      else items.push({ k: "run", run: (r: any) => r.place(p) });
    }
    return new Content(items);
  }
}

export class Equation {
  constructor(public body: C, public block: boolean, public src: string) {}
}

/** Math content outside an equation: what std.math.upright("s") and friends return. */
export class MathV { constructor(public c: C) {} }
export class Length { constructor(public v: number, public unit: string) {} }
export class Label { constructor(public name: string) {} }
export class Regex { constructor(public re: RegExp) {} }
export class Color { constructor(public hex: string) {} }
export class Selector { constructor(public kind: "heading", public level?: number) {} }
export class Sym { constructor(public s: string, public path: string) {} }

export class Module {
  constructor(public name: string, private lookup: (name: string) => V | undefined, public names: () => string[]) {}
  get(name: string): V | undefined { return this.lookup(name); }
}

export class Scope {
  vars = new Map<string, V | (() => V)>();
  constructor(public parent?: Scope) {}
  get(name: string): V | undefined {
    for (let s: Scope | undefined = this; s; s = s.parent) {
      if (s.vars.has(name)) {
        const v = s.vars.get(name)!;
        if (typeof v === "function") { const r = (v as () => V)(); s.vars.set(name, r); return r; }
        return v as V;
      }
    }
    return undefined;
  }
  set(name: string, v: V | (() => V)) { this.vars.set(name, v); }
}

export class TypstError extends Error {
  constructor(message: string, public pos?: number) { super(message); }
}

export function typeName(v: V): string {
  if (v === null) return "none";
  if (v instanceof Auto) return "auto";
  if (typeof v === "boolean") return "bool";
  if (typeof v === "number") return Number.isInteger(v) ? "int" : "float";
  if (v instanceof IntV) return "int";
  if (typeof v === "string") return "str";
  if (Array.isArray(v)) return "array";
  if (v instanceof Dict) return "dictionary";
  if (v instanceof Fn) return "function";
  if (v instanceof Content || v instanceof Equation || v instanceof MathV) return "content";
  if (v instanceof Length) return "length";
  if (v instanceof Label) return "label";
  if (v instanceof Regex) return "regex";
  if (v instanceof Color) return "color";
  if (v instanceof Selector) return "selector";
  if (v instanceof Module) return "module";
  if (v instanceof Sym) return "symbol";
  return "value";
}
