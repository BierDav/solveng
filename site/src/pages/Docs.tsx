// The documentation: sidebar of every page, the page itself, and what's on it.
import { createMemo, createSignal, For, Show } from "solid-js";
import Blocks from "../components/Blocks";
import Header, { REPO } from "../components/Header";
import { bySlug, GROUPS, pages } from "../content";
import type { Page } from "../content/types";
import { highlight } from "../lib/highlight";
import { inline, slugify } from "../lib/md";
import { Link, path } from "../lib/router";

const KIND: Record<string, string> = { functions: "function", modifiers: "modifier", operations: "operation", guides: "guide", start: "start", reference: "reference" };

function Sidebar(props: { current: string; onNavigate: () => void }) {
  return (
    <nav class="sidebar" aria-label="Documentation">
      <For each={GROUPS}>
        {g => {
          const list = pages.filter(p => p.group === g.id);
          const sections: { name: string | undefined; items: Page[] }[] = [];
          for (const p of list) {
            const last = sections[sections.length - 1];
            if (last && last.name === p.section) last.items.push(p); else sections.push({ name: p.section, items: [p] });
          }
          return (
            <Show when={list.length}>
              <div class="side-group">
                <div class="side-title">{g.title}</div>
                <For each={sections}>
                  {s => (
                    <>
                      <Show when={s.name}><div class="side-section">{s.name}</div></Show>
                      <For each={s.items}>
                        {p => (
                          <Link to={`/docs/${p.slug}`} class={["side-link", { active: props.current === p.slug, code: g.id !== "start" && g.id !== "guides" && g.id !== "reference" }]} onClick={props.onNavigate}>
                            {g.id === "functions" || g.id === "operations" || g.id === "modifiers" ? p.slug.split("/").pop() : p.title}
                          </Link>
                        )}
                      </For>
                    </>
                  )}
                </For>
              </div>
            </Show>
          );
        }}
      </For>
    </nav>
  );
}

export default function Docs() {
  const slug = createMemo(() => path().replace(/^\/docs\/?/, "").replace(/\/$/, "") || "introduction");
  const page = createMemo(() => bySlug.get(slug()));
  const [menu, setMenu] = createSignal(false);
  const index = createMemo(() => pages.findIndex(p => p.slug === slug()));
  const prev = () => (index() > 0 ? pages[index() - 1] : undefined);
  const next = () => (index() >= 0 && index() < pages.length - 1 ? pages[index() + 1] : undefined);
  const toc = createMemo(() => (page()?.blocks ?? []).flatMap(b => ("h2" in b ? [{ t: b.h2, l: 2 }] : "h3" in b ? [{ t: b.h3, l: 3 }] : [])));
  const file = () => {
    const p = page();
    if (!p) return "";
    const g = p.group === "operations" ? `operations-${({ Elementary: "elementary", Complex: "elementary", Signals: "signals", Transforms: "signals", Calculus: "calculus", "Vectors & matrices": "linalg", "Statistics & data": "numerics", Numerical: "numerics" } as Record<string, string>)[p.section ?? ""] ?? "elementary"}` : p.group;
    return `${REPO}/tree/main/site/src/content/pages/${g}.ts`;
  };

  return (
    <>
      <Header onMenu={() => setMenu(m => !m)} />
      <div class={["docs", { "menu-open": menu() }]}>
        <aside class="docs-side">
          <Sidebar current={slug()} onNavigate={() => setMenu(false)} />
        </aside>
        <div class="docs-scrim" onClick={() => setMenu(false)} />
        <main class="docs-main">
          <Show when={page()} fallback={<NotFoundDoc slug={slug()} />}>
            {p => (
              <article class="doc" data-kind={KIND[p().group]}>
                <div class="doc-crumbs">
                  <span>{GROUPS.find(g => g.id === p().group)?.title}</span>
                  <Show when={p().section}><span>›</span><span>{p().section}</span></Show>
                </div>
                <h1 class={["doc-title", { code: ["functions", "operations", "modifiers"].includes(p().group) }]}>
                  {["functions", "operations", "modifiers"].includes(p().group) ? p().slug.split("/").pop() : p().title}
                  <Show when={["functions", "operations", "modifiers"].includes(p().group)}><span class="kind-badge">{KIND[p().group]}</span></Show>
                </h1>
                <p class="doc-summary" innerHTML={inline(p().summary)} />
                <Show when={p().signature || p().import}>
                  <div class="sig-card">
                    <Show when={p().signature}><div class="sig-row"><span class="sig-k">call</span><code class="sig" innerHTML={highlight(p().signature!)} /></div></Show>
                    <Show when={p().import}><div class="sig-row"><span class="sig-k">import</span><code class="sig" innerHTML={highlight(p().import!)} /></div></Show>
                    <Show when={p().returns}><div class="sig-row"><span class="sig-k">returns</span><span class="sig-ret" innerHTML={inline(p().returns!)} /></div></Show>
                  </div>
                </Show>
                <Blocks blocks={p().blocks} />
                <Show when={p().related?.length}>
                  <h2 class="related-title">Related</h2>
                  <div class="related">
                    <For each={p().related!.map(s => bySlug.get(s)).filter(Boolean) as Page[]}>
                      {r => (
                        <Link to={`/docs/${r.slug}`} class="related-card">
                          <span class="t">{["functions", "operations", "modifiers"].includes(r.group) ? r.slug.split("/").pop() : r.title}</span>
                          <span class="s" innerHTML={inline(r.summary)} />
                        </Link>
                      )}
                    </For>
                  </div>
                </Show>
                <div class="doc-foot">
                  <a href={file()} target="_blank" rel="noopener" class="muted">Edit this page on GitHub ↗</a>
                </div>
                <nav class="pager">
                  <Show when={prev()} fallback={<span />}>{pv => <Link to={`/docs/${pv().slug}`} class="pager-link"><span class="muted">← Previous</span><span>{pv().title}</span></Link>}</Show>
                  <Show when={next()}>{nx => <Link to={`/docs/${nx().slug}`} class="pager-link next"><span class="muted">Next →</span><span>{nx().title}</span></Link>}</Show>
                </nav>
              </article>
            )}
          </Show>
        </main>
        <aside class="docs-toc">
          <Show when={toc().length}>
            <div class="toc-title">On this page</div>
            <For each={toc()}>{t => <a href={`#${slugify(t.t)}`} class={["toc-link", `l${t.l}`]} innerHTML={inline(t.t)} />}</For>
          </Show>
        </aside>
      </div>
    </>
  );
}

function NotFoundDoc(props: { slug: string }) {
  return (
    <article class="doc">
      <h1 class="doc-title">Not written yet</h1>
      <p class="doc-summary">There's no page at <code>/docs/{props.slug}</code> (yet). Try the search, or start with the <Link to="/docs/introduction">introduction</Link>.</p>
    </article>
  );
}
