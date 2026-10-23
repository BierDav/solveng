# solveng

Unit-aware engineering calculations that read like your notes, like a Mathcad worksheet. **Write the math as you normally would – solveng evaluates it.** `:=` defines, a trailing `=` shows the value, `->` the symbolic result or a solution, and every value carries its unit.

````typ
#import "@preview/solveng:1.0.0": *
#show: worksheet

= Battery runtime
The battery holds $Q_0 := 3450 mAh$ and the load draws $I := 30 mA$,
so it lasts $t := Q_0 / I = 1 h$.

At half charge and a supply of $U := 5 V$:
```solveng
t_half := Q_0 / (2 I) = [h]
P := U I =
```
````

![A worksheet: math defines and shows values, a code block calculates with every step](docs/document.png)

That is the whole document – no function calls for the calculation:

- **`:=` defines**, symbolically: `$I := 30 mA$` is a value, `$P := U I$` is computed, `$f(x) := x^2 + 1$` is a function. Names that aren't defined yet stay names until they are: `$h := 5 z$`, then `$z := 20 cm$`, and `$h =$` is 1 m.
- **`=` inside an expression compares**, as in Mathcad: `$M := 1 = 2 =$` gives 0, `$M := 2 A = 2000 mA$` gives 1; values of different units are never equal.
- **A trailing `=` shows the value**, with a bold =: `$t =$` → t **=** 115 h. After it come the unit and keywords: `$t = min, digits(2)$`.
- **`->` shows the symbolic result** with the known values put in, exactly, and undefined names kept (`$c := 1/3 -> frac$`, `$f(x) := (x^2 - 1)/(x - 1) -> simplify$` is x + 1), **or solves**, symbolically where it can: `$Q_0/2 = I t -> solve(t)$`, `$x^2 + 2x - 3 = 0 -> solve(x)$`, `$R <= U/I -> solve(R)$`, `$0 = 1 + 3/x + y^2 -> solve(x)$` is x = −3/(y² + 1).
- **Any other `=` states something:** it's checked when everything in it is known (`$U = R I$`), and left alone as prose when not (`$E = m c^2$`).
- **Code blocks** ```` ```solveng ```` calculate each line the same way and show it with all its steps.

## The worksheet

`#show: worksheet` calculates the whole document; `#worksheet[…]` only a part, whose definitions are dropped afterwards. `#worksheet($…$)` calculates single equations where it stands, also outside a worksheet. `hidden: true` calculates without showing what is calculated: the equations, or in `#worksheet(hidden: true)[…]` the math and code blocks – the text around them stays, and a `#worksheet($…$)` inside shows its equations again.

- **Keywords** after `=` or `->`: `solve(x, …)`, `simplify`, `frac` (exact fractions), `digits(n)` (n significant digits), `sci` and `eng` (scientific and engineering notation), and a unit.
- **Code:** each line of ```` ```solveng ```` is calculated like math and shown with its steps: `t := Q_0/I = [h]`, `Q_0/I =`, `x + y = 3; x - y = 1 -> solve(x, y)`; units in brackets, `;` separates a system, `//` is a comment. Inline code and code without a language stay code samples.
- **Evaluate:** `#worksheet(evaluate: false)[…]` leaves its math alone, only code blocks and `#worksheet(…)` calls are calculated; nested worksheets calculate theirs again. A code block passed to `#worksheet(…)` is calculated without the `solveng` tag.
- **Exercises:** `#show: worksheet.with(clear-at: heading)` starts over at every heading; `#clear("x", "f")` forgets names, `#clear(all: true)` everything.

```typ
#import "@preview/solveng:1.0.0": *
#show: worksheet
$Q_0 := 3450 mAh$, $I := 30 mA$, $U := 5 V$
$Q_0/2 = I t -> solve(t)$ and $t = 1 h$
$R <= U/I -> solve(R)$
$x^2 + 2x - 3 = 0 -> solve(x)$
Runtime: $Q_0/I = 1 h$
```

![Definitions, a solved equation, an inequality and a quadratic](docs/basics.png)

Separate equations with `\` in math (`#worksheet($t := 1 \ g := 2$)`) and strings (`#worksheet("t := 1 \\ g := 2")`). Complete string/code equations also end at a newline; parentheses and a trailing arithmetic operator continue the equation. A plain `:=` keeps its formula and definition marker; append `=` to evaluate it. Both forms render as Typst math.

Calculus uses the same notation in both modes: `integral_(0)^(1) x^2 dif x ->` gives 1/3, and `dif / (dif t) f(t) ->` differentiates a defined function. `diff(expression, variable)` and `integrate(expression, variable, lower, upper)` are equivalent calls. Elementary antiderivatives stay exact; other definite integrals use adaptive numerical quadrature, including infinite bounds. Unsupported symbolic integrals stay as integrals; nonconvergent numerical integrals report an error.

