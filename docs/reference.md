# solveng reference

<!-- Generated from the doc comments in src/ by `python3 tests/reference.py` – edit those, not this file. -->

Everything solveng does, for looking things up. For an introduction see the [README](../README.md).

- [`def`](#def)
- [`solve`](#solve)
- [`solve-block`](#solve-block)
- [`defined`](#defined)
- [`graph`](#graph)
- [`lambdify`](#lambdify)
- [`reset`](#reset)
- [Options](#options)
- [Writing values](#writing-values)
- [Units and variables](#units-and-variables)
- [Errors](#errors)
- [HTML](#html)
- [How it solves](#how-it-solves)
- [Limits](#limits)
- [Testing](#testing)

## Functions

| function | does |
|---|---|
| [`def`](#def) | Defines values, functions or unknowns from equations, silently. |
| [`solve`](#solve) | Solves equations and shows the way: equation ⇒ rearranged formula = numbers = result. |
| [`solve-block`](#solve-block) | A scope in which math and code are calculations. |
| [`defined`](#defined) | Lists the definitions made so far, in order. |
| [`graph`](#graph) | Plots an equation with `simple-plot`: ranges, nice ticks, axis labels and units are picked automatically. |
| [`lambdify`](#lambdify) | Turns an expression into a plain Typst function of numbers, for value tables, other plotting packages (lilaq, cetz-plot, simple-plot) or your own calculations. |
| [`reset`](#reset) | Forgets every definition (values and functions) made so far. |
| [`solve-config`](#options) | Sets options for number formatting, units and output. |

## `def`

Defines values, functions or unknowns from equations, silently.

Every positional argument is an equation, as a string or as math. Several are
defined in order; equations that can't be defined one by one are solved together
as a system. `def` shows nothing (except problems) unless `output: true`; `solve`
is the same with the output on.

```typ
#def("R = 7.2 kOhm", "I = 415 uA")     // values: a number with a unit
#def($Q_0 = 3450 "mAh"$)                // math works too
#def("U = R I")                          // computed from defined values
#def("f(x) = x^2 + 1")                   // a function, usable as f(3)
#def("Q_0/2 = I t", unit: "h")           // an equation: t is solved for
#def("x + y = 3", "x - y = 1")           // a system
```

Inside an equation `[5 V]` is a value and `@R` inlines the formula of `R`.

| parameter | type | default | |
|---|---|---|---|
| `..eqs` | str, content |  | The equations. A single one without `=` (`"Q0/I"`) is evaluated (only useful with `output: true`). |
| `unit` | none, str, content, dictionary | `none` | Unit of the result: `"mA"` (or math `$"mA"$`). For a system a dictionary per unknown: `(I: "mA", U: "V")`. Without it: the SI unit with an automatic prefix, or the non-SI unit of the inputs (h, mAh). If the result has another unit (`"x = f(1)"` gives volts, `unit: "A"`), that is an error. |
| `output` | bool | `false` | Show the definition (like `solve` does). |
| `steps` | auto, str | `auto` | What is shown: `"full"`: equation ⇒ formula = numbers = result · `"formula"`: without the numbers · `"result"`: `t = 57.5 h` · `"value"`: just `57.5 h`. `auto`: the option `steps` (default `"full"`), or `inline-steps` with `inline: true`. |
| `layout` | auto, str | `auto` | `"row"`: all steps in one line · `"column"`: one step per line, aligned at `=`. `auto`: the option `layout` (default `"row"`). |
| `inline` | bool | `false` | In the running text instead of a display block; `steps` default to `inline-steps` (`"result"`). |
| `unknown` | none, str, array | `none` | What to solve for, `"t"` or `("x", "y")`. Only needed when it isn't clear; by default all variables that aren't defined yet are the unknowns (one per equation). If everything is defined, a lone variable on either side is recomputed. |
| `root` | auto, int, function | `auto` | Which solution is stored when there are several: `2` (1-based), or a test like `x => x > 0` (for systems it gets a dictionary). `auto`: the only all-positive one, else the first. |
| `guess` | none, str, content, int, float, dictionary | `none` | Newton start value instead of the search: `"-1 A"`, or `(U: "5 V")` for systems. |
| `listed` | bool | `true` | `false` hides it from `defined`. |
| `digits`, `separator`, … | auto | `auto` | Any [option](#options) for this call; `auto` keeps the current setting. |

Returns content.

## `solve`

Solves equations and shows the way: equation ⇒ rearranged formula = numbers = result.

Solves for all unknowns, i.e. all variables that aren't defined yet (one unknown per
equation), and stores the results like `def`. The unknown can be on either side,
also inside a function call. If the unknown occurs once it is isolated symbolically;
polynomials give all real roots; anything else is solved numerically.

```typ
#def("Q0 = 3450 mAh", "I = 30 mA")
#solve("Q0/2 = I t", unit: "h")          // Q₀/2 = I t ⇒ t = Q₀/(2I) = … = 57.5 h
#solve("I", unit: "A")                    // no '=': convert, I = 30 mA = 0.03 A
#solve("x^2 + 2x - 3 = 0")                // all real roots
#def("P1 = 2 W", "R1 = 50 Ω")
#solve("P1 = U1 I1", "U1 = R1 I1")        // a system
Runtime: #solve("Q0/I", unit: "h", inline: true, steps: "value")   // 115 h
#solve("Q0/4 = I t2", unit: "h", layout: "column", steps: "formula")
```

| parameter | type | default | |
|---|---|---|---|
| `..eqs` | str, content |  | The equations (or a single expression without `=` to evaluate). |
| `unit` | none, str, content, dictionary | `none` | Unit of the result: `"mA"` (or math `$"mA"$`). For a system a dictionary per unknown: `(I: "mA", U: "V")`. Without it: the SI unit with an automatic prefix, or the non-SI unit of the inputs (h, mAh). If the result has another unit (`"x = f(1)"` gives volts, `unit: "A"`), that is an error. |
| `output` | bool | `true` | `false` defines silently, like `def`. |
| `steps` | auto, str | `auto` | What is shown: `"full"`: equation ⇒ formula = numbers = result · `"formula"`: without the numbers · `"result"`: `t = 57.5 h` · `"value"`: just `57.5 h`. `auto`: the option `steps` (default `"full"`), or `inline-steps` with `inline: true`. |
| `layout` | auto, str | `auto` | `"row"`: all steps in one line · `"column"`: one step per line, aligned at `=`. `auto`: the option `layout` (default `"row"`). |
| `inline` | bool | `false` | In the running text instead of a display block; `steps` default to `inline-steps` (`"result"`). |
| `unknown` | none, str, array | `none` | What to solve for, `"t"` or `("x", "y")`. Only needed when it isn't clear; by default all variables that aren't defined yet are the unknowns (one per equation). If everything is defined, a lone variable on either side is recomputed. |
| `root` | auto, int, function | `auto` | Which solution is stored when there are several: `2` (1-based), or a test like `x => x > 0` (for systems it gets a dictionary). `auto`: the only all-positive one, else the first. |
| `guess` | none, str, content, int, float, dictionary | `none` | Newton start value instead of the search: `"-1 A"`, or `(U: "5 V")` for systems. |
| `listed` | bool | `true` | `false` hides it from `defined`. |
| `digits`, `separator`, … | auto | `auto` | Any [option](#options) for this call; `auto` keeps the current setting. |

Returns content.

## `solve-block`

A scope in which math and code are calculations.

`#show: solve-block` calculates the whole rest of the document; `#solve-block[…]` only its body, whose definitions are dropped at its end. How results look is set with `solve-config`, not here.

```typ
#show: solve-block
The load draws $I = "30 mA"$ (#solve("I", unit: "A", steps: "value", inline: true)),
the battery holds $Q_0 = 3450 "mAh"$, so it lasts $t = Q_0 / I$, ```solveng = t -> h```.
```

**Math** (with the default `integration: "math"`). Every math equation with a `=` is a definition, exactly like `def`: `$I = "30 mA"$`, `$P = U I$`, `$Q_0/2 = I t$`. The equation stays as you wrote it; problems are shown next to it.

- An inequality with exactly one unknown (`$R <= U/I$`) defines it like `#solve("R <= U/I")`; `$0 < x < 1$` and inequalities between known values stay prose.
- Equations without `=` (`$x^2$`), derivations with `⇒`, `≈`, … (`$ Q = I t => t = Q/I $`), equations with alignment or line breaks (`$ U &= R I \ &= 6 V $`) or with embedded calls are left alone.
- In a worked chain `$R = 7.2 "k"Omega = 7200 Omega$` the first `=` defines, the rest are steps.
- A step that writes an undefined variable out defines it: `$R = rho dot frac(l, A) = rho dot frac(l, b dot h)$` defines `A = b h`; with `R` given instead, `b` is solved from the most written-out step (`R = ρ l/(b h)`) and `A` follows.
- A number followed by units is a value (`frac(3450 "mAh", 30 "mA")`, `9.81 "m"/"s"^2`), `0,95` is a decimal number. Quoted units (`"h"`) are always units; a bare letter that is a defined variable is that variable (`$3 h$` with a variable `h` is 3 times h).

**Code.** Code tagged `solveng` is calculated:

- A code block ```` ```solveng ````: every line is a definition that shows itself, exactly like `#solve(line)`.
- Inline code ```` ```solveng t = Q0/I -> h``` ```` defines and shows inline with `inline-steps` (default `"result"`: t = 115 h).
- ```` ```solveng = Q0/I -> h``` ````: just the value (115 h), nothing is defined.
- Per line: `-> unit` for the result unit, `;` separates the equations of a system (`x + y = 3; x - y = 1`), `//` starts a comment.

What else is calculated is set with `solve-config(integration: …)`: `"math"` (default) reads math as above and leaves code without a language alone, so code samples stay code samples; `"code"` calculates code without a language instead of math, `"full"` both, `"none"` neither. Code with another language (```` ```py ````) is always left alone. The output of `def`, `solve`, `defined` and `graph` is never read as a definition again.

**Editors.** Each equation and code element the block sees leaves a `metadata` labelled `<solveng>` with its `location()`, whether it was calculated and the `integration` there; at the end of the block (and before each `reset`) one lists everything defined, with values, and the units. Editors like Typewright read them after compiling: `query(<solveng>)`.

**Starting over.** `reset` starts over at every match; `#reset` inside a solve-block does the same once: back to how the block started (with `inherit: true` the outer definitions stay). After the block the outer definitions come back.

```typ
#show: solve-block.with(reset: heading)                    // every heading
#show: solve-block.with(reset: heading.where(level: 1))    // only top-level ones
#show: solve-block.with(reset: heading, inherit: true)     // keep the definitions made before the block
```

| parameter | type | default | |
|---|---|---|---|
| `..body` | content |  | The content. |
| `inherit` | bool | `false` | See the definitions made before the block (also after each `reset`). |
| `reset` | none, function, selector | `none` | Start over at every match, e.g. `heading` (every heading) or `heading.where(level: 1)`: the definitions made since are forgotten, like `#reset`. |

Returns content.

## `defined`

Lists the definitions made so far, in order.

```typ
#def("R = 7.2 kOhm", "U = 5 V", "I = U/R", "f(x) = x^2")
#defined()                                          // everything, in columns side by side
#defined(included: "given")                         // only plain values (also "computed", "function")
#defined(excluded: ("U", "function"))               // names and kinds
#defined(included: regex("^R"), style: "column")    // a regex, or a function name => bool
#defined(show-si: true, digits: 3)                  // any option, for this list only
```

Styles (`style:` here, option `defined-style` for all of them):

- `"grid"` (default): as many columns side by side as fit, filled top to bottom, each aligned at `=`. In HTML the columns reflow with the window width.
- `"column"`: one column aligned at `=` (in HTML a table).
- `"inline"`: comma separated in the text.
- `"list"`: bullets.

`included` / `excluded` take `"all"`, a kind (`"given"`: plain values, `"computed"`, `"function"`), a name, a regex, a function `name => bool` or an array of these; `excluded` wins. (`include` is a Typst keyword, hence the names.)

| parameter | type | default | |
|---|---|---|---|
| `included` | str, regex, function, array | `"all"` | What to list. Default `"all"`. |
| `excluded` | none, str, regex, function, array | `none` | What to leave out. |
| `style` | auto, str | `auto` | `"grid"`, `"column"`, `"inline"` or `"list"`, see above. `auto`: the option `defined-style` (default `"grid"`). |
| `digits`, `separator`, `form`, … | auto | `auto` | Every option of `solve-config` (except `units`) works here too, for this call only. See [Options](#options). |
| `digits`, `separator`, … | auto | `auto` | Any [option](#options) for this call; `auto` keeps the current setting. |

Returns content.

## `graph`

Plots an equation with `simple-plot`: ranges, nice ticks, axis labels and units are picked automatically.

Every positional string or math is a curve: `y = …` gives one curve (named `y` in the
legend) and marks the stored point with guide lines; other equations draw both sides
and mark the intersections. `xrange:` / `yrange:` list values that must be visible;
without them the x range is picked around the marks, else around the interesting part
(roots, extrema, poles, intersections). Marked values are always visible too.

```typ
#def("R = 50 Ω", "P0 = 2 W")
#solve("P0 = U^2/R")
#graph("P = U^2/R", x: "U")                                // 0…2·U, the solved point marked
#graph("P = U^2/R", x: "R", xrange: ("10 Ω", "1 kΩ"))
#graph("cos(w) = w")                                       // range picked, intersection marked
#graph("y = x^2", "y = 2x + 3", xrange: (-3, 5))           // several curves
#graph("P = U^2/R", x: "R", xrange: (0, "2 R"), mark: ("R", "extrema"))
#graph("P = U^2/R", x: "R", ymark: "P0")                  // where P reaches P0
```

Uses `@preview/simple-plot:1.1.0`. Ticks use the document's decimal separator, axis labels read `U / V`; curves of the same unit share one graph, with a legend.

**Ranges:** values and expressions in `xrange` / `yrange` must have the axis unit (`$f(20) Omega$` and `"f(20) Ω"` are fine) – `xrange: ("10 Ω", "f(20)")` with a unitless `f(20)` is an error, since it isn't clear what was meant. Without a range: if x is defined, 0…2·x with the solved point marked; else around the marks; else around what's interesting – roots, extrema, poles, edges of the domain (√, ln) and, above all, where curves cross – rounded to nice ticks, starting at 0 when everything interesting is positive. With `ymark`, the y range is around the marked values (steep curves leave the picture). If x has no unit yet, it follows from the curve and the units of the y values (`P = U0^2/R2` with `ymark: "2 W"`: R₂ in Ω).

**Marks** are always visible – the ranges grow to show them. Combine them in an array, `(auto, "R1")`; `mark: ()` marks nothing.

**Size:** `width` / `height` are the whole graph, labels included; `auto` makes the plot area 8 × 5 cm. Any other named argument goes straight to simple-plot's `plot` (e.g. `show-grid: false`), other positional ones (series from simple-plot) are added as they are. For full control, plot with simple-plot directly.

| parameter | type | default | |
|---|---|---|---|
| `..args` | str, content, any |  | The curves (equations or expressions) and options passed on to simple-plot's `plot` (e.g. `show-grid: false`); other positional ones are added as series. |
| `x` | auto, str | `auto` | The x variable. `auto`: the only variable that isn't defined. |
| `xrange` | auto, array, str, content, int, float | `auto` | x values that must be visible: numbers in `xunit` (`(1, 2)`), values (`("10 Ω", "1 kΩ")`) or expressions of defined variables (`(0, "2 R")`, math works too). Two or more give the range from the smallest to the largest; a single one widens the picked range. `auto`: 0…2·x if x is defined, else around the marked x values, else picked from the curves. |
| `yrange` | auto, array, str, content, int, float | `auto` | y values that must be visible, like `xrange` (numbers in `yunit`). Two or more give the exact y range. `auto`: fitted to the curves. |
| `xunit` | auto, str, content | `auto` | Unit of the x axis. `auto`: from the range or the variable. |
| `yunit` | auto, str, content | `auto` | Unit of the y axis. `auto`: SI unit with a fitting prefix. |
| `mark` | auto, none, array, str, content, int, float | `auto` | What to mark, with both coordinates written on the axes: x values like `xrange` (`"R"`, `2`, `"5 V"`; a point with guide lines on every curve, the range grows to show it), `"crossings"`, `"roots"`, `"extrema"` (searched within the range) and `auto`: the solved point or where the two sides cross. An array combines them: `(auto, "R1")`. `()` or `none`: no marks. |
| `xmark` | none, array, str, content, int, float | `()` | Like `mark`, but only the x value is written on the axis. |
| `ymark` | none, array, str, content, int, float | `()` | y values to mark, like `yrange`: every point where a curve reaches one, with guide lines; the value is written on the y axis. |
| `samples` | int | `400` | Number of sample points. |
| `width` | auto, length, ratio, relative | `auto` | Width of the whole graph, labels included: `12cm`, `300pt` or a share like `100%`. `auto`: a plot area of 8 cm. |
| `height` | auto, length, ratio, relative | `auto` | Like `width`; `auto`: a plot area of 5 cm. A share is of the container's height, or of the width when the page grows with its content. |
| `colors` | array, color | `(blue, red.darken(10%), green.darken(25%), orange.darken(10%))` | Colors of the curves, e.g. `(gray, blue, rgb("#f98eff"))`. |
| `digits`, `separator`, `form`, … | auto | `auto` | Every option of `solve-config` (except `units`) works here too, for this call only. See [Options](#options). |

Returns content.

## `lambdify`

Turns an expression into a plain Typst function of numbers, for value tables,
other plotting packages (lilaq, cetz-plot, simple-plot) or your own calculations.
Defined variables and functions are used with their current values. Needs `context`.

```typ
#def("R = 50 Ω")
#context {
  let P = lambdify("U^2/R", "U", unit: "W", units: (U: "V"))
  table(columns: 2, ..range(1, 6).map(u => ([#u V], [#P(u) W])).flatten())
}
```

The function takes one argument per parameter: a number (in the unit given in
`units`, else in the SI unit) or a value like `"5 V"`. It returns a float in
`unit` (else in the SI unit), or `none` where the expression isn't defined
(`sqrt(-1)`, division by zero).

Unlike the other functions, problems stop the compilation here, since the
result is a function and not content.

| parameter | type | default | |
|---|---|---|---|
| `expr` | str, content |  | The expression, using defined variables and the parameters: `"U^2/R"` or `$U^2 / R$`. |
| `..params` | str |  | The parameters in order, e.g. `"U"` or `"U", "R"`. A defined variable as a parameter is replaced by the argument. Without any: the undefined variables of the expression, in the order they appear. |
| `unit` | none, str, content | `none` | Unit of the result, e.g. `"W"`. Checked against the expression when all parameters have units. |
| `digits`, `separator`, … | auto | `auto` | Any [option](#options) for this call; `auto` keeps the current setting. |

Returns function.

## `reset`

Forgets every definition (values and functions) made so far. Inside a `solve-block`:
back to how the block started (with `inherit: true` the outer definitions stay).

```typ
#def("I = 30 mA")
#reset
#def("I = 1 A")
```

## Options

`solve-config` – Sets options for number formatting, units and output.

Without a body the options apply from here on; with a body only inside it. Every option defaults to `auto`, which keeps the current setting, so only what you pass changes. Every option except `units` can also be passed directly to `def`, `solve`, `defined` and `graph`, for that call only. Unknown options and values of the wrong type are shown as an error in the document.

```typ
#show: solve-config.with(digits: 3, form: "fraction")   // your defaults: at the top of the document
#solve-config(layout: "column")                          // from here on, definitions are kept
#solve-config(steps: "result")[ … ]                      // only inside the brackets
#def("U = 5 V", "I = 30 mA")
#solve("R = U/I", digits: 2)                             // only this call
#let opts = (digits: 2, separator: ",")
#solve("P = U I", ..opts)                                // options kept in a dictionary
```

`form`:

- `"auto"`: plain numbers as typed (`7.20 kΩ` keeps its zero), arithmetic evaluated (`3450/2 mAh` → 1725 mAh), results rounded to `digits`.
- `"decimal"`: everything rounded to `digits`, including what you typed.
- `"fraction"`: exact fractions, simplified (`$frac(2, 4)"A"$` → ½ A, `I/3` → ⅙ A); values that are no fraction with denominator ≤ `max-denominator` fall back to decimals.
- `"input"`: given values exactly as typed (`3450/2 mAh` stays a fraction).

An explicit unit (`#solve(eq, unit: "mA")`) always wins, then `preferred-units`, then the non-SI unit of the inputs (h, mAh, bar), then the SI unit with a prefix.

Typst only allows `#set` for its built-in elements, not for functions of packages, so `#set solve(steps: "result")` isn't possible (yet). Instead, `#let solve = solve.with(steps: "result")` changes your `solve` calls, `#show: solve-config.with(steps: "result")` everything, including code in a solve-block. To reuse your defaults in many documents, keep a small file of your own:

```typ
// my-defaults.typ
#import "@preview/solveng:0.2.0": *
#let setup = solve-config.with(digits: 3, multiplication: "times", layout: "column")
#let solve = solve.with(steps: "formula")

// in a document
#import "my-defaults.typ": *
#show: setup
```

### Numbers

| option | type | default | |
|---|---|---|---|
| `digits` | auto, int | `4` | Significant digits of computed values, 1…15. e.g. `57.5 h`, `123500`. |
| `trailing-zeros` | auto, bool | `false` | Keep trailing zeros up to `digits` (`2.500 V`). |
| `separator` | auto, str | `auto` | Decimal separator, any string like `","`. `auto` (default): from the document language, `,` for de, fr, it, … and `.` otherwise. |
| `grouping` | auto, none, str, symbol, content | `none` | Digit grouping for integer parts with 5+ digits, e.g. `sym.space.thin` → `12 345`. |
| `scientific` | auto, none, array | `(-3, 6)` | `(low, high)`: plain notation while the exponent `e` is in `low ≤ e < high`, else `2.5 · 10⁻⁷`. `none` never switches. |
| `engineering` | auto, bool | `false` | Engineering notation, exponents in steps of 3. |
| `multiplication` | auto, str | `"dot"` | `"dot"` (default, `2 · 10⁶`) or `"times"` (`2 × 10⁶`). |
| `form` | auto, str | `"auto"` | How values are written. `"auto"` (default): as typed, results rounded · `"decimal"`: everything rounded to `digits` · `"fraction"`: exact fractions (`⅙ A`) · `"input"`: given values exactly as typed. |
| `max-denominator` | auto, int | `1000` | Largest denominator for `form: "fraction"`. |
| `fractions` | auto, str | `"stacked"` | How fractions are written: `"stacked"` (default) or `"slash"` (`1/2`). |

### Units

| option | type | default | |
|---|---|---|---|
| `prefixes` | auto, str, none, array | `"all"` | Prefixes picked for results: `"all"` (default: a f p n µ m k M G T; times from a minute on in min or h, not ks), `none` (base unit only) or an array like `("m", "k")` (`"u"` = µ). |
| `show-si` | auto, bool | `false` | `defined` also shows the SI value (`Q₀ = 3450 mAh = 12420 C`). |
| `units` | auto, dictionary | `(:)` | Your own units, defined by a value in known units: `(bar: "100 kPa", rpm: "1/min")`; `none` removes one, `(T: none)`. Adds to the units set before. Only here, not per call. |
| `preferred-units` | auto, str, array | `()` | Units for results of their dimension, `("bar", "kWh")`, before the unit of the inputs and the SI unit. |
| `strict` | auto, bool | `false` | Unit names can't be variable names, and units in math must be quoted (`$5 "V"$`), see [Units and variables](#units-and-variables). |

### Output

| option | type | default | |
|---|---|---|---|
| `steps` | auto, str | `"full"` | Default for `steps` of `solve`: `"full"` (default), `"formula"`, `"result"` or `"value"`. |
| `inline-steps` | auto, str | `"result"` | Steps for `inline: true` and inline code: `"full"`, `"formula"`, `"result"` (default) or `"value"`. |
| `layout` | auto, str | `"row"` | Default for `layout` of `solve`: `"row"` (default) or `"column"`. |
| `implies` | auto, content, symbol, str | `sym.arrow.r.double` | What is written between steps, e.g. `sym.arrow.r.long` or `[and]`; ⇒ by default. |
| `approximate` | auto, bool | `true` | Write `≈` when the shown result is rounded. |
| `block` | auto, bool | `true` | Display equations (centred, default) or inline. |
| `max-solutions` | auto, int | `6` | How many solutions are listed; all are found, `root` picks among all. |

### Output of defined

| option | type | default | |
|---|---|---|---|
| `defined-style` | auto, str | `"grid"` | How `defined` looks: `"grid"` (default: columns side by side, aligned at `=`), `"column"`, `"inline"` or `"list"`. |

### Problems

| option | type | default | |
|---|---|---|---|
| `errors` | auto, str | `"document"` | `"document"` (default): problems are shown as red notes · `"panic"`: stop the compilation. |

### What solve-block calculates besides code tagged solveng

| option | type | default | |
|---|---|---|---|
| `integration` | auto, str | `"math"` | What a `solve-block` calculates besides code tagged `solveng`: `"math"` (default: math equations) · `"code"`: code without a language · `"full"`: both · `"none"`: only tagged code. Only here, not per call. |

## Writing values

```typ
#def("R = 4.7 kOhm")                // strings: name = number + unit ("I = 30mA" works without the space too)
#def($R_1 = 4.7 "k"Omega$)          // math content
#def($I = frac(2, 4) "A"$, "Q0 = 3450/2 mAh", "U = (1+2)*1,5 V", "A = 2^3 m^2")  // arithmetic in the number
#def("R2 = 2 R", "Z = [5 V] / I")   // [5 V] is a value inside an equation
#def("U2 = R [1 mA]")
#solve("Q = @U2 [2 s]")             // @U2 inlines its formula: Q = R [1 mA] [2 s]
```

- **Units:** SI base units (m kg s A K mol cd) and V A Ω W J C F H S Hz N Pa T Wb, min h Ah Wh As Ws eV l %, ‰, °, rad, °C (as a temperature *difference*), all SI prefixes; your own with the option `units`. Powers as `m^2`, `m²`, `s^-1`; quotients as `m/s^2`; products with spaces `kg m`.
- **Typeable spellings:** `Ohm`/`ohm`/`Omega` → Ω, `u` → µ, `degC` → °C, `deg` → °. They are printed with the proper symbols (`"20 uA"` → 20 µA).
- **Decimal comma:** `"7,2 V"` works; the output separator follows the document language (option `separator`).
- **Names:** `Q0`→Q₀, `R_min`→R_min, `rho`→ρ, `DeltaT`→ΔT, `tol`→tol (upright). In math equations `Q_0` is a different variable than `Q0`.
- **Operators:** `+ - * / ^ · ×`, implicit multiplication (`2 pi f C`), `sqrt exp ln log sin cos tan asin acos atan abs`, constants `pi e` (unless you define a variable with that name), your own functions.

## Units and variables

Many engineering symbols are also units: `A` (area, ampere), `V` (volume, volt), `T` (period, tesla), `h` (height, hour), `C`, `F`, `W`, `m`, `s`, `l`, `L`, … solveng keeps them apart by *how* they are written, never by guessing:

- **Strings:** a unit is read only where a value is written – right after a number (`"I = 30 mA"`) or in brackets (`[5 V]`, `[A]` for one ampere). Every other name in an equation is a variable.
- **Math:** quoted text is a unit (`$30 "mA"$`, `$3 "h"$` – set upright, as units should be); a bare letter is a variable if one with that name is defined (`$3 h$` with `h` defined is 3 times h, in italics), else a unit after a number (`$5 V$`).
- **Unclear is an error:** a value whose unit is also a defined variable (`"t = 3 h"` with `h` defined, `"f = 1/T"` with `T` defined) is refused, with a hint how to write either meaning: `[3 h]` for hours, `3*h` or `1/(T)` for the variable.

```typ
#def("A = 12")
#solve("x = A [A]")                  // x = A · A = 12 · A = 12 A
#solve("y = A A", unit: "A")         // error: the result has unit 1, not A – 'A' is read as a variable here; …
#def("T = 2 s")
#def("f = 1/T")                      // error: 'T' in '1/T' is both a defined variable and a unit – …
#solve("f = 1/(T)")                  // f = 1/T = 1/(2 s) = 500 mHz
```

For documents where nothing may be read two ways, `#solve-config(strict: true)` also refuses unit names as variable names (`A = 3 m^2` → pick `A_1`, or remove the unit) and bare units in math (`$5 V$` → `$5 "V"$`).

The available units can be changed with the option `units`: new units are defined by a value, `none` removes one (which frees its name, e.g. `T` for a period that is then never read as tesla):

```typ
#solve-config(units: (bar: "100 kPa", rpm: "1/min", t: "1000 kg", T: none))
#def("p = 3 bar", "n = 3000 rpm")              // prefixes work too: mbar, kbar
#solve-config(preferred-units: ("bar", "kWh"))  // results with these dimensions are shown in these units
```

## Errors

Problems (an unknown unit, a missing value, units that don't match, a typo) are shown as a red note where they happen; the rest of the document still compiles:

> ⚠ solveng: 1 equation but 2 unknowns (Q0, t) in 'Q0/2 = I t' – define the missing values first, or pass unknown: …

`#show: solve-config.with(errors: "panic")` stops the compilation instead (useful in CI). Unknown named arguments of `def`, `solve`, `defined`, `graph`, `solve-config` and `solve-block` are reported the same way; those of `lambdify` stop the compilation.

## HTML

With `typst compile --features html --format html`, equations become MathML, `defined` columns that reflow with the width (or a table with `style: "column"`), graphs inline SVGs (via `html.frame`) that shrink with the page, errors styled notes. One small stylesheet is added to the page (math font STIX Two Math / Cambria Math / Latin Modern Math, spacing, table stripes; classes `solveng`, `solveng-defs`, `solveng-list`, `solveng-plot`, `solveng-error`). HTML export is experimental in Typst; in column layout browsers drop the space after a leading `=`.

## How it solves

If the unknown occurs once it is isolated symbolically (with substitution shown). Equations linear in it are solved symbolically too. Polynomials → all real roots (any degree). Anything else → numeric search over ±10⁻¹⁵…10¹⁵ (times the target unit): sign changes are bisected and Newton starts from every local minimum of the absolute value. Systems: equations with a single unknown are solved symbolically first, the rest by damped Newton from 33 starts. Units of unknowns are inferred from the equations; mismatched units stop with an error.

## Limits

Found by the fuzzer (`tests/fuzz.py`: random equations, quantities, options and hostile inputs checked against Python):

- **Numbers** are 64-bit floats: about 15 significant digits, magnitudes up to 10³⁰⁰. Results that underflow show as 0 (`ln(x) = -800`).
- **Isolating** `x^2 = 4` or `sqrt` gives only the positive root, periodic functions only the principal value (`tan(x) = 1` → `atan(1)`). Pass `root:` or `guess:` to use the numeric search, which finds the others.
- **Numeric search** can miss roots squeezed between nearby poles, roots of very fast oscillations, and roots outside ±10¹⁵ (scaled by the target unit). In random nonlinear equations this hit about 0.1 %; the result then says "no real solution".
- **Systems** are solved numerically from a fixed set of starts; solutions far away from them can be missed. Underdetermined systems (`x + y = 2`, `2x + 2y = 4`) show one arbitrary solution.
- **Nesting:** up to about 20 levels of brackets (Typst's function call depth); sums and products of any length are fine.
- **Domains:** only real numbers; `(-8)^(1/3)` and `sqrt(-1)` are errors, as is an unknown that cancels out (`x - x`).
- **°C** is a temperature difference (no 273.15 offset); temperatures in formulas like `p V = n R T` need K. New base dimensions (bit, €) can't be added.
- **Prefixes:** a lone `m` before a unit is metre: `"m s"`, `$"m" Omega$` are m·s and Ω·m; write `"ms"`, `$"m"Omega$` for milli.
- **Functions** are expanded where they are used (no recursion); `f(x) = 9` with an existing `f` is an equation, not a new definition.
- **Math in solve-block:** every equation with `=` is evaluated, also ones meant as plain text (`$a^2 + b^2 = c^2$` with unknown a, b, c is an error). Put those outside the block or use `solve-config(integration: "none")`.

## Testing

The tests use [tytanic](https://github.com/typst-community/tytanic) (`cargo install tytanic`); the scripts need Python 3 and `typst` on the `PATH`:

```sh
tt run                                        # units, solving, lambdify and the README examples (reference images)
tt update readme                              # accept changed output of the README examples
python3 tests/examples.py                     # every example in README.md and docs/reference.md compiles on its own
python3 tests/reference.py                    # docs/reference.md from the doc comments in src/ (--check: is it up to date?)
python3 tests/fuzz.py                         # random inputs checked against Python, every category, 300 cases each
python3 tests/fuzz.py -n 2000 -c solve1 --seed 7 --save /tmp/fails
```

Fuzzer categories: `parse` (quantities and math content), `solve1` (random nonlinear equations), `poly`, `system`, `units` (physics formulas with random prefixes), `edge` (hostile inputs must be refused or solved right), `edge-doc` (the same inputs must never stop the compilation), `format`, `ratio`, `display` (random options must never crash). `--typst …` (fuzzer) and `TYPST=…` (examples) pick another compiler, e.g. the oldest supported version.
