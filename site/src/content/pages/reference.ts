import type { Page } from "../types";

export const pages: Page[] = [
  {
    slug: "reference/options",
    title: "Options",
    group: "reference",
    summary: "Every option of `math` – and of `defined` and `graph` – with its type, default and meaning.",
    keywords: ["options", "settings", "math.with", "defaults", "configuration", "digits", "prefixes", "notation", "errors", "graph colors"],
    blocks: [
      { md: "Options go to `#show: math.with(…)` for the document, to `#math(…)[…]` for a block, and – where it makes sense – to `defined(…)` and `graph(…)` for one call. Every option defaults to `auto`, which keeps the setting in force; the defaults below are what's in force when nothing set them. Closest settings win, option by option: see [Settings and scopes](/docs/scopes)." },
      { widget: "option-explorer" },
      { h2: "Calculation" },
      { md: "These belong to `math` itself, not to `defined` or `graph`." },
      {
        table: {
          head: ["option", "type", "default", "meaning"],
          rows: [
            ["`evaluate`", "auto, bool", "`true`", "Calculate ordinary math. `false` leaves math as written, even a single equation; nested blocks and scopes inherit it until an explicit `true`."],
            ["`hidden`", "auto, bool", "`false`", "Calculate without showing the math; prose and problems stay."],
            ["`clear-at`", "auto, none, selector, function, label", "`none`", "Reset the innermost variable scope at matching headings or labels: `heading.where(level: 1)`. An inner one replaces the outer; `none` turns resets off."],
          ],
        },
      },
      { h2: "Numbers" },
      {
        table: {
          head: ["option", "type", "default", "meaning"],
          rows: [
            ["`digits`", "auto, int", "`4`", "Significant digits of computed values, 1…15: `57.5 h`, `123500`."],
            ["`trailing-zeros`", "auto, bool", "`false`", "Keep trailing zeros up to `digits`: `2.500 V`."],
            ["`decimal-separator`", "auto, str", "`auto`", "Any string, like `\",\"`. `auto` follows the document language: `,` for de, fr, it, … and `.` otherwise. Input always uses a point."],
            ["`grouping`", "auto, none, str, symbol, content", "`none`", "Digit grouping for integer parts of 5+ digits: `sym.space.thin` → 12 345."],
            ["`notation-thresholds`", "auto, none, array", "`(-3, 6)`", "`(low, high)`: plain while the exponent `e` is in `low ≤ e < high`, scientific beyond. `none` never switches."],
            ["`notation`", "auto, str", "`\"auto\"`", "`\"auto\"`, `\"plain\"`, `\"scientific\"` or `\"engineering\"` (exponents in steps of 3)."],
            ["`multiplication`", "auto, str", "`\"dot\"`", "`\"dot\"` (2 · 10⁶) or `\"times\"` (2 × 10⁶)."],
            ["`number-form`", "auto, str", "`\"auto\"`", "`\"auto\"`: typed values as typed, results rounded · `\"decimal\"`: everything rounded · `\"fraction\"`: exact fractions (⅙ A) · `\"input\"`: given values exactly as typed."],
            ["`max-denominator`", "auto, int", "`1000`", "Largest denominator for `number-form: \"fraction\"`; larger ones fall back to decimals."],
            ["`fraction-style`", "auto, str", "`\"stacked\"`", "`\"stacked\"` or `\"slash\"` (1/2)."],
          ],
        },
      },
      { h2: "Units" },
      {
        table: {
          head: ["option", "type", "default", "meaning"],
          rows: [
            ["`prefixes`", "auto, str, none, array", "`\"all\"`", "Prefixes picked for results: `\"all\"` (a f p n µ m k M G T; times from a minute on in min or h, not ks), `none` (base units only) or an array like `(\"m\", \"k\")` (`\"u\"` is µ)."],
            ["`preferred-units`", "auto, content, array", "`()`", "Units for results and graph axes of their dimension, before input units and automatic prefixes: `($#m / #s$, $kWh$)`."],
            ["`show-si`", "auto, bool", "`false`", "`defined` also shows the SI value: Q₀ = 3450 mAh = 12420 C."],
          ],
        },
      },
      { h2: "Output" },
      {
        table: {
          head: ["option", "type", "default", "meaning"],
          rows: [
            ["`show-approximation`", "auto, bool", "`true`", "Documented to write `≈` for rounded results. In 2.0.0 the result marker stays ⇒ either way."],
            ["`block`", "auto, bool", "`true`", "Display equations centred (`true`) or inline."],
            ["`max-solutions`", "auto, int", "`6`", "How many solutions are listed; all are found."],
            ["`defined-style`", "auto, str", "`\"grid\"`", "How `defined` looks: `\"grid\"` (columns aligned at =), `\"column\"`, `\"inline\"` or `\"list\"`."],
            ["`show-types`", "auto, str", "`\"chip\"`", "Types like `f(I: mA)` in the equations as written: `\"chip\"` (a blue chip), `\"plain\"` (as written) or `\"none\"` (left out: f(I))."],
          ],
        },
      },
      { h2: "Problems" },
      {
        table: {
          head: ["option", "type", "default", "meaning"],
          rows: [
            ["`errors`", "auto, str", "`\"document\"`", "`\"document\"`: problems are red notes, the document still compiles · `\"panic\"`: stop the compilation (for CI)."],
          ],
        },
      },
      { h2: "How things look" },
      {
        table: {
          head: ["option", "type", "default", "meaning"],
          rows: [
            ["`words`", "auto, dictionary", "`(any: \"any\")`", "The word shown for any-unit types, for other languages: `(any: \"beliebig\")`. You still type `any`."],
            ["`chips`", "auto, dictionary, function", "`auto`", "Colours of type chips, `(type: (fill: …, mark: …, ink: …))`, with `dark: (…)` for dark HTML pages – or a function `(kind, mark, unit) => content`."],
            ["`unit-style`", "auto, function", "`auto`", "A function given every shown unit: `u => text(fill: blue, u)`."],
            ["`unit-separator`", "auto, content, str, symbol", "`auto`", "What goes between units, a thin dot by default (A·s): `sym.times`, `h(0.2em)`."],
            ["`constant-displays`", "auto, dictionary", "`(:)`", "How constants are shown, by identity: `(imaginary: $j$, \"standard-gravity\": $g_0$)`, over the built-in π, e, i and c. Nested settings add to it."],
          ],
        },
      },
      { h2: "Graphs" },
      { md: "Per curve, repeating. `auto` in an array keeps the surrounding setting for that curve: `(auto, red)` changes only the second (and fourth, …). A graph's own `colors`, `lines`, `dots` and `joined` go over these the same way." },
      {
        table: {
          head: ["option", "type", "default", "meaning"],
          rows: [
            ["`graph-colors`", "auto, color, array", "Petroff's ten colours", "Curve colours, chosen to stay apart for colour-blind readers: `(gray, blue, rgb(\"#f98eff\"))`."],
            ["`graph-lines`", "auto, str, array, dictionary, none", "`(\"solid\", \"dashed\", \"dotted\", \"dash-dotted\")`", "Typst dash patterns or lengths like `(4pt, 2pt)`; `none` draws no line."],
            ["`graph-dots`", "auto, str, array, none", "`(\"circle\", \"square\", \"triangle\", \"diamond\")`", "Also `\"triangle-down\"`, `\"pentagon\"`, `\"hexagon\"`, `\"star\"`, `\"cross\"`, `\"plus\"`; `none` draws no dots."],
            ["`graph-joined`", "auto, bool, array", "`(false,)`", "Whether everything a curve draws is joined with a line."],
          ],
        },
      },
      {
        example: {
          title: "Options at three levels",
          code: `#import "@preview/solveng:3.0.0": *
#import units: kΩ, nF
#import constants: pi
#show: math.with(digits: 3, decimal-separator: ",")

$R := 4.7 kΩ$, $C := 22 nF$
$tau := R C =>$
#math(digits: 6)[$f := 1/(2 pi tau) =>$]
#defined(style: "inline", digits: 2)`,
          caption: "The document says 3 digits, the block 6, the `defined` list 2 – and all three keep the decimal comma from the document.",
        },
      },
      { callout: "note", title: "A typo is a problem, not a silence", md: "An unknown option – `digit: 3` – is reported with the list of valid ones, and a value of the wrong type or outside its choices names what's allowed." },
    ],
    related: ["formatting", "scopes", "functions/math", "functions/defined", "functions/graph", "errors"],
  },

  {
    slug: "reference/units",
    title: "Units",
    group: "reference",
    summary: "Every unit binding of solveng's `units` module, how prefixes are spelled, and how to write them in math.",
    keywords: ["units", "SI", "prefix", "milli", "micro", "kilo", "ohm", "Omega", "degree", "celsius", "percent", "import units"],
    blocks: [
      { md: "Units are Typst bindings in solveng's `units` module: import the ones you use, `#import units: mA, kΩ, s`. Search the whole list:" },
      { widget: "unit-table" },
      { h2: "What's in it" },
      {
        table: {
          head: ["kind", "units"],
          rows: [
            ["SI base units", "`m` `kg` `s` `A` `K` `mol` `cd`"],
            ["Derived SI", "`V` `Ω` `W` `J` `C` `F` `H` `S` `Hz` `N` `Pa` `T` `Wb`"],
            ["Time", "`min` `h` (and `s`)"],
            ["Charge and energy", "`Ah` `As` `Wh` `Ws` `eV`"],
            ["Volume, mass", "`l` `L` (litre), `g`"],
            ["Angles", "`rad`, `deg` (°)"],
            ["Temperature", "`K`, `degC` (°C, a *difference*)"],
            ["Ratios", "`percent` (%), `permille` (‰)"],
          ],
        },
      },
      { h2: "Prefixes" },
      { md: "Every unit above except `kg`, `percent` and `permille` comes with the SI prefixes as ready-made bindings: `kΩ`, `mA`, `µs`, `GHz`, `mAh`, `kWh`, `mm`, `mg`." },
      {
        table: {
          head: ["prefix", "factor", "example"],
          rows: [
            ["`a` atto", "10⁻¹⁸", "`aJ`"],
            ["`f` femto", "10⁻¹⁵", "`fF`"],
            ["`p` pico", "10⁻¹²", "`pF`"],
            ["`n` nano", "10⁻⁹", "`nF`, `ns`"],
            ["`µ` `μ` `u` micro", "10⁻⁶", "`µA`, `uA`"],
            ["`m` milli", "10⁻³", "`mA`, `mm`"],
            ["`c` centi", "10⁻²", "`cm`"],
            ["`d` deci", "10⁻¹", "`dm`, `dl`"],
            ["`k` kilo", "10³", "`kΩ`, `km`"],
            ["`M` mega", "10⁶", "`MHz`"],
            ["`G` giga", "10⁹", "`GW`"],
            ["`T` tera", "10¹²", "`TWh`"],
          ],
        },
      },
      { h2: "Typeable spellings" },
      { md: "- `Omega` is Ω; prefixed ohms are spelled with Ω: `kΩ`, `MΩ`.\n- Micro has three spellings: `uA`, `µA` (micro sign) and `μA` (Greek mu) are the same binding.\n- `deg` is °, `degC` is °C.\n\nResults always show the proper symbols." },
      {
        example: {
          title: "Spellings and the # rule",
          code: `#import "@preview/solveng:3.0.0": *
#import units: kΩ, uA, Omega, V, deg, percent
#show: math

$R := 2.2 kΩ$, $I := 3 uA$, $R_s := 50 Omega$
$U := (R + R_s) I => #V$
$alpha := 30 deg$, $sin(alpha) =>$
$eta := 85 percent$, $eta dot 2 =>$`,
          caption: "One-letter units need `#` in math (`#V`); longer ones don't (`kΩ`, `uA`).",
        },
      },
      { h2: "Writing units in math" },
      { md: "- **One letter needs `#`**: `$2 #s$`, `$100 #Ω$`, `$#m^2$`. Without it, `s` is a variable – Typst only looks up bindings for names of two or more letters.\n- **Products** with spaces (`kg #m`), **quotients** with `/` (`#m / #s^2`), **powers** whole numbers (`#s^(-1)`).\n- **Names with `_` or `-`** need `#`: `$1 #fluid_ounce$`, else Typst reads a subscript or a minus.\n- **After `=>` or `->`** comes a unit expression: `$v => km/#h$`, `$P => mW, digits(2)$`.\n- **Dividing by a unit** gives a plain number: `U / #V` is U's number in volts. `$F(d: mm) := 3.2 (d / mm)^0.6 #N$` writes an empirical formula." },
      { callout: "warning", title: "Clashes worth knowing", md: "`ms` is the millisecond, not metre times second – write `#m #s`. `Pa` is the pascal, `pA` the picoampere. After `#import units: h`, Typst's spacing function is `std.h(1em)`. And `A` alone is a variable: the ampere is `#A`." },
      {
        quiz: {
          question: "After `#import units: mA, A`, what is `$I := 2 A$`?",
          options: ["2 times a variable `A`", "2 amperes", "An error"],
          answer: 0,
          explain: "A single letter in math is a letter. The binding is `#A`: `$I := 2 #A$`. Multi-letter units like `mA` don't need the `#`.",
        },
      },
      { h2: "Temperature" },
      { md: "`degC` is a temperature **difference**: 5 °C is 5 K. For absolute readings, `celsius(20)` and `fahrenheit(68)` from `operations` give kelvin – that's what `p V = n R T` needs." },
      { h2: "Your own units" },
      { md: "`base-unit(\"information\", display: \"bit\")` creates a new dimension, `unit($100 kPa$, display: \"bar\")` a unit of existing ones. See [Custom units](/docs/custom-units), [base-unit](/docs/functions/base-unit) and [unit](/docs/functions/unit)." },
    ],
    related: ["units", "custom-units", "functions/unit", "functions/base-unit", "operations/celsius", "reference/typst-math"],
  },

  {
    slug: "reference/cheatsheet",
    title: "Cheat sheet",
    group: "reference",
    summary: "solveng on one page: operators, imports, writing values and names, modifiers, common operations and options.",
    keywords: ["cheat sheet", "quick reference", "summary", "syntax", "overview", "card"],
    blocks: [
      {
        example: {
          title: "Everything at once",
          code: `#import "@preview/solveng:3.0.0": *
#import units: mA, mAh, h, V, Ω, W
#import operations: at
#import modifiers: solve, digits
#show: math.with(digits: 3)

$Q_0 := 3450 mAh$, $I := 30 mA$
$t := Q_0 / I => #h$
$R := 50 #Ω$, $P(U: #V) := U^2 / R$
$P(5 #V) => #W, digits(2)$
$Q_0 / 2 = I t_2 -> solve(t_2)$
$"r" := x^2 = 9 -> solve(x)$, $at("r", 1) =>$`,
        },
      },
      { h2: "Operators" },
      {
        table: {
          head: ["write", "means", "example"],
          rows: [
            ["`:=`", "define a value or function", "`$R := 4.7 kΩ$`, `$f(x) := x^2$`"],
            ["trailing `=>`", "show the value, numerically", "`$I =>$`, `$t => #h, digits(2)$`"],
            ["trailing `->`", "exact symbolic result, or solve", "`$1/3 + 1/6 ->$`, `$x^2 = 4 -> solve(x)$`"],
            ["`=` inside", "compare: 1 or 0", "`$M := 2 #A = 2000 mA$`"],
            ["other `=`", "a statement, checked when everything has a value", "`$U = R I$`"],
            ["`<` `<=` `>` `>=` `!=`", "compare; chains hold when every step does", "`$0 < x <= 1$`"],
            ["`and` `or`", "join conditions, `and` binds tighter", "`$x < 0 or x > 2$`"],
          ],
        },
      },
      { h2: "Imports" },
      {
        table: {
          head: ["import", "brings"],
          rows: [
            ["`#import \"@preview/solveng:3.0.0\": *`", "`math`, `mathscope`, `graph`, `defined`, `value`, `clear`, `compile`, `base-unit`, `unit`, `constant`, `operation`, `any`, and the modules"],
            ["`#import units: mA, kΩ, s`", "units – see [Units](/docs/reference/units)"],
            ["`#import constants: pi, e, i, c`", "π, Euler's e, the imaginary unit, the speed of light"],
            ["`#import operations: tri, integrate`", "operations beyond Typst's own `sqrt sin cos ln exp abs …`"],
            ["`#import modifiers: solve, digits`", "what goes after `=>` and `->`; `solve as löse` works too"],
            ["`#show: math`", "turn calculations on (nothing happens before it)"],
          ],
        },
      },
      { h2: "Writing" },
      {
        table: {
          head: ["what", "how"],
          rows: [
            ["one-letter binding", "`#`: `$2 #s$`, `$#e^x$`, `$3 + 4 #i$`"],
            ["number", "decimal point always, no e-notation: `$1.12 dot 10^4$`"],
            ["multi-letter name", "quote it: `$\"duty\" := 0.4$`; or subscript `t_\"on\"`"],
            ["join name pieces", "`Delta\"\"t` → Δt, `t\"\"2` → t2"],
            ["typed parameter", "`$P(I: mA) := …$`, `any` for every unit"],
            ["named parameter", "`$f(x, n: 2) := x^n$`, then `f(3, n: 3)`"],
            ["parameters from the body", "`$P(..) := U^2/R$` is `P(U, R)`"],
            ["vector, matrix", "`vec(1, 2, 3)`, `mat(1, 2; 3, 4)`"],
            ["element", "`x[0]`, `A[i, j]`, `A[i, :]` – counted from 0"],
            ["range", "`$i := 0..N$`, `range(0, 1, step: 0.1)`"],
            ["piecewise", "`cases(0 & x < 0, x & \"otherwise\")`"],
            ["sum", "`sum_(k=0)^10 k^2`"],
            ["number from a value", "`U / #V`, `number(U, mV)`"],
          ],
        },
      },
      { h2: "Modifiers" },
      {
        table: {
          head: ["modifier", "after", "does"],
          rows: [
            ["`solve(x)`, `solve(a, b)`", "`->`", "solutions, defining nothing"],
            ["`substitute(U)`", "`->`", "put formulas or constants' expressions in"],
            ["`assume(x > 0)`", "`->`", "local assumptions for the symbolic result"],
            ["`simplify` `expand` `factor`", "`->`", "the shape of the symbolic result"],
            ["`steps`", "`->`", "the way there, a line each"],
            ["`digits(n)`", "`=>`", "n significant digits"],
            ["`sci` `eng`", "`=>`", "scientific, engineering notation"],
            ["`fraction`", "`=>`", "an exact fraction"],
          ],
        },
      },
      { h2: "Functions" },
      {
        table: {
          head: ["call", "does"],
          rows: [
            ["`#math(digits: 8)[…]`", "settings for a block, no new scope"],
            ["`#mathscope(scope: \"inherit\")[…]`", "a variable scope: `isolated`, `inherit`, `shared`"],
            ["`#graph($P(U: #V) = U^2/R$)`", "a plot, ranges and units picked for you"],
            ["`#defined()`", "list the definitions so far"],
            ["`#value($U => mV$)`", "a result in running text"],
            ["`#clear(\"I\")`, `#clear(all: true)`", "forget definitions"],
            ["`compile($U^2/R$, parameters: (U: $#V$))`", "a numeric Typst function"],
          ],
        },
      },
      { h2: "Common operations" },
      {
        table: {
          head: ["area", "operations"],
          rows: [
            ["built into Typst math", "`sqrt exp ln log sin cos tan sinh cosh tanh abs arg`"],
            ["elementary", "`asin acos atan atan2 min max floor ceil sgn factorial number celsius`"],
            ["complex", "`Re Im conj`"],
            ["signals", "`rect tri sinc dirac`"],
            ["calculus", "`diff integrate taylor prod`, `laplace invlaplace convolve`"],
            ["linear algebra", "`at rank identity rows cols linear-solve eigenvals cond`"],
            ["statistics", "`mean median variance stdev pnorm qnorm linear-fit dft`"],
            ["numerical", "`minimize odesolve polyroots`"],
          ],
        },
      },
      { h2: "Options you'll use most" },
      {
        table: {
          head: ["option", "example"],
          rows: [
            ["`digits`", "`math.with(digits: 3)`"],
            ["`prefixes`", "`prefixes: (\"m\", \"k\")`, `prefixes: none`"],
            ["`preferred-units`", "`preferred-units: ($kWh$,)`"],
            ["`notation`", "`notation: \"engineering\"`"],
            ["`number-form`", "`number-form: \"fraction\"`"],
            ["`constant-displays`", "`constant-displays: (imaginary: $j$)`"],
            ["`clear-at`", "`clear-at: heading.where(level: 1)`"],
            ["`evaluate`, `hidden`", "`#math(evaluate: false)[…]`, `#math(hidden: true)[…]`"],
            ["`errors`", "`errors: \"panic\"` for CI"],
          ],
        },
      },
      { md: "All of them: [Options](/docs/reference/options)." },
    ],
    related: ["operators", "reference/options", "reference/units", "reference/typst-math"],
  },

  {
    slug: "reference/typst-math",
    title: "Typst math primer",
    group: "reference",
    summary: "The parts of Typst's math syntax solveng reads – and the ones that surprise people coming from Mathcad or LaTeX.",
    keywords: ["typst", "math syntax", "hash", "quotes", "subscript", "fraction", "vec", "mat", "cases", "line break", "identifier", "unknown variable"],
    blocks: [
      { md: "solveng doesn't have its own syntax: it reads the math Typst already parsed. Most of it is what you'd guess. This page is about the rest – the handful of rules that explain nearly every surprise." },
      { h2: "Letters, words and #" },
      { md: "In Typst math, **a single letter is just a letter**: `x`, `R`, `s`. A **word of two or more letters is looked up** as a Typst binding: `pi`, `sin`, `alpha`, `mA` (once imported) – and an unknown word is a Typst error, *unknown variable*.\n\nSo:\n\n- A one-letter **binding** (a unit, a constant) needs `#` to be looked up: `#s`, `#V`, `#e`, `#i`. Without it, `s` is a variable named s.\n- A variable name of several letters needs **quotes**: `\"duty\"`, `\"roots\"`. Quoted text is always a worksheet variable, even when it spells a unit.\n- Names with `_` or `-` need `#` too: `#fluid_ounce`." },
      {
        example: {
          title: "Three kinds of names",
          code: `#import "@preview/solveng:3.0.0": *
#import units: V, mA
#show: math

$U := 5 #V$, $I := 20 mA$
$"duty" := 0.25$
$P := "duty" U I =>$`,
          caption: "`#V` is the volt, `mA` needs no `#` (two letters), `\"duty\"` is a variable.",
        },
      },
      {
        quiz: {
          question: "Which of these defines a variable called *xy*?",
          options: ["`$\"xy\" := 3$`", "`$xy := 3$`", "`$x y := 3$`"],
          answer: 0,
          explain: "`xy` without quotes is a Typst lookup – *unknown variable: xy*. `x y` is x times y, which can't be defined. Quotes make a name.",
        },
      },
      { h2: "Subscripts, primes and joined names" },
      { md: "- `R_1`, `t_\"on\"`, `R_(a b)` are names with subscripts; `R_(a b)` is R with subscript ab.\n- `x'` is a name of its own.\n- **Digits:** `x1` is a Typst word, so unknown – write `x_1`, or `x\"\"1` for x1 on the baseline.\n- **`\"\"` joins** pieces Typst would otherwise read separately: `Delta\"\"t` is Δt – without it, `Deltat` isn't a symbol and `Delta t` is Δ times t.\n- Greek letters by name: `alpha`, `tau`, `Omega` (careful: after `#import units: Omega`, that's the ohm)." },
      {
        example: {
          code: `#import "@preview/solveng:3.0.0": *
#import units: ms
#show: math

$t_"on" := 2 ms$, $Delta""t := 3 ms$
$D := t_"on" / (t_"on" + Delta""t) =>$
$x' := 3$, $x_1 := 2$, $x' + x_1 =>$`,
        },
      },
      { h2: "Fractions and spaces" },
      { md: "`/` makes a fraction of the **atoms** next to it: a name, a number, a call, or something in parentheses. Spaces multiply, but they don't group. So `a/2b` and `a/2 b` are both (a/2)·b – and only `a/(2 b)` divides by 2b. `frac(a, b)` is the explicit form; parentheses directly around a fraction's part disappear in the output, as in Typst." },
      {
        example: {
          title: "Where does the fraction end?",
          code: `#import "@preview/solveng:3.0.0": *
#show: math

$a := 6$, $b := 3$
$a/2b =>$
$a/(2 b) =>$
$frac(a, 2 b) =>$`,
          caption: "9, 1, 1. The typeset output shows it too: the b sits next to the fraction, not under it.",
        },
      },
      {
        quiz: {
          question: "What does `$1/2 pi f C$` mean?",
          options: ["(1/2) · π · f · C", "1 / (2 π f C)", "An error"],
          answer: 0,
          explain: "The fraction takes only the 2. For the capacitor's reactance write `1/(2 pi f C)`.",
        },
      },
      { h2: "Multiplication" },
      { md: "Implicit multiplication needs a space between letters (`2 pi f C`, `x y`); a number directly before a letter works (`2x`). `dot`, `*` and `times` multiply explicitly. `dot` shows a centred dot – the usual choice." },
      { h2: "Calls" },
      { md: "`f(x)`, `sin(x)`, `integrate(x^2, x, 0, 1)`: a name directly followed by parentheses. Arguments are separated by commas – which is why numbers always use a decimal **point**: `at(A, 0,1)` passes three arguments. Named arguments look like Typst's: `f(3, n: 2)`, `range(0, 1, step: 0.1)`. Typst's own `sin`, `ln`, `abs` also work without parentheses for a single atom (`sin a`), but parentheses are clearer." },
      { h2: "vec, mat and cases" },
      { md: "- `vec(1, 2, 3)` is a column vector.\n- `mat(1, 2; 3, 4)` is a matrix: commas separate columns, semicolons rows.\n- `cases(…)` is a piecewise definition: value `&` condition, one case per comma. A condition `\"otherwise\"` catches the rest." },
      {
        example: {
          code: `#import "@preview/solveng:3.0.0": *
#show: math

$v := vec(1, 2, 3)$, $v[2] =>$
$M := mat(1, 2; 3, 4)$
$f(x) := cases(0 & x < 0, x & "otherwise")$
$f(-2) =>$, $f(3) =>$`,
        },
      },
      { h2: "Line breaks: \\" },
      { md: "`\\` breaks a line inside one equation – solveng reads each line as its own statement, so a block of definitions can be one equation. In `#graph`, `\\` separates curves: `#graph($sin(x) \\ cos(x)$)`." },
      {
        example: {
          code: `#import "@preview/solveng:3.0.0": *
#show: math

$a := 2 \\
 b := a + 1 =>$`,
        },
      },
      { h2: "Symbols Typst makes for you" },
      {
        table: {
          head: ["you type", "Typst shows", "solveng reads"],
          rows: [
            ["`:=`", "≔", "define"],
            ["`=>`", "⇒", "show the value"],
            ["`->`", "→", "symbolic result"],
            ["`<=` `>=` `!=`", "≤ ≥ ≠", "compare"],
            ["`and` `or`", "∧ ∨", "join conditions"],
            ["`dot` `times` `*`", "· × ∗", "multiply"],
            ["`infinity`", "∞", "infinity, in sums and integrals"],
            ["`\"…\"`", "upright text", "a name"],
          ],
        },
      },
      { h2: "Surprises, collected" },
      { md: "- `1.12e4` is 1.12 times a variable `e4` – write `1.12 dot 10^4`.\n- `A` is a variable, `#A` the ampere; `ms` is the millisecond, not metre·second.\n- `$...$` with spaces inside (`$ x := 2 $`) is a display equation in Typst, centred on its own line.\n- `a[b + c]` is an element of `a`; write `a dot [b + c]` for a product.\n- Importing `math` hides Typst's math module: native functions are `std.math.…`.\n- After `#import units: h`, Typst's spacing function is `std.h(…)`." },
    ],
    related: ["operators", "reference/units", "reference/cheatsheet", "values", "functions"],
  },
];