Sums use inclusive integer bounds: `sum_(k=0)^10 k^2` or `sum(k^2, k, 0, 10)`. The index is local and case-sensitive; parenthesize summands containing addition. Infinite sums stay symbolic with `->` and use numerical convergence checks with `=` or in graphs. Define `$s_2(t) := sum_(k=0)^infinity sigma(t-k+3/4)$`, then use `#graph("s_2(t)")` once `sigma` is defined. Piecewise linear signal sums are counted directly between their corners. Other numerical sums stop after at most 100000 terms; slow or nonconvergent series report an error.

For example, a sum of steps defined with `cases` can be plotted directly:

```typ
#import "@preview/solveng:1.0.0": *
#show: worksheet
$sigma(t) := cases(
  0 & "for" t < 0,
  1 & "for" t >= 0
)$
$g_2(t) := sum_(k=0)^infinity sigma(t-k+3/4)$
#graph("g_2(t)")
```

The acute accent `´` and the backtick (`` ` ``) are literal name characters: `$x´ := 3$` needs no escaping and defines a different variable from `x`. Variable names include digits: `t2`, `t""2`, and `"t2"` are the same name, distinct from `t_2`. Complete Latin letter runs normalize to Greek, so `Delta2tau` and `"Delta2tau"` both name Δ2τ; `Deltatau` stays unchanged. Empty strings join fragments: `Delta""tau` names Δτ, carried through Typst as `$"Delta\"\"tau"$`. Quotes provide no literal-name escape. Completion accepts partial spellings, symbols, joins, and quoted carriers, and inserts the normalized name with quotes only where Typst needs them.

Use `$t^2 -> laplace$` for 2/s³ and `$1/(s+1) -> invlaplace$` for exp(−t). The defaults are t → s and s → t; `laplace(x, p)` and `invlaplace(p, x)` after the arrow choose other variables. Callable forms such as `$F(p) := laplace(exp(-t), t, p)$` also work. Common polynomial, exponential, trigonometric, and rational transforms are supported; unsupported forms report an error. See [Laplace transforms](docs/reference.md#laplace) for the supported rules.

Matrices are Typst's: `mat(1, 2; 3, 4)`, rows separated by semicolons, `vec(a, b)` for a column, and matrices as blocks of a bigger one (`mat(A, b)`, `mat(A; B)`). Products keep their order (`A B`, `A v`, `v^T A`; `v dot w` of two vectors is the dot product), `A^(-1)` inverts, `A^T` or `A^top` transposes, `|A|` and `det(A)` give the determinant, `norm(v)` the length and `a times b` the cross product; `A^(-1) b` solves the system without forming the inverse. `tr`, `rank`, `identity(n)`, `rows`, `cols` and `at(A, i, j)` complete the set. Matrices of exact numbers stay exact, names give formulas (`$det(mat(a, b; c, d)) ->$` is a d − b c), and units follow the rows and columns, so a resistance matrix solves to currents. A matrix equation solves element by element:

```typ
#import "@preview/solveng:1.0.0": *
#show: worksheet
$R := mat(2 Ω, 1 Ω; 1 Ω, 3 Ω)$, $U := vec(1 V, 2 V)$
$I := R^(-1) U =$, $det(R) =$, $R^(-1) =$
$R vec(I_1, I_2) = U -> solve(I_1, I_2)$
```

See [Built-in operations](docs/reference.md#built-in-operations) for syntax, domains, units, and examples.

Signal functions `rect`, `tri`, and `sgn` are available directly in strings and math after importing solveng. `rect(x)` is 1 inside ±½, ½ at its endpoints, and 0 outside; `tri(x)` is max(1 − |x|, 0); `sgn(0)` is 0. Linear combinations simplify and solve normally; numerical integrals split shifted/scaled signals at their corners. Distributional derivatives remain symbolic.

Use a worksheet function to give an operation another name:

```typ
#import "@preview/solveng:1.0.0": *
#show: worksheet
$Lambda(t) := tri(t)$
$g_2(t) := sum_(k=0)^infinity Lambda(t-k+1/2) ->$
#graph("g_2(t)", xrange: (-2, 3))
```

Standard operation names are always available. Define an ordinary worksheet function such as `$Lambda(t) := tri(t)$` to use another name; `Λ` and `Lambda` name the same function in math and code strings.

`inherit` and `export` control worksheet scope independently: `inherit` reads definitions from the parent; `export` passes the final definitions back. Both default to `auto`: true for expression calls and false for content blocks.

```typ
#import "@preview/solveng:1.0.0": *
#show: worksheet
$U := 12 V$
#worksheet(inherit: true, export: true)[
  $R := 100 Ω$
  $I := U/R = mA$
]
$I = mA$
```

## Units and variables

Units are names: `$30 mA$`, `$4.7 kΩ$`, `$9.81 m/s^2$`, and `unit("bar")` for units without one (your own). A single letter is a unit only right after a number: `$5 V$`, `$3 h$`, and `$t = 1 h$` shows t in hours. Anywhere else it's a variable, also alone after `=`: `$t = h$` compares t with a variable h, and `$3 dot h$` is 3 times h. So `A` (area, ampere), `V`, `T`, `h`, `C`, `W`, `m`, `s` can be both, and it's always clear which is meant:

```typ
#import "@preview/solveng:1.0.0": *
#show: worksheet
$A := 12$, $x := A dot 2 A =$     // x ≔ A · 2 A = 24 A
$T := 2 s$, $f := 1/T = Hz$      // f ≔ 1/T = 0.5 Hz
```

![Units and variables](docs/units.png)

Your own units: `#worksheet-config(units: (bar: "100 kPa", rpm: "1/min", T: none))` – `none` removes one and frees its name.

