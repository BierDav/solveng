// The front page: what solveng is, shown rather than told – everything on it calculates live.
import { createSignal, For, onSettled, Show } from "solid-js";
import Editor from "../components/Editor";
import Header, { REPO } from "../components/Header";
import Output from "../components/Output";
import Workbench from "../components/Workbench";
import OperatorCards from "../components/widgets/OperatorCards";
import { runDocument, type RunResponse } from "../engine/client";
import { Link } from "../lib/router";

const PRELUDE = `#import "@preview/solveng:3.0.0": *
#import units: mA, mAh, h
#import modifiers: solve
#show: math
`;
const LINES = [
  "$Q_0 := 3450 mAh$",
  "$I := 30 mA$",
  "$t := Q_0 / I => #h$",
  "$Q_0/2 = I t -> solve(t)$",
];

function HeroNotebook() {
  const [typed, setTyped] = createSignal("");
  const [result, setResult] = createSignal<RunResponse | null>(null);
  onSettled(() => {
    let alive = true;
    const full = LINES.join("\n");
    let i = 0;
    const tick = () => {
      if (!alive) return;
      if (i <= full.length) {
        const text = full.slice(0, i);
        setTyped(text);
        const done = text.split("\n").filter(l => l.endsWith("$"));
        if (text.endsWith("$")) runDocument(PRELUDE + done.join("\n")).then(r => alive && setResult(r));
        i++;
        setTimeout(tick, full[i - 1] === "\n" ? 380 : 28 + Math.random() * 40);
      }
    };
    setTimeout(tick, 600);
    return () => { alive = false; };
  });
  return (
    <div class="hero-notebook">
      <div class="nb-chrome"><i /><i /><i /><span>battery.typ</span></div>
      <div class="nb-code">
        <Editor value={PRELUDE + typed()} onInput={() => {}} readOnly minRows={9} label="Example worksheet" />
      </div>
      <div class="nb-out paper">
        <Show when={result()} fallback={<div class="bench-loading"><span /><span /><span /></div>}>
          {r => <Output blocks={r().blocks} />}
        </Show>
      </div>
    </div>
  );
}

const FEATURES = [
  { icon: "⟦m⟧", title: "Units that check themselves", text: "30 mA × 2 h is 60 mAh. Volts plus amperes is an error, not a silent mistake. 480 unit bindings, SI prefixes, your own base units.", to: "/docs/units" },
  { icon: "½", title: "Exact until you ask", text: "Fractions, roots and π stay exact with ->. => rounds to the digits you choose, in the unit you name.", to: "/docs/symbolic" },
  { icon: "x=", title: "Solve where you stand", text: "-> solve(x) isolates symbolically, finds all complex roots of polynomials, and falls back to numerics when everything else has a value.", to: "/docs/solving" },
  { icon: "∿", title: "Graphs that pick their axes", text: "Ranges, ticks, units and labels come from the math. Mark roots, extrema and crossings with one word.", to: "/docs/graphs" },
  { icon: "∫", title: "Calculus and transforms", text: "Derivatives, antiderivatives, definite integrals, Taylor series, Laplace transforms and convolution – exact where possible.", to: "/docs/calculus" },
  { icon: "j", title: "Complex numbers, natively", text: "Impedances with units, ±2i solutions, Euler's formula proven. Electrical engineers can write j.", to: "/docs/complex" },
  { icon: "⊓", title: "Signals", text: "rect, tri, sinc and dirac with exact Fourier integrals and piecewise convolution: rect ∗ rect is tri.", to: "/docs/signals" },
  { icon: "[ ]", title: "Matrices and numerics", text: "Linear systems, eigenvalues, singular values, statistics, fits, DFTs, ODEs and minimization.", to: "/docs/numerics" },
];

