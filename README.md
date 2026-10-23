# solveng

Unit-aware engineering calculations that read like your notes, like a Mathcad worksheet. **Write the math as you normally would – solveng evaluates it.** `:=` defines, a trailing `=` shows the value, `->` the symbolic result or a solution, and every value carries its unit.

````typ
#import "@preview/solveng:1.0.0": *
#import units: mA, mAh
#show: worksheet

= Battery runtime
The battery holds $Q_0 := 3450 mAh$ and the load draws $I := 30 mA$,
so it lasts $t := Q_0 / I = 1 unit("h")$.

At half charge and a supply of $U := 5 unit("V")$:
```solveng
t_half := Q_0 / (2 I) = [h]
P := U I =
```
````

![A worksheet: math defines and shows values, a code block calculates with every step](docs/document.png)

That is the whole document – no function calls for the calculation:

- **`:=` defines**, symbolically: `$I := 30 unit("mA")$` is a value, `$P := U I$` is computed, `$f(x) := x^2 + 1$` is a function. Names that aren't defined yet stay names until they are: `$h := 5 z$`, then `$z := 20 unit("cm")$`, and `$h =$` is 1 m.
- **`=` inside an expression compares**, as in Mathcad: `$M := 1 = 2 =$` gives 0, `$M := 2 unit("A") = 2000 unit("mA")$` gives 1; values of different units are never equal.
- **A trailing `=` shows the value**, with a bold =: `$t =$` → t **=** 115 h. After it come the unit and keywords: `$t = min, digits(2)$`.
- **`->` shows the symbolic result** with the known values put in, exactly, and undefined names kept (`$c := 1/3 -> frac$`, `$f(x) := (x^2 - 1)/(x - 1) -> simplify$` is x + 1), **or solves**, symbolically where it can: `$Q_0/2 = I t -> solve(t)$`, `$x^2 + 2x - 3 = 0 -> solve(x)$`, `$R <= U/I -> solve(R)$`, `$0 = 1 + 3/x + y^2 -> solve(x)$` is x = −3/(y² + 1).
- **Any other `=` states something:** it's checked when everything in it is known (`$U = R I$`), and left alone as prose when not (`$E = m c^2$`).
- **Code blocks** ```` ```solveng ```` calculate each line the same way and show it with all its steps.

## The worksheet

`#show: worksheet` calculates the whole document; `#worksheet[…]` only a part, on its own: it starts empty and its definitions are dropped afterwards (see `scope` below). `#worksheet($…$)` calculates single equations where it stands, also outside a worksheet. `hidden: true` calculates without showing what is calculated: the equations, or in `#worksheet(hidden: true)[…]` the math and code blocks – the text around them stays, and a `#worksheet($…$)` inside shows its equations again.

