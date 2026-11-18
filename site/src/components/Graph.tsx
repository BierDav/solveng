// solveng's graphs, drawn from the engine's figure in the schoolbook style of the printed ones: axes through the
// origin with arrows, a faint grid at the regular ticks. Curves draw themselves in; the pointer reads every curve.
import { createMemo, createSignal, For, onSettled, Show } from "solid-js";
import type { GraphView } from "../engine/interp";

const DASH: Record<string, string> = {
  solid: "", dashed: "6 4", "densely-dashed": "4 2", "loosely-dashed": "6 8", dotted: "0.1 4", "densely-dotted": "0.1 2.5",
  "loosely-dotted": "0.1 7", "dash-dotted": "6 3 0.1 3", "densely-dash-dotted": "4 2 0.1 2", "loosely-dash-dotted": "6 6 0.1 6",
};

function dotPath(shape: string, r: number): string {
  const poly = (n: number, rot = -Math.PI / 2, rr = r) => Array.from({ length: n }, (_, i) => {
    const a = rot + (i * 2 * Math.PI) / n;
    return `${(rr * Math.cos(a)).toFixed(2)},${(rr * Math.sin(a)).toFixed(2)}`;
  }).join(" L");
  switch (shape) {
    case "square": return `M${-r * 0.85},${-r * 0.85} h${r * 1.7} v${r * 1.7} h${-r * 1.7} Z`;
    case "triangle": return `M${poly(3, -Math.PI / 2, r * 1.15)} Z`;
    case "triangle-down": return `M${poly(3, Math.PI / 2, r * 1.15)} Z`;
    case "diamond": return `M${poly(4, -Math.PI / 2, r * 1.15)} Z`;
    case "pentagon": return `M${poly(5)} Z`;
    case "hexagon": return `M${poly(6)} Z`;
    case "star": return "M" + Array.from({ length: 10 }, (_, i) => { const a = -Math.PI / 2 + (i * Math.PI) / 5; const rr = i % 2 ? r * 0.5 : r * 1.2; return `${(rr * Math.cos(a)).toFixed(2)},${(rr * Math.sin(a)).toFixed(2)}`; }).join(" L") + " Z";
    case "cross": return `M${-r},${-r} L${r},${r} M${r},${-r} L${-r},${r}`;
    case "plus": return `M${-r},0 L${r},0 M0,${-r} L0,${r}`;
    default: return `M${-r},0 a${r},${r} 0 1,0 ${2 * r},0 a${r},${r} 0 1,0 ${-2 * r},0`;
  }
}

const fmt = (v: number) => {
  if (!isFinite(v)) return String(v);
  const a = Math.abs(v);
  if (a !== 0 && (a < 1e-3 || a >= 1e6)) return v.toExponential(3).replace("e", " · 10^");
  return String(Number(v.toPrecision(4)));
};

