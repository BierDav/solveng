// Every example of the documentation as the document it runs, with its controls at their initial values.
import { pages } from "./index";
import type { Control, Example } from "./types";

export function fill(code: string, controls: Control[] | undefined, values?: Record<string, unknown>): string {
  if (!controls) return code;
  return code.replace(/\{\{(\w[\w-]*)\}\}/g, (all, name) => {
    const c = controls.find(c => c.name === name);
    if (!c) return all;
    const v = values?.[name] ?? c.value;
    if (c.kind === "toggle") return v ? c.on : c.off;
    return String(v);
  });
}

export function allExamples(): { where: string; code: string; expectError: boolean; example: Example }[] {
  const out = [];
  for (const p of pages) {
    let n = 0;
    for (const b of p.blocks) {
      if ("example" in b) out.push({ where: `${p.slug}#${++n}`, code: fill(b.example.code, b.example.controls), expectError: !!b.example.expectError, example: b.example });
    }
  }
  return out;
}
