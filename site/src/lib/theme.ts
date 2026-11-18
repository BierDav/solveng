// Light or dark: the reader's choice, else their system's.
import { createSignal } from "solid-js";

type Theme = "light" | "dark";
const KEY = "solveng-theme";

function stored(): Theme | null {
  try { const v = localStorage.getItem(KEY); return v === "light" || v === "dark" ? v : null; } catch { return null; }
}
const system = (): Theme => (matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light");

const [theme, setThemeSignal] = createSignal<Theme>(stored() ?? system());
export { theme };

function apply(t: Theme) { document.documentElement.dataset.theme = t; }
apply(theme());

export function toggleTheme() {
  const next: Theme = theme() === "dark" ? "light" : "dark";
  setThemeSignal(next);
  apply(next);
  try { localStorage.setItem(KEY, next); } catch { /* private mode */ }
}

matchMedia("(prefers-color-scheme: dark)").addEventListener("change", () => {
  if (stored()) return;
  const t = system();
  setThemeSignal(t);
  apply(t);
});
