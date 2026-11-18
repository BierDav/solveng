// A live example: the source, its controls, and the document solveng calculates from it – recalculated as you type
// or drag.
import { createEffect, createMemo, createSignal, For, Match, Show, Switch } from "solid-js";
import { fill } from "../content/examples";
import type { Example } from "../content/types";
import { runDocument, type RunResponse } from "../engine/client";
import { inline } from "../lib/md";
import { href } from "../lib/router";
import Editor from "./Editor";
import Output from "./Output";

export function playgroundLink(code: string) {
  return href("/playground") + "#code=" + encodeURIComponent(btoa(unescape(encodeURIComponent(code))));
}

export default function Workbench(props: { example: Example; compact?: boolean }) {
  const initial = () => props.example.code;
  const [code, setCode] = createSignal(props.example.code);
  const [values, setValues] = createSignal<Record<string, unknown>>(Object.fromEntries((props.example.controls ?? []).map(c => [c.name, c.value])));
  const [result, setResult] = createSignal<RunResponse | null>(null);
  const [error, setError] = createSignal<string | null>(null);
  const [busy, setBusy] = createSignal(false);
  const [copied, setCopied] = createSignal(false);
  const doc = createMemo(() => fill(code(), props.example.controls, values()));

  let seq = 0;
  let timer: ReturnType<typeof setTimeout> | undefined;
  createEffect(doc, source => {
    clearTimeout(timer);
    const id = ++seq;
    timer = setTimeout(() => {
      setBusy(true);
      runDocument(source).then(
        r => { if (id === seq) { setResult(r); setError(null); setBusy(false); } },
        e => { if (id === seq) { setError(String(e.message ?? e)); setBusy(false); } },
      );
    }, result() ? 140 : 0);
    return () => clearTimeout(timer);
  });

  const problems = () => result()?.problems.length ?? 0;
  const copy = async () => {
    try { await navigator.clipboard.writeText(doc()); setCopied(true); setTimeout(() => setCopied(false), 1400); } catch { /* no clipboard */ }
  };

  return (
    <figure class={["bench", { compact: props.compact }]}>
      <Show when={props.example.title}>
        <figcaption class="bench-title">{props.example.title}</figcaption>
      </Show>
      <div class="bench-tools">
        <span class={["bench-status", { busy: busy(), bad: problems() > 0 && !props.example.expectError }]}>
          <Show when={busy()} fallback={<Show when={result()} fallback="starting engine…">{r => <>{problems() ? `${problems()} problem${problems() > 1 ? "s" : ""}` : "calculated"} · {Math.max(1, Math.round(r().ms))} ms</>}</Show>}>
            calculating…
          </Show>
        </span>
        <span class="spacer" />
        <Show when={code() !== initial()}>
          <button class="chip-btn" onClick={() => setCode(initial())} title="Undo your changes">↺ Reset</button>
        </Show>
        <button class="chip-btn" onClick={copy} title="Copy the document">{copied() ? "✓ Copied" : "Copy"}</button>
        <a class="chip-btn" href={playgroundLink(doc())} title="Open in the playground">Playground ↗</a>
      </div>
      <Editor value={code()} onInput={setCode} label={props.example.title ?? "Example source"} />
      <Show when={props.example.controls?.length}>
        <div class="bench-controls">
          <For each={props.example.controls}>
            {c => (
              <label class="control">
                <span class="control-label">{c.label ?? c.name}</span>
                <Switch>
                  <Match when={c.kind === "slider" && c}>
                    {s => (
                      <>
                        <input type="range" min={s().min} max={s().max} step={s().step} value={values()[s().name] as number}
                          onInput={e => setValues(v => ({ ...v, [s().name]: Number(e.currentTarget.value) }))} />
                        <output class="control-value">{String(values()[s().name])}{s().suffix ?? ""}</output>
                      </>
                    )}
                  </Match>
                  <Match when={c.kind === "select" && c}>
                    {s => (
                      <select value={values()[s().name] as string} onChange={e => setValues(v => ({ ...v, [s().name]: e.currentTarget.value }))}>
                        <For each={s().options}>{o => <option value={o}>{o}</option>}</For>
                      </select>
                    )}
                  </Match>
                  <Match when={c.kind === "toggle" && c}>
                    {s => (
                      <button class={["toggle", { on: !!values()[s().name] }]} role="switch" aria-checked={values()[s().name] ? "true" : "false"}
                        onClick={() => setValues(v => ({ ...v, [s().name]: !v[s().name] }))}><i /></button>
                    )}
                  </Match>
                </Switch>
              </label>
            )}
          </For>
        </div>
      </Show>
      <div class={["bench-out", "paper", { stale: busy() }]}>
        <Show when={error()}>{e => <div class="sheet-err">⚠ {e()}</div>}</Show>
        <Show when={result()} fallback={<div class="bench-loading"><span /><span /><span /></div>}>
          {r => <Output blocks={r().blocks} />}
        </Show>
      </div>
      <Show when={props.example.caption}>
        <div class="bench-caption" innerHTML={inline(props.example.caption!)} />
      </Show>
    </figure>
  );
}
