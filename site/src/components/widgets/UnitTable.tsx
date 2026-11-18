// Every unit binding of solveng's units module, searchable: its name, how it's shown, and what it is in SI.
import { createMemo, createSignal, For, Show } from "solid-js";
import unitWords from "../../../../src/unit-words.typ?raw";

type Unit = { name: string; display: string; si: string; dim: string; base: boolean };

const DIMS: Record<string, string> = {
  "": "dimensionless", "m": "length", "kg": "mass", "s": "time", "A": "current", "K": "temperature", "mol": "amount", "cd": "luminous intensity",
  "m^2": "area", "m^3": "volume", "s^-1": "frequency", "m·s^-1": "velocity", "m·s^-2": "acceleration",
  "m·kg·s^-2": "force", "m^-1·kg·s^-2": "pressure", "m^2·kg·s^-2": "energy", "m^2·kg·s^-3": "power", "A·s": "charge",
  "A^-1·m^2·kg·s^-3": "voltage", "A^-2·m^2·kg·s^-3": "resistance", "A^2·m^-2·kg^-1·s^3": "conductance",
  "A^2·m^-2·kg^-1·s^4": "capacitance", "A^-2·m^2·kg·s^-2": "inductance", "A^-1·kg·s^-2": "magnetic flux density", "A^-1·m^2·kg·s^-2": "magnetic flux",
};

const SUP: Record<string, string> = { "-": "⁻", "0": "⁰", "1": "¹", "2": "²", "3": "³", "4": "⁴", "5": "⁵", "6": "⁶", "7": "⁷", "8": "⁸", "9": "⁹" };
const sup = (s: string) => [...s].map(c => SUP[c] ?? c).join("");

function parse(): Unit[] {
  const out: Unit[] = [];
  const re = /^#let ([^\s=]+) = (base-unit|unit)\((.*)\)$/gm;
  for (const m of unitWords.matchAll(re)) {
    const [, name, kind, args] = m;
    const display = args.match(/upright\("([^"]*)"\)/)?.[1] ?? name;
    if (kind === "base-unit") {
      out.push({ name, display, si: display, dim: DIMS[display] ?? "base", base: true });
      continue;
    }
    const expr = args.match(/^\$(.*?)\$/)?.[1] ?? "";
    const frac = expr.match(/frac\((\d+), (\d+)\)/);
    const factor = frac ? Number(frac[1]) / Number(frac[2]) : Number(expr.match(/^[\d.]+/)?.[0] ?? 1);
    const parts = [...expr.matchAll(/#(\w+)\^\((-?\d+)\)/g)].map(p => [p[1], p[2]] as const);
    const key = parts.map(([u, e]) => (e === "1" ? u : `${u}^${e}`)).join("·");
    const units = parts.map(([u, e]) => (e === "1" ? u : u + sup(e))).join("·");
    const num = factor === 1 ? "" : Math.abs(Math.log10(factor)) >= 4 ? factor.toExponential().replace(/e\+?(-?\d+)/, (_, e) => ` · 10${sup(e)}`).replace(/^1 · /, "") : String(Number(factor.toPrecision(10)));
    out.push({ name, display, si: [num, units].filter(Boolean).join(" ") || "1", dim: DIMS[key] ?? (key ? key.replace(/\^(-?\d+)/g, (_, e) => sup(e)) : "dimensionless"), base: false });
  }
  return out;
}

const UNITS = parse();
const COMMON = new Set(["m", "kg", "s", "A", "K", "mol", "cd", "V", "Ω", "W", "J", "C", "F", "H", "S", "Hz", "N", "Pa", "T", "Wb", "min", "h", "Ah", "Wh", "eV", "l", "L", "rad", "deg", "degC", "percent"]);

export default function UnitTable() {
  const [q, setQ] = createSignal("");
  const [all, setAll] = createSignal(false);
  const rows = createMemo(() => {
    const s = q().trim().toLowerCase();
    return UNITS.filter(u => (all() || s || COMMON.has(u.name)) && (!s || u.name.toLowerCase().includes(s) || u.display.toLowerCase().includes(s) || u.dim.includes(s)));
  });
  return (
    <div class="unit-table">
      <div class="ut-bar">
        <input placeholder={`Search ${UNITS.length} units — by name, symbol or quantity (“pressure”)…`} value={q()} onInput={e => setQ(e.currentTarget.value)} />
        <label class="ut-all"><input type="checkbox" checked={all()} onChange={e => setAll(e.currentTarget.checked)} /> with prefixes</label>
      </div>
      <div class="table-wrap ut-wrap">
        <table>
          <thead><tr><th>import</th><th>in math</th><th>shown</th><th>in SI</th><th>quantity</th></tr></thead>
          <tbody>
            <For each={rows().slice(0, 400)}>
              {u => (
                <tr>
                  <td><code>{u.name}</code></td>
                  <td><code>{[...u.name].length === 1 ? "#" + u.name : u.name.includes("_") || u.name.includes("-") ? "#" + u.name : u.name}</code></td>
                  <td class="ut-disp">{u.display}</td>
                  <td class="ut-si">{u.base ? <em>base unit</em> : u.si}</td>
                  <td class="muted">{u.dim}</td>
                </tr>
              )}
            </For>
          </tbody>
        </table>
      </div>
      <Show when={rows().length > 400}><div class="muted ut-more">{rows().length - 400} more — narrow the search.</div></Show>
      <div class="muted ut-more">{rows().length} shown · import with <code>#import units: {rows().slice(0, 3).map(u => u.name).join(", ") || "m"}</code></div>
    </div>
  );
}
