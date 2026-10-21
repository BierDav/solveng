# solveng

Unit-aware engineering calculations that read like your notes, like a Mathcad worksheet. **Write the math as you normally would – solveng evaluates it.** `:=` defines, a trailing `=` shows the value, `->` the symbolic result or a solution, and every value carries its unit.

````typ
#import "@preview/solveng:0.2.0": *
#show: worksheet

= Battery runtime
The battery holds $Q_0 := 3450 mAh$ and the load draws $I := 30 mA$,
so it lasts $t := Q_0 / I = h$.

At half charge and a supply of $U := 5 V$:
```solveng
t_half := Q_0 / (2 I) = [h]
P := U I =
```
````

![A worksheet: math defines and shows values, a code block calculates with every step](docs/document.png)

That is the whole document – no function calls for the calculation:

- **`:=` defines.** `$I := 30 mA$` is a value, `$P := U I$` is computed, `$f(x) := x^2 + 1$` is a function. Steps after it are checked: `$R := 7.2 kΩ = 7200 Ω$`.
- **A trailing `=` shows the value**, with a bold =, as in Mathcad: `$t =$` → t **=** 115 h. After it come the unit and keywords: `$t = min, digits(2)$`.
- **`->` shows the symbolic result** with the known values put in, exactly (`$c := 1/3 -> frac$`), **or solves**: `$Q_0/2 = I t -> solve(t)$`, `$x^2 + 2x - 3 = 0 -> solve(x)$`, `$R <= U/I -> solve(R)$`.
- **Any other `=` states something:** it's checked when everything in it is known (`$U = R I$`), and left alone as prose when not (`$E = m c^2$`).
- **Code blocks** ```` ```solveng ```` calculate each line the same way and show it with all its steps.

## The worksheet

`#show: worksheet` calculates the whole document; `#worksheet[…]` only a part, whose definitions are dropped afterwards. `#worksheet($…$)` calculates single equations where it stands, also outside a worksheet, and `hidden: true` silently.

- **Keywords** after `=` or `->`: `solve(x, …)`, `simplify`, `frac` (exact fractions), `digits(n)` (n significant digits), `sci` and `eng` (scientific and engineering notation), and a unit.
- **Code:** each line of ```` ```solveng ```` is calculated like math and shown with its steps: `t := Q_0/I = [h]`, `Q_0/I =`, `x + y = 3; x - y = 1 -> solve(x, y)`; units in brackets, `;` separates a system, `//` is a comment. Inline code and code without a language stay code samples.
- **Integration:** `#show: worksheet-config.with(integration: "none")` leaves math alone, only code blocks and `#worksheet(…)` calls are calculated; `"math"` is the default.
- **Exercises:** `#show: worksheet.with(reset: heading)` starts over at every heading.

```typ
#import "@preview/solveng:0.2.0": *
#show: worksheet
$Q_0 := 3450 mAh$, $I := 30 mA$, $U := 5 V$
$Q_0/2 = I t -> solve(t)$ and $t = h$
$R <= U/I -> solve(R)$
$x^2 + 2x - 3 = 0 -> solve(x)$
Runtime: $Q_0/I = h$
```

![Definitions, a solved equation, an inequality and a quadratic](docs/basics.png)

## Units and variables

Units are names: `$30 mA$`, `$4.7 kΩ$`, `$9.81 m/s^2$`, and `unit("bar")` for units without one (your own). A single letter is a unit right after a number (`$5 V$`, `$3 h$`) or alone after a trailing `=` (`$t = h$`); anywhere else it's a variable – `$3 dot h$` is 3 times a variable h. So `A` (area, ampere), `V`, `T`, `h`, `C`, `W`, `m`, `s` can be both, and it's always clear which is meant:

```typ
#import "@preview/solveng:0.2.0": *
#show: worksheet
$A := 12$, $x := A dot 2 A =$     // x ≔ A · 2 A = 24 A
$T := 2 s$, $f := 1/T = Hz$      // f ≔ 1/T = 0.5 Hz
```

![Units and variables](docs/units.png)

Your own units: `#worksheet-config(units: (bar: "100 kPa", rpm: "1/min", T: none))` – `none` removes one and frees its name.

## Plots

```typ
#import "@preview/solveng:0.2.0": *
#worksheet("R := 50 Ω", "P0 := 2 W", "U0 := 10 V", hidden: true)
#graph("P(U: V) = U^2/R", ymark: "P0")         // a function of U: where P reaches P0
#graph("cos(w)", "w")                           // two curves, the crossing marked
#graph("P(..) = U0^2/R_L", xrange: ("10 Ω", "1 kΩ"), width: 10cm)   // .. takes R_L
```

![A graph with where P reaches P0 marked](docs/graph.png)

Ranges, ticks, axis labels and units are picked automatically; marks, ranges and sizes can be set, see the [reference](https://github.com/BierDav/solveng/blob/v0.2.0/docs/reference.md#graph).

## Options

Every option works for the rest of the document, for a part, or for a few equations:

```typ
#import "@preview/solveng:0.2.0": *
#show: worksheet-config.with(digits: 3, layout: "column")   // your defaults
#show: worksheet
$U := 5 V$, $I := 30 mA$
#worksheet-config(form: "fraction")[$I/3 =$]            // only inside the brackets
#worksheet($R := U/I =$, digits: 2, separator: ",")     // only these equations
#defined(show-si: true, style: "inline")
```

The most used ones: `digits`, `separator` (`auto`: from the document language), `form` (`"auto"`, `"decimal"`, `"fraction"`, `"input"`), `steps` (`"full"`, `"formula"`, `"result"`, `"value"`), `layout` (`"row"`, `"column"`), `preferred-units`, `errors` (`"document"` shows problems as red notes, `"panic"` stops the compilation). All of them: [reference](https://github.com/BierDav/solveng/blob/v0.2.0/docs/reference.md#options).

## Reference

[docs/reference.md](https://github.com/BierDav/solveng/blob/v0.2.0/docs/reference.md) has every function, argument and option, how values and units are written, `lambdify` (expressions as plain Typst functions), HTML export, how the solver works and its limits.

## Editors

`solveng-lsp` (in [`editor/`](editor)) is a language server for solveng that runs next to tinymist and answers only where solveng reads the document: ```` ```solveng ```` code blocks, math in a `worksheet`, the strings of `worksheet`, `graph` and `lambdify`, and the options of solveng's calls. It completes defined names, functions, units (also your own from `worksheet-config(units: …)`), keywords after `=` and `->`, and option values like `steps: "value"`; highlights calculations (the evaluating `=` bold); shows values and units on hover; converts a code line ⇄ math as a code action; goes to a name's definition and finds its usages as the compile sees them (in its `worksheet`, up to a redefinition or reset, through `#include`d files when `main` is set); and warns about unknown units. Settings, imports and includes of local files count; set `main` to the file you compile so its `worksheet` counts in the files it includes.

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