export default function Graph(props: { graph: GraphView }) {
  const g = () => props.graph;
  const [width, setWidth] = createSignal(560);
  const [hover, setHover] = createSignal<number | null>(null);
  const [drawn, setDrawn] = createSignal(false);
  let box!: HTMLDivElement;

  onSettled(() => {
    const ro = new ResizeObserver(([e]) => setWidth(Math.max(260, Math.min(720, e.contentRect.width))));
    ro.observe(box);
    const io = new IntersectionObserver(es => { if (es.some(x => x.isIntersecting)) { setDrawn(true); io.disconnect(); } }, { threshold: 0.2 });
    io.observe(box);
    return () => { ro.disconnect(); io.disconnect(); };
  });

  const pad = { l: 46, r: 46, t: 26, b: 34 };
  const height = () => Math.round(width() * 0.6);
  const X = (x: number) => pad.l + ((x - g().x0) / (g().x1 - g().x0)) * (width() - pad.l - pad.r);
  const Y = (y: number) => height() - pad.b - ((y - g().y0) / (g().y1 - g().y0)) * (height() - pad.t - pad.b);
  const clampX = (x: number) => Math.max(pad.l, Math.min(width() - pad.r, x));
  const clampY = (y: number) => Math.max(pad.t, Math.min(height() - pad.b, y));
  const axisY = () => clampY(Y(Math.min(Math.max(0, g().y0), g().y1)));
  const axisX = () => clampX(X(Math.min(Math.max(0, g().x0), g().x1)));

  const paths = createMemo(() => g().series.map(s => {
    const st = g().styles[s.curve] ?? g().styles[0];
    const line = s.points.length > 1 && st.dash !== null;
    const d = line ? s.points.map(([x, y], i) => `${i ? "L" : "M"}${X(x).toFixed(1)},${Y(y).toFixed(1)}`).join("") : "";
    const showDots = st.dot !== null && !s.beyond && (s.data ? st.dash === null || s.dots : s.points.length === 1);
    return { s, st, d, dots: showDots ? s.points : [] };
  }));

  // the values of every curve under the pointer, interpolated along its samples
  const readout = createMemo(() => {
    const px = hover();
    if (px === null) return null;
    const x = g().x0 + ((px - pad.l) / (width() - pad.l - pad.r)) * (g().x1 - g().x0);
    const rows: { color: string; label: string | null; y: number; cy: number }[] = [];
    const seen = new Set<number>();
    for (const s of g().series) {
      if (seen.has(s.curve) || s.points.length < 2) continue;
      for (let k = 1; k < s.points.length; k++) {
        const [x1, y1] = s.points[k - 1], [x2, y2] = s.points[k];
        if ((x >= x1 && x <= x2) || (x >= x2 && x <= x1)) {
          const y = x2 === x1 ? y1 : y1 + ((y2 - y1) * (x - x1)) / (x2 - x1);
          if (isFinite(y) && y >= g().y0 - (g().y1 - g().y0) && y <= g().y1 + (g().y1 - g().y0)) {
            rows.push({ color: (g().styles[s.curve] ?? g().styles[0]).color, label: s.label, y, cy: Y(y) });
            seen.add(s.curve);
          }
          break;
        }
      }
    }
    return { x, px, rows };
  });

  const onMove = (e: PointerEvent) => {
    const r = (e.currentTarget as SVGElement).getBoundingClientRect();
    const px = ((e.clientX - r.left) / r.width) * width();
    setHover(px >= pad.l && px <= width() - pad.r ? px : null);
  };

  const legend = createMemo(() => {
    const out: { color: string; label: string; dash: string; dot: string | null }[] = [];
    const seen = new Set<number>();
    for (const s of g().series) {
      if (!s.label || seen.has(s.curve)) continue;
      seen.add(s.curve);
      const st = g().styles[s.curve] ?? g().styles[0];
      out.push({ color: st.color, label: s.label, dash: st.dash ?? "solid", dot: st.dot });
    }
    return out.length > 1 ? out : [];
  });

  return (
    <div class="graph" ref={el => (box = el)}>
      <div class="graph-stage" style={{ width: `${width()}px`, height: `${height()}px` }}>
        <svg viewBox={`0 0 ${width()} ${height()}`} width={width()} height={height()} onPointerMove={onMove} onPointerLeave={() => setHover(null)}>
          <defs>
            <marker id="sv-arrow" viewBox="0 0 10 10" refX="8" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse">
              <path d="M0,1 L9,5 L0,9 z" fill="currentColor" />
            </marker>
            <clipPath id={`clip-${width()}`}><rect x={pad.l - 2} y={pad.t - 2} width={width() - pad.l - pad.r + 4} height={height() - pad.t - pad.b + 4} /></clipPath>
          </defs>
          <g class="graph-grid">
            <For each={g().xticks.map((v, i) => ({ v, i })).filter(t => !g().xmarked[t.i])}>{t => <line x1={X(t.v)} x2={X(t.v)} y1={pad.t} y2={height() - pad.b} />}</For>
            <For each={g().yticks.map((v, i) => ({ v, i })).filter(t => !g().ymarked[t.i])}>{t => <line y1={Y(t.v)} y2={Y(t.v)} x1={pad.l} x2={width() - pad.r} />}</For>
          </g>
          <g class="graph-axes">
            <line x1={pad.l - 6} x2={width() - pad.r + 14} y1={axisY()} y2={axisY()} marker-end="url(#sv-arrow)" />
            <line y1={height() - pad.b + 6} y2={pad.t - 14} x1={axisX()} x2={axisX()} marker-end="url(#sv-arrow)" />
            <For each={g().xticks}>{v => <line x1={X(v)} x2={X(v)} y1={axisY() - 3} y2={axisY() + 3} />}</For>
            <For each={g().yticks}>{v => <line y1={Y(v)} y2={Y(v)} x1={axisX() - 3} x2={axisX() + 3} />}</For>
          </g>
          <g class="graph-guides">
            <For each={g().points}>
              {p => (
                <>
                  <Show when={p.guides !== ""}><line x1={X(p.x)} x2={X(p.x)} y1={axisY()} y2={Y(p.y)} /></Show>
                  <Show when={p.guides === "xy"}><line x1={axisX()} x2={X(p.x)} y1={Y(p.y)} y2={Y(p.y)} /></Show>
                </>
              )}
            </For>
          </g>
          <g clip-path={`url(#clip-${width()})`}>
            <For each={paths()}>
              {(p, i) => (
                <>
                  <Show when={p.d}>
                    <path
                      class={["graph-curve", { drawn: drawn() }]}
                      d={p.d}
                      stroke={p.st.color}
                      stroke-dasharray={DASH[p.st.dash ?? "solid"] || undefined}
                      style={{ "--delay": `${i() * 0.12}s` }}
                      pathLength={DASH[p.st.dash ?? "solid"] ? undefined : 1}
                    />
                  </Show>
                  <For each={p.dots}>
                    {([x, y]) => <path class={["graph-dot", { drawn: drawn() }]} d={dotPath(p.st.dot ?? "circle", 3.6)} transform={`translate(${X(x)},${Y(y)})`} fill={p.st.color} stroke={p.st.color} />}
                  </For>
                </>
              )}
            </For>
          </g>
          <For each={g().points}>{p => <circle class="graph-mark" cx={X(p.x)} cy={Y(p.y)} r="3.6" />}</For>
          <Show when={readout()}>
            {r => (
              <g class="graph-hover">
                <line x1={r().px} x2={r().px} y1={pad.t} y2={height() - pad.b} />
                <For each={r().rows}>{row => <circle cx={r().px} cy={row.cy} r="4.5" fill={row.color} />}</For>
              </g>
            )}
          </Show>
        </svg>
        <For each={g().xticks}>
          {(v, i) => <div class="graph-lab x" style={{ left: `${X(v)}px`, top: `${axisY() + 5}px` }} innerHTML={g().xlabels[i()]} />}
        </For>
        <For each={g().yticks}>
          {(v, i) => <div class="graph-lab y" style={{ left: `${axisX() - 6}px`, top: `${Y(v)}px` }} innerHTML={g().ylabels[i()]} />}
        </For>
        <div class="graph-lab xl" style={{ left: `${width() - pad.r + 18}px`, top: `${axisY()}px` }} innerHTML={g().xlabel} />
        <div class="graph-lab yl" style={{ left: `${axisX()}px`, top: `${pad.t - 20}px` }} innerHTML={g().ylabel} />
        <Show when={legend().length}>
          <div class="graph-legend">
            <For each={legend()}>
              {l => (
                <div class="row">
                  <svg width="26" height="10"><line x1="1" x2="25" y1="5" y2="5" stroke={l.color} stroke-width="2" stroke-dasharray={DASH[l.dash] || undefined} stroke-linecap="round" /></svg>
                  <span innerHTML={l.label} />
                </div>
              )}
            </For>
          </div>
        </Show>
        <Show when={readout()?.rows.length ? readout() : null}>
          {r => (
            <div class="graph-tip" style={{ left: `${Math.min(r().px + 14, width() - 150)}px`, top: `${pad.t + 6}px` }}>
              <div class="x">{g().xname || "x"} = {fmt(r().x)}</div>
              <For each={r().rows}>
                {row => (
                  <div class="row">
                    <i style={{ background: row.color }} />
                    <span>{fmt(row.y)}</span>
                  </div>
                )}
              </For>
            </div>
          )}
        </Show>
      </div>
    </div>
  );
}