## Functions

A function's body is symbolic too, so it needn't define everything it uses: what isn't a parameter and isn't defined stays a name, looked up when the function is evaluated – and cancels where the algebra lets it. Undefined function calls also stay symbolic: `$f(t) := integral_(-infinity)^t sigma(tau) dif tau$` is valid before `sigma` is defined, and uses its definition when evaluated later. Numerical evaluation requires its dependencies to be defined and the integral to converge. Parameters can have a unit (plain numbers passed get it), and `..` makes the body's other undefined names parameters.

```typ
#import "@preview/solveng:1.0.0": *
#show: worksheet
$h := 5 z$, $z := 20 cm$
$U(x: m) := h x a ->$                      // U(x) ≔ h x a → 1 m · a x
$g(x: m) := U(x)/a ->$                     // a cancels: g(x) ≔ U(x)/a → 1 m · x
$q(x: A, ..) := 1 + 3/3 x + y^2 -> simplify$   // q(x, y) → x + y² + 1
$g(3 m) =$                                 // 3 m²
```

## Plots

```typ
#import "@preview/solveng:1.0.0": *
#worksheet("R := 50 Ω", "P0 := 2 W", "U0 := 10 V", hidden: true)
#graph("P(U: V) = U^2/R", ymark: "P0")         // a function of U: where P reaches P0
#graph("cos(w) \\ w")                           // two curves, the crossing marked
#graph("P(..) = U0^2/R_L", xrange: ("10 Ω", "1 kΩ"), width: 10cm)   // .. takes R_L
```

![A graph with where P reaches P0 marked](docs/graph.png)

Ranges, ticks, axis labels and units are picked automatically; marks, ranges and sizes can be set, see the [reference](https://github.com/BierDav/solveng/blob/v1.0.0/docs/reference.md#graph).

## Options

Every option works for the rest of the document, for a part, or for a few equations:

```typ
#import "@preview/solveng:1.0.0": *
#show: worksheet-config.with(digits: 3, layout: "column")   // your defaults
#show: worksheet
$U := 5 V$, $I := 30 mA$
#worksheet-config(form: "fraction")[$I/3 =$]            // only inside the brackets
#worksheet($R := U/I =$, digits: 2, separator: ",")     // only these equations
#defined(show-si: true, style: "inline")
```

The most used ones: `digits`, `separator` (`auto`: from the document language), `form` (`"auto"`, `"decimal"`, `"fraction"`, `"input"`), `steps` (`"full"`, `"formula"`, `"result"`, `"value"`), `layout` (`"row"`, `"column"`), `preferred-units`, `errors` (`"document"` shows problems as red notes, `"panic"` stops the compilation). All of them: [reference](https://github.com/BierDav/solveng/blob/v1.0.0/docs/reference.md#options).

## Reference

[docs/reference.md](https://github.com/BierDav/solveng/blob/v1.0.0/docs/reference.md) has every function, argument and option, how values and units are written, `lambdify` (expressions as plain Typst functions), HTML export, how the solver works and its limits.

## How it calculates

solveng's engine is written in Rust and runs as a WebAssembly plugin (`src/solveng.wasm`, built from [`core/`](core) and [`plugin/`](plugin) with `plugin/build.sh`): a symbolic algebra system for the exact results and the solutions with names in them, and numeric solvers for the rest – see [how it solves](https://github.com/BierDav/solveng/blob/v1.0.0/docs/reference.md#how-it-solves) and its [limits](https://github.com/BierDav/solveng/blob/v1.0.0/docs/reference.md#limits).

## Editors

`solveng-lsp` (in [`editor/`](editor)) is a language server for solveng that runs next to tinymist and answers only where solveng reads the document: ```` ```solveng ```` code blocks, math in a `worksheet`, the strings of `worksheet`, `graph` and `lambdify`, and the options of solveng's calls. It completes defined names, functions, units (also your own from `worksheet-config(units: …)`), keywords after `=` and `->`, and option values like `steps: "value"`; highlights calculations (the evaluating `=` bold); shows values and units on hover; converts a code line ⇄ math as a code action; goes to a name's definition and finds its usages as the compile sees them (in its `worksheet`, up to a redefinition or `clear`, through `#include`d files when `main` is set); and warns about unknown units. Sum indices have local scopes for rename, definition, references, completion, and hover; nested sums can reuse an index without changing outer variables. Settings, imports and includes of local files count; set `main` to the file you compile so its `worksheet` counts in the files it includes.

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
