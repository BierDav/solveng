# solveng reference

<!-- Generated from the doc comments in src/ by `python3 tests/reference.py` – edit those, not this file. -->

Everything solveng does, for looking things up. For an introduction see the [README](../README.md).

- [`worksheet`](#worksheet)
- [`base-unit`](#base-unit)
- [`unit`](#unit)
- [`constant`](#constant)
- [`operation`](#operation)
- [`solve`](#solve)
- [`substitute`](#substitute)
- [`simplify`](#simplify)
- [`digits`](#digits)
- [`sci`](#sci)
- [`eng`](#eng)
- [`fraction`](#fraction)
- [`defined`](#defined)
- [`graph`](#graph)
- [`lambdify`](#lambdify)
- [`value`](#value)
- [`clear`](#clear)
- [`rect`](#rect)
- [`tri`](#tri)
- [`sgn`](#sgn)
- [`Re`](#re)
- [`Im`](#im)
- [`conj`](#conj)
- [`number`](#number)
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
- [Modifiers](#modifiers)
- [Calculating like Mathcad](#calculating-like-mathcad)
- [Writing values](#writing-values)
- [Complex numbers](#complex-numbers)
- [Units](#units)
- [How it solves](#how-it-solves)
- [Limits](#limits)
- [Errors](#errors)
- [HTML](#html)
- [Testing](#testing)

## Functions

| function | does |
|---|---|
| [`worksheet`](#worksheet) | Calculates the math you write, like a Mathcad worksheet. |
| [`base-unit`](#base-unit) | A unit of a new, independent dimension: `#let bit = base-unit("information")`. |
| [`unit`](#unit) | A unit defined by an expression of unit values: `#let minute = unit($60 #s$)`. |
| [`constant`](#constant) | Defines a constant: an identity and the math expression it stands for, which can carry units. |
| [`operation`](#operation) | An operation under a name of your choice: `#let conv = operation("convolve")`, then `$conv(x, h)$`. |
| [`solve`](#solve) | Solves an equation for these names (and defines them): `$x^2 - 4 = 0 -> solve(x)$`. |
| [`substitute`](#substitute) | The result with these variables' formulas and constants' expressions put in, every other name kept: `$P := U^2/R -> substitute(U)$`, `$E := m #c^2 -> substitute(#c)$`. |
| [`simplify`](#simplify) | The symbolic result, what `->` shows anyway: `$f := (x^2 - 1)/(x - 1) -> simplify$`. |
| [`digits`](#digits) | Decimals with n significant digits: `$t = #h, digits(2)$`. |
| [`sci`](#sci) | Scientific notation: `$C = sci$` → 2.2 · 10⁻⁶ F. |
| [`eng`](#eng) | Engineering notation, exponents in steps of 3: `$f = eng$`. |
| [`fraction`](#fraction) | Exact fractions: `$I / 3 = fraction$` → ⅙ A. |
| [`defined`](#defined) | Lists the definitions made so far, in order. |
| [`graph`](#graph) | Plots an equation: ranges, nice ticks, axis labels and units are picked automatically. |
| [`lambdify`](#lambdify) | Turns an expression into a plain Typst function of numbers, for value tables, other plotting packages (lilaq, cetz-plot, simple-plot) or your own calculations. |
| [`value`](#value) | The result of an expression alone, where the text needs it: what `$U =$` shows after its `=`. |
| [`clear`](#clear) | Forgets definitions: the names given, or with `all: true` every one made so far. |

## Operations

| operation | does |
|---|---|
| [`rect`](#rect) | Rectangular pulse: `rect(x)` is 1 for \|x\| < ½, ½ at x = ±½, and 0 otherwise. |
| [`tri`](#tri) | Triangular pulse: `tri(x)` is max(1 − \|x\|, 0), with height 1 at 0 and support from −1 to 1. |
| [`sgn`](#sgn) | Sign function: `sgn(x)` is −1 for x < 0, 0 for x = 0, and 1 for x > 0. |
| [`Re`](#re) | Real part of a complex number: `Re(3 + 4 #i)` is 3. |
| [`Im`](#im) | Imaginary part of a complex number: `Im(3 + 4 #i)` is 4, a real number. |
| [`conj`](#conj) | Complex conjugate: `conj(3 + 4 #i)` is 3 − 4i, shown with a bar. |
| [`number`](#number) | A value as a plain number in a unit: `number(U, mV)` is U in millivolts, without a unit. |
| [`floor`](#floor) | Greatest integer less than or equal to x: `floor(x)`. |
| [`factorial`](#factorial) | Factorial of a nonnegative integer: `factorial(k)`. |
| [`diff`](#diff) | Symbolically differentiates an expression with respect to a name: `diff(expression, variable)`. |
| [`integrate`](#integrate) | Integral: `integrate(expression, variable)` is the antiderivative, `integrate(expression, variable, lower, upper)` the definite integral. |
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
#import units: mA, mAh, h
#show: worksheet
The battery holds $Q_0 := 3450 mAh$ and the load draws $I := 30 mA$,
so it lasts $t := Q_0 / I = #h$.
```

**Math.** As in Mathcad:

- `:=` defines (shown as ≔), symbolically: `$I := 30 mA$`, `$P := U I$`, `$f(x) := x^2 + 1$`. Names not defined yet stay names until they are, see *Calculating like Mathcad*.
- `=` inside an expression compares, 1 or 0: `$M := 1 = 2 =$` is 0.
- `=` at the end shows the value, with a bold =: `$t =$` → t **=** 115 h, `$t := Q_0/I =$` defines and shows it. What follows it is the unit and modifiers: `$t = #h$`, `$t = min, digits(2)$`.
- `->` at the end shows the symbolic result, the known values put in, exactly: `$f := (x^2 - 1)/(x - 1) -> simplify$`. With `solve(x)` it solves an equation for x (and defines it when the solution is a value): `$Q_0/2 = I t -> solve(t)$`, `$x^2 - 4 = 0 -> solve(x)$`, `$x + y = 3, x - y = 1 -> solve(x, y)$`, `$R <= U/I -> solve(R)$`.

Modifiers, imported from `modifiers` where you use them (`#import modifiers: solve, digits`): `solve(x, …)`, `substitute(x, …)` (these variables' formulas put in, other names kept), `simplify`, `fraction` (exact fractions), `digits(n)` (decimals with n significant digits), `sci` (scientific notation), `eng` (engineering notation). Any other equation with `=` states something: it's checked when everything in it is known (`$U = R I$` after defining them) and left alone as prose when not (`$E = m c^2$`).

**Units, constants, operations and modifiers** are Typst bindings, imported where you use them: `#import units: mA, s`, `#import constants: pi, i`, `#import operations: tri`, `#import modifiers: solve`, then `$30 mA$`, `$2 #s$`, `$2 pi$`. A binding of a single letter needs a `#` in math. Quoted text names worksheet variables; it never means a unit or operation. Raw blocks, also ```` ```solveng ```` ones, are ordinary text.

**Calls.** `#worksheet($U := 5 #V$, $I := U/R =$)` calculates its equations where it stands, with the definitions made before. Its options are for these only; with content, for the content.

**Evaluate.** `#worksheet(evaluate: false)[…]` leaves its math alone: only `#worksheet(…)` calls in it are calculated. Nested worksheets don't inherit it, so a `#worksheet[…]` or `#worksheet($t =$)` inside calculates its math again.

```typ
#import units: s
#worksheet[
  $t := 5 #s$
  #worksheet(evaluate: false, scope: "inherit")[
    $E = m c^2$ stays as written, and so does $t = 3 #s$;
    #worksheet($t =$) is still calculated.
  ]
]
```

**Hidden.** `hidden: true` calculates without showing what is calculated: the equations of a call, or in `#worksheet(hidden: true)[…]` the math it calculates – its text, headings and prose equations (`$E = m c^2$`) stay. Problems are still shown. Nested worksheets don't inherit it, so a `#worksheet($t =$)` inside shows its equation again.

```typ
#import units: V, Ω, mA
#worksheet(hidden: true)[
  The values for the example:
  $U := 5 #V$, $R := 100 #Ω$, $I := U/R$
  The current is #worksheet($I = mA$).
]
```

**Scopes.** `scope` says what a worksheet sees and keeps: `"isolated"` starts empty and its definitions end with it; `"inherit"` sees the definitions made before, and its own end with it; `"shared"` sees them, and what it defines stays afterwards, replacing names of the same name (variables, functions and solved values alike). `auto`: isolated for content (`#worksheet[…]`, `#show: worksheet`), shared for equations passed to `worksheet`. Typst makes a content block holding only one equation the same value as that equation, so `#worksheet[$x := 1$]` is shared; write `scope: "isolated"` to keep it local. Nested worksheets don't inherit the scope. Clearing never reaches outside: `clear`, `clear-at` and `clear(all: true)` don't delete the outer definitions. Units, constants and operations aren't definitions: they are bindings, the same in every worksheet where they're visible.

```typ
#import units: V, Ω, mA
#show: worksheet
$U := 12 #V$
#worksheet(scope: "shared")[
  $R := 100 #Ω$
  $I := U/R = mA$
]
$I = mA$ // 120 mA, kept from the block
#worksheet($U := 5 #V$, scope: "inherit") // local to this call
$U =$ // still 12 V
```

**Editors.** Each equation the worksheet sees leaves a `metadata` labelled `<solveng>` with its `location()`, whether it was calculated and whether math is evaluated there; each problem one with `kind: "error"`, its `message`, the `element` it's in (`none` for equations passed to `worksheet`) and the `source` statement; at the end (and before each match of `clear-at`) one lists everything defined, with values. Editors like Typewright read them after compiling: `query(<solveng>)`.

**Starting over.** `clear-at` clears every definition at each match; `#clear(all: true)` inside a worksheet does the same once: back to how it started (with `scope: "inherit"` or `"shared"` the outer definitions stay), and `#clear("x", "f")` forgets only those names. After a `#worksheet[…]` the outer definitions come back; with `scope: "shared"`, its final definitions are merged over them.

```typ
#show: worksheet.with(clear-at: heading)                    // every heading
#show: worksheet.with(clear-at: heading.where(level: 1))    // only top-level ones
#show: worksheet.with(clear-at: <exercise>)                 // every element labelled, = Task <exercise>
#show: worksheet.with(clear-at: heading, scope: "inherit")  // keep the definitions made before
```

**Options.** Every option defaults to `auto`, which keeps the current setting, so only what you pass changes. For the whole document: `#show: worksheet.with(digits: 3)`; for a part: `#worksheet(evaluate: false, digits: 2)[…]`; for equations or content it calculates: `#worksheet($R := U/I =$, digits: 2)`; or keep them in a dictionary: `#let opts = (digits: 2, separator: ",")`, then `#worksheet($P := U I =$, ..opts)`. `defined` and `graph` take the same options. Unknown options and values of the wrong type are shown as an error in the document.

```typ
#import units: V, mA
#show: worksheet.with(digits: 3, form: "fraction")   // your defaults, the whole document
$U := 5 #V$, $I := 30 mA$
#worksheet($I/3 =$, digits: 2)                        // only this equation
#worksheet(evaluate: false, multiplication: "times")[Prose and $E = m c^2$ stay as written.]
```

`form`:

- `"auto"`: plain numbers as typed (`7.20 kΩ` keeps its zero), arithmetic evaluated (`3450/2 mAh` → 1725 mAh), results rounded to `digits`.
- `"decimal"`: everything rounded to `digits`, including what you typed.
- `"fraction"`: exact fractions, simplified (`$frac(2, 4) #A$` → ½ A, `I/3` → ⅙ A); values that are no fraction with denominator ≤ `max-denominator` fall back to decimals.
- `"input"`: given values exactly as typed (`3450/2 mAh` stays a fraction).

An explicit unit (`$I = mA$`) always wins, then `preferred-units`, then the non-SI unit of the inputs (h, mAh, bar), then the SI unit with a prefix.

To reuse your defaults in many documents, keep a small file of your own:

```typ
// my-defaults.typ
#import "@preview/solveng:1.0.0": *
#let setup = worksheet.with(digits: 3, multiplication: "times", constants: (imaginary: $j$))

// in a document
#import "my-defaults.typ": *
#show: setup
```

| parameter | type | default | |
|---|---|---|---|
| `..body` | content |  | The content, or equations to calculate. |
| `scope` | auto, str | `auto` | What the worksheet sees and keeps: `"isolated"` (starts empty, its definitions end with it), `"inherit"` (sees the definitions made before, its own end with it) or `"shared"` (sees them, and its definitions stay afterwards, over the outer ones). `auto` (default): isolated for content, shared for equations. |
| `clear-at` | none, function, selector, label | `none` | Clear every definition at each match: `heading` (every heading), `heading.where(level: n)` (headings of level n) or a label like `<exercise>` (every element with it, e.g. `= Task <exercise>`): the definitions made since are forgotten, like `#clear(all: true)`. |
| `hidden` | bool | `false` | Calculate without showing what is calculated (only problems are shown): the equations, or the math of the content. Not inherited by nested worksheets. |
| `evaluate` | bool | `true` | Calculate the math in the content; `false`: only its `#worksheet(…)` calls are. Not inherited by nested worksheets, and not for equations, which are always calculated. |
| `digits`, `separator`, … | auto | `auto` | Any [option](#options) for this call; `auto` keeps the current setting. |

Returns content.

## `base-unit`

A unit of a new, independent dimension: `#let bit = base-unit("information")`.

The identity names the dimension: base units with the same identity are the same dimension, also when
they're declared in other files or worksheets. Results of the dimension are shown in the unit.

`display` is how the unit is shown, in the input and in results: a string (set upright), content, or a
function returning content.

```typ
#let bit = base-unit("information", display: "bit")
#let byte = unit($8 bit$, display: $"B"$)
#show: worksheet
$d := 3 byte = bit$
```

| parameter | type | default | |
|---|---|---|---|
| `identity` | str |  | The dimension, the same in every file and worksheet. |
| `display` | auto, str, content, function | `auto` | How the unit is shown: a string (upright), content, or a function |

Returns content.

## `unit`

A unit defined by an expression of unit values: `#let minute = unit($60 #s$)`.

The expression is the unit's value: a positive number times units, from `units`, `base-unit` or other
`unit`s (`$100 kPa$`, `$1/min$`, `$#m / #s$`). Like every binding, the unit holds where the binding is
visible, in every worksheet.

`display` says how the unit is shown, in the input and in results, not what it means: a string (set upright),
content, or a function returning content. Binding names keep their spelling (a unit named `tau` isn't shown
as τ). Without a display the unit shows its expression, written as results are (numbers in the worksheet's
format, a thin space before units): `unit($2.54 #cm$)` is 2.54 cm. That display is one unit wherever it
appears: after a number or another factor it is joined by a dot, `$1 #inch$` is 1 · 2.54 cm, and under a
power or after a slash it is parenthesized, (2.54 cm)², m/(2.54 cm).

```typ
#import units: kPa, min, cm
#let bar = unit($100 kPa$, display: "bar")
#let rpm = unit($1/min$, display: $"rpm"$)
#let inch = unit($2.54 #cm$)
#show: worksheet
$p := 3 bar$, $n := 3000 rpm$, $p = kPa$, $l := 2 #inch = inch$
```

| parameter | type | default | |
|---|---|---|---|
| `expression` | content |  | The unit's value as math. |
| `display` | auto, str, content, function | `auto` | How the unit is shown: a string (upright), content, or a function |

Returns content.

## `constant`

Defines a constant: an identity and the math expression it stands for, which can carry units.

The identity says what the constant is and how it is shown: worksheets show it as their option `constants`
says, `#show: worksheet.with(constants: (imaginary: $j$))`, with built-in displays for `"pi"`, `"euler"`,
`"imaginary"` and `"speed-of-light"`; an identity without a display is shown as its expression. The engine
calculates exactly with the identities it knows – `"pi"` (π), `"euler"` (e) and `"imaginary"` (i) – once the
expression has that value: `$#calc.pi$`, `$sqrt(-1)$`. Any other identity is kept by it in exact results
(`$E := m #c^2 ->$` is m c²) and calculated with its expression, which must have a value. `substitute` puts
the expression in: `$E := m #c^2 -> substitute(#c)$`, and a constant alone shows it as written,
`$#i -> substitute(#i)$` is √−1.

```typ
#import units: m, s, kg
#let c0 = constant("speed-of-light", $299792458 #m / #s$)
#let g0 = constant("standard-gravity", $9.80665 #m / #s^2$)
#show: worksheet.with(constants: ("standard-gravity": $g_0$))
$F := 2 kg #g0 =$
```

| parameter | type | default | |
|---|---|---|---|
| `identity` | str |  | What the constant is, the same in every file and worksheet. |
| `expression` | content |  | Its value as math: numbers, units and other constants. |

Returns content.

## `operation`

An operation under a name of your choice: `#let conv = operation("convolve")`, then `$conv(x, h)$`.

The name selects one of solveng's operations (see *Built-in operations*), whatever the binding is called;
quoted names in math never do. Unlike `math.op("tri")`, the result takes named arguments, as Typst only
parses them for functions: `$conv(x, h, step: 2)$`.

`display` is how the operation is shown, in the input and in results: a string (an operator name, as
`math.op` writes it), content, or a function returning content.

```typ
#let triangle = operation("tri", display: "triangle")
#let step = operation("sgn", display: $op(sigma)$)
#show: worksheet
$triangle(1/2) =$, $step(-2) =$
```

| parameter | type | default | |
|---|---|---|---|
| `name` | str |  | The operation, e.g. `"tri"` or `"convolve"`. |
| `display` | auto, str, content, function | `auto` | How the operation is shown: an operator name, content, or a function |

Returns function.

## `solve`

Solves an equation for these names (and defines them): `$x^2 - 4 = 0 -> solve(x)$`.

| parameter | type | default | |
|---|---|---|---|
| `..names` | content |  | What to solve for. |

Returns content.

## `substitute`

The result with these variables' formulas and constants' expressions put in, every other name kept:
`$P := U^2/R -> substitute(U)$`, `$E := m #c^2 -> substitute(#c)$`. A constant alone shows its expression
as written: `$#i -> substitute(#i)$` is √−1, where a simplified result would be i again.

| parameter | type | default | |
|---|---|---|---|
| `..names` | content |  | The variables and constants whose formulas to put in. |

Returns content.

## `simplify`

The symbolic result, what `->` shows anyway: `$f := (x^2 - 1)/(x - 1) -> simplify$`.

## `digits`

Decimals with n significant digits: `$t = #h, digits(2)$`.

| parameter | type | default | |
|---|---|---|---|
| `n` | int, content |  | The digits, 1…15. |

Returns content.

## `sci`

Scientific notation: `$C = sci$` → 2.2 · 10⁻⁶ F.

## `eng`

Engineering notation, exponents in steps of 3: `$f = eng$`.

## `fraction`

Exact fractions: `$I / 3 = fraction$` → ⅙ A.

## `defined`

Lists the definitions made so far, in order.

```typ
#import units: kΩ, V
#show: worksheet
$R := 7.2 kΩ$, $U := 5 #V$, $I := U/R$, $f(x) := x^2$
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
| `digits`, `separator`, `form`, … | auto | `auto` | Every option of `worksheet` works here too, for this call only. See [Options](#options). |
| `digits`, `separator`, … | auto | `auto` | Any [option](#options) for this call; `auto` keeps the current setting. |

Returns content.

## `graph`

Plots an equation: ranges, nice ticks, axis labels and units are picked automatically.

Math holds the curves, separated by a backslash: `$cos(w) \ w$`, or a function such as
`$P(U: #V) = U^2/R$`. A parameter annotation supplies the axis unit; `..` takes the one undefined variable.
With `:=`, curves also define their names for later calculations. Curves share the same x axis.

```typ
#import units: Ω, W, V
#worksheet($R := 50 #Ω$, $P_0 := 2 #W$, hidden: true)
#graph($P(U: #V) = U^2/R$, ymark: $P_0$)
#graph($cos(w) \ w$, mark: auto)
#graph($y(x) = x^2 \ y(x) = 2x + 3$, xrange: (-3, 5))
```

Drawn with `@preview/lilaq:0.6.0` in the schoolbook style: axes through the origin with arrows, a faint grid at the regular ticks. Ticks use the document's decimal separator, axis labels read `U / V`; curves of the same unit share one graph, with a legend in the corner the curves leave emptiest. In HTML export the graph is drawn in the browser instead, in the same style: the curves draw themselves in when the graph comes into view, and the pointer shows the values of every curve with their units. The static picture stays in the page for readers without scripts.

**Ranges:** values and expressions in `xrange` / `yrange` must have the axis unit (`$f(20) #Ω$` is fine) – `xrange: ($10 #Ω$, $f(20)$)` with a unitless `f(20)` is an error, since it isn't clear what was meant. Without a range: around the marks; else around what's interesting – roots, extrema, poles, edges of the domain (√, ln) and, above all, where curves cross – rounded to nice ticks, starting at 0 when everything interesting is positive. With `ymark`, the y range is around the marked values (steep curves leave the picture). **Units** follow from what's plotted: y's from the curves (shown in the SI unit with a fitting prefix; `preferred-units` picks others), x's from the curve's parameter (`U(I: mA) = …`) or a function x is passed to whose parameter has one (`$q(I: mA) := I R$`, then `#graph($U(I) = q(I)$)` is over I in mA), else from the units in `xrange` and the marks, else from the curve and the units of the y values (`P(..) = U_0^2/R_2` with `ymark: $2 #W$`: R₂ in Ω).

**Data:** a vector of values is plotted as the points through its elements, joined in index order (with a dot at each when there are at most 30): `#graph($y$)` against the zero-based index k (as in `at(y, k)`), `#graph($y(τ)$)` over the vector `tau` of the same length – or over any expression without free names that is one, `y(2 tau)`. Units come from the elements (each vector needs one unit for all its elements; every element must have a value). Function curves can go alongside: `#graph($y(τ) \ c(τ)$)` with a function `c`, or an expression in `tau` (`$2 tau$`), where `tau` stands for the x axis and the function is sampled over it; the x range is from the data's first to last x value unless `xrange` says otherwise, and the units of x and y must agree. Marks work on data too, with linear interpolation between the points: x values, `"roots"` and `"crossings"` (with data and with functions) where the values change sign, `"extrema"` as the local extreme points, and `ymark`. A vector with a free name is several curves instead, one per element (`$vec(t, t^2)$`). For measurements, `lines: none` draws the data as points alone, without the line: `#graph($y(τ) \ c(τ)$, lines: (none, auto))` shows measured points next to the curve `c`.

**Piecewise functions** (`cases`) break their line where a branch changes with a jump; a value that holds at a single x – `2 & "for" x = 1`, or a point set apart from the line – is drawn as a dot. `joined: true` joins the pieces through them instead.

**Layers:** curves are drawn in the order written, each over the ones before and over the axes: a function written after measured points runs over them, one written before runs under them. Every dot has a thin ring of the page's color, so dots stand apart from lines and from each other; a dot right on a dot of another curve is drawn smaller, the one below showing as a ring around it.

**Marks** default to none. Requested marks are always visible – the ranges grow to show them. Combine them in an array, `(auto, $R_1$)`.

**Size:** `width` / `height` are the whole graph, labels included; `auto` makes the plot area 8 × 5 cm. Any other named argument goes straight to lilaq's `diagram` (e.g. `title: [Load]`, `grid: none`); it shapes the printed graph, not the one drawn in HTML. For full control, plot with lilaq directly.

| parameter | type | default | |
|---|---|---|---|
| `equation` | content |  | Math curves separated by a backslash. |
| `..args` | any |  | Named options passed on to lilaq's `diagram` (e.g. `title: [Load]`). |
| `xrange` | auto, array, content, int, float | `auto` | x values that must be visible: numbers in the x axis's unit (`(1, 2)`), values (`($10 #Ω$, $1 kΩ$)`) or expressions of defined variables (`(0, $2 R$)`). Two or more give the range from the smallest to the largest; a single one widens the picked range. `auto`: around the marked x values, else picked from the curves. |
| `yrange` | auto, array, content, int, float | `auto` | y values that must be visible, like `xrange` (numbers in the y axis's SI unit). Two or more give the exact y range. `auto`: fitted to the curves. |
| `mark` | none, auto, array, content, int, float | `none` | What to mark, with both coordinates written on the axes: x values like `xrange` (`$R$`, `2`, `$5 #V$`; a point with guide lines on every curve, the range grows to show it), `"crossings"`, `"roots"`, `"extrema"` (searched within the range) and `auto`: where two curves cross. An array combines them: `(auto, $R_1$)`. `()` or `none` (default): no marks. |
| `xmark` | none, array, str, content, int, float | `()` | Like `mark`, but only the x value is written on the axis. |
| `ymark` | none, array, content, int, float | `()` | y values to mark, like `yrange`: every point where a curve reaches one, with guide lines; the value is written on the y axis. |
| `samples` | int | `400` | Number of sample points. |
| `width` | auto, length, ratio, relative, fraction | `auto` | Width of the whole graph, labels included: `12cm`, `300pt` or a share like `100%`. `1fr` is the container's width – in a grid, the column's. `auto`: a plot area of 8 cm. |
| `height` | auto, length, ratio, relative, fraction | `auto` | Like `width`; `auto`: a plot area of 5 cm. A share is of the container's height, or of the width when the page grows with its content. |
| `colors` | auto, array, color | `auto` | Colors of the curves, curve after curve and repeating: `(gray, blue, rgb("#f98eff"))`. `auto` keeps the worksheet's `graph-colors`, also for a single curve: `(auto, red)` changes only the second (and fourth, …). By default Petroff's ten colors, chosen to stay apart for color-blind readers (lilaq's default too). `lines`, `dots` and `joined` work the same way. |
| `lines` | auto, array, str, dictionary, none | `auto` | The curves' lines, like `colors`: Typst dash patterns such as `"solid"`, `"dashed"`, `"dotted"`, `"dash-dotted"`, `"densely-dashed"`, `"loosely-dotted"`, or an array of lengths like `(4pt, 2pt)`, so curves can be told apart without color (color-blind readers, grayscale prints); `none` draws no line – data as points alone (a scatter plot), a function as its single points. `auto`: the worksheet's `graph-lines`, by default `("solid", "dashed", "dotted", "dash-dotted")`, the order of scientific plotting tools. |
| `dots` | auto, array, str, none | `auto` | The curves' dots, like `colors`: `"circle"`, `"square"`, `"triangle"`, `"triangle-down"`, `"diamond"`, `"pentagon"`, `"hexagon"`, `"star"`, `"cross"`, `"plus"`; `none` draws no dots. `auto`: the worksheet's `graph-dots`, by default `("circle", "square", "triangle", "diamond")`, so curves can be told apart without color too. Dots are drawn for data's points (with at most 30 points, or without a line) and a function's single points (`2 & "for" x = 1` in `cases`). |
| `joined` | auto, bool, array | `auto` | Whether everything a curve draws is joined with a line, like `colors`: a function's pieces, from the end of one to its single point and on to the start of the next, as data's points are (with `lines: none`: data's dots joined). Joins to points beyond the graph leave it; the pointer doesn't read them. `auto`: the worksheet's `graph-joined`, by default `false`. A curve with nothing to draw is an error. |
| `digits`, `separator`, `form`, … | auto | `auto` | Every option of `worksheet` works here too, for this call only. See [Options](#options). |

Returns content.

## `lambdify`

Turns an expression into a plain Typst function of numbers, for value tables,
other plotting packages (lilaq, cetz-plot, simple-plot) or your own calculations.
Defined variables and functions are used with their current values. Needs `context`.

```typ
#import units: Ω, W, V
#worksheet($R := 50 #Ω$, hidden: true)
#context {
  let P = lambdify($U^2/R$, "U", unit: $#W$, units: (U: $#V$))
  table(columns: 2, ..range(1, 6).map(u => ([#u V], [#P(u) W])).flatten())
}
```

The function takes one argument per parameter: a number (in the unit given in
`units`, else in the SI unit) or a math quantity like `$5 #V$`. It returns a float in
`unit` (else in the SI unit), or `none` where the expression isn't defined
(`sqrt(-1)`, division by zero).

Unlike the other functions, problems stop the compilation here, since the
result is a function and not content.

| parameter | type | default | |
|---|---|---|---|
| `expr` | content |  | The expression, using defined variables and the parameters: `$U^2 / R$`. |
| `..params` | str |  | The parameters in order, e.g. `"U"` or `"U", "R"`. A defined variable as a parameter is replaced by the argument. Without any: the undefined variables of the expression, in the order they appear. |
| `unit` | none, content | `none` | Unit of the result as math, e.g. `$#W$` or `$kW$`. Checked against the expression when all parameters have units. |
| `units` | dictionary | `(:)` | Units of numeric arguments as math, e.g. `(U: $#V$, R: $kΩ$)`. |

Returns function.

## `value`

The result of an expression alone, where the text needs it: what `$U =$` shows
after its `=`. Calculated with the worksheet's definitions at this point, which
it leaves as they are.

```typ
#show: worksheet
#import units: kΩ, mA, mV, Ω
$R := 4.7 kΩ$, $I := 2 mA$, $U := R I$, $f(x: #Ω) := 2 x$
The voltage is #value($U$) (#value($U = mV$)), and #value($f(3 #Ω)$) for 3 Ω.
```

A clause picks the unit or form as after `=`: `#value($U = mV, digits(2)$)`; a
trailing `->` shows the symbolic result. A function's name alone shows its body.
It shows one expression and defines nothing: definitions go in the worksheet.

| parameter | type | default | |
|---|---|---|---|
| `expr` | content |  | The expression as math: `$U$`, `$f(2)$` or `$U / 2 = mV$`. |

Returns content.

## `clear`

Forgets definitions: the names given, or with `all: true` every one made so far.

A value and a function share their name – defining one replaces the other – so `clear("f")` forgets `f` either way (`"f()"` and `"f(x)"` are read the same). Names are written as strings (`"Q_0"`, `"tau"`) or in math (`$Q_0$`). A name that isn't defined is shown as a problem. With `all: true` inside a `worksheet` it's back to how the worksheet started (with `scope: "inherit"` the outer definitions stay). Units, constants and operations are bindings, not definitions: `clear` leaves them alone.

```typ
#import units: mA, A, V
#show: worksheet
$I := 30 mA$, $f(x) := 2 x$, $U := 5 #V$
#clear("I", $f$)
$I := 1 #A$
#clear(all: true)
$U := 12 #V$
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

## `Re`

Real part of a complex number: `Re(3 + 4 #i)` is 3. Keeps the unit: `Re(Z)` of an impedance is its
resistance. Real numbers are their own real part.

| parameter | type | default | |
|---|---|---|---|
| `z` | expression |  | A complex or real value. |

Returns value.

## `Im`

Imaginary part of a complex number: `Im(3 + 4 #i)` is 4, a real number. Keeps the unit: `Im(Z)` of an
impedance is its reactance. Real numbers have the imaginary part 0.

| parameter | type | default | |
|---|---|---|---|
| `z` | expression |  | A complex or real value. |

Returns value.

## `conj`

Complex conjugate: `conj(3 + 4 #i)` is 3 − 4i, shown with a bar. Keeps the unit.

The magnitude is `abs(z)` and the angle is Typst's own `arg(z)`, in radians from −π to π; both need no
import.

| parameter | type | default | |
|---|---|---|---|
| `z` | expression |  | A complex or real value. |

Returns value.

## `number`

A value as a plain number in a unit: `number(U, mV)` is U in millivolts, without a unit.

The value must have the unit's dimension – `number(U, #A)` of a voltage is an error – so the number always
means what it says. Dividing by the unit, `U / mV`, gives the same number but lets any unit through; use
`number` where a formula needs a value in a given unit, like an empirical one:
`$F(d) := 3.2 number(d, mm)^0.6 #N$`.

| parameter | type | default | |
|---|---|---|---|
| `value` | expression |  | The value, with a unit. |
| `unit` | expression |  | The unit to express it in: a unit binding or an expression of them (`#m / #s`). |

Returns number.

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

In math, Leibniz's notation does the same: `$(dif x^3)/(dif x) ->$`, `$dif/(dif x) x^3 ->$`,
higher orders `$(dif^2 f(x))/(dif x^2)$` and partial derivatives `$(partial^2 f(x, y))/(partial x partial y)$`.
`dif/(dif x)` applies to the term after it: `$dif/(dif x) x^2 + x ->$` is 2x + x. A worksheet shows
`diff(f, x)` in Leibniz's notation, d/dx f.

| parameter | type | default | |
|---|---|---|---|
| `expression` | expression |  | Expression to differentiate. |
| `variable` | name |  | Differentiation variable, written as a name rather than a value. |

Returns expression.

## `integrate`

Integral: `integrate(expression, variable)` is the antiderivative, `integrate(expression, variable, lower, upper)`
the definite integral.

`$integrate(x^2, x, 0, 3) =$` gives 9, `$integrate(x^2, x) ->$` gives x³/3. The variable is local to the
operation.

Without bounds the result is an antiderivative, without a constant of integration: `$integral #e^x dif x ->$`
gives eˣ, `$integral x^2 sin(x) dif x ->$` gives −x² cos(x) + 2x sin(x) + 2 cos(x). It is found for sums of
constants times powers, exponentials, sin, cos, sinh, cosh, ln and tan of a linear argument; for products that
multiply out into such terms; and by parts for a polynomial times one exponential, sin, cos, sinh or cosh of a
linear argument. Other integrands (`sin(x^2)`, `#e^x cos(x)`, rational functions other than 1/(a x + b))
report an error.

With bounds, the bounds must have matching units (0 and ±∞ match any); the result has the integrand's unit
times the variable's unit. Either bound may be infinite. Recognized antiderivatives simplify exactly where they
are continuous between the bounds; remaining integrals stay symbolic with `->` and use adaptive quadrature for
`=` or graphs. Affine signal arguments are split at their corners so narrow pulses are not skipped. Undefined
integrands or failure to converge report an error.

In math, `$integral_0^3 x^2 dif x =$` and `$integral x^2 dif x ->$` do the same. The differential (`dif`, not
`diff`, which is ∂) ends the integrand wherever it stands: `$integral_0^1 x^2 + 1 dif x$` integrates x² + 1,
`$integral_0^1 (x dif x) + 1$` adds 1 to the integral, `$integral_0^1 (dif x)/(1 + x^2)$` and
`$integral_0^1 dif x$` are integrals too. A bound of 0 or ±∞ takes the other bound's unit
(`$integral_0^(2 #s) 3 #A dif t$`); when both are, write the unit on one:
`$integral_(0 #s)^infinity #e^(-t/tau) dif t$`. A worksheet shows `integrate(…)` as the integral sign.

| parameter | type | default | |
|---|---|---|---|
| `expression` | expression |  | Integrand. |
| `variable` | name |  | Integration variable, written as a name rather than a value. |
| `lower` | expression |  | Lower bound, including `-infinity`; none for an antiderivative. |
| `upper` | expression |  | Upper bound, including `infinity`; none for an antiderivative. |

Returns expression or value.

## `laplace`

Unilateral Laplace transform: `laplace(expression, variable, result-variable)`.

The variables default to t and s: `$laplace(t^2) ->$`, `$F(p) := laplace(exp(-x), x, p)$`.
(The keyword form `-> laplace` was removed; it's an error pointing here.)
A worksheet shows the call as ℒ{f}, with the result variable as ℒ{f}(p) when it's given; `invlaplace` as ℒ⁻¹{F}.
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
`$range(0 #s, 1 #s, step: 250 ms)$`. The bounds and the step need values when the range is defined. A
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

The argument is unitless; outside [−1, 1] the result is the principal complex value (`asin(2)` is
1.571 − 1.317i). `$asin(1) =$` gives π/2.

| parameter | type | default | |
|---|---|---|---|
| `x` | expression |  | Unitless sine value. |

Returns unitless angle in radians.

## `acos`

Inverse cosine, also spelled `arccos`: `acos(x)` returns an angle in radians from 0 to π.

The argument is unitless; outside [−1, 1] the result is the principal complex value.
`$acos(1) =$` gives 0.

| parameter | type | default | |
|---|---|---|---|
| `x` | expression |  | Unitless cosine value. |

Returns unitless angle in radians.

## `atan`

Inverse tangent, also spelled `arctan`: `atan(x)` returns an angle in radians between −π/2 and π/2.

The argument and result are unitless. `$atan(1) =$` gives π/4.

| parameter | type | default | |
|---|---|---|---|
| `x` | expression |  | Unitless tangent value. |

Returns unitless angle in radians.

## Options

Options of `worksheet`, and of `defined` and `graph`. Each one defaults to `auto`, which keeps the current setting.

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
| `preferred-units` | auto, content, array | `()` | Units for results and graph axes of their dimension, as math expressions of unit values: `($#m / #s$, $kWh$)`, before input units and automatic SI prefixes. |

### Output

| option | type | default | |
|---|---|---|---|
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

### Types (f(x

| option | type | default | |
|---|---|---|---|
| `show-types` | auto, str | `"chip"` | How types (`f(I: mA)`) are shown in the equations as written: `"chip"` (default): as a blue chip · `"plain"`: as written, f(I: mA) · `"no"`: left out (`$f(I: mA) := 2 I$` shows as f(I) ≔ 2 I). |

### How they look

| option | type | default | |
|---|---|---|---|

### Function drawing them, and units

| option | type | default | |
|---|---|---|---|
| `words` | auto, dictionary | `(any: "any")` | The word shown for any-unit types, for documents in other languages: `(any: "beliebig")`. What you type stays `any`. |
| `chips` | auto, dictionary, function | `auto` | The colours of the type chips, `(type: (fill: …, mark: …, ink: …))` (`mark` is the colour of `:`, `ink` the unit's), or a function `(kind, mark, unit) => content` drawing them, with `kind` `"type"` and `mark` and `unit` as content. |
| `unit-style` | auto, function | `auto` | A function given every shown unit as content, `u => text(fill: blue, u)`. |
| `unit-separator` | auto, content, str, symbol | `auto` | What goes between units, a thin dot by default (A·as): `sym.times`, `h(0.2em)`. |

### How constants are shown, by identity, over the built-in displays

| option | type | default | |
|---|---|---|---|
| `constants` | auto, dictionary | `(:)` | How constants are shown, by identity: `(imaginary: $j$, "standard-gravity": $g_0$)`, over the built-in displays of `pi` (π), `euler` (e), `imaginary` (i) and `speed-of-light` (c) and the ones of the worksheets around. Results show them the same way. See `constant`. |

### Graphs

| option | type | default | |
|---|---|---|---|

### Line styles of scientific plotting tools, and dot shapes that tell curves apart without color too

| option | type | default | |
|---|---|---|---|
| `graph-colors` | auto, color, array | `(rgb("#3f90da"), rgb("#ffa90e"), rgb("#bd1f01"), rgb("#94a4a2"), rgb("#832db6"), rgb("#a96b59"), rgb("#e76300"), rgb("#b9ac70"), rgb("#717581"), rgb("#92dadd"))` | The colors of the curves in graphs, curve after curve and repeating: `(gray, blue, rgb("#f98eff"))`. Default: Petroff's ten colors, chosen to stay apart for color-blind readers. `auto` keeps the worksheet's around, also for a single curve: `(auto, red)` changes only the second (and fourth, …). A graph's `colors` goes over it, the same way. |
| `graph-lines` | auto, str, array, dictionary, none | `("solid", "dashed", "dotted", "dash-dotted")` | The lines of the curves in graphs, like `graph-colors`: Typst dash patterns such as `"solid"`, `"dashed"`, `"dotted"`, `"dash-dotted"`, or lengths like `(4pt, 2pt)`; `none` draws a curve without lines. A graph's `lines` goes over it. |
| `graph-dots` | auto, str, array, none | `("circle", "square", "triangle", "diamond")` | The dots of the curves in graphs, like `graph-colors`: `"circle"`, `"square"`, `"triangle"`, `"triangle-down"`, `"diamond"`, `"pentagon"`, `"hexagon"`, `"star"`, `"cross"`, `"plus"`; `none` draws a curve without dots. A graph's `dots` goes over it. |
| `graph-joined` | auto, bool, array | `(false,)` | Whether everything a curve in a graph draws is joined with a line, like `graph-colors`. A graph's `joined` goes over it. |

## Modifiers

The clause after a trailing `=` or `->` takes a unit and modifiers, separated by commas: `$t = #h, digits(2)$`,
`$x^2 = 4 -> solve(x)$`. Modifiers are Typst bindings of solveng's `modifiers` module, and importing them is the
only way to use them: `#import modifiers: solve, simplify` (or `#import modifiers: *`). Like any import, it holds
only where it's written; `#import "@preview/solveng:1.0.0": *` brings none. Renamed imports
(`#import modifiers: solve as löse`) work as the original. `solve`, `substitute` and `digits` are functions,
called with what they take; `simplify`, `sci`, `eng` and `fraction` are written on their own.

```typ
#import "@preview/solveng:1.0.0": *
#import modifiers: solve, digits, sci
#import units: h, mA, mAh
#show: worksheet
$I := 30 mA$, $Q := 3450 mAh$
$Q/2 = I t -> solve(t)$
$t = #h, digits(3)$
$I = sci$
```

## Calculating like Mathcad

Three operators, and `=` keeps its meaning from maths:

- **`:=` defines**, symbolically. Names defined by then are put in right away, names that aren't stay names until they are: `$h := 5 z$` works before `z` is defined, and after `$z := 20 cm$`, `$h =$` shows 1 m. What's put in is the value at that point – defining a name again later changes what comes after, not what came before.
- **`=` inside an expression compares**, as Mathcad's bold =: `$M := 1 = 2 =$` gives 0, `$M := 2 #A = 2000 mA$` gives 1. Values of different units are never equal (`1 A = 2 V` is 0, `1 A != 2 V` is 1); `<`, `<=`, `>`, `>=` and `!=` compare too, chains like `$0 < x <= 1$` hold when every step does. `and` and `or` (Typst shows ∧ and ∨) join them, `and` binding tighter: `$x < 0 or x > 2$`, also as a condition in `cases`.
- **A trailing `=` shows the value**, numerically, with a bold =. After it come a unit and modifiers (from `modifiers`): `$t = #h, digits(3)$`. Every name in it must have a value by then.
- **A trailing `->` shows the symbolic result**: defined values put in, exactly (fractions, roots and π stay), undefined names kept: `$x + 3/3 y -> simplify$` is x + y. With `solve(x)` it solves – symbolically when it can (`$0 = 1 + 3/x + y^2 -> solve(x)$` is x = −3/(y² + 1)), numerically when every other name has a value – and defines x when the solution is a value. `substitute(U, …)` shows the result with these variables' formulas put in and every other name kept: after `$U := R I$`, `$P := U^2/R -> substitute(U)$` is I² R.
- **Any other `=`** states something: it's checked when everything in it has a value (`$U = R I$`), and left alone as prose when not (`$E = m c^2$`).

**Functions** are defined the same way: `$f(x) := x^2 + 1$`. Their body is symbolic too, so a function needn't define everything it uses: names that aren't parameters and aren't defined stay names, looked up when the function is evaluated, and cancel where the algebra lets them (`$U(x: #m) := h x a$`, then `$g(x: #m) := U(x)/a ->$` is `1 m · x` without any `a`). Undefined function calls stay symbolic too: `$f(t) := integral_(-infinity)^t sigma(tau) dif tau$` is valid before `sigma` is defined; numerical evaluation requires its definition and convergence. A parameter can have a unit, `f(I: mA)`: arguments must carry its dimension (`f(2 mA)` is valid, `f(2)` reports a missing unit); `any` takes every unit. `..` makes the body's other undefined names parameters, in the order they appear: `$P(..) := U^2/R$` is `P(U, R)`. A parameter passed on to one with a unit takes that unit: with `$q(x: #A) := 2 x$`, `$f(y) := q(y)$` makes y an A. **Named parameters** have a default value and are passed by name, as in Typst: after `$f(x, n: 2) := x^n$`, `$f(3) =$` is 9 and `$f(3, n: 3) =$` is 27; a named argument that isn't a parameter, a positional one passed by name and one passed twice are errors. After the colon, an expression made only of unit bindings is a type (`I: mA`, `v: #m / #s`), anything else a default (`n: 2`, `R: 2 #Ω`, `s: k`). Defaults are taken where the function is defined, as Typst evaluates them, and come after the other parameters.

```typ
#import units: cm, m, A, V
#show: worksheet
$h := 5 z$, $z := 20 cm$, $h =$             // h ≔ 5z, z ≔ 20 cm, h = 1 m
$U(x: #m) := h x a ->$                     // U(x) ≔ h x a → 1 m · a x
$g(x: #m) := U(x)/a ->$                    // g(x) ≔ U(x)/a → 1 m · x
$M := 1 #A = 2 #V ->$                      // M ≔ 1 A = 2 V → 0
$k := 2 #m = 200 cm =$                     // k = 1: the comparison holds
```

## Writing values

```typ
#import units: kΩ, A, V, m, mA, mAh, s
#import modifiers: substitute
#show: worksheet
$R := 4.7 kΩ$                                 // a number and a unit
$I := frac(2, 4) #A$, $Q_0 := 3450/2 mAh$     // arithmetic in the number
$U := (1+2) dot 1.5 #V$, $A := 2^3 #m^2$      // A the variable, #A the ampere
$U_2 := R dot 1 mA$
$Q := U_2 dot 2 #s -> substitute(U_2)$        // U_2's formula put in: Q ≔ U_2 · 2 s → 1/500 A·s·R
```

- **Units** are Typst bindings, imported from solveng's `units` module where you use them: `#import units: mA, kΩ, s`. It has the SI base units (m kg s A K mol cd), V Ω W J C F H S Hz N Pa T Wb, min h Ah Wh As Ws eV l L, rad, deg (°) and degC (°C, a temperature *difference*), with the SI prefixes (`mA`, `kΩ`, `µs`). In math a binding of one letter needs a `#` (`$2 #s$`, `$100 #Ω$`), longer ones don't (`$30 mA$`). Powers as `#m^2`, quotients as `#m / #s^2`, products with spaces, `kg #m`. Your own with `base-unit` and `unit`, see [Units](#units).
- **Numbers from values:** dividing by a unit gives a value's number in it, checked and prefix-aware: `U / #V` is U's number in volts, `(U / #V) #A` gives it another unit. Empirical formulas are written this way: `$F(d: mm) := 3.2 (d / mm)^0.6 #N$`. `number(U, #mV)` from `operations` is U's value in mV as a plain number, an error when U doesn't have mV's dimension.
- **Typeable spellings:** `Omega` is Ω, the prefix `u` is µ (`uA`, also `µA` and `μA`), `deg` and `degC` are ° and °C. Results show the proper symbols.
- **Decimal point:** numbers are written with a point, `$7.2 #V$`, also in German documents; results use the document language's separator (option `separator`), so they show 7,2 there. A comma always separates: `at(A, 0,1)` and `mat(2,4,6)` are lists of numbers.
- **No e-notation:** numbers are written out (`$11200$`, or `$1.12 dot 10^4$`). `1.12e4` is the coefficient 1.12 times the variable `e4` (undefined unless defined; quote it, `$"e4"$`, for a name of several letters). Scientific notation is only how results are written (option `scientific`, keyword `sci`).
- **Names:** the acute accent `´` and the backtick (`` ` ``) are literal name characters (`$x´ := 3$`), with no escaping needed. Digits continue a name: `t2` differs from `t_2` and keeps its digits on the baseline. Complete Latin letter runs normalize to Greek: `Delta2tau` → Δ2τ, while `Deltatau` stays unchanged. `""` joins name fragments: `Delta""tau` → Δτ and `t""2` → t2. Quotes only carry a name through Typst: `$"Delta2tau"$` names Δ2τ, and `$"Delta\"\"tau"$` names Δτ. Quoted text is always a name, also when it spells a unit or an operation: `$"mA" := 3$` is a variable. Explicit subscripts may be grouped: `$R_(a b)$` names R_ab. Operators and whitespace end a name; a leading number remains a coefficient (`2t2` is 2 times t2).
- **Sums:** `sum_(k=0)^10 k^2` or `sum(k^2, k, 0, 10)` sums over inclusive integer bounds. The index is local and case-sensitive. Parenthesize a summand containing addition. Small finite sums simplify exactly; infinite sums stay symbolic after `->` and are evaluated numerically for `=` and graphs. For example, `$s_2(t) := sum_(k=0)^infinity sigma(t-k+3/4)$` defines a function usable as `#graph($s_2(t)$)` once `sigma` is defined. Piecewise linear signal sums are counted directly between their corners. Other numerical sums use at most 100000 terms and a shrinking-tail convergence estimate; slow or nonconvergent series report an error. Bounds and the index are unitless, while summands may carry units.
- **Signals:** `rect(x)` is 1 for |x| < ½, ½ at ±½, otherwise 0; `tri(x)` is max(1 − |x|, 0); `sgn(x)` is −1, 0 or 1. Arguments are unitless. Import the names from `operations` (`#import operations: rect, tri, sgn`); a worksheet function such as `$Lambda(t) := tri(t)$` or `#let triangle = math.op("tri")` gives another name. Linear combinations simplify and can be solved; definite integrals split affine arguments at their boundaries. Distributional derivatives stay symbolic.
- **Ranges and elements:** `$i := 0..N$` (also `0 .. N`, `0...N`) is a range variable: the values from 0 to N in steps of 1, both included, counting down when N < 0; with units, use `range(start, end, step: …)`. `x[i]` and `A[i, j]` are elements, counted from 0, written right after the name without a space; a vector of indices selects several (`x[i]` with i a range is a vector). `:` selects a whole row or column: `A[i, :]` is row i (a matrix of one row), `A[:, j]` column j (a vector), both plottable as data: `#graph($A[1, :]$)`. `$x[i] := …$` defines an element for each value of the range variable i, the body calculated with i standing for that value: `$t[i] := i Delta""t$`, `$x[i] := tri(t[i])$`. Elements not defined yet are 0; defining one past the end extends the vector, as in Mathcad. `A[i, j] := …` defines a matrix.
- **Convolution:** `convolve(x, h)` of sequences and `convolve(f(τ), g(τ), τ, t)` of functions, see `convolve`. Sequences of values are convolved numerically (whole numbers exactly), so long sampled signals stay fast; functions exactly where they're piecewise polynomial.
- **Operators:** `+ - * / ^ · ×`, implicit multiplication (`2 pi f C`), Typst's math operators `sqrt exp ln log sin cos tan arcsin arccos arctan sinh cosh tanh abs arg` (no import needed), the other operations from `operations` (see [Built-in operations](#built-in-operations)), your own functions.
- **Constants** come from solveng's `constants` module: `#import constants: pi, e, i, c` (π, Euler's e, the imaginary unit, the speed of light), then `$2 pi$`, `$#e^x$`, `$3 + 4 #i$`. Without the import `pi` is only the symbol π, a name without a value. `constant(identity, expression)` makes your own; the option `constants` says how they are shown.

## Complex numbers

Values are complex numbers: `$sqrt(-4) =$` is 2i, `$ln(-1) ->$` is iπ, also without `i` written. Operations keep their real value where they have one, so `(-8)^(1/3)` is −2; elsewhere they take their principal value. `#import constants: i` brings the imaginary unit to write them, and results show it whether imported or not.

- **Exact** numbers stay exact: `(1 + 2i)(3 − i)` → 5 + 5i, `e^(iπ)` → −1; sin and cos are exact at multiples of 30° and 45°.
- **Units:** complex values carry units like any other, `(100 + 94.25i) Ω`.
- **Parts:** `abs(z)` is the magnitude, `arg(z)` the angle in radians (−π…π); `Re`, `Im` and `conj` come from `operations` (`#import operations: Re, Im, conj`).
- **Solving** is complex: `$x^2 + 4 = 0 -> solve(x)$` gives ±2i, polynomials with numbers get all their roots (`$x^3 = 8 -> solve(x)$`: 2 and −1 ± 1.732i). The numeric search for other equations finds real solutions only.
- **Showing:** electrical engineers write `j`: `#show: worksheet.with(constants: (imaginary: $j$))`.
- **Real only:** graphs, `lambdify`, comparisons with `<` and `>`, inequalities and the numeric search need real values; a complex one there is an error.

```typ
#import units: Ω, Hz, mH
#import constants: pi, e, i
#import operations: conj
#import modifiers: solve
#show: worksheet.with(constants: (imaginary: $j$))
$(1 + 2 #i)(3 - #i) ->$                       // 5 + 5j
$#e^(#i pi) ->$, $sin(pi/4) ->$               // −1, √2/2
$Z := 100 #Ω + #i 2 pi 50 Hz dot 300 mH =$    // (100 + 94.25j) Ω
$abs(Z) =$, $arg(Z) =$, $conj(Z) =$           // 137.4 Ω, 0.7558, (100 − 94.25j) Ω
$x^2 - 2 #i x - 2 = 0 -> solve(x)$            // x₁ = −1 + j, x₂ = 1 + j
```

## Units

Units, constants, operations and modifiers are Typst bindings, so solveng never guesses what a name means: in math `A` is a variable and, after `#import units: A`, `#A` is the ampere; quoted text (`$"mA"$`) is always a variable. Imports are lexical: they hold in the file or block where they're written and the blocks nested in it, whatever worksheet is around, and nothing comes along with `#import "@preview/solveng:1.0.0": *`. A unit binding hides a Typst function of the same name in code: after `#import units: h`, spacing is `std.h(1em)`.

After a trailing `=` or `->`, the unit to show is an expression of unit values too: `$t = #h$`, `$v -> km/#h$`, `$P = mW, digits(2)$`. Without one, results are shown in the unit of the option `preferred-units` (`preferred-units: ($#m / #s$,)`), else in a unit of the inputs, else in SI units with a prefix.

```typ
#import units: Hz, ms, A, s
#show: worksheet
$A := 12$, $x := A dot 2 #A =$        // x ≔ A · 2 A = 24 A
$T := 2 #s$, $f := 1/T = Hz$          // f ≔ 1/T = 0.5 Hz
$T -> ms$                             // 2000 ms
```

**Units of your own.** `base-unit(identity, display: …)` creates a unit of a new, independent dimension; base units with the same identity are the same dimension, also in other files and worksheets. `unit(expression, display: …)` defines a unit as an expression of unit values with a positive scale: `unit($60 #s$)` is a minute. `display` is a function returning the unit's content; it changes how the unit is shown, not what it means. Like every binding, they hold wherever the binding is visible, in every worksheet.

```typ
#import units: kPa, min
#let bit = base-unit("information", display: () => math.upright("bit"))
#let byte = unit($8 bit$, display: () => math.upright("B"))
#let bar = unit($100 kPa$, display: () => math.upright("bar"))
#let rpm = unit($1/min$, display: () => math.upright("rpm"))
#show: worksheet
$d := 3 byte = bit$, $p := 3 bar$, $n := 3000 rpm$
#worksheet(preferred-units: ($bar$,), scope: "inherit")[$p/2 =$]   // results of this dimension in bar
```

## How it solves

solveng calculates in a WebAssembly plugin written in Rust (`core/` in the repository). Equations are solved symbolically first, other names as parameters: an unknown that occurs once is isolated; an equation that is a polynomial in it, once both sides are over one denominator, is solved by formula up to degree 2 and for all real roots numerically above; a system is solved one equation at a time where an equation has a single unknown, and by elimination where it's linear. Equations that contain `i` are solved in the complex numbers, the others in the real numbers. Results are simplified: like terms combined, fractions cancelled (`(x^2 - 1)/(x - 1)` is x + 1), exact numbers kept exact. What can't be done symbolically is solved numerically, once every other name has a value: sign changes over ±10⁻¹⁵…10¹⁵ (times the target unit) are bisected and Newton starts from every local minimum of the absolute value; systems by damped Newton from 41 starts. Units of unknowns are inferred from the equations; mismatched units stop with an error.

A solved definition keeps its solutions: `circle(x, r) := x^2 + b^2 = r^2 -> solve(b)` returns a vector of both branches. A relation function can supply the equation: `p(a, b, c) := a^2 + b^2 = c^2`, then `circle(x, r) := p(x, b, r) -> solve(b)`. The solve variable is local, not a parameter. One solution returns a scalar; several return a vector, with `at(vector, 0)` selecting its first component. Graphing a vector plots every branch. Solved definitions require a symbolic solution; equations in `vec(...)` with several solve variables return components in the requested order, nested by branch when needed.

## Limits

- **Numbers** are exact fractions as long as they fit (128-bit numerator and denominator), else 64-bit floats: about 15 significant digits, magnitudes up to 10³⁰⁰.
- **Isolating** `sqrt(x) = 2` gives only the positive root, periodic functions only the principal value (`tan(x) = 1` → `arctan(1)`); polynomials like `x^2 = 4` give all their roots.
- **Symbolically**, polynomials are solved up to degree 2; higher degrees need values for their coefficients. Simplifying cancels common factors of polynomials (with exact coefficients) but doesn't factor or use trigonometric identities.
- **Numeric search** can miss roots squeezed between nearby poles, roots of very fast oscillations, and roots outside ±10¹⁵ (scaled by the target unit); the result then says "no real solution".
- **Systems** are solved numerically from a fixed set of starts; solutions far away from them can be missed. Underdetermined systems show one arbitrary solution.
- **Domains:** values are complex, with the real value where there is one: `sqrt(-1)` is i, `(-8)^(1/3)` is −2. Complex roots of polynomials with names in their coefficients and of degree 3 or more aren't found.
- **Complex numbers:** complex roots of polynomials of degree 3 and more aren't found. Graphs, `lambdify`, numeric solving and inequalities need real values, and complex numbers have no order (`<` on them is an error).
- **°C** (`degC`) is a temperature difference (no 273.15 offset); temperatures in formulas like `p V = n R T` need K.
- **Unit expressions** take whole-number powers (`#m^2`, `#s^(-1)`); fractional ones (`#m^(1/2)`) aren't units.
- **Functions** can't call themselves.
- **Ranges** have at most 100000 values, and their bounds and steps need values when they're defined. In math an index follows its name directly, so `a[b + c]` is an element of a – write `a dot [b + c]` or `a (b + c)` for a product.
- **Convolution** of functions is exact for piecewise polynomials with numeric breakpoints (rect, tri, sgn, abs, cases with affine conditions, polynomials); exponentials, trigonometric functions, symbolic shifts and integrals that diverge stay the integral, evaluated numerically.
- **Matrices:** names without a value are scalars, so they commute in products until they get a value (define matrices before multiplying them, or pass them to a function). Symbolic elimination treats an entry as zero only when it simplifies to 0, so entries that vanish by an identity like sin² + cos² = 1 count as nonzero. Floats count as zero within 10⁻¹³ of the largest element (times the size). Unknowns of `solve` are scalars: write a vector of unknowns as `vec(x, y)`.
- **Math in a worksheet:** an equation with `=` and only known names is checked, so a formula meant as plain text that happens to use defined names (`$a^2 + b^2 = c^2$` after defining a, b and c) must hold. Put those outside the worksheet or use `#worksheet(evaluate: false)[…]` around them.

## Errors

Problems (a missing value, units that don't match, a real calculation that would need complex numbers, a typo) are shown as a red note where they happen; the rest of the document still compiles:

> ⚠ solveng: 'U' in 'P := U I' isn't defined – define it first with :=, or solve for it: … -> solve(U)

`#show: worksheet.with(errors: "panic")` stops the compilation instead (useful in CI). Unknown named arguments of `worksheet`, `defined` and `graph` are reported the same way; those of `lambdify` stop the compilation.

## HTML

With `typst compile --features html --format html`, equations become MathML, `defined` columns that reflow with the width (or a table with `style: "column"`), graphs inline SVGs (via `html.frame`) that shrink with the page, errors styled notes. One small stylesheet is added to the page (math font STIX Two Math / Cambria Math / Latin Modern Math, spacing, table stripes; classes `solveng`, `solveng-defs`, `solveng-list`, `solveng-plot`, `solveng-error`; types are `solveng-chip` with `solveng-type`, coloured inline). HTML export is experimental in Typst; in column layout browsers drop the space after a leading `=`.

## Testing

The engine is tested in Rust, the package with [tytanic](https://github.com/typst-community/tytanic) (`cargo install tytanic`); the scripts need Python 3 and `typst` on the `PATH`:

```sh
cargo test                                    # the engine (units, parsing, algebra, solving, worksheets, graphs, random inputs) and the editor (IDE, language server)
plugin/build.sh                               # src/solveng.wasm from the engine (rustup target add wasm32-unknown-unknown; wasm-opt if installed)
tt run                                        # units, complex numbers, solving, names, lambdify, … and the README examples (reference images)
tt update readme document                     # accept changed output of the README and document examples
python3 tests/examples.py                     # every example in README.md and docs/reference.md compiles on its own
python3 tests/reference.py                    # docs/reference.md from the doc comments in src/ (--check: is it up to date?)
python3 scripts/unit-words.py                # src/unit-words.typ (the units module) from the unit table (--check: is it up to date?)
uv run --no-project --with sympy --with mpmath python tests/oracle/generate.py   # the reference results
```

`cargo test` includes a fuzzer (`core/tests/fuzz.rs`): random nonlinear equations, polynomials, systems, physics formulas with random prefixes and random numbers to format, checked against direct calculation, and functions whose shown return unit must match what their calls produce; and one for the editor (`editor/ide/tests/fuzz.rs`): random and mutated documents through every editor feature at random positions, which must neither panic nor return ranges outside the text (`FUZZ_N=5000 cargo test --release fuzz` for more cases). `core/tests/oracle.rs` checks families of matrices, integrals, series, simplifications and equations against SymPy and mpmath (the results are in `core/tests/oracle/`, so `cargo test` needs neither); what the engine can't do yet is listed in `core/tests/oracle/known.txt`, which only shrinks: a listed case that passes fails the test until `ORACLE_BLESS=1 cargo test --test oracle` updates the list. `TYPST=…` picks another compiler for the examples, e.g. the oldest supported version.
