// Runs documents through Typst with the package and through the site's interpreter, and compares what they defined and
// which problems they reported. Documents: the examples of README.md and docs/reference.md, and the site's examples.
//   bun scripts/conformance.ts [--site] [filter]      (needs typst on the PATH; TYPST=… picks another)
//   bun scripts/conformance.ts --file docs.typ        (documents in one file, separated by lines of ---)
import { execFile } from "node:child_process";
import { mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { cpus } from "node:os";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { loadInterp } from "./engine-node";

const ROOT = fileURLToPath(new URL("../../", import.meta.url));
const TYPST = process.env.TYPST ?? "typst";
const IMPORT = '#import "@preview/solveng:2.0.0": *';

type Doc = { where: string; code: string; expectError?: boolean; variants?: string[] };

function readmeExamples(): Doc[] {
  const out: Doc[] = [];
  for (const path of ["README.md", "docs/reference.md"]) {
    const text = readFileSync(ROOT + path, "utf8");
    for (const m of text.matchAll(/^(`{3,})typ\n([\s\S]*?)^\1$/gm)) {
      let code = m[2];
      if (/#import "(?!@preview\/solveng)/.test(code)) continue;
      if (!code.includes('"@preview/solveng:')) code = IMPORT + "\n" + code;
      out.push({ where: `${path}:${text.slice(0, m.index).split("\n").length}`, code });
    }
  }
  return out;
}

async function siteExamples(): Promise<Doc[]> {
  const { allExamples, fill } = await import("../src/content/examples");
  return allExamples().map(e => {
    // controls at their extremes too: an example must work wherever its sliders go
    const variants: string[] = [];
    for (const c of e.example.controls ?? []) {
      if (c.kind === "slider") for (const v of [c.min, c.max]) variants.push(fill(e.example.code, e.example.controls, { [c.name]: v }));
      if (c.kind === "select") for (const v of c.options) variants.push(fill(e.example.code, e.example.controls, { [c.name]: v }));
      if (c.kind === "toggle") variants.push(fill(e.example.code, e.example.controls, { [c.name]: !c.value }));
    }
    return { where: e.where, code: e.code, expectError: e.expectError, variants };
  });
}

const norm = (s: string) => s.replace(/\s+/g, " ").trim();

type Want = { defined: string[]; errors: string[]; fatal: string | null };

function typst(code: string, dir: string): Promise<Want> {
  const doc = code.replace(/"@preview\/solveng:[\d.]+"/g, `"${ROOT}lib.typ"`);
  const file = join(dir, "doc.typ");
  writeFileSync(file, "#set page(height: auto)\n" + doc);
  return new Promise(resolve => {
    execFile(TYPST, ["query", "--root", "/", file, "<solveng>", "--field", "value"], { encoding: "utf8", maxBuffer: 1 << 26 }, (err, out, stderr) => {
      if (err) return resolve({ defined: [], errors: [], fatal: String(stderr || err.message).split("\n").slice(0, 4).join(" ") });
      const notes = JSON.parse(out) as any[];
      const blocks = notes.filter(n => n.kind === "block");
      const defined = blocks.length ? blocks[blocks.length - 1].defined.map((d: any) => `${d.name} = ${norm(d.value)}`) : [];
      const errors = notes.filter(n => n.kind === "error").map(n => norm(n.message));
      resolve({ defined, errors, fatal: null });
    });
  });
}

const args = process.argv.slice(2);
const fileArg = args.indexOf("--file");
const fileDocs = (path: string): Doc[] => readFileSync(path, "utf8").split(/^---$/m).map((code, i) => ({ where: `${path}#${i + 1}`, code: code.trim() + "\n" }));
const docs = [...(fileArg >= 0 ? fileDocs(args.splice(fileArg, 2)[1]) : args.includes("--site") ? await siteExamples() : readmeExamples())].filter(d => !args.find(a => !a.startsWith("--")) || d.where.includes(args.find(a => !a.startsWith("--"))!));
const it = await loadInterp();
// Typst runs in parallel, one directory per document; the interpreter then checks each in order
const jobs = Math.max(2, Math.min(8, cpus().length - 1));
const wants: Promise<Want>[] = new Array(docs.length);
let nextJob = 0;
const slots = Array.from({ length: jobs }, () => (async () => {
  while (nextJob < docs.length) { const k = nextJob++; wants[k] = typst(docs[k].code, mkdtempSync(join(tmpdir(), "solveng-site-"))); await wants[k]; }
})());
let failed = 0;
for (let k = 0; k < docs.length; k++) {
  const d = docs[k];
  while (!wants[k]) await new Promise(r => setTimeout(r, 20));
  const want = await wants[k];
  const got = it.run(d.code);
  const gotDefined = got.definitions.map(x => `${x.name} = ${norm(x.value)}`);
  const gotErrors = got.problems.map(p => norm(p.message.replace(/^solveng: /, "")));
  const same = (a: string[], b: string[]) => a.length === b.length && a.every((x, i) => x === b[i]);
  const problems: string[] = [];
  if (want.fatal) problems.push(`typst fails: ${want.fatal}`);
  if (!same(want.defined, gotDefined)) problems.push(`defined:\n    typst: ${want.defined.join(" | ")}\n    site:  ${gotDefined.join(" | ")}`);
  const wantErrors = [...new Set(want.errors)], gotErrs = [...new Set(gotErrors)];
  if (!same(wantErrors, gotErrs)) problems.push(`errors:\n    typst: ${wantErrors.join(" | ")}\n    site:  ${gotErrs.join(" | ")}`);
  if (d.expectError !== undefined) {
    if (d.expectError && !gotErrors.length) problems.push("expects a problem, but the site reports none");
    if (!d.expectError && gotErrors.length) problems.push("reports a problem but isn't marked expectError");
  }
  for (const v of d.variants ?? []) {
    const r = it.run(v);
    if (!d.expectError && r.problems.length) problems.push(`with a control moved: ${r.problems.map(p => p.message).join(" | ")}`);
  }
  if (problems.length) { failed++; console.log(`✗ ${d.where}\n  ${problems.join("\n  ")}`); }
  else console.log(`✓ ${d.where}`);
}
await Promise.all(slots);
console.log(`\n${docs.length - failed}/${docs.length} documents agree`);
process.exit(failed ? 1 : 0);
