import { createSignal, For, onSettled, Show } from "solid-js";
import { Link, navigate, path } from "../lib/router";
import { search, type Hit } from "../lib/search";
import { theme, toggleTheme } from "../lib/theme";
import { slugify } from "../lib/md";

export const REPO = "https://github.com/BierDav/solveng";
const [searchOpen, setSearchOpen] = createSignal(false);
export const openSearch = () => setSearchOpen(true);

export function Logo(props: { size?: number }) {
  const s = () => props.size ?? 26;
  return (
    <svg width={s()} height={s()} viewBox="0 0 32 32" aria-hidden="true">
      <rect x="1" y="1" width="30" height="30" rx="9" fill="var(--ink)" />
      <path d="M9 12.5h6M9 15.5h6" stroke="var(--bg)" stroke-width="2" stroke-linecap="round" />
      <circle cx="12" cy="9" r="0" />
      <path d="M17.5 21.5l3-3 3 3" stroke="#ffa857" stroke-width="2" fill="none" stroke-linecap="round" stroke-linejoin="round" transform="rotate(90 20.5 20)" />
      <path d="M8.5 21h7" stroke="#7c9cff" stroke-width="2" stroke-linecap="round" />
    </svg>
  );
}

export default function Header(props: { overHero?: boolean; onMenu?: () => void }) {
  const [scrolled, setScrolled] = createSignal(false);
  onSettled(() => {
    const on = () => setScrolled(window.scrollY > 24);
    window.addEventListener("scroll", on, { passive: true });
    const key = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k") { e.preventDefault(); setSearchOpen(o => !o); }
      else if (e.key === "/" && !/input|textarea|select/i.test((e.target as Element).tagName)) { e.preventDefault(); setSearchOpen(true); }
    };
    window.addEventListener("keydown", key);
    return () => { window.removeEventListener("scroll", on); window.removeEventListener("keydown", key); };
  });
  const isDocs = () => path().startsWith("/docs");
  return (
    <>
      <header class={["header", { "over-hero": props.overHero, scrolled: scrolled() }]}>
        <Show when={props.onMenu}>
          <button class="icon-btn menu-btn" aria-label="Menu" onClick={() => props.onMenu?.()}>
            <svg width="18" height="18" viewBox="0 0 24 24"><path d="M4 7h16M4 12h16M4 17h16" stroke="currentColor" stroke-width="2" stroke-linecap="round" /></svg>
          </button>
        </Show>
        <Link to="/" class="brand"><Logo /> solveng <span class="ver">{__SOLVENG_VERSION__}</span></Link>
        <nav class="nav">
          <Link to="/docs/introduction" class={{ active: isDocs() && !path().startsWith("/docs/operations") && !path().startsWith("/docs/functions") }}>Docs</Link>
          <Link to="/docs/functions/math" class={{ active: path().startsWith("/docs/functions") || path().startsWith("/docs/modifiers") }}>API</Link>
          <Link to="/docs/operations/tri" class={{ active: path().startsWith("/docs/operations") }}>Operations</Link>
          <Link to="/playground" class={{ active: path() === "/playground" }}>Playground</Link>
        </nav>
        <div class="spacer" />
        <button class="search-btn" onClick={() => setSearchOpen(true)} aria-label="Search">
          <svg width="15" height="15" viewBox="0 0 24 24"><circle cx="11" cy="11" r="7" stroke="currentColor" stroke-width="2" fill="none" /><path d="M20 20l-3.5-3.5" stroke="currentColor" stroke-width="2" stroke-linecap="round" /></svg>
          <span>Search the docs…</span>
          <kbd>⌘K</kbd>
        </button>
        <button class="icon-btn" aria-label="Toggle theme" onClick={toggleTheme} title="Light / dark">
          <Show when={theme() === "dark"} fallback={<svg width="18" height="18" viewBox="0 0 24 24"><path d="M21 12.8A9 9 0 1 1 11.2 3a7 7 0 0 0 9.8 9.8z" fill="currentColor" /></svg>}>
            <svg width="18" height="18" viewBox="0 0 24 24"><circle cx="12" cy="12" r="4.5" fill="currentColor" /><g stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4" /></g></svg>
          </Show>
        </button>
        <a class="icon-btn" href={REPO} target="_blank" rel="noopener" aria-label="GitHub">
          <svg width="18" height="18" viewBox="0 0 16 16"><path fill="currentColor" d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z" /></svg>
        </a>
      </header>
      <Show when={searchOpen()}><SearchDialog onClose={() => setSearchOpen(false)} /></Show>
    </>
  );
}

function SearchDialog(props: { onClose: () => void }) {
  const [q, setQ] = createSignal("");
  const [sel, setSel] = createSignal(0);
  const hits = () => search(q());
  const go = (h: Hit) => { props.onClose(); navigate(`/docs/${h.page.slug}${h.anchor ? "#" + slugify(h.anchor) : ""}`); };
  const SUGGEST = ["units", "solve", "graph", "integrate", "tri", "complex", "digits", "matrices"];
  return (
    <div class="search-overlay" onClick={e => { if (e.target === e.currentTarget) props.onClose(); }}>
      <div class="search-dialog" role="dialog" aria-label="Search">
        <input
          ref={el => requestAnimationFrame(() => el.focus())}
          placeholder="Search functions, operations, guides…"
          value={q()}
          onInput={e => { setQ(e.currentTarget.value); setSel(0); }}
          onKeyDown={e => {
            if (e.key === "Escape") props.onClose();
            else if (e.key === "ArrowDown") { e.preventDefault(); setSel(s => Math.min(s + 1, hits().length - 1)); }
            else if (e.key === "ArrowUp") { e.preventDefault(); setSel(s => Math.max(s - 1, 0)); }
            else if (e.key === "Enter" && hits()[sel()]) go(hits()[sel()]);
          }}
        />
        <div class="search-results">
          <Show when={q().trim()} fallback={
            <div class="search-empty">Try <For each={SUGGEST}>{(s, i) => <>{i() ? ", " : ""}<a href="#" onClick={e => { e.preventDefault(); setQ(s); }}>{s}</a></>}</For></div>
          }>
            <Show when={hits().length} fallback={<div class="search-empty">Nothing found for “{q()}”</div>}>
              <For each={hits()}>
                {(h, i) => (
                  <div class={["search-item", { sel: i() === sel() }]} onMouseEnter={() => setSel(i())} onClick={() => go(h)}>
                    <span class="t">{h.label}</span>
                    <span class="s">{h.detail}</span>
                    <span class="g">{h.group}</span>
                  </div>
                )}
              </For>
            </Show>
          </Show>
        </div>
      </div>
    </div>
  );
}
