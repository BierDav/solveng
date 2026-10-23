# solveng reference

<!-- Generated from the doc comments in src/ by `python3 tests/reference.py` – edit those, not this file. -->

Everything solveng does, for looking things up. For an introduction see the [README](../README.md).

- [`worksheet`](#worksheet)
- [`unit`](#unit)
- [`operation`](#operation)
- [`solve`](#solve)
- [`substitute`](#substitute)
- [`simplify`](#simplify)
- [`digits`](#digits)
- [`sci`](#sci)
- [`eng`](#eng)
- [`defined`](#defined)
- [`graph`](#graph)
- [`lambdify`](#lambdify)
- [`value`](#value)
- [`clear`](#clear)
- [`rect`](#rect)
- [`tri`](#tri)
- [`sgn`](#sgn)
- [`floor`](#floor)
- [`factorial`](#factorial)
- [`diff`](#diff)
- [`integrate`](#integrate)
- [`laplace`](#laplace)
- [`invlaplace`](#invlaplace)
- [`convolve`](#convolve)
- [`range`](#range)
- [`at`](#at)
- [`rank`](#rank)
- [`identity`](#identity)
- [`rows`](#rows)
- [`cols`](#cols)
- [`asin`](#asin)
- [`acos`](#acos)
- [`atan`](#atan)
- [Options](#options)
- [Built-in operations](#built-in-operations)
- [Calculating like Mathcad](#calculating-like-mathcad)
- [Writing values](#writing-values)
- [Units and variables](#units-and-variables)
- [How it solves](#how-it-solves)
- [Limits](#limits)
- [Errors](#errors)
- [HTML](#html)
- [Testing](#testing)

## Functions

| function | does |
|---|---|
| [`worksheet`](#worksheet) | Calculates the math you write, like a Mathcad worksheet. |
| [`unit`](#unit) | An explicit unit reference: `#let ing = unit("iwef")` makes a unit shortcut for `$1ing$`. |
| [`operation`](#operation) | An operation for math, by its name: `#let conv = operation("convolve")`, then `$conv(x, h, step: Delta""t)$`. |
| [`solve`](#solve) | Solves an equation for these names (and defines them): `$x^2 - 4 = 0 -> solve(x)$`. |
| [`substitute`](#substitute) | The result with these variables' formulas put in, every other name kept: `$P := U^2/R -> substitute(U)$`. |
| [`simplify`](#simplify) | The symbolic result, what `->` shows anyway: `$f := (x^2 - 1)/(x - 1) -> simplify$`. |
| [`digits`](#digits) | Decimals with n significant digits: `$t = 1 "h", digits(2)$`. |
| [`sci`](#sci) | Scientific notation: `$C = sci$` → 2.2 · 10⁻⁶ F. |
| [`eng`](#eng) | Engineering notation, exponents in steps of 3: `$f = eng$`. |
| [`defined`](#defined) | Lists the definitions made so far, in order. |
| [`graph`](#graph) | Plots an equation with `simple-plot`: ranges, nice ticks, axis labels and units are picked automatically. |
| [`lambdify`](#lambdify) | Turns an expression into a plain Typst function of numbers, for value tables, other plotting packages (lilaq, cetz-plot, simple-plot) or your own calculations. |
| [`value`](#value) | The result of an expression alone, where the text needs it: what `$U =$` shows after its `=`. |
| [`clear`](#clear) | Forgets definitions: the names given, or with `all: true` every one made so far. |
| [`rect`](#rect) | Rectangular pulse: `rect(x)` is 1 for \|x\| < ½, ½ at x = ±½, and 0 otherwise. |
| [`tri`](#tri) | Triangular pulse: `tri(x)` is max(1 − \|x\|, 0), with height 1 at 0 and support from −1 to 1. |
| [`sgn`](#sgn) | Sign function: `sgn(x)` is −1 for x < 0, 0 for x = 0, and 1 for x > 0. |
| [`floor`](#floor) | Greatest integer less than or equal to x: `floor(x)`. |
| [`factorial`](#factorial) | Factorial of a nonnegative integer: `factorial(k)`. |
| [`diff`](#diff) | Symbolically differentiates an expression with respect to a name: `diff(expression, variable)`. |
| [`integrate`](#integrate) | Definite integral: `integrate(expression, variable, lower, upper)`. |
| [`laplace`](#laplace) | Unilateral Laplace transform: `laplace(expression, variable, result-variable)`. |
| [`invlaplace`](#invlaplace) | Inverse unilateral Laplace transform: `invlaplace(expression, variable, result-variable)`. |
| [`convolve`](#convolve) | Convolution: `convolve(x, h)` of two sequences, `convolve(f(τ), g(τ), τ, t)` of two functions. |
| [`range`](#range) | Values from start to end, as Typst's range: `range(end)`, `range(start, end, step: 1)`, the end left out. |
| [`at`](#at) | Selects a vector component with a zero-based index: `at(vector, index)`, or a matrix element: `at(matrix, row, column)`. |
| [`rank`](#rank) | Rank of a matrix: `rank(A)`, the number of independent rows. |
| [`identity`](#identity) | Identity matrix: `identity(n)` has n rows and columns, ones on the diagonal. |
| [`rows`](#rows) | Number of rows of a vector or matrix: `rows(A)`. |
| [`cols`](#cols) | Number of columns of a vector or matrix: `cols(A)`; a vector has 1. |
| [`asin`](#asin) | Inverse sine, also spelled `arcsin`: `asin(x)` returns an angle in radians from −π/2 to π/2. |
| [`acos`](#acos) | Inverse cosine, also spelled `arccos`: `acos(x)` returns an angle in radians from 0 to π. |
| [`atan`](#atan) | Inverse tangent, also spelled `arctan`: `atan(x)` returns an angle in radians between −π/2 and π/2. |

## `worksheet`

Calculates the math you write, like a Mathcad worksheet.

`#show: worksheet` calculates the rest of the document, `#worksheet[…]` its content (on its own unless `scope` says otherwise), and `#worksheet($…$)` single equations, also outside a worksheet.

```typ
#show: worksheet
#import units: mA, mAh
The battery holds $Q_0 := 3450 mAh$ and the load draws $I := 30 mA$,
so it lasts $t := Q_0 / I = 1 "h"$.
```

**Math.** As in Mathcad:

- `:=` defines (shown as ≔), symbolically: `$I := 30 mA$`, `$P := U I$`, `$f(x) := x^2 + 1$`. Names not defined yet stay names until they are, see *Calculating like Mathcad*.
- `=` inside an expression compares, 1 or 0: `$M := 1 = 2 =$` is 0.
- `=` at the end shows the value, with a bold =: `$t =$` → t **=** 115 h, `$t := Q_0/I =$` defines and shows it. What follows it is the unit and keywords: `$t = 1 "h"$`, `$t = min, digits(2)$`.
- `->` at the end shows the symbolic result, the known values put in, exactly: `$f := (x^2 - 1)/(x - 1) -> simplify$`. With `solve(x)` it solves an equation for x (and defines it when the solution is a value): `$Q_0/2 = I t -> solve(t)$`, `$x^2 - 4 = 0 -> solve(x)$`, `$x + y = 3, x - y = 1 -> solve(x, y)$`, `$R <= U/I -> solve(R)$`.

Keywords: `solve(x, …)`, `substitute(x, …)` (these variables' formulas put in, other names kept), `simplify`, `frac` (exact fractions), `digits(n)` (decimals with n significant digits), `sci` (scientific notation), `eng` (engineering notation). Any other equation with `=` states something: it's checked when everything in it is known (`$U = R I$` after defining them) and left alone as prose when not (`$E = m c^2$`).

**Units** are quoted names, `$9.81 "m"/"s"^2$`, `$3 "bar"$` for your own units (with `worksheet(units: …)`), or unit words imported from solveng's `units` module: after `#import units: mA, kΩ`, `$30 mA$`, `$4.7 kΩ$`; single letters need a hash, `#import units: V` then `$5#V$`. An unquoted single letter is a variable, also alone after `=`: `$t = h$` compares t with a variable h, and `$3 dot h$` is 3 times h; after `->` it's an error, so write `$t -> "h"$`.

**Code.** Each line of a `solveng` code block is calculated the same way and shown with all its steps: `t := Q0/I = [h]`, `Q0/I =`, `x^2 = 4 -> solve(x)`; units in brackets, `;` separates the equations of a system, `//` starts a comment. Inline code and code with another language are left alone.

**Calls.** `#worksheet($U := 5 "V"$, $I := U/R =$)` calculates its equations (and strings, read like a code line) where it stands, with the definitions made before. A code block passed to it is calculated line by line, like a `solveng` code block but without the tag. Its options are for these only; with content, for the content.

**Evaluate.** `#worksheet(evaluate: false)[…]` leaves its math alone: only its `solveng` code blocks and `#worksheet(…)` calls are calculated. Nested worksheets don't inherit it, so a `#worksheet[…]` or `#worksheet($t =$)` inside calculates its math again.

````typ
#worksheet(```
  t := 2 s
  v := 3 m / t =
```)
````

```typ
#worksheet[
  $t := 5 "s"$
  #worksheet(evaluate: false, scope: "inherit")[
    $E = m c^2$ stays as written, and so does $t = 3 "s"$;
    #worksheet($t =$) is still calculated.
  ]
]
```

**Hidden.** `hidden: true` calculates without showing what is calculated: the equations of a call, or in `#worksheet(hidden: true)[…]` the math and code blocks it calculates – its text, headings and prose equations (`$E = m c^2$`) stay. Problems are still shown. Nested worksheets don't inherit it, so a `#worksheet($t =$)` inside shows its equation again.

````typ
#worksheet(hidden: true)[
  The values for the example:
  $U := 5 "V"$, $R := 100 "Ω"$
  ```solveng
  I := U/R
  ```
  The current is #worksheet($I = "mA"$).
]
````

**Scopes.** `scope` says what a worksheet sees and keeps: `"isolated"` starts empty and its definitions end with it; `"inherit"` sees the definitions made before, and its own end with it; `"shared"` sees them, and what it defines stays afterwards, replacing names of the same name (variables, functions and solved values alike). `auto`: isolated for content (`#worksheet[…]`, `#show: worksheet`), shared for equations, strings and code blocks passed to `worksheet`. Typst makes a content block holding only one equation or code block the same value as that equation, so `#worksheet[$x := 1$]` is shared; write `scope: "isolated"` to keep it local. Nested worksheets don't inherit the scope. Clearing never reaches outside: `clear`, `clear-at` and `clear(all: true)` don't delete the outer definitions.

Units change what names mean, so `units` and `builtin-units` need an isolated scope, which starts a unit table of its own: its `units` over the built-in ones (none with `builtin-units: false`, which nested worksheets keep like every other option), never the outer worksheet's `units`. Other options are scoped as before.

```typ
#show: worksheet
$U := 12 "V"$
#worksheet(scope: "shared")[
  $R := 100 "Ω"$
  $I := U/R = "mA"$
]
$I = "mA"$ // 120 mA, kept from the block
#worksheet($U := 5 "V"$, scope: "inherit") // local to this call
$U =$ // still 12 V
```

**Editors.** Each equation and code block the worksheet sees leaves a `metadata` labelled `<solveng>` with its `location()`, whether it was calculated and whether math is evaluated there; each problem one with `kind: "error"`, its `message`, the `element` it's in (`none` for equations and code passed to `worksheet`) and the `source` statement (a code line; empty for math); at the end (and before each match of `clear-at`) one lists everything defined, with values, and the units. Editors like Typewright read them after compiling: `query(<solveng>)`.

**Starting over.** `clear-at` clears every definition at each match; `#clear(all: true)` inside a worksheet does the same once: back to how it started (with `scope: "inherit"` or `"shared"` the outer definitions stay), and `#clear("x", "f")` forgets only those names. After a `#worksheet[…]` the outer definitions come back; with `scope: "shared"`, its final definitions are merged over them.

```typ
#show: worksheet.with(clear-at: heading)                    // every heading
#show: worksheet.with(clear-at: heading.where(level: 1))    // only top-level ones
#show: worksheet.with(clear-at: <exercise>)                 // every element labelled, = Task <exercise>
#show: worksheet.with(clear-at: heading, scope: "inherit")  // keep the definitions made before
```

**Options.** Every option defaults to `auto`, which keeps the current setting, so only what you pass changes. For the whole document: `#show: worksheet.with(digits: 3)`; for a part: `#worksheet(evaluate: false, steps: "result")[…]`; for equations or content it calculates: `#worksheet($R := U/I =$, digits: 2)`; or keep them in a dictionary: `#let opts = (digits: 2, separator: ",")`, then `#worksheet($P := U I =$, ..opts)`. `defined` and `graph` take the same options except `units`. Unknown options and values of the wrong type are shown as an error in the document.

```typ
#show: worksheet.with(digits: 3, form: "fraction")   // your defaults, the whole document
$U := 5 "V"$, $I := 30 "mA"$
#worksheet($I/3 =$, digits: 2)                        // only this equation
#worksheet(evaluate: false, multiplication: "times")[Prose and $E = m c^2$ stay as written.]
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
#import "@preview/solveng:1.0.0": *
#let setup = worksheet.with(digits: 3, multiplication: "times", layout: "column")

// in a document
#import "my-defaults.typ": *
#show: setup
```

| parameter | type | default | |
|---|---|---|---|
| `..body` | content, str |  | The content, or equations to calculate: math, strings (code lines) and code blocks. |
| `scope` | auto, str | `auto` | What the worksheet sees and keeps: `"isolated"` (starts empty, its definitions end with it), `"inherit"` (sees the definitions made before, its own end with it) or `"shared"` (sees them, and its definitions stay afterwards, over the outer ones). `auto` (default): isolated for content, shared for equations, strings and code blocks. `units` and `builtin-units` need `"isolated"`. |
| `clear-at` | none, function, selector, label | `none` | Clear every definition at each match: `heading` (every heading), `heading.where(level: n)` (headings of level n) or a label like `<exercise>` (every element with it, e.g. `= Task <exercise>`): the definitions made since are forgotten, like `#clear(all: true)`. |
| `hidden` | bool | `false` | Calculate without showing what is calculated (only problems are shown): the equations, or the math and code blocks of the content. Not inherited by nested worksheets. |
| `evaluate` | bool | `true` | Calculate the math in the content; `false`: only its code blocks and `#worksheet(…)` calls are. Not inherited by nested worksheets, and not for equations, which are always calculated. |
| `steps` | auto, str | `auto` | Default for `steps` of `solve`: `"full"` (default), `"formula"`, `"result"` or `"value"`. |
| `layout` | auto, str | `auto` | Default for `layout` of `solve`: `"row"` (default) or `"column"`. |
| `digits`, `separator`, … | auto | `auto` | Any [option](#options) for this call; `auto` keeps the current setting. |

Returns content.

## `unit`

An explicit unit reference: `#let ing = unit("iwef")` makes a unit shortcut for `$1ing$`.

Unlike an ordinary string binding, this explicitly marks its value as a unit and completes unit names
inside the string. Only the unit string is highlighted in the declaration; evaluated uses get unit styling.
Hover identifies the binding as a unit shortcut and shows the target unit's definition.
Inline references also work, with or without a space: `$y := 1unit("km")$`, `$y := 1 unit("km")$`.
Completion and unit styling apply inside the string; the constructor collapses to the unit in the editor.
The unit resolves through the units of the worksheet at each use, not where the binding is declared.
It also reaches a name hidden with `none` (`T: none`); with `builtin-units: false` only your own units exist.
Multiletter bindings need no hash; single letters do: `#let i = unit("iwef")`, then `$1#i$`.
`$1i$` stays multiplication by a worksheet variable. Complex numbers are unsupported; `i` is no imaginary constant.

```typ
#show: worksheet.with(units: (iwef: "km"))
#let ing = unit("iwef")
#let i = unit("iwef")
$t := 1ing =$
$u := 1#i =$
```

| parameter | type | default | |
|---|---|---|---|
| `name` | str |  | The unit name or expression, such as `"km"`, `"iwef"`, or `"m/s^2"`. |

Returns content.

## `operation`

An operation for math, by its name: `#let conv = operation("convolve")`, then `$conv(x, h, step: Delta""t)$`.

Like `math.op("convolve")`, the result explicitly selects the underlying operation, independently of
`builtin-functions`. Unlike `math.op`, it is a Typst
function, so math can pass it named arguments: in Typst math, `name: value` in parentheses parses only for
a function. solveng's own operations with named arguments (`convolve`, `range`) are made this way.

| parameter | type | default | |
|---|---|---|---|
| `name` | str |  | The underlying operation's name, such as `"convolve"` or `"tri"`. |

Returns function.

## `solve`

Solves an equation for these names (and defines them): `$x^2 - 4 = 0 -> solve(x)$`.

| parameter | type | default | |
|---|---|---|---|
| `..names` | content |  | What to solve for. |

Returns content.

## `substitute`

The result with these variables' formulas put in, every other name kept: `$P := U^2/R -> substitute(U)$`.

| parameter | type | default | |
|---|---|---|---|
| `..names` | content |  | The variables whose formulas to put in. |

Returns content.

## `simplify`

The symbolic result, what `->` shows anyway: `$f := (x^2 - 1)/(x - 1) -> simplify$`.

## `digits`

Decimals with n significant digits: `$t = 1 "h", digits(2)$`.

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
$R := 7.2 "kΩ"$, $U := 5 "V"$, $I := U/R$, $f(x) := x^2$
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
| `digits`, `separator`, `form`, … | auto | `auto` | Every option of `worksheet` (except `units` and `builtin-units`, which need a worksheet of their own) works here too, for this call only. See [Options](#options). |
| `digits`, `separator`, … | auto | `auto` | Any [option](#options) for this call; `auto` keeps the current setting. |

Returns content.

## `graph`

Plots an equation with `simple-plot`: ranges, nice ticks, axis labels and units are picked automatically.

One string or math holds the curves, separated by a backslash: an expression (`"cos(w)"`), or a function of
what it's plotted against, named in the legend and on the axis: `"U(x: A) = x * [2 Ω]"`, with
the parameter's unit for the x axis, or `"P(..) = U^2/R"`, where `..` takes the one variable
that isn't defined (with its unit from the range or the marks). Like every function, a curve
may use only its parameters and defined variables. With `:=` the curve also defines what it names, as the same equation in a worksheet does: `#graph($y := X(x)$)` plots y over x and defines y as X(x), with x left free; `#graph($y := 2 y$)` plots and defines twice that; `#graph($y(t) := X(t)$)` defines the function y. All curves share x. Where two cross is marked:
`#graph("cos(w) \\ w", mark: auto)` shows the solution of cos(w) = w. `xrange:` / `yrange:` list values that must be visible;
without them the x range is picked around the marks, else around the interesting part
(roots, extrema, poles, intersections). Marked values are always visible too.

```typ
#worksheet("R := 50 Ω", "P0 := 2 W", "U0 := 10 V", hidden: true)
#graph("P(U: V) = U^2/R", ymark: "P0")                    // where P reaches P0
#graph("P(..) = U0^2/R_L", xrange: ("10 Ω", "1 kΩ"))       // R_L, in Ω from the range
#graph("cos(w) \\ w", mark: auto)                          // range picked, crossing marked
#graph("y(x) = x^2 \\ y(x) = 2x + 3", xrange: (-3, 5))     // several curves
#graph("P(R_L: Ω) = U0^2/R_L", xrange: (0, "2 R"), mark: ("R", "extrema"))
#worksheet("tau := vec(0, 1, 2, 3) * 1 s", "y := vec(0, 2, 1, 3) * 1 V", hidden: true)
#graph("y(tau) \\ 1 V/s * tau", mark: "crossings")         // data over tau and a line
```

Uses `@preview/simple-plot:1.1.0`. Ticks use the document's decimal separator, axis labels read `U / V`; curves of the same unit share one graph, with a legend.

**Ranges:** values and expressions in `xrange` / `yrange` must have the axis unit (`$f(20) Omega$` and `"f(20) Ω"` are fine) – `xrange: ("10 Ω", "f(20)")` with a unitless `f(20)` is an error, since it isn't clear what was meant. Without a range: around the marks; else around what's interesting – roots, extrema, poles, edges of the domain (√, ln) and, above all, where curves cross – rounded to nice ticks, starting at 0 when everything interesting is positive. With `ymark`, the y range is around the marked values (steep curves leave the picture). **Units** follow from what's plotted: y's from the curves (shown in the SI unit with a fitting prefix; `preferred-units` picks others), x's from the curve's parameter (`U(I: mA) = …`) or a function x is passed to whose parameter has one (`$q(I: mA) := I R$`, then `#graph("U(I) = q(I)")` is over I in mA), else from the units in `xrange` and the marks, else from the curve and the units of the y values (`P(..) = U0^2/R2` with `ymark: "2 W"`: R₂ in Ω).

**Data:** a vector of values is plotted as the points through its elements, joined in index order (with a dot at each when there are at most 30): `#graph("y")` against the zero-based index k (as in `at(y, k)`), `#graph("y(tau)")` over the vector `tau` of the same length – or over any expression without free names that is one, `y(2 tau)`. Units come from the elements (each vector needs one unit for all its elements; every element must have a value). Function curves can go alongside: `#graph("y(tau) \\ c(tau)")` with a function `c`, or an expression in `tau` (`"2 tau"`), where `tau` stands for the x axis and the function is sampled over it; the x range is from the data's first to last x value unless `xrange` says otherwise, and the units of x and y must agree. Marks work on data too, with linear interpolation between the points: x values, `"roots"` and `"crossings"` (with data and with functions) where the values change sign, `"extrema"` as the local extreme points, and `ymark`. A vector with a free name is several curves instead, one per element (`"vec(t, t^2)"`).

**Marks** default to none. Requested marks are always visible – the ranges grow to show them. Combine them in an array, `(auto, "R1")`.

**Size:** `width` / `height` are the whole graph, labels included; `auto` makes the plot area 8 × 5 cm. Any other named argument goes straight to simple-plot's `plot` (e.g. `show-grid: false`), other positional ones (series from simple-plot) are added as they are. For full control, plot with simple-plot directly.

| parameter | type | default | |
|---|---|---|---|
| `equation` | str, content |  | Curves separated by a backslash; strings also accept complete equations on separate lines. |
| `..args` | any |  | Named options passed on to simple-plot's `plot` (e.g. `show-grid: false`). |
| `xrange` | auto, array, str, content, int, float | `auto` | x values that must be visible: numbers in the x axis's unit (`(1, 2)`), values (`("10 Ω", "1 kΩ")`) or expressions of defined variables (`(0, "2 R")`, math works too). Two or more give the range from the smallest to the largest; a single one widens the picked range. `auto`: around the marked x values, else picked from the curves. |
| `yrange` | auto, array, str, content, int, float | `auto` | y values that must be visible, like `xrange` (numbers in the y axis's SI unit). Two or more give the exact y range. `auto`: fitted to the curves. |
| `mark` | none, auto, array, str, content, int, float | `none` | What to mark, with both coordinates written on the axes: x values like `xrange` (`"R"`, `2`, `"5 V"`; a point with guide lines on every curve, the range grows to show it), `"crossings"`, `"roots"`, `"extrema"` (searched within the range) and `auto`: where two curves cross. An array combines them: `(auto, "R1")`. `()` or `none` (default): no marks. |
| `xmark` | none, array, str, content, int, float | `()` | Like `mark`, but only the x value is written on the axis. |
| `ymark` | none, array, str, content, int, float | `()` | y values to mark, like `yrange`: every point where a curve reaches one, with guide lines; the value is written on the y axis. |
| `samples` | int | `400` | Number of sample points. |
| `width` | auto, length, ratio, relative, fraction | `auto` | Width of the whole graph, labels included: `12cm`, `300pt` or a share like `100%`. `1fr` is the container's width – in a grid, the column's. `auto`: a plot area of 8 cm. |
| `height` | auto, length, ratio, relative, fraction | `auto` | Like `width`; `auto`: a plot area of 5 cm. A share is of the container's height, or of the width when the page grows with its content. |
| `colors` | array, color | `(blue, red.darken(10%), green.darken(25%), orange.darken(10%))` | Colors of the curves, e.g. `(gray, blue, rgb("#f98eff"))`. |
| `digits`, `separator`, `form`, … | auto | `auto` | Every option of `worksheet` (except `units` and `builtin-units`, which need a worksheet of their own) works here too, for this call only. See [Options](#options). |

Returns content.

## `lambdify`

Turns an expression into a plain Typst function of numbers, for value tables,
other plotting packages (lilaq, cetz-plot, simple-plot) or your own calculations.
Defined variables and functions are used with their current values. Needs `context`.

```typ
#worksheet($R := 50 "Ω"$, hidden: true)
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

## `value`

The result of an expression alone, where the text needs it: what `$U =$` shows
after its `=`. Calculated with the worksheet's definitions at this point, which
it leaves as they are.

```typ
#show: worksheet
#import units: kΩ, mA, mV
$R := 4.7 kΩ$, $I := 2 mA$, $U := R I$, $f(x: Ω) := 2 x$
The voltage is #value("U") (#value($U = mV$)), and #value("f(3 Ω)") for 3 Ω.
```

A clause picks the unit or form as after `=`: `#value("U = mV, digits(2)")`; a
trailing `->` shows the symbolic result. A function's name alone shows its body.
It shows one expression and defines nothing: definitions go in the worksheet.

| parameter | type | default | |
|---|---|---|---|
| `expr` | str, content |  | The expression: `"U"`, `"f(2)"`, `"U / 2 = mV"` or `$U$`. |

Returns content.

## `clear`

Forgets definitions: the names given, or with `all: true` every one made so far.

A value and a function share their name – defining one replaces the other – so `clear("f")` forgets `f` either way (`"f()"` and `"f(x)"` are read the same). Names are written as in code (`"Q_0"`, `"tau"`) or in math (`$Q_0$`). A name that isn't defined is shown as a problem. With `all: true` inside a `worksheet` it's back to how the worksheet started (with `inherit: true` the outer definitions stay).

```typ
#show: worksheet
$I := 30 "mA"$, $f(x) := 2 x$, $U := 5 "V"$
#clear("I", $f$)
$I := 1 "A"$
#clear(all: true)
$U := 12 "V"$
```

| parameter | type | default | |
|---|---|---|---|
| `..names` | str, content |  | The names to forget, as strings or math. |
| `all` | bool | `false` | Forget every definition instead. |

Returns content.

## `rect`

Rectangular pulse: `rect(x)` is 1 for |x| < ½, ½ at x = ±½, and 0 otherwise.

The argument and result are unitless. Shift and scale the argument to place a pulse:
`$p(t) := rect((t-2)/4)$` has width 4 and is centered at 2.
`$rect(1/2) =$` gives ½. Integrals split at the pulse boundaries; derivatives stay symbolic.

| parameter | type | default | |
|---|---|---|---|
| `x` | expression |  | Unitless position relative to the pulse center. |

Returns unitless value.

## `tri`

Triangular pulse: `tri(x)` is max(1 − |x|, 0), with height 1 at 0 and support from −1 to 1.

The argument and result are unitless. `$p(t) := tri((t-2)/4)$` is centered at 2 and
reaches zero at −2 and 6. `$tri(1/4) =$` gives ¾. Integrals split at its corners;
the first derivative is piecewise constant and undefined at the three corners.

| parameter | type | default | |
|---|---|---|---|
| `x` | expression |  | Unitless position relative to the pulse center. |

Returns unitless value.

## `sgn`

Sign function: `sgn(x)` is −1 for x < 0, 0 for x = 0, and 1 for x > 0.

The argument and result are unitless. `$sgn(-3) =$` gives −1.
Its distributional derivative stays symbolic.

| parameter | type | default | |
|---|---|---|---|
| `x` | expression |  | Unitless value whose sign to take. |

Returns unitless value.

## `floor`

Greatest integer less than or equal to x: `floor(x)`.

The argument and result are unitless. Used for finite time-delay sums.
Its derivative stays symbolic.

| parameter | type | default | |
|---|---|---|---|
| `x` | number |  | Unitless real value. |

Returns integer.

## `factorial`

Factorial of a nonnegative integer: `factorial(k)`.

Zero factorial is 1. Values through 33! are exact; larger values can be
evaluated numerically through 170!. Its derivative stays symbolic.

| parameter | type | default | |
|---|---|---|---|
| `k` | integer |  | Nonnegative unitless integer. |

Returns number.

## `diff`

Symbolically differentiates an expression with respect to a name: `diff(expression, variable)`.

`$diff(x^3, x) ->$` gives 3x². The variable is local to the operation, so an existing
worksheet value for it does not replace it while differentiating. Unsupported piecewise
or distributional derivatives stay symbolic and cannot be evaluated numerically.

| parameter | type | default | |
|---|---|---|---|
| `expression` | expression |  | Expression to differentiate. |
| `variable` | name |  | Differentiation variable, written as a name rather than a value. |

Returns expression.

## `integrate`

Definite integral: `integrate(expression, variable, lower, upper)`.

`$integrate(x^2, x, 0, 3) =$` gives 9. The variable is local to the operation.
Bounds must have matching units; the result has the integrand's unit times the
variable's unit. Either bound may be infinite. Recognized antiderivatives simplify
exactly; remaining integrals stay symbolic with `->` and use adaptive quadrature for
`=` or graphs. Affine signal arguments are split at their corners so narrow pulses
are not skipped. Undefined integrands or failure to converge report an error.

| parameter | type | default | |
|---|---|---|---|
| `expression` | expression |  | Integrand. |
| `variable` | name |  | Integration variable, written as a name rather than a value. |
| `lower` | expression |  | Lower bound, including `-infinity`. |
| `upper` | expression |  | Upper bound, including `infinity`. |

Returns expression or value.

## `laplace`

Unilateral Laplace transform: `laplace(expression, variable, result-variable)`.

The variables default to t and s: `$laplace(t^2) ->$`, `$F(p) := laplace(exp(-x), x, p)$`.
(The keyword form `-> laplace` was removed; it's an error pointing here.)
The input variable is local, so a worksheet value for it is ignored.
Supports constants, polynomials through degree 33, affine exponentials,
sine, cosine, hyperbolic sine and cosine, linear combinations, exponential
shifts and multiplication by powers of the input variable (up to 20).
Also supports `rect(a*t)`, `tri(a*t)` and `sgn(a*t)` for nonzero a,
and shifted affine signal arguments with known real numeric coefficients.
These are unilateral transforms: only the signal at t >= 0 contributes.
For example, `laplace(rect(t))` is (1 − exp(−s/2))/s and
`laplace(tri(2*t))` is 1/s − 2(1 − exp(−s/2))/s².
Define `$Lambda(t) := tri(t)$` before using Λ as the triangular signal.
Parameters are formal constants; results apply in the half-plane of convergence.
The result variable must differ from the input and must not occur in the input
expression. Unsupported transforms report an error; no numerical transform is used.

| parameter | type | default | |
|---|---|---|---|
| `expression` | expression |  | Function to transform. |
| `variable` | name |  | Input variable, t by default. |
| `result-variable` | name |  | Output variable, s by default. |

Returns expression.

## `invlaplace`

Inverse unilateral Laplace transform: `invlaplace(expression, variable, result-variable)`.

The variables default to s and t: `$invlaplace(1/s^2) ->$`, `$f(x) := invlaplace(1/(p+1), p, x)$`.
(The keyword form `-> invlaplace` was removed; it's an error pointing here.)
Proper rational functions are inverted symbolically using repeated poles,
quadratic factors and exact partial fractions where available. The general
numerical evaluator uses a state-space matrix exponential, including complex
poles and repeated irreducible factors; it does not require numerical roots.

Rational delay expressions `N(s,q)/D(s,q)`, with `q = exp(-d*s)`, are handled
by formal power-series division and causal shifts. Delays must be commensurate
rational numbers. Products, powers, `e^x` and hyperbolic functions are normalized
before inversion. The constant delay coefficient D_0(s) must be nonzero;
each numerator coefficient must have lower degree in s than D_0, and no
denominator coefficient may have higher degree. Polynomial parts requiring
Dirac distributions remain unsupported.

Separable delay denominators use generating-function partial fractions to
produce exact finite sums when their delay poles can be determined exactly.
Other supported inverses remain written as `invlaplace(...)` and can be
evaluated or graphed. Numerical evaluation uses at most 256 delay intervals
and a state dimension of 128; exceeding these limits reports an error.

For a system taking `rect(t)` to `tri(2*t)`, shift both signals to preserve
their negative-time parts under the unilateral transform:
`$x(t) := rect(t-1/2)$`, `$y(t) := tri(2*(t-1/2))$`,
`$H(s) := laplace(y(t), t, s)/laplace(x(t), t, s) ->$`, then
`$g_2(t) := invlaplace(H(s)/s, s, t) ->$` gives the step response.
It is a periodic triangular wave: 0 at integer times and 1 at half-integers.
Results describe the causal function for t >= 0.
The input variable is local. The output name must be distinct and absent from
the input expression. Unsupported forms report an error.
Arrow transforms cannot be combined with `solve` or a second transform keyword.

| parameter | type | default | |
|---|---|---|---|
| `expression` | expression |  | Expression in the transform domain. |
| `variable` | name |  | Input variable, s by default. |
| `result-variable` | name |  | Output variable, t by default. |

Returns expression.

## `convolve`

Convolution: `convolve(x, h)` of two sequences, `convolve(f(τ), g(τ), τ, t)` of two functions.

Of sequences (vectors) it is the discrete convolution, (x ∗ h)ₖ = Σᵢ xᵢ hₖ₋ᵢ, with n + m − 1
elements; `step: Δt` multiplies it by the sampling step, the Riemann sum of the continuous convolution of
the sampled signals. Of functions it is (f ∗ g)(t) = ∫ f(τ) g(t − τ) dτ over the whole line, τ local to
the operation. Without `at`, the result is in the variable itself: `convolve(rect(t), rect(t), t)` is
`tri(t)`. Piecewise polynomial signals – `rect`, `tri`, `sgn`, `abs`, `cases` steps, polynomials, with
numeric affine arguments such as `tri((τ - 2 s)/(4 s))` – convolve exactly into a piecewise polynomial;
`$y(t) := convolve(tri(tau), tri(tau), tau, t) ->$` shows the five pieces, `$y(1/2) =$` gives 23/48.
Other signals (exp, sin, symbolic shifts) stay the integral, evaluated numerically for `=` and graphs.
Units multiply: the result has f's unit times g's unit (times τ's unit, or the step's).

| parameter | type | default | |
|---|---|---|---|
| `f` | expression |  | A sequence (vector), or a function of the variable. |
| `g` | expression |  | A sequence (vector), or a function of the variable. |
| `variable` | name |  | The integration variable of a convolution of functions. |
| `at` | expression |  | Where the convolution of functions is taken, usually the result's variable; default: the variable. |
| `step` | expression |  | Named: the sampling step of sequences, the result is multiplied by it. |

Returns vector or expression.

## `range`

Values from start to end, as Typst's range: `range(end)`, `range(start, end, step: 1)`, the end left out.

`$range(0, 1, step: 1/4)$` is `vec(0, 1/4, 1/2, 3/4)`. Values may have a unit, then the step needs it too:
`$range(0 "s", 1 "s", step: 250 ms)$`. The bounds and the step need values when the range is defined. A
range variable, as Mathcad writes it, includes the end: `$i := 0 .. N$` (or `0...N`) counts from 0 to N in
steps of 1. Used as an index, `$x[i] := tri(t[i])$` defines an element for each value. In Typst code
(`#for i in range(5)`), `range` stays Typst's own.

| parameter | type | default | |
|---|---|---|---|
| `start` | expression |  | The first value; default 0 with one argument. |
| `end` | expression |  | The bound, not included. |
| `step` | expression |  | Named: the distance between values, default 1; negative counts down. |

Returns vector.

## `at`

Selects a vector component with a zero-based index: `at(vector, index)`, or a matrix
element: `at(matrix, row, column)`.

`$at(vec(3, 5), 0) =$` gives 3. The index must be a nonnegative integer smaller
than the number of components. This also selects a solution from a function that
returns several solutions as a vector. The selected component keeps its unit.

| parameter | type | default | |
|---|---|---|---|
| `vector` | expression |  | A vector or matrix, including the result of a function returning a vector. |
| `index` | int |  | Component number, or the row of a matrix, starting at 0. |
| `column` | int |  | Column of a matrix element, starting at 0. |

Returns expression or value.

## `rank`

Rank of a matrix: `rank(A)`, the number of independent rows.

Exact for exact numbers; floats count as zero within rounding of the largest element.

| parameter | type | default | |
|---|---|---|---|
| `matrix` | expression |  | A vector or matrix. |

Returns integer.

## `identity`

Identity matrix: `identity(n)` has n rows and columns, ones on the diagonal.

| parameter | type | default | |
|---|---|---|---|
| `n` | int |  | Number of rows and columns, 1 or more. |

Returns matrix.

## `rows`

Number of rows of a vector or matrix: `rows(A)`.

| parameter | type | default | |
|---|---|---|---|
| `matrix` | expression |  | A vector or matrix. |

Returns integer.

## `cols`

Number of columns of a vector or matrix: `cols(A)`; a vector has 1.

| parameter | type | default | |
|---|---|---|---|
| `matrix` | expression |  | A vector or matrix. |

Returns integer.

## `asin`

Inverse sine, also spelled `arcsin`: `asin(x)` returns an angle in radians from −π/2 to π/2.

The argument is unitless and must lie in [−1, 1]; the result is unitless.
`$asin(1) =$` gives π/2.

| parameter | type | default | |
|---|---|---|---|
| `x` | expression |  | Unitless sine value in [−1, 1]. |

Returns unitless angle in radians.

## `acos`

Inverse cosine, also spelled `arccos`: `acos(x)` returns an angle in radians from 0 to π.

The argument is unitless and must lie in [−1, 1]; the result is unitless.
`$acos(1) =$` gives 0.

| parameter | type | default | |
|---|---|---|---|
| `x` | expression |  | Unitless cosine value in [−1, 1]. |

Returns unitless angle in radians.

## `atan`

Inverse tangent, also spelled `arctan`: `atan(x)` returns an angle in radians between −π/2 and π/2.

The argument and result are unitless. `$atan(1) =$` gives π/4.

| parameter | type | default | |
|---|---|---|---|
| `x` | expression |  | Unitless tangent value. |

Returns unitless angle in radians.

## Options

Options of `worksheet`, and of `defined` and `graph` (except `units`). Each one defaults to `auto`, which keeps the current setting.

### Numbers

| option | type | default | |
|---|---|---|---|
| `digits` | auto, int | `4` | Significant digits of computed values, 1…15. e.g. `57.5 h`, `123500`. |
| `trailing-zeros` | auto, bool | `false` | Keep trailing zeros up to `digits` (`2.500 V`). |
| `separator` | auto, str | `auto` | Decimal separator, any string like `","`. `auto` (default): from the document language, `,` for de, fr, it, … and `.` otherwise. |
| `grouping` | auto, none, str, symbol, content | `none` | Digit grouping for integer parts with 5+ digits, e.g. `sym.space.thin` → `12 345`. |
| `scientific` | auto, none, array | `(-3, 6)` | `(low, high)`: plain notation while the exponent `e` is in `low ≤ e < high`, else `2.5 · 10⁻⁷`. `none` never switches. Only how results are written: `1.12e4` is 1.12 times the variable `e4`, not a number. |
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
| `builtin-units` | auto, bool | `true` | Whether solveng's built-in units exist, true by default. `false`: none at all – not quoted, not as unit words, not with `unit("km")`, not after `=` or `->` – only the units of the option `units`. Needs an isolated scope; nested worksheets keep it. |
| `units` | auto, dictionary | `(:)` | Units of your own, a user overlay over the built-in ones: `(bar: "100 kPa", rpm: "1/min", dozen: 12, ü: auto)`. A value is an expression of units and numbers (`"10^3 m"`, `"1/min"`), a plain number (a factor without unit), `auto` (a base unit of its own) or `none` (hides a built-in name, `T: none`: `T` is then a variable, while `mT` and definitions still reach it). Entries may use each other in any order; a self reference is the built-in unit (`(s: "2s")` doubles the second), circular ones are errors. Prefixes work on your units too (`"mbar"`). Needs an isolated scope, which starts a unit table of its own: the units of the outer worksheet don't apply inside. Write them quoted (`$1 "bar"$`) or declare a shorthand, `#let bar = unit("bar")`, then `$1 bar$` (single letters with a hash, `$1#i$`). |
| `preferred-units` | auto, str, array | `()` | Units for results and graph axes of their dimension, `("bar", "kWh")`, before input units and automatic SI prefixes. |
| `strict` | auto, bool | `false` | Unit names can't be variable names, and units in math must be quoted (`$5 "V"$`), see *Units and variables*. |

### Operations

| option | type | default | |
|---|---|---|---|
| `builtin-functions` | auto, bool | `true` | Enable automatic built-in operation names, true by default. False disables automatic lookup in code and the math names imported from solveng's `functions`. Explicit `math.op("tri")` and `operation("convolve")` references always select the underlying operations. Quoted math names never select built-in operations. |

### Output

| option | type | default | |
|---|---|---|---|
| `steps` | auto, str | `"full"` | Default for `steps` of `solve`: `"full"` (default), `"formula"`, `"result"` or `"value"`. |
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

## Built-in operations

Built-in operations work in worksheet math, calculation strings and graphs. In strings and code their
names work as they are: `#worksheet("tri(0) =")`. In math, Typst needs a binding for every name it doesn't
know, and `#import "@preview/solveng:1.0.0": *` brings none: import the ones you use from solveng's
`functions` module, `#import functions: tri, sgn` (or `#import functions: *`), then `$tri(t)$`. Like any
import, it holds only where it's written, so no name appears in a document unasked. `at` needs it most:
without the import, `$at(v, 0)$` is Typst's symbol @. Typst's own math names (`sin`, `sqrt`, `vec`, …)
need no import.
`builtin-functions: false` disables automatic operation names, including the names imported from `functions`.
A worksheet's own functions take precedence over automatic names.

Explicit references always select the underlying operation: `#let triangle = math.op("tri")`,
then `$triangle(x)$`, also with automatic names disabled. The editor completes the operation's name
inside `math.op` and highlights the string and its binding's uses in evaluated worksheets.
Quoted names such as `$"tri"(x)$` call worksheet functions only; they never select built-in operations.
Typst's native operation content, such as `math.sin`, is also an explicit reference.
In math, use the native constructor directly: `$y := op("tri")(x)$`. Completion inside its string offers
the underlying operations even with automatic names disabled. The operation string gets built-in styling;
the constructor collapses to its contents in the editor.

For named arguments, use `operation("convolve")`: Typst only parses named math arguments for functions.

```typ
#import "@preview/solveng:1.0.0": *
#show: worksheet.with(builtin-functions: false)
#let triangle = math.op("tri")
#let conv = operation("convolve")
$a := triangle(1/2) =$
$y := conv(vec(1, 2), vec(1, 1), step: 2) =$
```

The following operations use Typst's existing math names or syntax:

| operation | behavior |
|---|---|
| `sqrt(x)` | Real square root; x must be nonnegative. Units are square-rooted too: `sqrt([9 m^2])` is 3 m. |
| `abs(x)` | Absolute value, preserving x's unit. |
| `exp(x)`, `ln(x)`, `log(x)` | Exponential, natural logarithm and base-10 logarithm. Arguments are unitless; logarithms require x > 0. |
| `sin(x)`, `cos(x)`, `tan(x)` | Trigonometric functions of a unitless angle in radians. |
| `arcsin(x)`, `arccos(x)`, `arctan(x)` | Inverse trigonometric functions; also spelled `asin`, `acos`, `atan`, documented below. |
| `sinh(x)`, `cosh(x)`, `tanh(x)` | Hyperbolic functions of a unitless argument. |
| `vec(a, b, …)` | Column vector. Use `at(vector, index)` to select a component, starting at 0. |
| `mat(a, b; c, d)` | Matrix, rows separated by semicolons, blocks allowed: `mat(A, b)`, `mat(A; B)`. `A B` multiplies, `A^(-1)` inverts, `A^T` transposes, `a times b` is the cross product. Use `at(matrix, row, column)` for an element. |
| `det(A)`, `tr(A)`, `norm(v)` | Determinant and trace of a square matrix; length of a vector (‖v‖). |
| `cases(value, condition, …, default)` | First matching branch; conditions are unitless, and only the selected value is evaluated. The final default is optional. In math use `cases(0 & t < 0, 1 & "otherwise")`. |
| `laplace(expression, variable, result-variable)`, `invlaplace(expression, variable, result-variable)` | Symbolic unilateral transforms. Defaults: t → s and s → t. See their entries below for supported forms. |
| `sum(expression, index, lower, upper)` | Sum over inclusive integer bounds; the index is local and unitless. In math use `sum_(k=0)^10 k^2`. Small finite sums simplify exactly; infinite sums stay symbolic with `->` and are evaluated for `=` and graphs. |

For sums, the upper bound may be `infinity`. Piecewise linear signal sums are counted
directly between their corners; other numerical sums use a shrinking-tail convergence
estimate with a limit of 100000 terms. Nonconvergent or slower-converging series report an error.

```typ
#import "@preview/solveng:1.0.0": *
#show: worksheet
$a := sqrt(9 "m"^2) =$
$b := sum_(k=0)^3 k^2 =$
$c := cases(0 & b < 0, b & "otherwise") =$
```

## Calculating like Mathcad

Three operators, and `=` keeps its meaning from maths:

- **`:=` defines**, symbolically. Names defined by then are put in right away, names that aren't stay names until they are: `$h := 5 z$` works before `z` is defined, and after `$z := 20 cm$`, `$h =$` shows 1 m. What's put in is the value at that point – defining a name again later changes what comes after, not what came before.
- **`=` inside an expression compares**, as Mathcad's bold =: `$M := 1 = 2 =$` gives 0, `$M := 2 "A" = 2000 mA$` gives 1. Values of different units are never equal (`1 A = 2 V` is 0, `1 A != 2 V` is 1); `<`, `<=`, `>`, `>=` and `!=` compare too, chains like `$0 < x <= 1$` hold when every step does.
- **A trailing `=` shows the value**, numerically, with a bold =. After it come a unit and keywords: `$t = 1 "h", digits(3)$`. Every name in it must have a value by then.
- **A trailing `->` shows the symbolic result**: defined values put in, exactly (fractions, roots and π stay), undefined names kept: `$x + 3/3 y -> simplify$` is x + y. With `solve(x)` it solves – symbolically when it can (`$0 = 1 + 3/x + y^2 -> solve(x)$` is x = −3/(y² + 1)), numerically when every other name has a value – and defines x when the solution is a value. `substitute(U, …)` shows the result with these variables' formulas put in and every other name kept: after `$U := R I$`, `$P := U^2/R -> substitute(U)$` is I² R.
- **Any other `=`** states something: it's checked when everything in it has a value (`$U = R I$`), and left alone as prose when not (`$E = m c^2$`).

**Functions** are defined the same way: `$f(x) := x^2 + 1$`. Their body is symbolic too, so a function needn't define everything it uses: names that aren't parameters and aren't defined stay names, looked up when the function is evaluated, and cancel where the algebra lets them (`$U(x: m) := h x a$`, then `$g(x: m) := U(x)/a ->$` is `1 m · x` without any `a`). Undefined function calls stay symbolic too: `$f(t) := integral_(-infinity)^t sigma(tau) dif tau$` is valid before `sigma` is defined; numerical evaluation requires its definition and convergence. A parameter can have a unit, `f(I: mA)`: arguments must carry its dimension (`f(2"mA")` is valid, `f(2)` reports a missing unit); `any` takes every unit. `..` makes the body's other undefined names parameters, in the order they appear: `$P(..) := U^2/R$` is `P(U, R)`. A parameter passed on to one with a unit takes that unit: with `$q(x: A) := 2 x$`, `$f(y) := q(y)$` makes y an A. **Named parameters** have a default value and are passed by name, as in Typst: after `$f(x, n: 2) := x^n$`, `$f(3) =$` is 9 and `$f(3, n: 3) =$` is 27; a named argument that isn't a parameter, a positional one passed by name and one passed twice are errors. After the colon, a unit is a type (`I: mA`) and a value a default: what starts with a number or a sign (`n: 2`, `R: 2 Ω`), a defined variable (`s: k`), or parentheses holding no unit (`n: (N0)`). Defaults are taken where the function is defined, as Typst evaluates them, and come after the other parameters.

````typ
#show: worksheet
#import units: cm
$h := 5 z$, $z := 20 cm$, $h =$             // h ≔ 5z, z ≔ 20 cm, h = 1 m
$U(x: m) := h x a ->$                      // U(x) ≔ h x a → 1 m · a x
$g(x: m) := U(x)/a ->$                     // g(x) ≔ U(x)/a → 1 m · x
$M := 1 "A" = 2 "V" ->$                        // M ≔ 1 A = 2 V → 0
```solveng
k := 2 m = 200 cm =                        // k = 1: the comparison holds
```
````

## Writing values

````typ
#show: worksheet
#import units: kΩ, A
$R := 4.7 kΩ$                        // math: a number and a unit
$I := frac(2, 4) #A$, $Q_0 := 3450/2 "mAh"$, $U := (1+2) dot 1,5 "V"$, $A := 2^3 "m"^2$  // arithmetic in the number
```solveng
R2 := 2 R; Z := [5 V] / I            // code: units in brackets inside an equation
U2 := R [1 mA]
Q := U2 [2 s] -> substitute(U2)     // U2's formula put in: Q ≔ U2 · 2 s → 1/500 C · R
```
````

- **Units:** SI base units (m kg s A K mol cd) and V A Ω W J C F H S Hz N Pa T Wb, min h Ah Wh As Ws eV l %, ‰, °, rad, °C (as a temperature *difference*), all SI prefixes; your own with the option `units`. Powers as `m^2`, `m²`, `s^-1`; quotients as `m/s^2`; products with spaces `kg m`; numbers with powers in definitions, `"10^3 m"`.
- **Typeable spellings:** `Omega` → Ω, with a prefix glued as in math: `k""Omega` → kΩ (`kOmega` is not a unit, like `DeltaT` is not ΔT); `u` → µ, `degC` → °C, `deg` → °. They are printed with the proper symbols (`[20 uA]` → 20 µA). In math, units are quoted (`$4.7 "kΩ"$`, `$20 "µA"$`) or imported unit words, see [Units and variables](#units-and-variables).
- **Decimal comma:** `7,2 V` works; the output separator follows the document language (option `separator`).
- **No e-notation:** numbers are written out (`$11200$`, or `$1.12 dot 10^4$`). `1.12e4` is the coefficient 1.12 times the variable `e4` (undefined unless defined; quote it, `$"e4"$`, for a name of several letters). Scientific notation is only how results are written (option `scientific`, keyword `sci`).
- **Names:** the acute accent `´` and the backtick (`` ` ``) are literal name characters (`$x´ := 3$`), with no escaping needed. Digits continue a name: `t2` differs from `t_2` and keeps its digits on the baseline. Complete Latin letter runs normalize to Greek: `Delta2tau` → Δ2τ, while `Deltatau` stays unchanged. `""` joins name fragments: `Delta""tau` → Δτ and `t""2` → t2. Quotes only carry a name through Typst: `$"Delta2tau"$` names Δ2τ, and `$"Delta\"\"tau"$` names Δτ. Explicit subscripts may be grouped: `$R_(a b)$` names R_ab. Operators and whitespace end a name; a leading number remains a coefficient (`2t2` is 2 times t2).
- **Sums:** `sum_(k=0)^10 k^2` or `sum(k^2, k, 0, 10)` sums over inclusive integer bounds. The index is local and case-sensitive. Parenthesize a summand containing addition. Small finite sums simplify exactly; infinite sums stay symbolic after `->` and are evaluated numerically for `=` and graphs. For example, `$s_2(t) := sum_(k=0)^infinity sigma(t-k+3/4)$` defines a function usable as `#graph("s_2(t)")` once `sigma` is defined. Piecewise linear signal sums are counted directly between their corners. Other numerical sums use at most 100000 terms and a shrinking-tail convergence estimate; slow or nonconvergent series report an error. Bounds and the index are unitless, while summands may carry units.
- **Signals:** `rect(x)` is 1 for |x| < ½, ½ at ±½, otherwise 0; `tri(x)` is max(1 − |x|, 0); `sgn(x)` is −1, 0 or 1. Arguments are unitless. In math, import the names from `functions` (`#import functions: rect, tri, sgn`); a worksheet function such as `$Lambda(t) := tri(t)$` or `#let triangle = math.op("tri")` gives another name. Linear combinations simplify and can be solved; definite integrals split affine arguments at their boundaries. Distributional derivatives stay symbolic.
- **Ranges and elements:** `$i := 0..N$` (also `0 .. N`, `0...N`) is a range variable: the values from 0 to N in steps of 1, both included, counting down when N < 0; with units, use `range(start, end, step: …)`. `x[i]` and `A[i, j]` are elements, counted from 0, written right after the name without a space; a vector of indices selects several (`x[i]` with i a range is a vector). `$x[i] := …$` defines an element for each value of the range variable i, the body calculated with i standing for that value: `$t[i] := i Delta""t$`, `$x[i] := tri(t[i])$`. Elements not defined yet are 0; defining one past the end extends the vector, as in Mathcad. `A[i, j] := …` defines a matrix. In code, `x[i]` is an index and `R [1 mA]`, with a space, a value with a unit.
- **Convolution:** `convolve(x, h)` of sequences and `convolve(f(τ), g(τ), τ, t)` of functions, see `convolve`. Sequences of values are convolved numerically (whole numbers exactly), so long sampled signals stay fast; functions exactly where they're piecewise polynomial.
- **Operators:** `+ - * / ^ · ×`, implicit multiplication (`2 pi f C`), `sqrt exp ln log sin cos tan asin acos atan abs`, constants `pi e` (unless you define a variable with that name), your own functions.

## Units and variables

Many engineering symbols are also units: `A` (area, ampere), `V` (volume, volt), `T` (period, tesla), `h` (height, hour), `C`, `F`, `W`, `m`, `s`, `l`, `L`, … solveng keeps them apart by *how* they are written, never by guessing:

- **Math:** units are quoted names, `$9.81 "m"/"s"^2$`, `$3 "bar"$` (your own too), or unit words: solveng's `units` module has every built-in unit and every prefixed one as a name, imported where you want them – `#import units: mA, kΩ, s`, or `#import units: *` – and then `$30 mA$`, `$4.7 kΩ$`, `$2#s$` (a single letter needs a hash, since Typst reads a lone letter in math as the letter). Names of Typst's global scope (`h`, `v`) are left out, so `#import units: *` never hides `#h(1em)`; Typst's math names are in (`#import units: min`, then `$t := 2 min$` and `$t -> min$` read the minute, and `min(a, b)` is no longer Typst's operator there). Nothing comes with `#import "@preview/solveng:1.0.0": *`: a shorthand in your document is one you imported or declared, so it's always clear where a name comes from, and like any import it holds only in its scope (a `#worksheet[…]` block, a file). A unit word means what the worksheet where it's used has under that name: your own unit if `units` redefines it, an error with `builtin-units: false` unless `units` defines it, a variable when `units` hides it (`T: none`). Unquoted single letters are variables, including after numbers (`$5 V$` multiplies by the variable V) and after a trailing `=` (`$t = h$` compares); after `->`, where only a unit can follow, one is an error: write `$t -> "s"$`, or `$t -> #s$` with `#import units: s`. Parameter annotations such as `$f(x: A) := x$` accept unit names; calls must carry the dimension, as in `$f(1"A") =$`. Quoted text is a name of several letters (`$"tews" := 2$`), as Typst needs; `"as"` casts.
- **Code:** a unit is read only where a value is written – right after a number (`I := 30 mA`) or in brackets (`[5 V]`, `[A]` for one ampere) – and in the clause after `=` or `->` (`= mA`; a single letter in brackets, `-> [h]`). Every other name in an equation is a variable. A value whose unit is also a defined variable (`t := 3 h` with `h` defined, `f := 1/T` with `T` defined) is refused, with a hint how to write either meaning: `[3 h]` for hours, `3*h` or `1/(T)` for the variable.

````typ
#show: worksheet
#import units: Hz, ms
$A := 12$, $x := A dot 2"A" =$        // x ≔ A · 2 A = 24 A
$T := 2"s"$, $f := 1/T = Hz$          // f ≔ 1/T = 0.5 Hz
$T -> ms$                             // 2000 ms
```solveng
g := 1/T                             // error: 'T' in '1/T' is both a defined variable and a unit – …
```
````

For documents where nothing may be read two ways, `#worksheet(strict: true)` (or `#show: worksheet.with(strict: true)`) also refuses unit names as variable names (`A := 3 m^2` → pick `A_1`, or remove the unit).

**Units of your own.** `units: (…)` defines units over the built-in ones: an expression of units and numbers (`bar: "100 kPa"`, `km_old: "10^3 m"`, `rpm: "1/min"`), a plain number (`dozen: 12`, a factor without unit), `auto` (a base unit of its own, `bit: auto`) or `none`. Units change what names mean, so they belong to an isolated worksheet (`scope: "isolated"`, the default for content); each starts a unit table of its own, built-in units (unless `builtin-units: false`) and its `units`, never the outer worksheet's. `inherit` and `shared` worksheets and equations passed to `worksheet` use the table around them and can't set `units` or `builtin-units`.

Definitions can refer to each other regardless of their order: `(km: "m", tui: "km")` makes both units one metre. A self reference is the built-in unit: `(s: "2s")` doubles the second. Circular dependencies such as `(zu: "uz", uz: "zu")` are rejected. A name defined anew is that unit everywhere, also where it would read as a prefix: with `(km: "s")`, `km` is a second and results are never shown in "km" for kilometres. `none` hides only the exact name: `(s: none)` makes `s` a variable, while `ms` and definition expressions such as `(mys: "s")` still work.

`builtin-units: false` leaves no built-in unit at all – not quoted, not as unit words, not with `unit("km")`, not after `=` or `->`: only what `units` defines exists, built from `auto` base units and numbers (`units: (m: auto, km: "1000 m", s: auto, min: "60 s")`). Prefixes still apply to your units (`mm`, `ms` above). Nested worksheets keep the setting, like every option.

Unit names support the same symbols as variable names. `(alpha: "m")` defines the unit `α`: `$34"alpha"$` and `$34"α"$` agree, and completion inserts `"α"`. With explicit Typst unit bindings, `$34alpha$` and `$34#α$` work too: `#let alpha = unit("alpha")` and `#let α = unit("alpha")`. The editor collapses spelled names to `α` in equations.

Your own units have no unit words: write them quoted, `$34"s"$`, or declare a shorthand. A worksheet variable with that quoted name takes precedence over the unit. Completion inserts quoted units, and can replace `#s` with `"s"` after a number.

An explicit unit shortcut uses `unit`: `#let ing = unit("ing")`, used as `$1ing$`. A single letter
needs an explicit reference: `#let i = unit("i")`, `$1#i$`. The unit name resolves through the units of the
worksheet where it's used, so the same binding can denote different units in nested worksheets. Completion is available inside
the string; hover identifies the binding as a unit shortcut. Plain string bindings are not detected as unit shortcuts.
`$1i$` stays multiplication
by a worksheet variable. Complex numbers are unsupported; `i` is not a built-in imaginary constant.
Typst options cannot create lexical bindings. Use the editor action *Declare unit overlay bindings* to
insert declarations for an option's names, or write them directly.

```typ
#show: worksheet.with(units: (ing: "km", i: "km"))
#let ing = unit("ing")
#let i = unit("i")
$a := 1ing =$
$b := 1#i =$
$c := 1"i" =$
```

```typ
#show: worksheet.with(units: (bar: "100 kPa", rpm: "1/min", t: "1000 kg", T: none, km_old: "10^3 m"))
$p := 3 "bar"$, $n := 3000 "rpm"$          // prefixes work too: "mbar"
#worksheet(preferred-units: ("bar",), scope: "inherit")[$p/2 =$]   // results of this dimension in bar
```

```typ
#show: worksheet.with(builtin-units: false, units: (m: auto, km: "1000 m", s: auto, min: "60 s"))
$d := 3 "km"$, $t := 90 "s" -> min$, $v := d/t -> "km"/"min"$   // "h", "g", "V": no units here
```

## How it solves

solveng calculates in a WebAssembly plugin written in Rust (`core/` in the repository). Equations are solved symbolically first, other names as parameters: an unknown that occurs once is isolated; an equation that is a polynomial in it, once both sides are over one denominator, is solved by formula up to degree 2 and for all real roots numerically above; a system is solved one equation at a time where an equation has a single unknown, and by elimination where it's linear. Results are simplified: like terms combined, fractions cancelled (`(x^2 - 1)/(x - 1)` is x + 1), exact numbers kept exact. What can't be done symbolically is solved numerically, once every other name has a value: sign changes over ±10⁻¹⁵…10¹⁵ (times the target unit) are bisected and Newton starts from every local minimum of the absolute value; systems by damped Newton from 41 starts. Units of unknowns are inferred from the equations; mismatched units stop with an error.

A solved definition keeps its solutions: `circle(x, r) := x^2 + b^2 = r^2 -> solve(b)` returns a vector of both branches. A relation function can supply the equation: `p(a, b, c) := a^2 + b^2 = c^2`, then `circle(x, r) := p(x, b, r) -> solve(b)`. The solve variable is local, not a parameter. One solution returns a scalar; several return a vector, with `at(vector, 0)` selecting its first component. Graphing a vector plots every branch. Solved definitions require a symbolic solution; equations in `vec(...)` with several solve variables return components in the requested order, nested by branch when needed.

## Limits

- **Numbers** are exact fractions as long as they fit (128-bit numerator and denominator), else 64-bit floats: about 15 significant digits, magnitudes up to 10³⁰⁰.
- **Isolating** `sqrt(x) = 2` gives only the positive root, periodic functions only the principal value (`tan(x) = 1` → `arctan(1)`); polynomials like `x^2 = 4` give all their roots.
- **Symbolically**, polynomials are solved up to degree 2; higher degrees need values for their coefficients. Simplifying cancels common factors of polynomials (with exact coefficients) but doesn't factor or use trigonometric identities.
- **Numeric search** can miss roots squeezed between nearby poles, roots of very fast oscillations, and roots outside ±10¹⁵ (scaled by the target unit); the result then says "no real solution".
- **Systems** are solved numerically from a fixed set of starts; solutions far away from them can be missed. Underdetermined systems show one arbitrary solution.
- **Domains:** only real numbers; `(-8)^(1/3)` is −2, `sqrt(-1)` an error.
- **°C** is a temperature difference (no 273.15 offset); temperatures in formulas like `p V = n R T` need K. New base dimensions (bit, €) can be added with `units: (bit: auto)`, at most 8.
- **Prefixes:** a lone `m` before a unit is metre: `"m s"`, `$"m" Omega$` are m·s and Ω·m; write `"ms"`, `$"m"Omega$` for milli.
- **Unit expressions** in `units` take whole-number powers (`"10^3 m"`, `"10^(-3) m"`, `"m^2"`); fractional ones (`"m^(1/2)"`) aren't read.
- **Functions** can't call themselves.
- **Ranges** have at most 100000 values, and their bounds and steps need values when they're defined. In math an index follows its name directly, so `a[b + c]` is an element of a – write `a dot [b + c]` or `a (b + c)` for a product.
- **Convolution** of functions is exact for piecewise polynomials with numeric breakpoints (rect, tri, sgn, abs, cases with affine conditions, polynomials); exponentials, trigonometric functions, symbolic shifts and integrals that diverge stay the integral, evaluated numerically.
- **Matrices:** names without a value are scalars, so they commute in products until they get a value (define matrices before multiplying them, or pass them to a function). Symbolic elimination treats an entry as zero only when it simplifies to 0, so entries that vanish by an identity like sin² + cos² = 1 count as nonzero. Floats count as zero within 10⁻¹³ of the largest element (times the size). Unknowns of `solve` are scalars: write a vector of unknowns as `vec(x, y)`.
- **Math in a worksheet:** an equation with `=` and only known names is checked, so a formula meant as plain text that happens to use defined names (`$a^2 + b^2 = c^2$` after defining a, b and c) must hold. Put those outside the worksheet or use `#worksheet(evaluate: false)[…]` around them.

## Errors

Problems (an unknown unit, a missing value, units that don't match, a typo) are shown as a red note where they happen; the rest of the document still compiles:

> ⚠ solveng: 'U' in 'P := U I' isn't defined – define it first with :=, or solve for it: … -> solve(U)

`#show: worksheet.with(errors: "panic")` stops the compilation instead (useful in CI). Unknown named arguments of `worksheet`, `defined` and `graph` are reported the same way; those of `lambdify` stop the compilation.

## HTML

With `typst compile --features html --format html`, equations become MathML, `defined` columns that reflow with the width (or a table with `style: "column"`), graphs inline SVGs (via `html.frame`) that shrink with the page, errors styled notes. One small stylesheet is added to the page (math font STIX Two Math / Cambria Math / Latin Modern Math, spacing, table stripes; classes `solveng`, `solveng-defs`, `solveng-list`, `solveng-plot`, `solveng-error`; casts and types are `solveng-chip` with `solveng-cast` or `solveng-type`, coloured inline). HTML export is experimental in Typst; in column layout browsers drop the space after a leading `=`.

## Testing

The engine is tested in Rust, the package with [tytanic](https://github.com/typst-community/tytanic) (`cargo install tytanic`); the scripts need Python 3 and `typst` on the `PATH`:

```sh
cargo test                                    # the engine (units, parsing, algebra, solving, worksheets, graphs, random inputs) and the editor (IDE, language server)
plugin/build.sh                               # src/solveng.wasm from the engine (rustup target add wasm32-unknown-unknown; wasm-opt if installed)
tt run                                        # units, solving, names, lambdify and the README examples (reference images)
tt update readme                              # accept changed output of the README examples
python3 tests/examples.py                     # every example in README.md and docs/reference.md compiles on its own
python3 tests/reference.py                    # docs/reference.md from the doc comments in src/ (--check: is it up to date?)
python3 scripts/unit-words.py                # src/unit-words.typ (the units module) from the unit table (--check: is it up to date?)
uv run --no-project --with sympy --with mpmath python tests/oracle/generate.py   # the reference results
```

`cargo test` includes a fuzzer (`core/tests/fuzz.rs`): random nonlinear equations, polynomials, systems, physics formulas with random prefixes and random numbers to format, checked against direct calculation, and functions whose shown return unit must match what their calls produce; and one for the editor (`editor/ide/tests/fuzz.rs`): random and mutated documents through every editor feature at random positions, which must neither panic nor return ranges outside the text (`FUZZ_N=5000 cargo test --release fuzz` for more cases). `core/tests/oracle.rs` checks families of matrices, integrals, series, simplifications and equations against SymPy and mpmath (the results are in `core/tests/oracle/`, so `cargo test` needs neither); what the engine can't do yet is listed in `core/tests/oracle/known.txt`, which only shrinks: a listed case that passes fails the test until `ORACLE_BLESS=1 cargo test --test oracle` updates the list. `TYPST=…` picks another compiler for the examples, e.g. the oldest supported version.
