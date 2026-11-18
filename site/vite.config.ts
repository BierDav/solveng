import { copyFileSync, readFileSync } from "node:fs";
import { resolve } from "node:path";
import solid from "@solidjs/vite-plugin";
import { defineConfig, type Plugin } from "vite";

// GitHub Pages serves the site under /<repository>/; SITE_BASE overrides it (e.g. "/" for a custom domain)
const base = process.env.SITE_BASE ?? "/solveng/";

// the header shows the package version, so a release needs no edit here
const version = /^version\s*=\s*"([^"]+)"/m.exec(readFileSync(resolve(import.meta.dirname, "../typst.toml"), "utf8"))![1];

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
  define: { __SOLVENG_VERSION__: JSON.stringify(version) },
  // the engine and solveng's unit table come from the repository itself
  server: { fs: { allow: [resolve(import.meta.dirname, "..")] } },
  worker: { format: "es" },
  build: { target: "es2022", chunkSizeWarningLimit: 1500 },
});
