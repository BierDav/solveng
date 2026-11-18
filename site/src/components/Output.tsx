// A calculated document, as the engine's interpreter laid it out: paragraphs with inline math, display equations,
// graphs, lists of definitions and problems.
import { For, Match, Switch } from "solid-js";
import type { Block } from "../engine/interp";
import { highlight } from "../lib/highlight";
import Graph from "./Graph";

export default function Output(props: { blocks: Block[] }) {
  return (
    <div class="sheet">
      <For each={props.blocks}>
        {b => (
          <Switch>
            <Match when={b.t === "p" && b}>{p => <p class="sheet-p" innerHTML={p().html} />}</Match>
            <Match when={b.t === "h" && b}>{h => <div class={["sheet-h", `l${h().level}`]} innerHTML={h().html} />}</Match>
            <Match when={b.t === "eq" && b}>{e => <div class="sheet-eq" innerHTML={e().html} />}</Match>
            <Match when={b.t === "err" && b}>{e => <div class="sheet-err" innerHTML={e().html} />}</Match>
            <Match when={b.t === "list" && b}>{l => <ul class="sheet-list"><li innerHTML={l().html} /></ul>}</Match>
            <Match when={b.t === "raw" && b}>{r => <pre class="sheet-raw" innerHTML={r().lang === "typ" || r().lang === "typst" ? highlight(r().code) : escapeHtml(r().code)} />}</Match>
            <Match when={b.t === "graph" && b}>{g => <Graph graph={g().graph} />}</Match>
            <Match when={b.t === "defs" && b}>
              {d => (
                <div class={["sheet-defs", d().style]}>
                  <For each={d().rows}>
                    {r => (
                      <div class="def">
                        <span class="def-name" innerHTML={r.name} />
                        <span class="def-value" innerHTML={r.value} />
                      </div>
                    )}
                  </For>
                </div>
              )}
            </Match>
          </Switch>
        )}
      </For>
    </div>
  );
}

const escapeHtml = (s: string) => s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
