// A small path router: the site lives under BASE_URL (/solveng/ on GitHub Pages), every page is a path below it.
import { createSignal } from "solid-js";
import type { JSX } from "@solidjs/web";

const BASE = import.meta.env.BASE_URL.replace(/\/$/, "");

const strip = (p: string) => {
  const rest = p.startsWith(BASE) ? p.slice(BASE.length) : p;
  return rest === "" ? "/" : rest;
};

const [path, setPath] = createSignal(strip(location.pathname));
const [hash, setHash] = createSignal(location.hash.slice(1));
export { path, hash };

export const href = (to: string) => BASE + (to.startsWith("/") ? to : "/" + to);

export function navigate(to: string, opts: { replace?: boolean } = {}) {
  const url = new URL(href(to), location.origin);
  if (url.pathname === location.pathname && url.hash === location.hash) return;
  history[opts.replace ? "replaceState" : "pushState"](null, "", url.pathname + url.search + url.hash);
  const samePage = strip(url.pathname) === path();
  setPath(strip(url.pathname));
  setHash(url.hash.slice(1));
  requestAnimationFrame(() => {
    if (url.hash) document.getElementById(decodeURIComponent(url.hash.slice(1)))?.scrollIntoView({ behavior: samePage ? "smooth" : "auto" });
    else if (!samePage) window.scrollTo({ top: 0 });
  });
}

window.addEventListener("popstate", () => {
  setPath(strip(location.pathname));
  setHash(location.hash.slice(1));
});

/** Internal links anywhere in the page (also inside rendered Markdown) go through the router. */
document.addEventListener("click", e => {
  if (e.defaultPrevented || e.button !== 0 || e.metaKey || e.ctrlKey || e.shiftKey || e.altKey) return;
  const a = (e.target as Element).closest?.("a");
  if (!a || a.target || a.hasAttribute("download")) return;
  const url = new URL(a.href, location.href);
  if (url.origin !== location.origin || !url.pathname.startsWith(BASE + "/") && url.pathname !== BASE) return;
  e.preventDefault();
  navigate(strip(url.pathname) + url.hash);
});

export function Link(props: { to: string; class?: string | Record<string, boolean> | unknown[]; children?: JSX.Element; title?: string; onClick?: () => void }) {
  return (
    <a href={href(props.to)} class={props.class as any} title={props.title} onClick={() => props.onClick?.()}>
      {props.children}
    </a>
  );
}
