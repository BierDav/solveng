# solveng reference

<!-- Generated from the doc comments in src/ by `python3 tests/reference.py` – edit those, not this file. -->

Everything solveng does, for looking things up. For an introduction see the [README](../README.md).

- [`worksheet`](#worksheet)
- [`unit`](#unit)
- [`solve`](#solve)
- [`simplify`](#simplify)
- [`digits`](#digits)
- [`sci`](#sci)
- [`eng`](#eng)
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
| [`worksheet`](#worksheet) | Calculates the math you write, like a Mathcad worksheet. |
| [`unit`](#unit) | A unit in math, for units without a name of their own: `$p := 3 unit("bar")$`, `$5 unit("V")$`. |
| [`solve`](#solve) | Solves an equation for these names (and defines them): `$x^2 - 4 = 0 -> solve(x)$`. |
| [`simplify`](#simplify) | The symbolic result, what `->` shows anyway: `$f := (x^2 - 1)/(x - 1) -> simplify$`. |
| [`digits`](#digits) | Decimals with n significant digits: `$t = h, digits(2)$`. |
| [`sci`](#sci) | Scientific notation: `$C = sci$` → 2.2 · 10⁻⁶ F. |
| [`eng`](#eng) | Engineering notation, exponents in steps of 3: `$f = eng$`. |
| [`defined`](#defined) | Lists the definitions made so far, in order. |
| [`graph`](#graph) | Plots an equation with `simple-plot`: ranges, nice ticks, axis labels and units are picked automatically. |
| [`lambdify`](#lambdify) | Turns an expression into a plain Typst function of numbers, for value tables, other plotting packages (lilaq, cetz-plot, simple-plot) or your own calculations. |
| [`reset`](#reset) | Forgets every definition (values and functions) made so far. |
| [`worksheet-config`](#options) | Sets options for number formatting, units and output. |

## `worksheet`

Calculates the math you write, like a Mathcad worksheet.

`#show: worksheet` calculates the rest of the document, `#worksheet[…]` its content (whose definitions are dropped at its end), and `#worksheet($…$)` single equations, also outside a worksheet.

```typ
#show: worksheet
The battery holds $Q_0 := 3450 mAh$ and the load draws $I := 30 mA$,
so it lasts $t := Q_0 / I = h$.
```

**Math.** Three operators, as in Mathcad:

- `:=` defines (shown as ≔): `$I := 30 mA$`, `$P := U I$`, `$f(x) := x^2 + 1$`. Steps after it are checked: `$R := 7.2 kΩ = 7200 Ω$`.
- `=` at the end shows the value, with a bold =: `$t =$` → t **=** 115 h, `$t := Q_0/I =$` defines and shows it. What follows it is the unit and keywords: `$t = h$`, `$t = min, digits(2)$`.
- `->` at the end shows the symbolic result, the known values put in, exactly: `$f := (x^2 - 1)/(x - 1) -> simplify$`. With `solve(x)` it solves an equation for x (and defines it): `$Q_0/2 = I t -> solve(t)$`, `$x^2 - 4 = 0 -> solve(x)$`, `$x + y = 3, x - y = 1 -> solve(x, y)$`, `$R <= U/I -> solve(R)$`.

Keywords: `solve(x, …)`, `simplify`, `frac` (exact fractions), `digits(n)` (decimals with n significant digits), `sci` (scientific notation), `eng` (engineering notation). Any other equation with `=` states something: it's checked when everything in it is known (`$U = R I$` after defining them) and left alone as prose when not (`$E = m c^2$`).

**Units** are names: `$30 mA$`, `$4.7 kΩ$`, `$9.81 m/s^2$`, and `unit("bar")` for units without one (your own, from `worksheet-config(units: …)`). A single letter is a unit right after a number (`$5 V$`, `$3 h$`) or alone after a trailing `=` (`$t = h$`); anywhere else it's a variable, and `$3 dot h$` is 3 times a variable h.

**Code.** Each line of a ```` ```solveng ```` code block is calculated the same way and shown with all its steps: `t := Q0/I = [h]`, `Q0/I =`, `x^2 = 4 -> solve(x)`; units in brackets, `;` separates the equations of a system, `//` starts a comment. Inline code and code with another language are left alone.

**Calls.** `#worksheet($U := 5 V$, $I := U/R =$)` calculates its equations (and strings, read like a code line) where it stands, `hidden: true` without showing them. Its options are for these only; with content, for the content.

**Editors.** Each equation and code block the worksheet sees leaves a `metadata` labelled `<solveng>` with its `location()`, whether it was calculated and the `integration` there; at the end (and before each `reset`) one lists everything defined, with values, and the units. Editors like Typewright read them after compiling: `query(<solveng>)`.

**Starting over.** `reset` starts over at every match; `#reset` inside a worksheet does the same once: back to how it started (with `inherit: true` the outer definitions stay). After a `#worksheet[…]` the outer definitions come back.

```typ
#show: worksheet.with(reset: heading)                    // every heading
#show: worksheet.with(reset: heading.where(level: 1))    // only top-level ones
#show: worksheet.with(reset: heading, inherit: true)     // keep the definitions made before
```

| parameter | type | default | |
|---|---|---|---|
| `..body` | content, str |  | The content, or equations (math or code lines) to calculate. |
| `inherit` | bool | `false` | See the definitions made before the worksheet (also after each `reset`). |
| `reset` | none, function, selector | `none` | Start over at every match, e.g. `heading` (every heading) or `heading.where(level: 1)`: the definitions made since are forgotten, like `#reset`. |
| `hidden` | bool | `false` | For equations: calculate them without showing them (only problems are shown). |
| `digits`, `separator`, `form`, … | auto | `auto` | Every option of `worksheet-config` (except `units`, `integration` and `implicit-parameters`), for this worksheet or these equations only. See [Options](#options). |

Returns content.

## `unit`

A unit in math, for units without a name of their own: `$p := 3 unit("bar")$`, `$5 unit("V")$`.

| parameter | type | default | |
|---|---|---|---|
| `name` | str |  | The unit as solveng writes it, `"mAh"`, `"kΩ"`, `"m/s^2"`. |

Returns content.

## `solve`

Solves an equation for these names (and defines them): `$x^2 - 4 = 0 -> solve(x)$`.

| parameter | type | default | |
|---|---|---|---|
| `..names` | content |  | What to solve for. |

Returns content.

## `simplify`

The symbolic result, what `->` shows anyway: `$f := (x^2 - 1)/(x - 1) -> simplify$`.

## `digits`

Decimals with n significant digits: `$t = h, digits(2)$`.

| parameter | type | default | |
|---|---|---|---|
| `n` | int, content |  | The digits, 1…15. |

Returns content.

## `sci`

Scientific notation: `$C = sci$` → 2.2 · 10⁻⁶ F.

## `eng`

Engineering notation, exponents in steps of 3: `$f = eng$`.

## `defined`

Lists the definitions made so far, in order.

```typ
#show: worksheet
$R := 7.2 kΩ$, $U := 5 V$, $I := U/R$, $f(x) := x^2$
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
| `digits`, `separator`, `form`, … | auto | `auto` | Every option of `worksheet-config` (except `units`) works here too, for this call only. See [Options](#options). |
| `digits`, `separator`, … | auto | `auto` | Any [option](#options) for this call; `auto` keeps the current setting. |

Returns content.

## `graph`

Plots an equation with `simple-plot`: ranges, nice ticks, axis labels and units are picked automatically.

Every positional string or math is a curve: an expression (`"cos(w)"`), or a function of
what it's plotted against, named in the legend and on the axis: `"U(x: A) = x * [2 Ω]"`, with
the parameter's unit for the x axis, or `"P(..) = U^2/R"`, where `..` takes the one variable
that isn't defined (with its unit from the range or the marks). Like every function, a curve
may use only its parameters and defined variables. All curves share x. Where two cross is marked:
`#graph("cos(w)", "w")` shows the solution of cos(w) = w. `xrange:` / `yrange:` list values that must be visible;
without them the x range is picked around the marks, else around the interesting part
(roots, extrema, poles, intersections). Marked values are always visible too.

```typ
#worksheet("R := 50 Ω", "P0 := 2 W", "U0 := 10 V", hidden: true)
#graph("P(U: V) = U^2/R", ymark: "P0")                    // where P reaches P0
#graph("P(..) = U0^2/R_L", xrange: ("10 Ω", "1 kΩ"))       // R_L, in Ω from the range
#graph("cos(w)", "w")                                      // range picked, crossing marked
#graph("y(x) = x^2", "y(x) = 2x + 3", xrange: (-3, 5))     // several curves
#graph("P(R_L: Ω) = U0^2/R_L", xrange: (0, "2 R"), mark: ("R", "extrema"))
```

Uses `@preview/simple-plot:1.1.0`. Ticks use the document's decimal separator, axis labels read `U / V`; curves of the same unit share one graph, with a legend.

**Ranges:** values and expressions in `xrange` / `yrange` must have the axis unit (`$f(20) Omega$` and `"f(20) Ω"` are fine) – `xrange: ("10 Ω", "f(20)")` with a unitless `f(20)` is an error, since it isn't clear what was meant. Without a range: around the marks; else around what's interesting – roots, extrema, poles, edges of the domain (√, ln) and, above all, where curves cross – rounded to nice ticks, starting at 0 when everything interesting is positive. With `ymark`, the y range is around the marked values (steep curves leave the picture). **Units** follow from what's plotted: y's from the curves (shown in the SI unit with a fitting prefix; `preferred-units` picks others), x's from the curve's parameter (`U(I: mA) = …`) or a function x is passed to whose parameter has one (`$q(I: mA) := I R$`, then `#graph("U(I) = q(I)")` is over I in mA), else from the units in `xrange` and the marks, else from the curve and the units of the y values (`P(..) = U0^2/R2` with `ymark: "2 W"`: R₂ in Ω).

**Marks** are always visible – the ranges grow to show them. Combine them in an array, `(auto, "R1")`; `mark: ()` marks nothing.

**Size:** `width` / `height` are the whole graph, labels included; `auto` makes the plot area 8 × 5 cm. Any other named argument goes straight to simple-plot's `plot` (e.g. `show-grid: false`), other positional ones (series from simple-plot) are added as they are. For full control, plot with simple-plot directly.

| parameter | type | default | |
|---|---|---|---|
| `..args` | str, content, any |  | The curves (equations or expressions) and options passed on to simple-plot's `plot` (e.g. `show-grid: false`); other positional ones are added as series. |
| `xrange` | auto, array, str, content, int, float | `auto` | x values that must be visible: numbers in the x axis's unit (`(1, 2)`), values (`("10 Ω", "1 kΩ")`) or expressions of defined variables (`(0, "2 R")`, math works too). Two or more give the range from the smallest to the largest; a single one widens the picked range. `auto`: around the marked x values, else picked from the curves. |
| `yrange` | auto, array, str, content, int, float | `auto` | y values that must be visible, like `xrange` (numbers in the y axis's SI unit). Two or more give the exact y range. `auto`: fitted to the curves. |
| `mark` | auto, none, array, str, content, int, float | `auto` | What to mark, with both coordinates written on the axes: x values like `xrange` (`"R"`, `2`, `"5 V"`; a point with guide lines on every curve, the range grows to show it), `"crossings"`, `"roots"`, `"extrema"` (searched within the range) and `auto`: where two curves cross. An array combines them: `(auto, "R1")`. `()` or `none`: no marks. |
| `xmark` | none, array, str, content, int, float | `()` | Like `mark`, but only the x value is written on the axis. |
| `ymark` | none, array, str, content, int, float | `()` | y values to mark, like `yrange`: every point where a curve reaches one, with guide lines; the value is written on the y axis. |
| `samples` | int | `400` | Number of sample points. |
| `width` | auto, length, ratio, relative | `auto` | Width of the whole graph, labels included: `12cm`, `300pt` or a share like `100%`. `auto`: a plot area of 8 cm. |
| `height` | auto, length, ratio, relative | `auto` | Like `width`; `auto`: a plot area of 5 cm. A share is of the container's height, or of the width when the page grows with its content. |
| `colors` | array, color | `(blue, red.darken(10%), green.darken(25%), orange.darken(10%))` | Colors of the curves, e.g. `(gray, blue, rgb("#f98eff"))`. |
| `digits`, `separator`, `form`, … | auto | `auto` | Every option of `worksheet-config` (except `units`) works here too, for this call only. See [Options](#options). |

Returns content.

## `lambdify`

Turns an expression into a plain Typst function of numbers, for value tables,
other plotting packages (lilaq, cetz-plot, simple-plot) or your own calculations.
Defined variables and functions are used with their current values. Needs `context`.

```typ
#worksheet($R := 50 Ω$, hidden: true)
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

Forgets every definition (values and functions) made so far. Inside a `worksheet`:
back to how the block started (with `inherit: true` the outer definitions stay).

```typ
#show: worksheet
$I := 30 mA$
#reset
$I := 1 A$
```

## Options

`worksheet-config` – Sets options for number formatting, units and output.

Without a body the options apply from here on; with a body only inside it. Every option defaults to `auto`, which keeps the current setting, so only what you pass changes. Most options can also be passed directly to `worksheet`, `defined` and `graph`, for that worksheet or call only. Unknown options and values of the wrong type are shown as an error in the document.

```typ
#show: worksheet-config.with(digits: 3, form: "fraction")   // your defaults: at the top of the document
#worksheet-config(layout: "column")                          // from here on, definitions are kept
#worksheet-config(steps: "result")[ … ]                      // only inside the brackets
#show: worksheet
$U := 5 V$, $I := 30 mA$
#worksheet($R := U/I =$, digits: 2)                     // only these equations
#let opts = (digits: 2, separator: ",")
#worksheet($P := U I =$, ..opts)                          // options kept in a dictionary
```

`form`:

- `"auto"`: plain numbers as typed (`7.20 kΩ` keeps its zero), arithmetic evaluated (`3450/2 mAh` → 1725 mAh), results rounded to `digits`.
- `"decimal"`: everything rounded to `digits`, including what you typed.
- `"fraction"`: exact fractions, simplified (`$frac(2, 4) A$` → ½ A, `I/3` → ⅙ A); values that are no fraction with denominator ≤ `max-denominator` fall back to decimals.
- `"input"`: given values exactly as typed (`3450/2 mAh` stays a fraction).

An explicit unit (`$I = mA$`) always wins, then `preferred-units`, then the non-SI unit of the inputs (h, mAh, bar), then the SI unit with a prefix.

To reuse your defaults in many documents, keep a small file of your own:

```typ
// my-defaults.typ
#import "@preview/solveng:0.2.0": *
#let setup(body) = worksheet-config(digits: 3, multiplication: "times", layout: "column", worksheet(body))

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
| `units` | auto, dictionary | `(:)` | Your own units, defined by a value in known units: `(bar: "100 kPa", rpm: "1/min", dozen: 12)`; `auto` makes a base unit of its own, a dimension no other unit has (`(ü: auto)`, then `mü`, `ü/s`); `none` removes one, `(T: none)`. Adds to the units set before. Only here, not per call. |
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

### What worksheet calculates besides code tagged solveng

| option | type | default | |
|---|---|---|---|
| `integration` | auto, str | `"math"` | Whether a `worksheet` calculates the math in it: `"math"` (default) or `"none"`, then only ```` ```solveng ```` code blocks and `#worksheet(…)` calls are calculated. Only here, not per call. |

### Casts (u as a) and types (f(x

| option | type | default | |
|---|---|---|---|
| `show-casts` | auto, str | `"chip"` | How casts (`U as A`) are shown in the steps and results and in the equations as written: `"chip"` (default): as an amber chip, since a cast isn't maths · `"plain"`: as written, U as A · `"no"`: left out (`$I = U "as" A$` shows as I = U). |
| `show-types` | auto, str | `"chip"` | How types (`f(I: mA)`) are shown in the steps and in the equations as written: `"chip"` (default): as a blue chip · `"plain"`: as written, f(I: mA) · `"no"`: left out (`$f(I: "mA") = 2 I$` shows as f(I) = 2 I). |

### How they look

| option | type | default | |
|---|---|---|---|

### Function drawing them, and units

| option | type | default | |
|---|---|---|---|
| `words` | auto, dictionary | `(cast: "as", any: "any")` | The words shown for casts and any-unit types, for documents in other languages: `(cast: "als", any: "beliebig")`. What you type stays `as` and `any`. |
| `chips` | auto, dictionary, function | `auto` | The colours of the chips per kind, `(cast: (fill: …, mark: …, ink: …), type: (…))` (`mark` is the colour of `as` and `:`, `ink` the unit's), or a function `(kind, mark, unit) => content` drawing them, with `kind` `"cast"` or `"type"` and `mark` and `unit` as content. |
| `unit-style` | auto, function | `auto` | A function given every shown unit as content, `u => text(fill: blue, u)`. |
| `unit-separator` | auto, content, str, symbol | `auto` | What goes between units, a thin dot by default (A·as): `sym.times`, `h(0.2em)`. |

### Functions

| option | type | default | |
|---|---|---|---|
| `implicit-parameters` | auto, str | `"no"` | What a function takes of what its body leaves undefined, besides `f(x, ..)`, which always takes it: `"no"` (default): nothing, `f()` takes nothing and `f(x) = x + k` with an undefined `k` is an error · `"empty"`: empty parentheses take it all, `f() = x^2 + k` is `f(x, k)` · `"allow"`: every function takes it after its listed parameters, `f(x) = x + k` is `f(x, k)`. Only here, not per call. |

## Writing values

````typ
#show: worksheet
$R := 4.7 kΩ$                        // math: a number and a unit
$I := frac(2, 4) A$, $Q_0 := 3450/2 mAh$, $U := (1+2) dot 1,5 V$, $A := 2^3 m^2$  // arithmetic in the number
```solveng
R2 := 2 R; Z := [5 V] / I            // code: units in brackets inside an equation
U2 := R [1 mA]
Q := @U2 [2 s]                       // @U2 inlines its formula: Q = R [1 mA] [2 s]
```
````

- **Units:** SI base units (m kg s A K mol cd) and V A Ω W J C F H S Hz N Pa T Wb, min h Ah Wh As Ws eV l %, ‰, °, rad, °C (as a temperature *difference*), all SI prefixes; your own with the option `units`. Powers as `m^2`, `m²`, `s^-1`; quotients as `m/s^2`; products with spaces `kg m`.
- **Typeable spellings:** `Omega` → Ω, with a prefix glued as in math: `k""Omega` → kΩ (`kOmega` is not a unit, like `DeltaT` is not ΔT); `u` → µ, `degC` → °C, `deg` → °. They are printed with the proper symbols (`[20 uA]` → 20 µA). In math, units are names (`$4.7 kΩ$`, `$20 µA$`), see [Units and variables](#units-and-variables).
- **Decimal comma:** `7,2 V` works; the output separator follows the document language (option `separator`).
- **Names:** `Q0`→Q₀, `R_min`→R_min, `rho`→ρ, `tol`→tol (upright). In math equations `Q_0` is a different variable than `Q0`. Typst reads a bare word in math as its own variable, so names of more than one letter are quoted there: `$"tews" := 23$`, `$"x"_"in"$` (x_in); subscripts may be grouped, `$R_(a b)$` is R_ab. A Greek letter spelled out alone or before an index is the letter, as in math: `Delta_R` is `$Delta_R$` (Δ_R), `alpha2` is α2; `DeltaT` and `Deltas` stay names of their own, set as written; `Delta""T` is ΔT. Empty strings glue names as math needs them to: `$Delta""r_R$` is Δr_R, which code writes `Δr_R` or `Delta""r_R` (in a string `"Delta\\"\\"r_R"`).
- **Operators:** `+ - * / ^ · ×`, implicit multiplication (`2 pi f C`), `sqrt exp ln log sin cos tan asin acos atan abs`, constants `pi e` (unless you define a variable with that name), your own functions.

## Units and variables

Many engineering symbols are also units: `A` (area, ampere), `V` (volume, volt), `T` (period, tesla), `h` (height, hour), `C`, `F`, `W`, `m`, `s`, `l`, `L`, … solveng keeps them apart by *how* they are written, never by guessing:

- **Math:** units are names, `$30 mA$`, `$4.7 kΩ$`, `$9.81 m/s^2$` (solveng brings every unit of several letters and every prefixed one, set upright); `unit("bar")` writes any other, also your own. A single letter is a unit right after a number (`$5 V$`, `$3 h$`, whatever is defined) or alone after a trailing `=` (`$t = h$`); anywhere else it's a variable, so `$3 dot h$` is 3 times h. Quoted text is a name of several letters (`$"tews" := 2$`), as Typst needs; `"as"` casts.
- **Code:** a unit is read only where a value is written – right after a number (`I := 30 mA`) or in brackets (`[5 V]`, `[A]` for one ampere). Every other name in an equation is a variable. A value whose unit is also a defined variable (`t := 3 h` with `h` defined, `f := 1/T` with `T` defined) is refused, with a hint how to write either meaning: `[3 h]` for hours, `3*h` or `1/(T)` for the variable.

````typ
#show: worksheet
$A := 12$, $x := A dot 2 A =$        // x ≔ A · 2 A = 24 A
$T := 2 s$, $f := 1/T = Hz$          // f ≔ 1/T = 0.5 Hz
```solveng
g := 1/T                             // error: 'T' in '1/T' is both a defined variable and a unit – …
```
````

For documents where nothing may be read two ways, `#worksheet-config(strict: true)` also refuses unit names as variable names (`A := 3 m^2` → pick `A_1`, or remove the unit).

The available units can be changed with the option `units`: new units are defined by a value, `none` removes one (which frees its name, e.g. `T` for a period that is then never read as tesla):

```typ
#worksheet-config(units: (bar: "100 kPa", rpm: "1/min", t: "1000 kg", T: none))
#show: worksheet
$p := 3 unit("bar")$, $n := 3000 unit("rpm")$          // prefixes work too: unit("mbar")
#worksheet-config(preferred-units: ("bar", "kWh"))  // results with these dimensions are shown in these units
```

## Errors

Problems (an unknown unit, a missing value, units that don't match, a typo) are shown as a red note where they happen; the rest of the document still compiles:

> ⚠ solveng: 'U' in 'P := U I' isn't defined – define it first with :=, or solve for it: … -> solve(U)

`#show: worksheet-config.with(errors: "panic")` stops the compilation instead (useful in CI). Unknown named arguments of `worksheet`, `worksheet-config`, `defined` and `graph` are reported the same way; those of `lambdify` stop the compilation.

## HTML

With `typst compile --features html --format html`, equations become MathML, `defined` columns that reflow with the width (or a table with `style: "column"`), graphs inline SVGs (via `html.frame`) that shrink with the page, errors styled notes. One small stylesheet is added to the page (math font STIX Two Math / Cambria Math / Latin Modern Math, spacing, table stripes; classes `solveng`, `solveng-defs`, `solveng-list`, `solveng-plot`, `solveng-error`). HTML export is experimental in Typst; in column layout browsers drop the space after a leading `=`.

## How it solves

If the unknown occurs once it is isolated symbolically (with substitution shown). Equations linear in it are solved symbolically too. Polynomials → all real roots (any degree). Anything else → numeric search over ±10⁻¹⁵…10¹⁵ (times the target unit): sign changes are bisected and Newton starts from every local minimum of the absolute value. Systems: equations with a single unknown are solved symbolically first, the rest by damped Newton from 33 starts. Units of unknowns are inferred from the equations; mismatched units stop with an error.

## Limits

Found by the fuzzer (`tests/fuzz.py`: random equations, quantities, options and hostile inputs checked against Python):

- **Numbers** are 64-bit floats: about 15 significant digits, magnitudes up to 10³⁰⁰. Results that underflow show as 0 (`ln(x) = -800`).
- **Isolating** `sqrt(x) = 2` gives only the positive root, periodic functions only the principal value (`tan(x) = 1` → `atan(1)`); polynomials like `x^2 = 4` give all their roots.
- **Numeric search** can miss roots squeezed between nearby poles, roots of very fast oscillations, and roots outside ±10¹⁵ (scaled by the target unit). In random nonlinear equations this hit about 0.1 %; the result then says "no real solution".
- **Systems** are solved numerically from a fixed set of starts; solutions far away from them can be missed. Underdetermined systems (`x + y = 2`, `2x + 2y = 4`) show one arbitrary solution.
- **Nesting:** up to about 20 levels of brackets (Typst's function call depth); sums and products of any length are fine.
- **Domains:** only real numbers; `(-8)^(1/3)` and `sqrt(-1)` are errors, as is an unknown that cancels out (`x - x`).
- **°C** is a temperature difference (no 273.15 offset); temperatures in formulas like `p V = n R T` need K. New base dimensions (bit, €) can't be added.
- **Prefixes:** a lone `m` before a unit is metre: `"m s"`, `$"m" Omega$` are m·s and Ω·m; write `"ms"`, `$"m"Omega$` for milli.
- **Functions** are expanded where they are used (no recursion).
- **Math in a worksheet:** an equation with `=` and only known names is checked, so a formula meant as plain text that happens to use defined names (`$a^2 + b^2 = c^2$` after defining a, b and c) must hold. Put those outside the worksheet or use `worksheet-config(integration: "none")`.

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
