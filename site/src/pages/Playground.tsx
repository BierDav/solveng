// A whole worksheet to play with: the source on the left, the calculated document on the right, shareable by link.
import { createEffect, createSignal, For, Show } from "solid-js";
import Editor from "../components/Editor";
import Header from "../components/Header";
import Output from "../components/Output";
import { allExamples } from "../content/examples";
import { runDocument, type RunResponse } from "../engine/client";

const START = `#import "@preview/solveng:2.0.0": *
#import units: V, Ω, mA, mW, kHz, nF, kΩ
#import constants: pi
#import modifiers: solve, digits
#show: math

= Voltage divider with a load

$U_0 := 12 #V$, $R_1 := 4.7 kΩ$, $R_2 := 2.2 kΩ$
$U_2 := U_0 R_2 / (R_1 + R_2) => #V$
$I := U_0 / (R_1 + R_2) => mA$
$P := U_2 I => mW, digits(3)$

= Low-pass corner

$C := 10 nF$
$f_c := 1 / (2 pi R_2 C) => kHz$
$1/sqrt(2) = 1/sqrt(1 + (f/f_c)^2) -> solve(f)$

#graph($H(f: kHz) = 1/sqrt(1 + (f/f_c)^2)$, mark: $f_c$)
`;

function fromHash(): string | null {
  const m = location.hash.match(/code=([^&]+)/);
  if (!m) return null;
  try { return decodeURIComponent(escape(atob(decodeURIComponent(m[1])))); } catch { return null; }
}

export default function Playground() {
  const [code, setCode] = createSignal(fromHash() ?? START);
  const [result, setResult] = createSignal<RunResponse | null>(null);
  const [error, setError] = createSignal<string | null>(null);
  const [busy, setBusy] = createSignal(false);
  const [shared, setShared] = createSignal(false);
  const examples = allExamples();

  let seq = 0;
  createEffect(code, src => {
    const id = ++seq;
    const t = setTimeout(() => {
      setBusy(true);
      runDocument(src).then(
        r => { if (id === seq) { setResult(r); setError(null); setBusy(false); } },
        e => { if (id === seq) { setError(String(e.message ?? e)); setBusy(false); } },
      );
    }, 180);
    return () => clearTimeout(t);
  });

  const share = async () => {
    const url = location.origin + location.pathname + "#code=" + encodeURIComponent(btoa(unescape(encodeURIComponent(code()))));
    history.replaceState(null, "", url);
    try { await navigator.clipboard.writeText(url); setShared(true); setTimeout(() => setShared(false), 1500); } catch { /* no clipboard */ }
  };

  return (
    <>
      <Header />
      <div class="pg">
        <div class="pg-bar">
          <span class="pg-title display">Playground</span>
          <select class="pg-select" onChange={e => { const v = e.currentTarget.value; if (v === "__start") setCode(START); else { const ex = examples.find(x => x.where === v); if (ex) setCode(ex.code); } e.currentTarget.value = ""; }}>
            <option value="">Load an example…</option>
            <option value="__start">Voltage divider (start)</option>
            <For each={examples}>{x => <option value={x.where}>{x.example.title ? `${x.example.title} — ` : ""}{x.where}</option>}</For>
          </select>
          <span class="spacer" />
          <span class={["bench-status", { busy: busy() }]}>
            <Show when={busy()} fallback={<Show when={result()}>{r => <>{r().problems.length ? `${r().problems.length} problems` : "calculated"} · {Math.max(1, Math.round(r().ms))} ms</>}</Show>}>calculating…</Show>
          </span>
          <button class="btn small" onClick={share}>{shared() ? "✓ Link copied" : "Share"}</button>
        </div>
        <div class="pg-split">
          <div class="pg-editor">
            <Editor value={code()} onInput={setCode} minRows={30} label="Worksheet source" />
          </div>
          <div class="pg-out paper">
            <Show when={error()}>{e => <div class="sheet-err">⚠ {e()}</div>}</Show>
            <Show when={result()} fallback={<div class="bench-loading"><span /><span /><span /></div>}>
              {r => (
                <>
                  <Output blocks={r().blocks} />
                  <Show when={r().definitions.length}>
                    <details class="pg-defs">
                      <summary>{r().definitions.length} definitions</summary>
                      <table>
                        <tbody>
                          <For each={r().definitions}>{d => <tr><td class="mono">{d.name}</td><td>{d.kind}</td><td class="mono">{d.value}</td></tr>}</For>
                        </tbody>
                      </table>
                    </details>
                  </Show>
                </>
              )}
            </Show>
          </div>
        </div>
      </div>
    </>
  );
}
