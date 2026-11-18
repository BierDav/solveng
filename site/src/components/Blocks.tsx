// A page's blocks: prose, live examples, callouts, parameter tables, quizzes and widgets.
import { createSignal, For, Match, Show, Switch } from "solid-js";
import type { Block, Quiz as QuizT } from "../content/types";
import { highlight } from "../lib/highlight";
import { inline, md, slugify } from "../lib/md";
import Workbench from "./Workbench";
import OperatorCards from "./widgets/OperatorCards";
import OptionExplorer from "./widgets/OptionExplorer";
import UnitTable from "./widgets/UnitTable";

export default function Blocks(props: { blocks: Block[] }) {
  return (
    <For each={props.blocks}>
      {b => (
        <Switch fallback={null}>
          <Match when={"md" in b && b}>{x => <div class="prose" innerHTML={md((x() as { md: string }).md)} />}</Match>
          <Match when={"h2" in b && b}>{x => { const t = (x() as { h2: string }).h2; return <h2 id={slugify(t)} class="anchor"><a href={`#${slugify(t)}`} innerHTML={inline(t)} /></h2>; }}</Match>
          <Match when={"h3" in b && b}>{x => { const t = (x() as { h3: string }).h3; return <h3 id={slugify(t)} class="anchor"><a href={`#${slugify(t)}`} innerHTML={inline(t)} /></h3>; }}</Match>
          <Match when={"example" in b && b}>{x => <Workbench example={(x() as any).example} />}</Match>
          <Match when={"code" in b && b}>{x => <pre class="code-block" innerHTML={highlight((x() as { code: string }).code)} />}</Match>
          <Match when={"callout" in b && b}>
            {x => {
              const c = x() as Extract<Block, { callout: string }>;
              const icon = { tip: "💡", note: "ℹ️", warning: "⚠️", fun: "🎲" }[c.callout];
              return (
                <aside class={["callout", c.callout]}>
                  <div class="callout-icon">{icon}</div>
                  <div>
                    <Show when={c.title}><div class="callout-title">{c.title}</div></Show>
                    <div class="prose" innerHTML={md(c.md)} />
                  </div>
                </aside>
              );
            }}
          </Match>
          <Match when={"params" in b && b}>
            {x => (
              <div class="params">
                <For each={(x() as Extract<Block, { params: unknown }>).params}>
                  {p => (
                    <div class="param">
                      <div class="param-head">
                        <code class="param-name">{p.name}</code>
                        <span class="param-type">{p.type}</span>
                        <Show when={p.default}><span class="param-default">default <code>{p.default}</code></span></Show>
                      </div>
                      <div class="prose" innerHTML={md(p.md)} />
                    </div>
                  )}
                </For>
              </div>
            )}
          </Match>
          <Match when={"table" in b && b}>
            {x => {
              const t = (x() as Extract<Block, { table: unknown }>).table;
              return (
                <div class="table-wrap">
                  <table>
                    <thead><tr><For each={t.head}>{h => <th innerHTML={inline(h)} />}</For></tr></thead>
                    <tbody><For each={t.rows}>{r => <tr><For each={r}>{c => <td innerHTML={inline(c)} />}</For></tr>}</For></tbody>
                  </table>
                </div>
              );
            }}
          </Match>
          <Match when={"quiz" in b && b}>{x => <Quiz quiz={(x() as { quiz: QuizT }).quiz} />}</Match>
          <Match when={"widget" in b && b}>
            {x => {
              const w = (x() as Extract<Block, { widget: string }>).widget;
              return (
                <Switch fallback={<div class="callout note">This interactive widget is coming soon.</div>}>
                  <Match when={w === "operator-cards"}><OperatorCards /></Match>
                  <Match when={w === "option-explorer"}><OptionExplorer /></Match>
                  <Match when={w === "unit-table"}><UnitTable /></Match>
                </Switch>
              );
            }}
          </Match>
        </Switch>
      )}
    </For>
  );
}

function Quiz(props: { quiz: QuizT }) {
  const [picked, setPicked] = createSignal<number | null>(null);
  return (
    <div class={["quiz", { answered: picked() !== null }]}>
      <div class="quiz-tag">Quick check</div>
      <div class="quiz-q prose" innerHTML={md(props.quiz.question)} />
      <Show when={props.quiz.code}><pre class="code-block" innerHTML={highlight(props.quiz.code!)} /></Show>
      <div class="quiz-options">
        <For each={props.quiz.options}>
          {(o, i) => (
            <button
              class={["quiz-opt", { right: picked() !== null && i() === props.quiz.answer, wrong: picked() === i() && i() !== props.quiz.answer }]}
              disabled={picked() !== null}
              onClick={() => setPicked(i())}
            >
              <span class="quiz-letter">{String.fromCharCode(65 + i())}</span>
              <span innerHTML={inline(o)} />
            </button>
          )}
        </For>
      </div>
      <Show when={picked() !== null}>
        <div class={["quiz-explain", picked() === props.quiz.answer ? "right" : "wrong"]}>
          <strong>{picked() === props.quiz.answer ? "Correct! " : "Not quite. "}</strong>
          <span innerHTML={inline(props.quiz.explain)} />
          <button class="chip-btn" onClick={() => setPicked(null)}>Try again</button>
        </div>
      </Show>
    </div>
  );
}