- **Keywords** after `=` or `->`: `solve(x, …)`, `substitute(x, …)` (these variables' formulas put in), `simplify`, `frac` (exact fractions), `digits(n)` (n significant digits), `sci` and `eng` (scientific and engineering notation), and a unit.
- **Code:** each line of ```` ```solveng ```` is calculated like math and shown with its steps: `t := Q_0/I = [h]`, `Q_0/I =`, `x + y = 3; x - y = 1 -> solve(x, y)`; units in brackets, `;` separates a system, `//` is a comment. Inline code and code without a language stay code samples.
- **Values in the text:** `$U =$` shows a value with its name; `#value($U$)` or `#value($U$)` only the value, also of any expression (`#value($f(2)$)`, `#value($U = unit("mV")$)`).
- **Evaluate:** `#worksheet(evaluate: false)[…]` leaves its math alone, only code blocks and `#worksheet(…)` calls are calculated; nested worksheets calculate theirs again. A code block passed to `#worksheet(…)` is calculated without the `solveng` tag.
- **Exercises:** `#show: worksheet.with(clear-at: heading)` starts over at every heading (`heading.where(level: 1)` at top-level ones, `<exercise>` at every element labelled so, like `= Task <exercise>`); `#clear("x", "f")` forgets names, `#clear(all: true)` everything.

```typ
#import "@preview/solveng:1.0.0": *
#import units: mA, mAh
#show: worksheet
$Q_0 := 3450 mAh$, $I := 30 mA$, $U := 5 unit("V")$
$Q_0/2 = I t -> solve(t)$ and $t = 1 unit("h")$
$R <= U/I -> solve(R)$
$x^2 + 2x - 3 = 0 -> solve(x)$
Runtime: $Q_0/I = 1 unit("h")$
```

![Definitions, a solved equation, an inequality and a quadratic](docs/basics.png)

Separate equations with `\` in math (`#worksheet($t := 1 \ g := 2$)`). Complete code equations also end at a newline; parentheses and a trailing arithmetic operator continue the equation. A plain `:=` keeps its formula and definition marker; append `=` to evaluate it. Both forms render as Typst math.

Calculus uses the same notation in both modes: `integral_(0)^(1) x^2 dif x ->` gives 1/3, and `dif / (dif t) f(t) ->` differentiates a defined function. `diff(expression, variable)` and `integrate(expression, variable, lower, upper)` are equivalent calls. Elementary antiderivatives stay exact; other definite integrals use adaptive numerical quadrature, including infinite bounds. Unsupported symbolic integrals stay as integrals; nonconvergent numerical integrals report an error.

Sums use inclusive integer bounds: `sum_(k=0)^10 k^2` or `sum(k^2, k, 0, 10)`. The index is local and case-sensitive; parenthesize summands containing addition. Infinite sums stay symbolic with `->` and use numerical convergence checks with `=` or in graphs. Define `$s_2(t) := sum_(k=0)^infinity sigma(t-k+3/4)$`, then use `#graph($s_2(t)$)` once `sigma` is defined. Piecewise linear signal sums are counted directly between their corners. Other numerical sums stop after at most 100000 terms; slow or nonconvergent series report an error.

For example, a sum of steps defined with `cases` can be plotted directly:

```typ
#import "@preview/solveng:1.0.0": *
#show: worksheet
$sigma(t) := cases(
  0 & "for" t < 0,
  1 & "for" t >= 0
)$
$g_2(t) := sum_(k=0)^infinity sigma(t-k+3/4)$
#graph($g_2(t)$)
```

The acute accent `´` and the backtick (`` ` ``) are literal name characters: `$x´ := 3$` needs no escaping and defines a different variable from `x`. Variable names include digits: `t2`, `t""2`, and `"t2"` are the same name, distinct from `t_2`. Complete Latin letter runs normalize to Greek, so `Delta2tau` and `"Delta2tau"` both name Δ2τ; `Deltatau` stays unchanged. Empty strings join fragments: `Delta""tau` names Δτ, carried through Typst as `$"Delta\"\"tau"$`. Quotes provide no literal-name escape. Completion accepts partial spellings, symbols, joins, and quoted carriers, and inserts the normalized name with quotes only where Typst needs them.

Laplace transforms are functions (in math with `#import functions: laplace, invlaplace`): `$laplace(t^2) ->$` is 2/s³ and `$invlaplace(1/(s+1)) ->$` is exp(−t). The variables default to t → s and s → t; `$F(p) := laplace(exp(-t), t, p)$` and `$f(x) := invlaplace(1/(p+1), p, x)$` choose others. Common polynomial, exponential, trigonometric, and rational transforms are supported; unsupported forms report an error. See [Laplace transforms](docs/reference.md#laplace) for the supported rules.

Matrices are Typst's: `mat(1, 2; 3, 4)`, rows separated by semicolons, `vec(a, b)` for a column, and matrices as blocks of a bigger one (`mat(A, b)`, `mat(A; B)`). Products keep their order (`A B`, `A v`, `v^T A`; `v dot w` of two vectors is the dot product), `A^(-1)` inverts, `A^T` or `A^top` transposes, `|A|` and `det(A)` give the determinant, `norm(v)` the length and `a times b` the cross product; `A^(-1) b` solves the system without forming the inverse. `tr`, `rank`, `identity(n)`, `rows`, `cols` and `at(A, i, j)` complete the set. Matrices of exact numbers stay exact, names give formulas (`$det(mat(a, b; c, d)) ->$` is a d − b c), and units follow the rows and columns, so a resistance matrix solves to currents. A matrix equation solves element by element:

```typ
#import "@preview/solveng:1.0.0": *
#show: worksheet
$R := mat(2 unit("Ω"), 1 unit("Ω"); 1 unit("Ω"), 3 unit("Ω"))$, $U := vec(1 unit("V"), 2 unit("V"))$
$I := R^(-1) U =$, $det(R) =$, $R^(-1) =$
$R vec(I_1, I_2) = U -> solve(I_1, I_2)$
```

See [Built-in operations](docs/reference.md#built-in-operations) for syntax, domains, units, and examples.

Signal functions `rect`, `tri`, and `sgn` are available directly in code blocks; in math after importing them from solveng's `functions` module (`#import functions: rect, tri, sgn`). `rect(x)` is 1 inside ±½, ½ at its endpoints, and 0 outside; `tri(x)` is max(1 − |x|, 0); `sgn(0)` is 0. Linear combinations simplify and solve normally; numerical integrals split shifted/scaled signals at their corners. Distributional derivatives remain symbolic.

Use a worksheet function to give an operation another name:

```typ
#import "@preview/solveng:1.0.0": *
#import functions: tri
#show: worksheet
$Lambda(t) := tri(t)$
$g_2(t) := sum_(k=0)^infinity Lambda(t-k+1/2) ->$
#graph($g_2(t)$, xrange: (-2, 3))
```

`Λ` and `Lambda` name the same function in math and code. `builtin-functions: false` disables automatic operation names, including the names imported from `functions`. Explicit references always select the underlying operation: `#let triangle = math.op("tri")`, then `$triangle(x)$`; use `operation("convolve")` for named math arguments. Quoted math names such as `$"tri"(x)$` only call worksheet functions. See [Built-in operations](docs/reference.md#built-in-operations).

`scope` says what a worksheet sees and keeps: `"isolated"` starts empty and its definitions end with it, `"inherit"` sees the definitions made before and its own end with it, `"shared"` sees them and what it defines stays afterwards. `auto` is `"isolated"` for content (`#worksheet[…]`, `#show: worksheet`) and `"shared"` for equations passed to `worksheet`.

```typ
#import "@preview/solveng:1.0.0": *
#show: worksheet
$U := 12 unit("V")$
#worksheet(scope: "shared")[
  $R := 100 unit("Ω")$
  $I := U/R = unit("mA")$
]
$I = unit("mA")$
```

## Units and variables

Units use explicit constructors or shortcuts: `$9.81 unit("m")/unit("s")^2$`, `$30 unit("mA")$`, `$3 unit("bar")$` for your own. For unquoted unit words, import them from solveng's `units` module where you want them: `#import units: mA, kΩ` gives `$30 mA$`, `$4.7 kΩ$`; single letters need a hash in math (`#import units: V`, then `$5#V$`), since Typst reads a lone letter as a letter. Nothing comes with `#import "@preview/solveng:1.0.0": *`: a shorthand is always one you imported (or declared, see below), and like any import it holds only where it's written. Names of Typst's global scope (`h`, `v`) aren't unit words, so `#import units: *` never hides `#h(1em)`. An unquoted single letter is a variable everywhere in math, also alone after `=` (`$t = h$` compares t with a variable h, `$3 dot h$` is 3 times h) and after `->`, where it's an error: write `$t -> unit("h")$`, or `$t -> #s$` with `#import units: s`. Typst's math names (`min`, `deg`) are unit words too, when you import them: `#import units: min`, then `$t := 2 min$`. So `A` (area, ampere), `V`, `T`, `h`, `C`, `W`, `m`, `s` can be both, and it's always clear which is meant:

```typ
#import "@preview/solveng:1.0.0": *
#import units: Hz, ms
#show: worksheet
$A := 12$, $x := A dot 2 unit("A") =$     // x ≔ A · 2 A = 24 A
$T := 2 unit("s")$, $f := 1/T = Hz$      // f ≔ 1/T = 0.5 Hz
$T -> ms$                          // 2000 ms
```

![Units and variables](docs/units.png)

Units of your own: `#show: worksheet.with(units: (bar: $100 unit("kPa")$, ing: $unit("km")$, i: $unit("km")$, T: none))` defines names (math expressions like `$10^3 unit("m")$`, numbers, or `auto` for a base unit of its own) or hides a built-in one with `none`. Units change what names mean, so `units` and `builtin-units` belong to an isolated worksheet (the default for content), which starts a unit table of its own: the outer worksheet's units don't apply inside. `builtin-units: false` leaves no built-in unit at all – only yours exist (`units: (m: auto, km: $1000 m$, s: auto)`), prefixes included. Use `$1unit("ing")$`, or declare `#let ing = unit("ing")` for `$1ing$`. Single-letter bindings need a hash: `#let i = unit("i")`, `$1#i$`. The editor action *Declare unit overlay bindings* inserts these declarations; Typst options cannot inject lexical bindings. Units resolve through the units of the worksheet where they're used. Complex numbers are unsupported; `i` is an ordinary variable unless referenced through `unit("i")`. See [Units and variables](docs/reference.md#units-and-variables).

Use an explicit unit shortcut when you want a readable name in formulas, completion inside its definition, and hover documentation:

```typ
#import "@preview/solveng:1.0.0": *
#show: worksheet.with(units: (iwef: $unit("km")$))
#let ing = unit("iwef")
#let i = unit("iwef")
$t := 1ing =$  // multiletter binding: no hash
$u := 1#i =$   // single-letter binding: explicit hash
```

Only the unit string is teal in the declaration; evaluated uses receive unit styling. Hover identifies the binding as a unit shortcut and explains the target unit. Ordinary string bindings keep normal highlighting. `unit` resolves the name through the units of the worksheet where it's used, so shortcuts also work across nested worksheets; with `builtin-units: false` only your own units resolve.

Explicit references can also be written directly in math. Both unit spacings work; completion and highlighting apply inside the strings, which collapse to their contents in the editor:

```typ
#import "@preview/solveng:1.0.0": *
#show: worksheet.with(builtin-functions: false, builtin-units: false, units: (m: auto, km: $1000 unit("m")$))
$x := 1/2$
$y := op("tri")(x)$
$d := 1unit("km")$
$e := 1 unit("km")$
```


## Functions

A function's body is symbolic too, so it needn't define everything it uses: what isn't a parameter and isn't defined stays a name, looked up when the function is evaluated – and cancels where the algebra lets it. Undefined function calls also stay symbolic: `$f(t) := integral_(-infinity)^t sigma(tau) dif tau$` is valid before `sigma` is defined, and uses its definition when evaluated later. Numerical evaluation requires its dependencies to be defined and the integral to converge. Parameters can have a unit (arguments must carry that dimension), and `..` makes the body's other undefined names parameters.

```typ
#import "@preview/solveng:1.0.0": *
#import units: cm
#show: worksheet
$h := 5 z$, $z := 20 cm$
$U(x: m) := h x a ->$                      // U(x) ≔ h x a → 1 m · a x
$g(x: m) := U(x)/a ->$                     // a cancels: g(x) ≔ U(x)/a → 1 m · x
$q(x: A, ..) := 1 + 3/3 x + y^2 -> simplify$   // q(x, y) → x + y² + 1
$g(3 unit("m")) =$                                 // 3 m²
```

Named parameters have a default value, as in Typst: after `$f(x, n: 2) := x^n$`, `$f(3) =$` is 9 and `$f(3, n: 3) =$` is 27. A unit after the colon is a type (`I: mA`), a value a default (`n: 2`, `R: 2 Ω`); defaults are taken where the function is defined.

## Sequences and convolution

Range variables and indices work as in Mathcad: `$i := 0..N$` counts from 0 to N, `x[i]` is an element (counted from 0), and `$x[i] := …$` defines one for every value of `i`. `convolve(x, h)` convolves sequences, `step: Δt` makes it the Riemann sum of the convolution integral; `convolve(f(τ), g(τ), τ, t)` convolves functions – exactly for piecewise polynomial signals (`rect`, `tri`, `sgn`, steps written with `cases`), numerically otherwise. Graphs plot a vector over its index, `#graph($y$)`, or over another vector, `#graph($y(τ)$)`, with function curves alongside:

```typ
#import "@preview/solveng:1.0.0": *
#import units: ms
#import functions: tri, convolve
#show: worksheet
$N := 200$, $Delta""t := 10 ms$, $i := 0..N$
$t[i] := -1 unit("s") + i Delta""t$, $x[i] := tri(t[i] / (1 unit("s")))$
$y := convolve(x, x, step: Delta""t)$              // the samples' convolution: 401 values
$k := 0..2N$, $tau[k] := -2 unit("s") + k Delta""t$
$c(t) := convolve(tri(tau), tri(tau), tau, t) ->$  // exact: five cubic pieces
$c(1/2) =$                                          // 23/48
#graph($y(τ)$)                                    // the samples over their time axis
```

`range(start, end, step: …)` (from `functions`) is Typst's range, the end left out, with units too: `$range(0 unit("s"), 1 unit("s"), step: 250 unit("ms"))$`; in Typst code, `range(5)` stays Typst's own. In math, named arguments like `step:` parse only for functions, which `convolve`, `range` and `operation("…")` are. An index follows its name without a space; in math, `a[b + c]` is therefore an element of a, write `a dot [b + c]` for the product.

## Plots

```typ
#import "@preview/solveng:1.0.0": *
#worksheet($R := 50 unit("Ω")$, $"P0" := 2 unit("W")$, $"U0" := 10 unit("V")$, hidden: true)
#graph($P(U: unit("V")) = U^2/R$, ymark: $"P0"$)         // a function of U: where P reaches P0
#graph($cos(w) \ w$)                           // two curves, the crossing marked
#graph($P(..) = "U0"^2/R_L$, xrange: ($10 unit("Ω")$, $1 unit("kΩ")$), width: 10cm)   // .. takes R_L
```

![A graph with where P reaches P0 marked](docs/graph.png)

Ranges, ticks, axis labels and units are picked automatically; marks, ranges and sizes can be set, see the [reference](https://github.com/BierDav/solveng/blob/v1.0.0/docs/reference.md#graph).

## Options

Every option works for the rest of the document, for a part, or for a few equations:

```typ
#import "@preview/solveng:1.0.0": *
#show: worksheet.with(digits: 3, layout: "column")     // your defaults, the whole document
$U := 5 unit("V")$, $I := 30 unit("mA")$
#worksheet($I/3 =$, form: "fraction")                  // only this equation
#worksheet(evaluate: false)[Equations like $E = m c^2$ stay as written.]  // options, no calculation
#worksheet($R := U/I =$, digits: 2, separator: ",")     // only these equations
#defined(show-si: true, style: "inline")
```

The most used ones: `digits`, `separator` (`auto`: from the document language), `form` (`"auto"`, `"decimal"`, `"fraction"`, `"input"`), `steps` (`"full"`, `"formula"`, `"result"`, `"value"`), `layout` (`"row"`, `"column"`), `preferred-units`, `errors` (`"document"` shows problems as red notes, `"panic"` stops the compilation). All of them: [reference](https://github.com/BierDav/solveng/blob/v1.0.0/docs/reference.md#options).

## Reference

[docs/reference.md](https://github.com/BierDav/solveng/blob/v1.0.0/docs/reference.md) has every function, argument and option, how values and units are written, `lambdify` (expressions as plain Typst functions), HTML export, how the solver works and its limits.

## How it calculates

solveng's engine is written in Rust and runs as a WebAssembly plugin (`src/solveng.wasm`, built from [`core/`](core) and [`plugin/`](plugin) with `plugin/build.sh`): a symbolic algebra system for the exact results and the solutions with names in them, and numeric solvers for the rest – see [how it solves](https://github.com/BierDav/solveng/blob/v1.0.0/docs/reference.md#how-it-solves) and its [limits](https://github.com/BierDav/solveng/blob/v1.0.0/docs/reference.md#limits).

## Editors

`solveng-lsp` (in [`editor/`](editor)) is a language server for solveng that runs next to tinymist and answers only where solveng reads the document: ```` ```solveng ```` code blocks, math in a `worksheet`, equations passed to solveng calls, unit and operation substitutions, and the options of solveng's calls. It completes defined names, functions, units (also your own from the `units` option, `worksheet(units: …)`), keywords after `=` and `->`, and option values like `steps: "value"`; highlights calculations (the evaluating `=` bold); shows values and units on hover; converts a code line ⇄ math as a code action; goes to a name's definition and finds its usages as the compile sees them (in its `worksheet`, up to a redefinition or `clear`, through `#include`d files when `main` is set); and warns about unknown units. A client that compiles can send what the compile reported (`query(<solveng>)`) as the notification `solveng/report`; then solveng's errors show as errors where they happened. Sum indices have local scopes for rename, definition, references, completion, and hover; nested sums can reuse an index without changing outer variables. Settings, imports and includes of local files count; set `main` to the file you compile so its `worksheet` counts in the files it includes.

```sh
cargo install --git https://github.com/BierDav/solveng solveng-lsp
```

Neovim (0.11+):

```lua
vim.lsp.config("solveng", { cmd = { "solveng-lsp" }, filetypes = { "typst" }, root_markers = { ".git", "typst.toml" },
  init_options = { main = "main.typ" } })
vim.lsp.enable("solveng")
```

Helix (`languages.toml`):

```toml
[language-server.solveng]
command = "solveng-lsp"
config = { main = "main.typ" }

[[language]]
name = "typst"
language-servers = ["tinymist", "solveng"]
```

VS Code: install [Generic LSP Client](https://marketplace.visualstudio.com/items?itemName=zsol.vscode-glspc) and set `"glspc.server.command": "solveng-lsp"` and `"glspc.server.languageId": ["typst"]`. VS Code uses one semantic token provider per file, so tinymist's highlighting stays; completions, hovers and code actions merge.

Units are highlighted as `enumMember`. Editors can also link the `solveng-ide` crate directly, natively or as WebAssembly, as [Typewright](https://github.com/BierDav/typewright) does.
