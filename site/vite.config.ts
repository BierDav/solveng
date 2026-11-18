import { copyFileSync } from "node:fs";
import { resolve } from "node:path";
import solid from "@solidjs/vite-plugin";
import { defineConfig, type Plugin } from "vite";

// GitHub Pages serves the site under /<repository>/; SITE_BASE overrides it (e.g. "/" for a custom domain)
const base = process.env.SITE_BASE ?? "/solveng/";

// Pages has no rewrites: a 404.html that is the app itself makes every /docs/… link load it
const spaFallback = (): Plugin => ({
  name: "spa-fallback",
  closeBundle() {
    const out = resolve(import.meta.dirname, "dist");
    copyFileSync(resolve(out, "index.html"), resolve(out, "404.html"));
  },
});

export default defineConfig({
  base,
  plugins: [solid(), spaFallback()],
  // the engine and solveng's unit table come from the repository itself
  server: { fs: { allow: [resolve(import.meta.dirname, "..")] } },
  worker: { format: "es" },
  build: { target: "es2022", chunkSizeWarningLimit: 1500 },
});
