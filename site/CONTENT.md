# Writing the documentation site's content

The site (`site/`, SolidJS) documents solveng, a Typst package for engineering calculations. Pages are TypeScript data
in `site/src/content/pages/*.ts`; the shapes are in `site/src/content/types.ts` (read it first, including the Markdown
dialect at its top). `site/src/content/pages/start.ts` holds a finished model page – match its tone and structure.

## Sources of truth

- `README.md` and `docs/reference.md` (generated from the doc comments in `src/*.typ`): what every function, operation,
  modifier and option does, its parameters and limits. Don't invent behaviour; when unsure, run an example.
- `src/*.typ` doc comments (`///`) and `src/builtins.typ` for operations.

## Examples run for real

Every `example` runs in the browser on solveng's real engine through a small Typst interpreter
(`site/src/engine/`). An example is a complete document:

```typ
#import "@preview/solveng:3.0.0": *
#import units: kΩ, V, mA          // only what it uses
#import operations: tri           // operations, constants, modifiers likewise
#show: math

$R := 4.7 kΩ$, $U := 5 #V$
$I := U / R => mA$
```

The interpreter supports: `#import` (package, `units`, `constants`, `operations`, `modifiers`, renames with `as`),
`#let` (values, `base-unit`, `unit`, `constant`, `operation`, simple closures), `#show: math` / `math.with(…)`,
`#set text(lang: …)`, `#math(…)[…]`, `#mathscope(…)[…]`, `#graph(…)`, `#defined(…)`, `#value(…)`, `#clear(…)`,
`#context { let f = compile(…); [#f(12)] }`, headings, paragraphs, `*strong*`, `_emph_`, raw text, comments.
Not supported: `#if`, `#for`, `#table`, `#figure`, file imports other than the package. Keep examples focused: one
idea each, a few lines, readable on a phone. Prefer showing results with `=>` / `->` over prose that states them.

### Make them interactive and fun

- `controls` put sliders, selects or toggles on an example: `{{name}}` in the code is replaced by the control's
  value and the example recalculates as you drag. Use them where moving a number teaches something (a pulse's width,
  a resistor's value, `digits(…)`, an option). A graph that changes as you drag is the best kind.
- `quiz` blocks: a question with 2–4 answers and an explanation. One per page where a common misconception exists.
- `callout` `fun` for a surprising fact or a challenge ("Try changing … to …"). Use sparingly.
- Graphs: `#graph($…$)` draws in the browser, interactive. Use them on pages about signals, functions, solving.

### Check every example against real Typst

```sh
cd site && nix shell nixpkgs#typst -c bun scripts/conformance.ts --site <slug-prefix>
```

runs each of your examples (and each control at its extremes) through Typst with the package and through the site's
interpreter, and compares what they define and which problems they report. Every example must pass (`✓`). An example
that shows an error on purpose has `expectError: true`. If Typst itself fails or reports a problem you didn't intend,
fix the example. If Typst succeeds but the site disagrees, that's a bug in the site's interpreter – don't edit
`site/src/engine/`; write the example another way if you can, and report the smallest document that shows the
disagreement in your final message.

## Rules

- Edit only the page files assigned to you. Don't commit. Don't touch other files.
- Slugs and links: link pages as `[text](/docs/<slug>)` using the slugs below; `related` lists slugs.
- Every page has a precise `summary` (one sentence), and functions/operations have `signature` and `import`.
- Tone: friendly, exact, short sentences, no filler. Explain *why* as well as *what*. Engineering-flavoured examples
  (circuits, mechanics, signals) beat abstract `x` and `y` where they fit.
- Typecheck what you wrote: `cd site && bunx tsc --noEmit -p .` must report no errors in your files (once tsconfig exists;
  if it doesn't yet, rely on the conformance run, which imports your files).

## Slugs

start (`group: "start"`): introduction, getting-started, operators, tour
guides (`group: "guides"`): values, units, custom-units, constants, functions, comparisons, piecewise, vectors,
matrices, sums, complex, symbolic, solving, calculus, signals, transforms, numerics, graphs, scopes, formatting, errors,
html, editors, how-it-solves, limits
functions (`group: "functions"`): functions/math, functions/mathscope, functions/graph, functions/defined,
functions/value, functions/clear, functions/compile, functions/base-unit, functions/unit, functions/constant,
functions/operation, functions/any
modifiers (`group: "modifiers"`): modifiers/solve, modifiers/substitute, modifiers/assume, modifiers/simplify,
modifiers/expand, modifiers/factor, modifiers/steps, modifiers/digits, modifiers/sci, modifiers/eng, modifiers/fraction
operations (`group: "operations"`, `section` as given): operations/<name> for
- Elementary: asin, acos, atan, atan2, min, max, floor, ceil, factorial, sgn, number, celsius, fahrenheit, gamma, erf, besselj
- Complex: Re, Im, conj
- Signals: rect, tri, sinc, cosc, tanc, dirac
- Calculus: diff, integrate, taylor, jacobian, hessian, prod
- Transforms: laplace, invlaplace, convolve
- Vectors & matrices: range, at, rank, identity, rows, cols, linear-solve, eigenvals, singularvals, cond
- Statistics & data: mean, median, variance, stdev, pnorm, qnorm, linear-interpolate, linear-fit, dft, idft
- Numerical: minimize, odesolve, polyroots
reference (`group: "reference"`): reference/options, reference/units, reference/cheatsheet, reference/typst-math

Widgets (`{ widget: … }`): `operator-cards` (the three operators, used on introduction/operators), `option-explorer`
(every option of `math` with live toggles, for reference/options), `unit-table` (every unit binding, searchable, for
reference/units).