export default function Landing() {
  const [copied, setCopied] = createSignal(false);
  const install = `#import "@preview/solveng:3.0.0": *`;
  return (
    <div class="landing">
      <Header overHero />
      <section class="hero">
        <div class="hero-bg" aria-hidden="true" />
        <div class="hero-inner">
          <div class="hero-copy">
            <div class="hero-kicker"><span class="dot" /> A Typst package · runs in your document</div>
            <h1 class="display hero-title">Calculate the math<br /><em>you write.</em></h1>
            <p class="hero-sub">solveng turns the equations in your Typst document into a live engineering worksheet — with units, exact results, solving and graphs. Write the formula once; the result appears next to it.</p>
            <div class="hero-cta">
              <Link to="/docs/getting-started" class="btn primary">Get started →</Link>
              <Link to="/playground" class="btn">Open the playground</Link>
            </div>
            <button class="install" onClick={async () => { try { await navigator.clipboard.writeText(install); setCopied(true); setTimeout(() => setCopied(false), 1400); } catch { /* */ } }}>
              <code>{install}</code>
              <span>{copied() ? "✓" : "copy"}</span>
            </button>
          </div>
          <HeroNotebook />
        </div>
      </section>

      <section class="band">
        <div class="band-head">
          <div class="eyebrow">Three operators</div>
          <h2 class="display">Everything else is just math.</h2>
          <p>A worksheet adds three operators to Typst's math. <code>=</code> keeps its meaning. Edit the cards — they're live.</p>
        </div>
        <OperatorCards />
      </section>

      <section class="band alt">
        <div class="band-head">
          <div class="eyebrow">Drag it</div>
          <h2 class="display">Move a slider, watch the physics.</h2>
          <p>An RC low-pass filter. The corner frequency, the solved −3 dB point and the graph all come from solveng's engine — the same one Typst runs — right here in your browser.</p>
        </div>
        <div class="demo-wrap">
          <Workbench example={{
            title: "RC low-pass",
            code: `#import "@preview/solveng:3.0.0": *
#import units: kΩ, nF, kHz, Hz
#import constants: pi
#show: math

$R := {{R}} kΩ$, $C := {{C}} nF$
$f_c := 1 / (2 pi R C) => kHz$
#graph($H(f: kHz) = 1/sqrt(1 + (f/f_c)^2)$, mark: $f_c$)`,
            controls: [
              { kind: "slider", name: "R", label: "R", min: 0.5, max: 20, step: 0.1, value: 4.7, suffix: " kΩ" },
              { kind: "slider", name: "C", label: "C", min: 1, max: 100, step: 1, value: 10, suffix: " nF" },
            ],
          }} />
        </div>
      </section>

      <section class="band">
        <div class="band-head">
          <div class="eyebrow">What's inside</div>
          <h2 class="display">An engineering toolbox, in your document.</h2>
        </div>
        <div class="features">
          <For each={FEATURES}>
            {f => (
              <Link to={f.to} class="feature">
                <div class="feature-icon">{f.icon}</div>
                <div class="feature-title">{f.title}</div>
                <p>{f.text}</p>
                <span class="feature-more">Learn more →</span>
              </Link>
            )}
          </For>
        </div>
      </section>

      <section class="stats">
        <div><b>60+</b><span>operations</span></div>
        <div><b>480</b><span>unit bindings</span></div>
        <div><b>Rust → Wasm</b><span>one engine, Typst and browser</span></div>
        <div><b>100%</b><span>of examples run live</span></div>
      </section>

      <section class="final">
        <h2 class="display">Your notes, calculated.</h2>
        <p>Start with a single <code>#show: math</code>.</p>
        <div class="hero-cta center">
          <Link to="/docs/introduction" class="btn primary">Read the docs →</Link>
          <a href={REPO} class="btn" target="_blank" rel="noopener">GitHub</a>
        </div>
      </section>

      <footer class="footer">
        <div class="inner">
          <span>solveng · MIT licensed · made for Typst</span>
          <span class="spacer" />
          <Link to="/docs/introduction">Docs</Link>
          <Link to="/playground">Playground</Link>
          <a href={REPO} target="_blank" rel="noopener">GitHub</a>
        </div>
      </footer>
    </div>
  );
}
