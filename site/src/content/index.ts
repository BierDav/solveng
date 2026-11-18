// Every page of the documentation, in the sidebar's order.
import type { Group, Page } from "./types";
import { pages as start } from "./pages/start";
import { pages as guidesBasics } from "./pages/guides-basics";
import { pages as guidesMath } from "./pages/guides-math";
import { pages as guidesAdvanced } from "./pages/guides-advanced";
import { pages as functions } from "./pages/functions";
import { pages as modifiers } from "./pages/modifiers";
import { pages as opsElementary } from "./pages/operations-elementary";
import { pages as opsSignals } from "./pages/operations-signals";
import { pages as opsCalculus } from "./pages/operations-calculus";
import { pages as opsLinalg } from "./pages/operations-linalg";
import { pages as opsNumerics } from "./pages/operations-numerics";
import { pages as reference } from "./pages/reference";

export const pages: Page[] = [
  ...start, ...guidesBasics, ...guidesMath, ...guidesAdvanced,
  ...functions, ...modifiers,
  ...opsElementary, ...opsSignals, ...opsCalculus, ...opsLinalg, ...opsNumerics,
  ...reference,
];

export const GROUPS: { id: Group; title: string; blurb: string }[] = [
  { id: "start", title: "Start here", blurb: "What solveng is, and your first worksheet" },
  { id: "guides", title: "Guides", blurb: "Every feature, explained with live examples" },
  { id: "functions", title: "Functions", blurb: "What you call in Typst code: math, graph, defined…" },
  { id: "modifiers", title: "Modifiers", blurb: "What goes after => and ->: solve, digits, expand…" },
  { id: "operations", title: "Operations", blurb: "What you call in math: tri, integrate, eigenvals…" },
  { id: "reference", title: "Reference", blurb: "Options, units and the fine print" },
];

export const bySlug = new Map(pages.map(p => [p.slug, p]));
