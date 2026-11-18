import type { Page } from "../types";

const PKG = `#import "@preview/solveng:2.0.0": *`;

// Every option of `math` is accepted by `graph` and `defined` as well, for that call only.
const OPTIONS_PARAM = {
  name: "digits, decimal-separator, number-form, …",
  type: "auto",
  default: "auto",
  md: "Any [option](/docs/reference/options) of `math`, for this call only. `auto` keeps the current setting.",
};

export const pages: Page[] = [
  // ---------------------------------------------------------------- math
  {
    slug: "functions/math",
    title: "math",
    group: "functions",
    summary: "Turns calculation on for the math that follows and sets its options – for the whole document as a show rule, or for one block – without starting a new variable scope.",
    signature: "math(..body, clear-at: auto, hidden: auto, evaluate: auto, digits: auto, …) → content",
    import: `#import "@preview/solveng:2.0.0": math`,
    returns: "content: the body, with its math calculated.",
    keywords: ["show rule", "settings", "options", "evaluate", "hidden", "clear-at", "worksheet"],
    blocks: [
      { md: "Importing solveng changes nothing on its own. `math` is the switch, and it works two ways:\n\n- `#show: math` – a show rule: every equation after it is calculated. `#show: math.with(digits: 3)` sets options too.\n- `#math(digits: 8)[…]` – only the content in the brackets gets these settings. They end with the block.\n\nEither way, definitions land in the scope you're already in: the document's global one, or the [`mathscope`](/docs/functions/mathscope) around you. `math` never creates a scope of its own – that's what makes it safe to sprinkle around." },
      {
        example: {
          title: "A show rule with options",
          code: `${PKG}
#import units: V, kΩ, mA, mW
#show: math.with(digits: {{d}})

$U := 5 #V$, $R := 4.7 kΩ$
$I := U / R => mA$
$P := U I => mW$`,
          controls: [{ kind: "slider", name: "d", label: "digits", min: 1, max: 10, step: 1, value: 4 }],
          caption: "Drag `digits`: every result follows the show rule. Given values (5 V, 4.7 kΩ) are shown as typed.",
        },
      },
      { h2: "Settings for one block" },
      { md: "Inside `#math(…)[…]` the closest setting wins, option by option; everything you don't set is inherited. When the block ends, the settings revert – but a definition made inside stays, because the block shares the scope around it." },
      {
        example: {
          code: `${PKG}
#show: math.with(digits: 4)

$x := 12$
#math(digits: 8)[$y := x/7 =>$]
$y =>$ // still defined, back to 4 digits`,
          caption: "`y` is defined inside the block and used after it.",
        },
      },
      { h2: "evaluate and hidden" },
      { md: "`evaluate: false` leaves math exactly as written, even an equation that would otherwise be checked. Nested `math` and `mathscope` inherit it; switch back on with an explicit `evaluate: true`.\n\n`hidden: true` calculates but doesn't show the math – handy for values a graph or a sentence needs, without a wall of definitions. Prose and problems stay visible." },
      {
        example: {
          title: "Calculated or as written",
          code: `${PKG}
#import units: m
#show: math

$a := 3 #m$, $b := 4 #m$
#math(evaluate: {{ev}})[$sqrt(a^2 + b^2) =>$]`,
          controls: [{ kind: "toggle", name: "ev", label: "evaluate", value: true, on: "true", off: "false" }],
          caption: "Switch `evaluate` off and the trailing `=` is just a symbol again.",
        },
      },
      {
        example: {
          title: "Hidden givens",
          code: `${PKG}
#import units: Ω, V, W
#show: math

#math(hidden: {{hide}})[$R := 50 #Ω$, $U := 12 #V$]
The resistor dissipates #value($U^2 / R$).`,
          controls: [{ kind: "toggle", name: "hide", label: "hidden", value: true, on: "true", off: "false" }],
          caption: "Hidden or not, R and U are defined – [`value`](/docs/functions/value) uses them in the sentence.",
        },
      },
      { h2: "clear-at: a fresh start per section" },
      { md: "`clear-at` resets the innermost variable scope at every matching heading or labelled element, back to how it started. Exercises, load cases, chapters: each starts clean. An inner `clear-at` replaces the outer one, and `clear-at: none` turns resets off locally." },
      {
        example: {
          code: `${PKG}
#import units: V, kΩ, mA
#show: math.with(clear-at: heading.where(level: 1))

= Case A
$R := 1 kΩ$, $I := (5 #V) / R => mA$

= Case B
$U := 12 #V$
#defined(style: "inline")`,
          caption: "At “Case B” the definitions of case A are gone: `defined` lists only U.",
        },
      },
      { h2: "Parameters" },
      {
        params: [
          { name: "..body", type: "content", md: "Content to calculate in the current variable scope. Leave it out in a show rule: `#show: math`." },
          { name: "clear-at", type: "auto, none, function, selector, label", default: "auto", md: "Reset the innermost variable scope at matching headings or labels, e.g. `heading.where(level: 1)`. `auto` inherits; `none` disables resets here." },
          { name: "hidden", type: "auto, bool", default: "auto", md: "Hide calculated math, keeping prose and problems. `auto` inherits (initially `false`)." },
          { name: "evaluate", type: "auto, bool", default: "auto", md: "Calculate ordinary math. `auto` inherits, initially `true`. `false` leaves every equation as written." },
          { name: "digits, decimal-separator, notation, …", type: "auto", default: "auto", md: "Every [option](/docs/reference/options): numbers, units, output, graphs, constant displays. `auto` keeps the current setting." },
        ],
      },
      { h2: "Notes" },
      { md: "- **Applying it again** – `#show: math.with(digits: 2)` after an earlier `#show: math` – overrides settings and keeps the same scope, also when a template already turned calculation on.\n- **Heading levels** follow the number of `=`: `====` is level 4.\n- **Unknown options** are reported as problems in the document, like any other." },
      { callout: "warning", title: "It shadows Typst's math module", md: "`#import \"@preview/solveng:2.0.0\": *` replaces Typst's built-in `math`. Reach the native functions as `std.math.upright`, `std.math.equation`, … – or import solveng's under another name: `#import \"@preview/solveng:2.0.0\": math as engineering-math`, then `#show: engineering-math`." },
      {
        example: {
          title: "Renamed",
          code: `#import "@preview/solveng:2.0.0": math as engineering-math
#show: engineering-math.with(digits: 3)

$x := 1/7 =>$`,
        },
      },
      {
        quiz: {
          question: "What does the last line show?",
          code: `#show: math.with(digits: 4)
$x := 12$
#math(digits: 8)[$y := x/7$]
$y =>$`,
          options: ["`y ⇒ 1.714`", "`y ⇒ 1.7142857`", "A problem: y was defined inside the block"],
          answer: 0,
          explain: "`math` doesn't create a scope, so `y` is still defined. Its settings ended with the block, so the result uses the outer 4 digits.",
        },
      },
    ],
    related: ["functions/mathscope", "reference/options", "scopes", "formatting"],
  },

  // ---------------------------------------------------------------- mathscope
  {
    slug: "functions/mathscope",
    title: "mathscope",
    group: "functions",
    summary: "Gives its content a variable scope of its own – isolated, inheriting or shared – while keeping the math settings around it.",
    signature: `mathscope(..body, scope: "isolated") → content`,
    import: `#import "@preview/solveng:2.0.0": mathscope`,
    returns: "content: the body, calculated in the new scope.",
    keywords: ["scope", "isolated", "inherit", "shared", "variants", "local"],
    blocks: [
      { md: "The document already has one global scope, and [`math`](/docs/functions/math) never makes another. Reach for `mathscope` when definitions should stay apart: two design variants, the solution of an exercise, a side calculation that mustn't overwrite `R`." },
      {
        example: {
          title: "Three kinds of scope",
          code: `${PKG}
#import units: V, kΩ, mA
#show: math

$R := 1 kΩ$
#mathscope(scope: "{{scope}}")[
  $R := 2.2 kΩ$, $I := (5 #V) / R => mA$
]
$R =>$`,
          controls: [{ kind: "select", name: "scope", label: "scope", options: ["isolated", "inherit", "shared"], value: "isolated" }],
          caption: "Only `\"shared\"` keeps the inner R = 2.2 kΩ after the block.",
        },
      },
      {
        table: {
          head: ["scope", "sees outer definitions", "keeps its changes"],
          rows: [
            ["`\"isolated\"` (default)", "no – starts empty", "no"],
            ["`\"inherit\"`", "yes", "no"],
            ["`\"shared\"`", "yes", "yes, in the parent"],
          ],
        },
      },
      { h2: "Isolated means empty" },
      { md: "The default is the strictest: nothing from outside is visible. A name defined outside is just an undefined name inside." },
      {
        example: {
          code: `${PKG}
#import units: V, Ω
#show: math

$U := 5 #V$
#mathscope[$P := U^2 / (100 #Ω) =>$]`,
          expectError: true,
          caption: "U isn't defined in an isolated scope. `scope: \"inherit\"` fixes it.",
        },
      },
      { h2: "Variants side by side" },
      { md: "`\"inherit\"` is made for what-if calculations: shared givens outside, each variant changes what it likes inside, and nothing leaks into the next one." },
      {
        example: {
          title: "An LED's series resistor",
          code: `${PKG}
#import units: V, mA, Ω, mW
#show: math

$U_0 := 5 #V$, $U_F := 2 #V$, $I_F := {{i}} mA$
$R := (U_0 - U_F) / I_F => #Ω$

#mathscope(scope: "inherit")[
  $U_0 := 3.3 #V$
  $R := (U_0 - U_F) / I_F => #Ω$, $P := (U_0 - U_F) I_F => mW$
]
$P := (U_0 - U_F) I_F => mW$ // back on 5 V`,
          controls: [{ kind: "slider", name: "i", label: "I_F", min: 2, max: 30, step: 1, value: 10, suffix: " mA" }],
        },
      },
      { h2: "Settings inside and around" },
      { md: "A scope inherits the math settings around it. To change them, put a [`math`](/docs/functions/math) block inside the scope or around it – `mathscope` itself only takes `scope`." },
      {
        example: {
          code: `${PKG}
#show: math

$x := 12$
#mathscope(scope: "inherit")[#math(digits: 2)[$y := x / 7 =>$]]
#mathscope($z := 1$) // one equation is a scope, too`,
        },
      },
      { h2: "Parameters" },
      {
        params: [
          { name: "..body", type: "content", md: "Content in the new variable scope: a block `[…]` or a single equation `$…$`." },
          { name: "scope", type: "str", default: `"isolated"`, md: "`\"isolated\"` starts empty; `\"inherit\"` reads outer definitions and keeps changes local; `\"shared\"` also keeps changes in the parent." },
        ],
      },
      { h2: "Notes" },
      { md: "- **One equation or a block** behave the same way.\n- **`#clear(all: true)`** inside a scope goes back to how the scope started – with `\"inherit\"`, the outer definitions are still there. See [`clear`](/docs/functions/clear).\n- **`clear-at`** resets the *innermost* scope, so headings inside a `mathscope` reset that scope only.\n- **Imports are lexical**, not scoped by `mathscope`: a `#import units: …` holds in its file or block whatever worksheet is around." },
      {
        quiz: {
          question: "Which scope lets a block *read* the outer `R` but keeps its own `R := 2 kΩ` to itself?",
          options: ["`\"isolated\"`", "`\"inherit\"`", "`\"shared\"`"],
          answer: 1,
          explain: "`\"inherit\"` sees outer definitions and forgets its changes at the end. `\"isolated\"` sees nothing; `\"shared\"` writes its changes back.",
        },
      },
    ],
    related: ["functions/math", "functions/clear", "scopes"],
  },

  // ---------------------------------------------------------------- graph
  {
    slug: "functions/graph",
    title: "graph",
    group: "functions",
    summary: "Plots one or more curves – functions, expressions or measured data – and picks ranges, ticks, axis labels and units by itself.",
    signature: "graph(equation, x-range: auto, y-range: auto, mark: none, xmark: (), ymark: (), samples: 400, …) → content",
    import: `#import "@preview/solveng:2.0.0": graph`,
    returns: "content: the graph. Printed with lilaq; in HTML (and on this site) drawn live, with values under the pointer.",
    keywords: ["plot", "chart", "curve", "diagram", "lilaq", "marks", "axis", "data", "scatter"],
    blocks: [
      { md: "This is the complete parameter reference, with an example for each group. For the ideas – how ranges are picked, how units reach the axes – read the [graphs guide](/docs/graphs) first." },
      {
        example: {
          title: "The power of a resistor",
          code: `${PKG}
#import units: Ω, W, V
#show: math

#math(hidden: true)[$R := {{r}} #Ω$, $P_0 := 2 #W$]
#graph($P(U: #V) = U^2/R$, ymark: $P_0$)`,
          controls: [{ kind: "slider", name: "r", label: "R", min: 10, max: 200, step: 5, value: 50, suffix: " Ω" }],
          caption: "The parameter's unit names the x axis (U / V), the curve's unit the y axis. `ymark` marks where the curve reaches 2 W – drag R and watch the marked voltages move.",
        },
      },
      { h2: "Curves: equation" },
      { md: "The first argument is math. Curves are separated by a backslash `\\`. Each curve is:\n\n- a **function** with its parameter: `$P(U: #V) = U^2/R$` – the annotation gives the x axis its unit;\n- a **bare expression** with one free name: `$cos(w)$` – that name is the x axis;\n- a **call** of a function you defined: `$f(t)$`;\n- **data**: a vector, `$u$`, or a vector over another, `$u(t)$` (see below).\n\nCurves share one x axis. With `:=` instead of `=`, a curve also defines its name for later calculations." },
      {
        example: {
          title: "Two curves, and one that stays defined",
          code: `${PKG}
#import units: Ω, V
#show: math

$R := 50 #Ω$
#graph($cos(w) \\ w$, mark: auto)
#graph($P(U: #V) := U^2/R$, x-range: (0, 10))
$P(3 #V) =>$`,
          caption: "`mark: auto` marks where the two curves cross (the solution of cos w = w). The second graph defines P.",
        },
      },
      { h2: "Ranges: x-range, y-range, x-include" },
      { md: "Without ranges, solveng looks for what's interesting – roots, extrema, poles, crossings, the edge of a domain – and rounds to nice ticks. A range is a list of values that must be visible: two or more give the range from the smallest to the largest, a single one widens the picked range. Values are numbers in the axis unit, values with units (`$10 #Ω$`), or expressions of defined variables (`$2 R$`). `x-include` widens the automatic range without fixing it." },
      {
        example: {
          code: `${PKG}
#show: math

#graph($y(x) = x^2 \\ y(x) = 2x + 3$, x-range: (-3, {{xmax}}), y-range: ({{ymin}}, 30))`,
          controls: [
            { kind: "slider", name: "xmax", label: "x-range end", min: 1, max: 8, step: 1, value: 5 },
            { kind: "slider", name: "ymin", label: "y-range start", min: -10, max: 0, step: 1, value: -5 },
          ],
          caption: "Two values each: exactly that range.",
        },
      },
      {
        example: {
          title: "Values with units",
          code: `${PKG}
#import units: ms, s, V
#import constants: e
#show: math

$tau := 2 ms$
#graph($u(t: ms) = 5 #V (1 - #e^(-t/tau))$, x-range: ($0 #s$, $5 tau$))`,
          caption: "The end of the range is an expression: five time constants.",
        },
      },
      { h2: "Marks: mark, xmark, ymark" },
      { md: "Marks default to none, and a requested mark is always visible – the range grows to show it.\n\n- `mark` writes both coordinates on the axes, with guide lines: x values (`2`, `$R$`, `$5 #V$`), `\"roots\"`, `\"extrema\"`, `\"crossings\"`, or `auto` for where two curves cross.\n- `xmark` takes the same, but writes only the x value.\n- `ymark` takes y values and marks every point where a curve reaches one.\n\nCombine them in an array: `(auto, \"extrema\")`." },
      {
        example: {
          code: `${PKG}
#show: math

#graph($x^2 - 2 \\ x$, mark: {{m}})`,
          controls: [{ kind: "select", name: "m", label: "mark", options: ["none", "auto", "\"roots\"", "\"extrema\"", "1.5", "(auto, \"extrema\")"], value: "(auto, \"extrema\")" }],
          caption: "Pick a mark. `\"roots\"` and `\"extrema\"` are searched on every curve, within the range.",
        },
      },
      {
        example: {
          title: "Only the x value: xmark",
          code: `${PKG}
#show: math

#graph($x^3 - x$, xmark: 0.5)`,
        },
      },
      { h2: "Sampling: samples" },
      { md: "A function curve is drawn through `samples` points, 400 by default – plenty for smooth curves. Fewer points make a fast oscillation lie." },
      {
        example: {
          code: `${PKG}
#show: math

#graph($sin(12 x)$, x-range: (0, 3), samples: {{n}})`,
          controls: [{ kind: "slider", name: "n", label: "samples", min: 10, max: 400, step: 5, value: 400 }],
          caption: "Drag to 15 or 20 samples: the curve turns into a slow wave that isn't there. That's aliasing.",
        },
      },
      { h2: "Units and the variable: variable, x-unit, y-unit" },
      { md: "Units normally follow from what's plotted: y from the curves (SI with a fitting prefix, or [`preferred-units`](/docs/reference/options)), x from the curve's parameter, the ranges or the marks. Override them when you want another: `x-unit` and `y-unit` are unit expressions, checked against the curve's dimension. `variable` names the independent variable of a bare expression – and shadows a worksheet definition of that name for the plot." },
      {
        example: {
          code: `${PKG}
#import units: Ω, V, mW
#show: math

$R := 50 #Ω$, $U := 3 #V$
#graph($U^2/R$, variable: $U$, x-unit: $#V$, y-unit: $mW$, x-range: (0, 2))`,
          caption: "U has a value, but `variable: $U$` plots over it anyway – in volts, with the power in mW.",
        },
      },
      { h2: "Data" },
      { md: "A vector is plotted as points joined in order, with a dot at each when there are at most 30. `$u$` alone is over the index k; `$u(t)$` is over another vector of the same length. Function curves can go alongside, and `lines: none` draws the measurement as points alone." },
      {
        example: {
          title: "Measured points and a line",
          code: `${PKG}
#show: math

$t := vec(0, 1, 2, 3, 4)$, $u := vec(0.1, 0.9, 2.1, 2.9, 4.2)$
#graph($u(t) \\ t$, lines: (none, auto), dots: ("diamond", auto))`,
          caption: "Measured u as diamonds, without a line; the expression t is drawn as the line u = t.",
        },
      },
      { h2: "Looks: colors, lines, dots, joined" },
      { md: "Each takes a value or an array that's used curve after curve, repeating; `auto` in the array keeps the default for that curve. The defaults – Petroff's ten colors, solid/dashed/dotted/dash-dotted lines, circle/square/triangle/diamond dots – keep curves apart for color-blind readers and in grayscale prints. Change them for every graph with the options `graph-colors`, `graph-lines`, `graph-dots`, `graph-joined`." },
      {
        example: {
          code: `${PKG}
#show: math

#graph($sin(x) \\ cos(x)$, x-range: (0, 6), colors: (gray, red), lines: ("{{line}}", auto))`,
          controls: [{ kind: "select", name: "line", label: "first line", options: ["solid", "dashed", "dotted", "dash-dotted", "densely-dashed", "loosely-dotted"], value: "dotted" }],
        },
      },
      { md: "**Piecewise functions** break their line at a jump and draw a value that holds at a single x as a dot. `joined: true` connects the pieces instead – for `floor`, `rect`, `sgn` and `ceil` that's a vertical line at each step." },
      {
        example: {
          code: `${PKG}
#show: math

$f(x) := cases(1 & "for" x < 0, 2 & "for" x = 1, x & "else")$
#graph($f(x)$, x-range: (-2, 3), joined: {{j}})`,
          controls: [{ kind: "toggle", name: "j", label: "joined", value: false, on: "true", off: "false" }],
        },
      },
      { h2: "Size: width, height" },
      { md: "`width` and `height` are the whole graph, labels included; `auto` makes the plot area 8 × 5 cm. `1fr` is the container's width – in a grid, the column's." },
      {
        example: {
          code: `${PKG}
#show: math

#graph($x^2 - 1$, width: {{w}}cm, height: 4cm)`,
          controls: [{ kind: "slider", name: "w", label: "width", min: 5, max: 14, step: 1, value: 10, suffix: " cm" }],
        },
      },
      { h2: "Options for one graph" },
      { md: "Every option of [`math`](/docs/functions/math) works here too, for this graph only – `digits` for the values at the marks, `decimal-separator` for the ticks, `preferred-units` for the axes." },
      {
        example: {
          code: `${PKG}
#show: math

#graph($x^2$, mark: 1.2345, digits: {{d}})`,
          controls: [{ kind: "slider", name: "d", label: "digits", min: 1, max: 6, step: 1, value: 2 }],
        },
      },
      { h2: "Advanced curves: curves" },
      { md: "`curves` takes dictionaries, one per curve, each with its own `expression` and, optionally, `color`, `line`, `dot` and `joined` – when per-curve settings read better than parallel arrays." },
      {
        example: {
          code: `${PKG}
#show: math

#graph(curves: (
  (expression: $sin(x)$, color: red, line: "dashed"),
  (expression: $cos(x)$),
))`,
        },
      },
      { h2: "Parameters" },
      {
        params: [
          { name: "equation", type: "none, content", default: "none", md: "Math curves separated by a backslash: `$cos(w) \\ w$`, `$P(U: #V) = U^2/R$`." },
          { name: "x-range", type: "auto, array, content, int, float", default: "auto", md: "x values that must be visible: numbers in the x axis's unit (`(1, 2)`), values (`($10 #Ω$, $1 kΩ$)`) or expressions of defined variables (`(0, $2 R$)`). Two or more give the range; one widens the picked range. `auto`: around the marked x values, else picked from the curves." },
          { name: "y-range", type: "auto, array, content, int, float", default: "auto", md: "y values that must be visible, like `x-range` (numbers in the y axis's SI unit). Two or more give the exact y range. `auto`: fitted to the curves." },
          { name: "x-include", type: "array, content, int, float", default: "()", md: "Values to include in an automatic x range, without drawing marks." },
          { name: "variable", type: "auto, str, content", default: "auto", md: "The independent name, explicitly. Shadows a worksheet definition of it for plotting." },
          { name: "x-unit, y-unit", type: "auto, content", default: "auto", md: "Axis units as unit expressions (`$mA$`), checked against the curves' dimensions." },
          { name: "mark", type: "none, auto, array, content, int, float", default: "none", md: "What to mark, both coordinates written on the axes: x values like `x-range`, `\"crossings\"`, `\"roots\"`, `\"extrema\"` (searched within the range), `auto` (where two curves cross), or an array of these. `()` or `none`: no marks." },
          { name: "xmark", type: "none, array, str, content, int, float", default: "()", md: "Like `mark`, but only the x value is written on the axis." },
          { name: "ymark", type: "none, array, content, int, float", default: "()", md: "y values to mark, like `y-range`: every point where a curve reaches one, with guide lines. With `ymark` the y range is around the marked values." },
          { name: "samples", type: "int", default: "400", md: "Number of sample points per function curve." },
          { name: "width", type: "auto, length, ratio, relative, fraction", default: "auto", md: "Width of the whole graph, labels included: `12cm`, `300pt`, `100%`; `1fr` is the container's width. `auto`: a plot area of 8 cm." },
          { name: "height", type: "auto, length, ratio, relative, fraction", default: "auto", md: "Like `width`; `auto`: a plot area of 5 cm. A share is of the container's height, or of the width when the page grows with its content." },
          { name: "colors", type: "auto, array, color", default: "auto", md: "Curve colors, curve after curve and repeating: `(gray, blue, rgb(\"#f98eff\"))`. `auto` in the array keeps that curve's default. Default: the option `graph-colors`, Petroff's ten colors. Gradients work in print only; in HTML they're an error." },
          { name: "lines", type: "auto, array, str, dictionary, none", default: "auto", md: "Line styles like `colors`: `\"solid\"`, `\"dashed\"`, `\"dotted\"`, `\"dash-dotted\"`, `\"densely-dashed\"`, `\"loosely-dotted\"`, an array of lengths like `(4pt, 2pt)`, or `none` – points alone. Default: the option `graph-lines`." },
          { name: "dots", type: "auto, array, str, none", default: "auto", md: "Dot shapes like `colors`: `\"circle\"`, `\"square\"`, `\"triangle\"`, `\"triangle-down\"`, `\"diamond\"`, `\"pentagon\"`, `\"hexagon\"`, `\"star\"`, `\"cross\"`, `\"plus\"`, or `none`. Drawn for data (at most 30 points, or without a line) and a function's single points. Default: the option `graph-dots`." },
          { name: "joined", type: "auto, bool, array", default: "auto", md: "Join everything a curve draws with a line – a function's pieces and single points, steps of `floor`/`rect`/`sgn`/`ceil`. Like `colors`. Default: the option `graph-joined`, `false`." },
          { name: "curves", type: "array", default: "()", md: "Advanced curves: dictionaries with `expression`, `color`, `line`, `dot` and `joined`." },
          { name: "..args", type: "any", md: "Any other named argument is reported as a problem (`unknown argument 'title' for graph`). For lilaq's own settings – titles, grids – plot with lilaq directly." },
          OPTIONS_PARAM,
        ],
      },
      { h2: "Notes" },
      { md: "- **Units in ranges must match.** A plain number counts in the axis unit; a value with another dimension (`$3 #A$` on a volt axis) is an error.\n- **Complex values** can't be plotted. A sample whose imaginary part isn't zero (within its numerical error) is a gap; if everything is complex, the graph asks for `Re(…)`, `Im(…)`, `abs(…)` or `arg(…)`.\n- **Layers:** curves are drawn in the order written, each over the ones before.\n- **Dirac** impulses have no pointwise value and can't be plotted.\n- **Full control:** for anything beyond this, plot with lilaq directly – [`compile`](/docs/functions/compile) turns an expression into a Typst function for it." },
      {
        quiz: {
          question: "You plot `$sin(12 x)$` over 0…3 with `samples: 15`. What do you see?",
          options: ["The same curve, just a little less smooth", "A slower wave that isn't really there", "An error: too few samples"],
          answer: 1,
          explain: "With about 5 samples per unit and a period of about 0.5, the points skip whole oscillations – the joined points trace an alias. Keep the default 400 unless you have a reason.",
        },
      },
    ],
    related: ["graphs", "signals", "functions/math", "reference/options"],
  },

  // ---------------------------------------------------------------- defined
  {
    slug: "functions/defined",
    title: "defined",
    group: "functions",
    summary: "Lists the definitions made so far, in order – all of them, or a selection by name or kind, as a grid, column, list or inline.",
    signature: `defined(names: "all", kinds: ("given", "computed", "function"), where: name => true, style: auto, …) → content`,
    import: `#import "@preview/solveng:2.0.0": defined`,
    returns: "content: the list of definitions with their values.",
    keywords: ["list", "summary", "variables", "overview", "table", "givens", "results"],
    blocks: [
      { md: "`#defined()` is the summary box of a calculation: every value and function defined so far in the current scope, in the order they were made. Filters pick what to show – the givens for a parts list, the results for a summary – and a style picks the layout." },
      {
        example: {
          title: "Pick a style",
          code: `${PKG}
#import units: kΩ, V
#show: math

$R := 7.2 kΩ$, $U := 5 #V$, $I := U/R$, $f(x) := x^2$
#defined(style: "{{style}}")`,
          controls: [{ kind: "select", name: "style", label: "style", options: ["grid", "column", "inline", "list"], value: "grid" }],
          caption: "`\"grid\"` fills as many columns as fit, aligned at `=`; `\"column\"` is one column; `\"inline\"` runs in the text; `\"list\"` uses bullets.",
        },
      },
      { h2: "Filtering: kinds, names, where" },
      { md: "A definition is `\"given\"` (a plain value like `R := 7.2 kΩ`), `\"computed\"` (calculated from others, `I := U/R`) or a `\"function\"`. `names` takes a string, an array of strings, a regex, or a function from name to bool; `where` adds a predicate on top." },
      {
        example: {
          code: `${PKG}
#import units: kΩ, V
#show: math

$R_1 := 7.2 kΩ$, $R_2 := 2.2 kΩ$, $U := 5 #V$
$I := U / (R_1 + R_2)$, $f(x) := x^2$
#defined(kinds: {{kinds}}, style: "inline")`,
          controls: [{ kind: "select", name: "kinds", label: "kinds", options: ["\"given\"", "\"computed\"", "\"function\"", "(\"given\", \"computed\")"], value: "\"given\"" }],
        },
      },
      {
        example: {
          title: "By name",
          code: `${PKG}
#import units: kΩ, V
#show: math

$R_1 := 7.2 kΩ$, $R_2 := 2.2 kΩ$, $U := 5 #V$, $I := U / (R_1 + R_2)$
#defined(names: regex("^R"), style: "column")
#defined(names: ("U", "I"), style: "inline")
#defined(where: n => n != "U", style: "inline")`,
          caption: "A regex for every R, a list of exact names, and a predicate that leaves U out.",
        },
      },
      { h2: "Options for this list" },
      { md: "Every [option](/docs/reference/options) of `math` works here, for this list only. `show-si: true` adds the SI value next to each, `digits` rounds." },
      {
        example: {
          code: `${PKG}
#import units: mAh, mA, h
#show: math

$Q_0 := 3450 mAh$, $I := 30 mA$, $t := Q_0 / I$
#defined(show-si: {{si}}, digits: {{d}})`,
          controls: [
            { kind: "toggle", name: "si", label: "show-si", value: true, on: "true", off: "false" },
            { kind: "slider", name: "d", label: "digits", min: 1, max: 8, step: 1, value: 3 },
          ],
        },
      },
      { h2: "Parameters" },
      {
        params: [
          { name: "names", type: "str, regex, function, array", default: `"all"`, md: "Names to show: strings match literally (except `\"all\"`), a regex matches, a function `name => bool` decides." },
          { name: "kinds", type: "str, array", default: `("given", "computed", "function")`, md: "`\"given\"`, `\"computed\"`, `\"function\"`, or an array of them. All three by default." },
          { name: "where", type: "function", default: "name => true", md: "An additional predicate on each name." },
          { name: "style", type: "auto, str", default: "auto", md: "`\"grid\"`, `\"column\"`, `\"inline\"` or `\"list\"`. `auto`: the option `defined-style`, by default `\"grid\"`." },
          OPTIONS_PARAM,
        ],
      },
      { h2: "Notes" },
      { md: "- **The current scope**: inside a [`mathscope`](/docs/functions/mathscope), `defined` lists that scope's definitions.\n- **In HTML** the grid's columns reflow with the window, and `\"column\"` becomes a table.\n- **All lists at once**: `#show: math.with(defined-style: \"column\")` changes the default style.\n- **Functions** show their body: `f(x) = x²`." },
    ],
    related: ["functions/value", "functions/clear", "reference/options", "formatting"],
  },

  // ---------------------------------------------------------------- value
  {
    slug: "functions/value",
    title: "value",
    group: "functions",
    summary: "The result of one expression on its own, for running text: what `$U =>$` would show after its `=`.",
    signature: "value(expr) → content",
    import: `#import "@preview/solveng:2.0.0": value`,
    returns: "content: the result, inline.",
    keywords: ["inline", "text", "sentence", "prose", "result"],
    blocks: [
      { md: "Reports read better when numbers live in sentences. `#value($U$)` drops the result right into the text – calculated with the definitions at that point, which it leaves untouched." },
      {
        example: {
          code: `${PKG}
#import units: kΩ, mA, mV, Ω
#show: math

$R := {{r}} kΩ$, $I := 2 mA$, $U := R I$, $f(x: #Ω) := 2 x$
The voltage is #value($U$) (#value($U => mV$)), and #value($f(3 #Ω)$) for 3 Ω.`,
          controls: [{ kind: "slider", name: "r", label: "R", min: 0.5, max: 10, step: 0.1, value: 4.7, suffix: " kΩ" }],
          caption: "The sentence follows the slider.",
        },
      },
      { h2: "The clause after =" },
      { md: "Everything a trailing `=` takes works here: a unit, [modifiers](/docs/modifiers/digits) like `digits(2)` or `sci`. A trailing `->` gives the symbolic result. A function's name alone shows its body." },
      {
        example: {
          code: `${PKG}
#import units: V, kΩ, µA
#import modifiers: digits, sci
#show: math

$U := 5 #V$, $R := 7.2 kΩ$, $g(x) := x^2 + 1$
- #value($U / R => µA, digits(2)$)
- #value($U / R => sci$)
- #value($(a + b)^2 - a^2 ->$)
- #value($g$)`,
        },
      },
      { h2: "Parameters" },
      {
        params: [
          { name: "expr", type: "content", md: "One expression as math: `$U$`, `$f(2)$`, `$U / 2 => mV$`, `$x^2 - 1 ->$`." },
        ],
      },
      { h2: "Notes" },
      { md: "- **It defines nothing.** `#value($x := 2$)` is a problem – definitions belong in the worksheet, and `value` only shows them.\n- **One expression** per `value`; for several lines, use several.\n- **Hidden givens** pair well with it: `#math(hidden: true)[…]` defines without showing, then sentences use the values. See [`math`](/docs/functions/math)." },
    ],
    related: ["functions/defined", "functions/math", "formatting"],
  },

  // ---------------------------------------------------------------- clear
  {
    slug: "functions/clear",
    title: "clear",
    group: "functions",
    summary: "Forgets definitions: the names given, or with `all: true` every one made so far in the current scope.",
    signature: "clear(..names, all: false) → content",
    import: `#import "@preview/solveng:2.0.0": clear`,
    returns: "content: nothing visible, unless a name wasn't defined (then a problem).",
    keywords: ["forget", "reset", "undefine", "delete", "remove"],
    blocks: [
      { md: "Defining a name again already replaces it. `clear` is for when the name should be *undefined* again – so a later `->` keeps it as a symbol, or so a typo can't silently reuse an old value." },
      {
        example: {
          code: `${PKG}
#import units: mA, A, V
#show: math

$I := 30 mA$, $f(x) := 2 x$, $U := 5 #V$
#clear("I", $f$)
#defined(style: "inline")
$P := U I ->$ // I is a symbol again`,
          caption: "Names as strings or in math. A value and a function share their name, so `\"f\"` forgets either.",
        },
      },
      {
        example: {
          title: "Everything",
          code: `${PKG}
#import units: V, A
#show: math

$U := 5 #V$, $I := 2 #A$
#clear({{what}})
$U := 12 #V$
#defined(style: "inline")`,
          controls: [{ kind: "toggle", name: "what", label: "all: true", value: true, on: "all: true", off: "\"U\"" }],
          caption: "Toggle between forgetting everything and forgetting U alone. `clear()` with neither is a problem: it wants names or `all: true`.",
        },
      },
      { h2: "Inside a mathscope" },
      { md: "`#clear(all: true)` returns the current scope to how it started. In an `\"inherit\"` scope the outer definitions are still visible afterwards – only the local ones go." },
      {
        example: {
          code: `${PKG}
#show: math

$x := 12$
#mathscope(scope: "inherit")[
  $x := 1$
  #clear(all: true)
  $x =>$
]`,
          caption: "Back to the inherited 12.",
        },
      },
      { h2: "Parameters" },
      {
        params: [
          { name: "..names", type: "str, content", md: "Names to forget: strings (`\"Q_0\"`, `\"tau\"`) or math (`$Q_0$`). `\"f()\"` and `\"f(x)\"` read as `\"f\"`." },
          { name: "all", type: "bool", default: "false", md: "Forget every definition of the current scope instead. Without names and without `all: true`, `clear` reports a problem." },
        ],
      },
      { h2: "Notes" },
      { md: "- **Undefined names** are reported as a problem – a typo like `#clear(\"R1\")` for `R_1` shouldn't pass unnoticed.\n- **Bindings stay:** units, constants and operations are Typst bindings, not definitions. `clear` leaves them alone.\n- **Automatic resets** at headings: the option `clear-at` of [`math`](/docs/functions/math)." },
    ],
    related: ["functions/mathscope", "functions/defined", "functions/math", "scopes"],
  },

  // ---------------------------------------------------------------- compile
  {
    slug: "functions/compile",
    title: "compile",
    group: "functions",
    summary: "Turns a worksheet expression into an ordinary Typst function that takes numbers and returns a float – for tables, loops and your own plots.",
    signature: "compile(expr, parameters: (:), result-unit: none) → function",
    import: `#import "@preview/solveng:2.0.0": compile`,
    returns: "function: one argument per parameter, returning a float in `result-unit` (else SI), or `none` where the expression isn't defined.",
    keywords: ["function", "numeric", "float", "export", "typst function", "loop", "table"],
    blocks: [
      { md: "Sometimes you need a number in Typst code, not in math: a table of operating points, a custom lilaq plot, a value for a layout. `compile` takes a snapshot of an expression – with the worksheet's definitions at that point – and hands back a plain Typst function. It reads the worksheet's state, so it needs `context`." },
      {
        example: {
          code: `${PKG}
#import units: V, W, Ω
#show: math

#math(hidden: true)[$R := 100 #Ω$]
#context {
  let P = compile($U^2/R$, parameters: (U: $#V$), result-unit: $#W$)
  [At {{u}} V the resistor takes #P({{u}}) W.]
}`,
          controls: [{ kind: "slider", name: "u", label: "U", min: 1, max: 30, step: 1, value: 12, suffix: " V" }],
          caption: "`P` takes volts, returns watts – a float, ready for Typst code.",
        },
      },
      { h2: "Arguments and units" },
      { md: "`parameters` is an ordered dictionary: names in argument order, each mapped to its unit (`$mA$`) or `auto` for SI. An argument is a number in that unit, or a quantity as math (`$5 #V$`). Without `parameters`, the free names of the expression are taken, in SI units – fine for one name; with several, list them so you know their order." },
      {
        example: {
          code: `${PKG}
#import units: V, mA, Ω
#show: math

#context {
  let g = compile($x y$, parameters: (x: auto, y: $mA$))
  let P = compile($U^2/R$, parameters: (U: $#V$, R: $#Ω$))
  let s = compile($sqrt(x)$)
  [#g(2, 3) and #P($5 #V$, 50) and #s(9)]
}`,
          caption: "`g(2, 3)` is 2 × 3 mA = 0.006 – in A, SI. `P` takes a number or a quantity. `s` has no `parameters`: its one free name x is inferred.",
        },
      },
      { h2: "A snapshot" },
      { md: "The function remembers the definitions where it was compiled. Redefining `R` later changes later snapshots, not this one." },
      {
        example: {
          code: `${PKG}
#import units: V, W, Ω
#show: math

$R := 100 #Ω$
#context { let P = compile($U^2/R$, parameters: (U: $#V$)); [#P(10) W] }
$R := 50 #Ω$
#context { let P = compile($U^2/R$, parameters: (U: $#V$)); [#P(10) W] }`,
        },
      },
      { h2: "Undefined points" },
      { md: "Where the expression has no real value the function returns `none`, which Typst prints as nothing. Test for it in code: `if f(x) != none { … }`." },
      {
        example: {
          code: `${PKG}
#show: math

#context {
  let f = compile($sqrt(x)$)
  [f(4) = #f(4), f(−1) = #repr(f(-1))]
}`,
        },
      },
      { h2: "Parameters" },
      {
        params: [
          { name: "expr", type: "content", md: "The expression as math, using defined variables and the parameters." },
          { name: "parameters", type: "dictionary", default: "(:)", md: "Names mapped to unit expressions or `auto` (SI), in argument order. Empty: the free names are inferred." },
          { name: "result-unit", type: "none, content", default: "none", md: "Unit of the returned number, as math (`$#W$`). `none`: SI." },
        ],
      },
      { h2: "Notes" },
      { md: "- **Needs context**: call it inside `#context { … }`.\n- **Mistakes stop the compilation.** Unlike the worksheet, a wrong argument count, a non-math expression or an unknown named argument panics – it's Typst code, and code should fail loudly.\n- **Real only**: complex results can't become floats.\n- **No units out**: the float is in `result-unit`. Keep the unit next to it in your text." },
    ],
    related: ["functions/value", "functions/graph", "numerics"],
  },

  // ---------------------------------------------------------------- base-unit
  {
    slug: "functions/base-unit",
    title: "base-unit",
    group: "functions",
    summary: "Creates a unit of a new, independent dimension – bits, pixels, people, items – that never mixes with the SI dimensions.",
    signature: "base-unit(identity, display: auto) → content",
    import: `#import "@preview/solveng:2.0.0": base-unit`,
    returns: "content: a unit binding to use in math.",
    keywords: ["custom unit", "dimension", "own unit", "bit", "currency", "new unit"],
    blocks: [
      { md: "SI has seven base dimensions. Engineering has more: information, pixels, cycles, money. `base-unit` adds one. Its **identity** names the dimension – two base units with the same identity are the same dimension, also when declared in different files – and its **display** says how it's written." },
      {
        example: {
          code: `${PKG}
#import units: s
#let bit = base-unit("information", display: "bit")
#let byte = unit($8 bit$, display: $"B"$)
#show: math

$d := 3 byte = bit$
$r := (12 byte) / (2 #s) =>$`,
          caption: "A byte is derived with [`unit`](/docs/functions/unit). Bits per second come out by themselves.",
        },
      },
      {
        example: {
          title: "It doesn't mix",
          code: `${PKG}
#import units: kg
#let pcs = base-unit("count", display: "pcs")
#show: math

$n := 12 pcs$, $m := 3 kg$
$n + m =>$`,
          expectError: true,
          caption: "Adding pieces to kilograms is a unit error, as it should be.",
        },
      },
      { h2: "Parameters" },
      {
        params: [
          { name: "identity", type: "str", md: "The dimension, the same in every file and worksheet: `\"information\"`." },
          { name: "display", type: "auto, str, content, function", default: "auto", md: "How the unit is shown in input and results: a string (set upright), content, or a function returning content." },
        ],
      },
      { h2: "Notes" },
      { md: "- **Results of the dimension** are shown in the base unit (or a [`unit`](/docs/functions/unit) of it, if you ask).\n- **Bindings**: like every unit, it holds where the `#let` is visible. A one-letter name needs `#` in math.\n- **Names with `_` or `-`** need `#` in math too: `$3 #my_unit$`." },
    ],
    related: ["functions/unit", "custom-units", "units"],
  },

  // ---------------------------------------------------------------- unit
  {
    slug: "functions/unit",
    title: "unit",
    group: "functions",
    summary: "Defines a unit as an expression of other units – bar, rpm, inch – with an optional display.",
    signature: "unit(expression, display: auto) → content",
    import: `#import "@preview/solveng:2.0.0": unit`,
    returns: "content: a unit binding to use in math.",
    keywords: ["custom unit", "derived unit", "bar", "rpm", "inch", "psi", "own unit"],
    blocks: [
      { md: "The expression *is* the unit's value: a positive number times units from `units`, [`base-unit`](/docs/functions/base-unit) or other `unit`s. `display` changes only how it's written, never what it means." },
      {
        example: {
          code: `${PKG}
#import units: kPa, min, s
#let bar = unit($100 kPa$, display: "bar")
#let rpm = unit($1/min$, display: $"rpm"$)
#show: math

$p := {{p}} bar$, $n := 3000 rpm$
$p => kPa$, $n => 1/#s$`,
          controls: [{ kind: "slider", name: "p", label: "p", min: 0.5, max: 10, step: 0.5, value: 3, suffix: " bar" }],
        },
      },
      { h2: "Without a display" },
      { md: "A unit without `display` shows its expression, written as results are: `unit($2.54 #cm$)` is 2.54 cm. That display counts as one unit: next to a number it's joined with a dot (1 · 2.54 cm), under a power or after a slash it's parenthesized ((2.54 cm)², m/(2.54 cm))." },
      {
        example: {
          code: `${PKG}
#import units: cm
#let inch = unit($2.54 #cm$)
#let inches = unit($2.54 #cm$, display: $"in"$)
#show: math

$a := 3 #inch = inch$
$b := 3 inches => cm$`,
          caption: "`inch` shows its expression, `inches` its display. Only one-letter bindings need `#` in math; `#inch` works too.",
        },
      },
      { h2: "In results" },
      { md: "A result is shown in a unit of its inputs, so `$p / 2 =>$` stays in bar. A pressure calculated from other units comes out in SI – ask for your unit after `=>`, or make it the default for its dimension with the option `preferred-units`." },
      {
        example: {
          code: `${PKG}
#import units: kPa, N, cm
#let bar = unit($100 kPa$, display: "bar")
#show: math

$F := 500 #N$, $A := 25 cm^2$
$F / A =>$, $F / A => bar$
#math(preferred-units: ($bar$,))[$F / A =>$]`,
        },
      },
      { h2: "Parameters" },
      {
        params: [
          { name: "expression", type: "content", md: "The unit's value as math: `$100 kPa$`, `$1/min$`, `$#m / #s$`. Its number must be positive." },
          { name: "display", type: "auto, str, content, function", default: "auto", md: "How the unit is shown: a string (upright), content, or a function returning content. `auto`: its expression." },
        ],
      },
      { h2: "Notes" },
      { md: "- **Names keep their spelling**: a unit bound as `tau` isn't shown as τ.\n- **Whole-number powers only** in unit expressions: `#m^2` yes, `#m^(1/2)` no.\n- **Affine units** like °C as a reading aren't units – use [`celsius`](/docs/operations/celsius)." },
    ],
    related: ["functions/base-unit", "custom-units", "units", "reference/units"],
  },

  // ---------------------------------------------------------------- constant
  {
    slug: "functions/constant",
    title: "constant",
    group: "functions",
    summary: "Names a constant by an identity and the expression it stands for – kept by name in exact results, calculated with its value.",
    signature: "constant(identity, expression) → content",
    import: `#import "@preview/solveng:2.0.0": constant`,
    returns: "content: a constant binding to use in math.",
    keywords: ["constant", "standard gravity", "physical constant", "identity", "symbol", "g0"],
    blocks: [
      { md: "A constant is more than a number: `$E := m #c^2 ->$` should read m c², not m · 8.988·10¹⁶ m²/s². `constant` gives a value an **identity** (what it is) and an **expression** (what it's worth, with units). Exact results keep it by identity; numbers use the expression." },
      {
        example: {
          code: `${PKG}
#import units: m, s, kg
#let g0 = constant("standard-gravity", $9.80665 #m / #s^2$)
#show: math.with(constant-displays: ("standard-gravity": $g_0$))

$F := 2 kg #g0 =>$
$F ->$`,
          caption: "`=>` calculates with 9.80665 m/s²; `->` keeps g₀ as a symbol.",
        },
      },
      { h2: "How it's shown" },
      { md: "The look belongs to the math settings, not the binding: the option `constant-displays` maps identities to content. Built in are `pi` (π), `euler` (e), `imaginary` (i) and `speed-of-light` (c). An identity without a display is shown as its expression." },
      {
        example: {
          code: `${PKG}
#import units: m, s, kg
#let g0 = constant("standard-gravity", $9.80665 #m / #s^2$)
#show: math

$F := 2 kg #g0 ->$`,
          caption: "No display for `\"standard-gravity\"`: the constant shows as its expression.",
        },
      },
      { h2: "Identities the engine knows" },
      { md: "With the identities `\"pi\"`, `\"euler\"` and `\"imaginary\"`, the engine calculates exactly once the expression has that value – your own π is the real π. Any other identity is kept by name in exact results and calculated with its expression, which must have a value." },
      {
        example: {
          code: `${PKG}
#import constants: pi
#import modifiers: substitute
#let p = constant("pi", $#calc.pi$)
#let turn = constant("turn", $2 pi$)
#show: math.with(constant-displays: (turn: $tau$))

$sin(#p/6) ->$
$A := turn r^2/2 ->$
$A -> substitute(turn)$`,
          caption: "`p`'s expression is π, so it calculates as exact π: sin(π/6) is ½. The turn τ stays a symbol until [`substitute`](/docs/modifiers/substitute) puts its expression in.",
        },
      },
      { h2: "Parameters" },
      {
        params: [
          { name: "identity", type: "str", md: "What the constant is, the same in every file and worksheet: `\"standard-gravity\"`." },
          { name: "expression", type: "content", md: "Its value as math: numbers, units and other constants." },
        ],
      },
      { h2: "Notes" },
      { md: "- **Built-in constants** come from `#import constants: pi, e, i, c`. Without the import, `pi` in math is just the letter π, a name without a value.\n- **One-letter bindings** need `#` in math: `#c`, `#e`, `#i`.\n- **Nested settings** add to the displays around them." },
    ],
    related: ["constants", "modifiers/substitute", "reference/options"],
  },

  // ---------------------------------------------------------------- operation
  {
    slug: "functions/operation",
    title: "operation",
    group: "functions",
    summary: "Binds one of solveng's operations under a name and look of your choice – `triangle` for `tri`, `conv` for `convolve`.",
    signature: "operation(name, display: auto) → function",
    import: `#import "@preview/solveng:2.0.0": operation`,
    returns: "function: an operation binding to call in math, also with named arguments.",
    keywords: ["rename", "alias", "operator", "custom name", "display"],
    blocks: [
      { md: "Operations come from `#import operations: tri, rect, …` under their own names. `operation(\"tri\")` gives you the same operation under any binding name, with any look. The **name** picks the operation, whatever the binding is called." },
      {
        example: {
          code: `${PKG}
#let triangle = operation("tri", display: "triangle")
#let step = operation("sgn", display: $op(sigma)$)
#let conv = operation("convolve")
#show: math

$triangle({{x}}) =>$, $step({{x}} - 1/2) =>$
$conv(vec(1, 2), vec(1, 1)) =>$`,
          controls: [{ kind: "slider", name: "x", label: "x", min: -1.5, max: 1.5, step: 0.25, value: 0.5 }],
        },
      },
      { h2: "Named arguments" },
      { md: "An operation binding is a Typst function, so Typst parses named arguments for it – `$conv(x, h, step: 2)$`. `std.math.op(\"tri\")` would only be a symbol." },
      { h2: "Parameters" },
      {
        params: [
          { name: "name", type: "str", md: "The operation: `\"tri\"`, `\"convolve\"`, `\"linear-solve\"` – any of the [operations](/docs/operations/tri)." },
          { name: "display", type: "auto, str, content, function", default: "auto", md: "How it's shown in input and results: a string (an operator name, as `std.math.op` writes it), content, or a function returning content." },
        ],
      },
      { h2: "Notes" },
      { md: "- **Quoted names never select an operation**: `$\"tri\"(x)$` is a variable called tri.\n- **A worksheet function** is the other way to rename: `$Lambda(t) := tri(t)$`. It's shown as Λ and works the same in calculations." },
    ],
    related: ["operations/tri", "operations/convolve", "functions"],
  },

  // ---------------------------------------------------------------- any
  {
    slug: "functions/any",
    title: "any",
    group: "functions",
    summary: "The type of a function parameter that accepts every unit: `$f(x: any) := 2 x$`.",
    signature: "any",
    import: `#import "@preview/solveng:2.0.0": any`,
    keywords: ["type", "parameter", "generic", "unit", "annotation"],
    blocks: [
      { md: "A parameter's annotation is a type: `f(I: mA)` takes currents only, and `f(2)` without a unit is a problem. `any` says the opposite on purpose – volts, metres, plain numbers, all welcome. Use it for helpers that don't care about dimensions: doubling, averaging, a safety factor." },
      {
        example: {
          code: `${PKG}
#import units: V, mA
#show: math

$d(x: any) := 2 x$
$d(3 #V) =>$, $d(2 mA) =>$, $d(4) =>$`,
        },
      },
      {
        example: {
          title: "The difference to a unit type",
          code: `${PKG}
#import units: V
#show: math

$h(x: #V) := 2 x$
$h(3) =>$`,
          expectError: true,
          caption: "A typed parameter insists on its dimension.",
        },
      },
      { h2: "Shown in other languages" },
      { md: "Types appear as chips in the equation. The option `words` changes the word shown – what you type stays `any`." },
      {
        example: {
          code: `${PKG}
#show: math.with(words: (any: "beliebig"))

$d(x: any) := 2 x$`,
        },
      },
      { h2: "Notes" },
      { md: "- **Unannotated parameters** already take anything – unless the body passes them to a typed parameter, which gives them that type. `any` makes the intent explicit and shows it.\n- **It comes with the package import**, `*` included – unlike units and modifiers.\n- **How types are shown**: option `show-types` – `\"chip\"`, `\"plain\"` or `\"none\"`." },
    ],
    related: ["functions", "units", "reference/options"],
  },
];
