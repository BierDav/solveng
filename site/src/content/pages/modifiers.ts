import type { Page } from "../types";

const PKG = `#import "@preview/solveng:2.0.0": *`;

export const pages: Page[] = [
  // ---------------------------------------------------------------- solve
  {
    slug: "modifiers/solve",
    title: "solve",
    group: "modifiers",
    summary: "After `->`, solves an equation for the names given – symbolically when it can, numerically when every other name has a value – and shows the solutions without defining anything.",
    signature: "solve(..names)",
    import: "#import modifiers: solve",
    returns: "The solutions, shown after →: one, several (x₁, x₂, …), a range for an inequality, or one per name for a system.",
    keywords: ["solve", "equation", "unknown", "roots", "isolate", "system", "inequality", "numeric"],
    blocks: [
      { md: "Write the equation as you would on paper, then `-> solve(t)`. The other names are either put in (they have values) or kept as parameters (they don't), so the same modifier gives you a formula or a number." },
      {
        example: {
          title: "How long does half a battery last?",
          code: `${PKG}
#import units: mA, mAh, h
#import modifiers: solve
#show: math

$Q_0/2 = I t -> solve(t)$ // nothing defined yet: a formula
$Q_0 := 3450 mAh$, $I := {{i}} mA$
$Q_0/2 = I t -> solve(t)$ // now a time`,
          controls: [{ kind: "slider", name: "i", label: "I", min: 5, max: 500, step: 5, value: 30, suffix: " mA" }],
          caption: "The same equation twice: before the definitions it gives t = Q₀/(2I), after them a time in hours.",
        },
      },
      { h2: "Symbolic, all roots, complex" },
      { md: "An unknown that occurs once is isolated. A polynomial is solved by formula up to degree 2 and numerically above – all its roots, complex ones included." },
      {
        example: {
          code: `${PKG}
#import modifiers: solve
#show: math

$0 = 1 + 3/x + y^2 -> solve(x)$
$x^3 = 8 -> solve(x)$
$x^2 + 4 = 0 -> solve(x)$`,
        },
      },
      { h2: "Numerically, with a unit and digits" },
      { md: "When the equation can't be rearranged but every other name has a value, solveng searches numerically for real solutions. The clause takes a unit and other modifiers after a comma: `-> solve(t), µs, digits(3)`." },
      {
        example: {
          title: "When does an RC charge reach a threshold?",
          code: `${PKG}
#import units: V, kΩ, nF, µs
#import constants: e
#import modifiers: solve, digits
#show: math

$R := {{r}} kΩ$, $C := 100 nF$, $U_0 := 5 #V$
$U_0 dot (1 - #e^(-t/(R C))) = 3.3 #V -> solve(t), µs, digits(3)$
$cos(x) = x -> solve(x)$`,
          controls: [{ kind: "slider", name: "r", label: "R", min: 1, max: 47, step: 1, value: 10, suffix: " kΩ" }],
          caption: "The first is isolated through the logarithm; `cos(x) = x` has no closed form and is found numerically. (`U_0 dot (…)`: without the dot, `U_0(…)` would be a function call.)",
        },
      },
      { h2: "Systems and inequalities" },
      { md: "Several equations go in a `vec(…)`, the unknowns in `solve(…)`. Each solution is shown by name. An inequality gives the range where it holds." },
      {
        example: {
          title: "Two mesh currents",
          code: `${PKG}
#import units: V, kΩ
#import modifiers: solve, digits
#show: math

$U_0 := 10 #V$, $R_1 := 1 kΩ$, $R_2 := 2 kΩ$, $R_3 := {{r3}} kΩ$
$vec(U_0 = R_1 I_1 + R_3 dot (I_1 - I_2),
     0 = R_2 I_2 + R_3 dot (I_2 - I_1)) -> solve(I_1, I_2), digits(3)$
$x^2 < 1 -> solve(x)$`,
          controls: [{ kind: "slider", name: "r3", label: "R₃", min: 1, max: 20, step: 1, value: 3, suffix: " kΩ" }],
          caption: "Note `R_3 dot (…)`: written `R_3 (…)`, it would be a call of a function R₃.",
        },
      },
      { h2: "Keeping the solutions" },
      { md: "`solve` shows solutions and **defines nothing** – the unknown stays unknown. To keep them, define a name with the equation: `$R_2 := … -> solve(R_2)$`. One solution is a value, several a vector; pick one with [`at`](/docs/operations/at). This needs a symbolic solution." },
      {
        example: {
          title: "A voltage divider, designed",
          code: `${PKG}
#import units: V, kΩ, mA
#import operations: at
#import modifiers: solve
#show: math

$U_0 := 12 #V$, $R_1 := 4.7 kΩ$, $U_2 := {{u2}} #V$
$R_2 := U_2 = U_0 R_2 / (R_1 + R_2) -> solve(R_2)$
$R_2 => kΩ$, $I := U_0 / (R_1 + R_2) => mA$
$x := x^2 = 4 -> solve(x)$, $at(x, 1) =>$`,
          controls: [{ kind: "slider", name: "u2", label: "U₂", min: 1, max: 11, step: 0.5, value: 5, suffix: " V" }],
          caption: "R₂ is defined by its equation, so I can use it. The second line keeps both roots of x² = 4 in a vector.",
        },
      },
      {
        example: {
          title: "The classic mistake",
          code: `${PKG}
#import modifiers: solve
#show: math

$x^2 = 4 -> solve(x)$
$x =>$`,
          expectError: true,
          caption: "After a bare `solve`, x is still undefined. The message says how to keep it.",
        },
      },
      { h2: "Renamed imports" },
      { md: "`solve` is a Typst binding like any other, so a renamed import works the same – handy for documents in another language." },
      {
        example: {
          code: `${PKG}
#import modifiers: solve as löse
#show: math

$x^2 - 2 x = 3 -> löse(x)$`,
        },
      },
      { h2: "Pitfalls" },
      { md: "- **Import it.** `#import \"@preview/solveng:2.0.0\": *` brings no modifiers; without `#import modifiers: solve`, `solve` is an unknown variable.\n- **After `->`.** One clause: `-> solve(t), ms`, not `-> solve(t) => ms`.\n- **One unit for one unknown.** With several unknowns, leave the unit out (`digits` still works).\n- **Principal values.** Periodic functions give one solution: `tan(x) = 1` is x ≈ 0.785, arctan(1). Polynomials give all their roots.\n- **Real numeric search.** The numeric search finds real roots only, between ±10¹⁵; nearby poles or fast oscillations can hide a root.\n- **Many solutions.** The option `max-solutions` (default 6) limits how many are listed; all are found.\n- **Not with `steps`.** `steps` shows how `->` simplifies, not how `solve` solves.\n\nMore background: [Solving](/docs/solving), [How it solves](/docs/how-it-solves)." },
      {
        quiz: {
          question: "What does the second line show?",
          code: `$x^2 - 4 = 0 -> solve(x)$
$y := x + 1 =>$`,
          options: ["`y ⇒ 3` (taking x = 2)", "`y ⇒ (−1, 3)`", "A problem: x isn't defined"],
          answer: 2,
          explain: "`solve` only shows solutions. Keep them with a definition – `$x := x^2 - 4 = 0 -> solve(x)$` – and then pick one with `at(x, 1)`.",
        },
      },
    ],
    related: ["solving", "how-it-solves", "modifiers/substitute", "operations/at", "operators"],
  },

  // ---------------------------------------------------------------- substitute
  {
    slug: "modifiers/substitute",
    title: "substitute",
    group: "modifiers",
    summary: "After `->`, puts in the formulas of the variables and the expressions of the constants you name – and keeps every other name as a symbol.",
    signature: "substitute(..names)",
    import: "#import modifiers: substitute",
    returns: "The symbolic result with exactly these names replaced.",
    keywords: ["substitute", "replace", "put in", "formula", "constant value", "symbolic"],
    blocks: [
      { md: "A bare `->` puts in **every** value it knows and simplifies – often all the way to a number. `substitute(U)` is selective: only U's formula goes in, R and I stay letters even though they have values. That's how you show *which* formula a result comes from." },
      {
        example: {
          code: `${PKG}
#import units: kΩ, mA, mW
#import modifiers: substitute
#show: math

$R := 2 kΩ$, $I := 3 mA$, $U := R I$
$P := U^2/R {{clause}}$`,
          controls: [{ kind: "select", name: "clause", label: "clause", options: ["-> substitute(U)", "->", "=> mW"], value: "-> substitute(U)" }],
          caption: "`substitute(U)`: I²·R. Bare `->`: the exact power, 9/500 W. `=>`: the rounded value.",
        },
      },
      { h2: "Constants" },
      { md: "A [constant](/docs/functions/constant) is kept by name in exact results – `m₀ c²` stays readable. `substitute(#c)` puts its expression in. For a constant alone it shows the expression exactly as written, where simplifying would give the constant again." },
      {
        example: {
          code: `${PKG}
#import constants: c, i
#import modifiers: substitute
#show: math

$E := m_0 #c^2 ->$
$E -> substitute(#c)$
$#i -> substitute(#i)$`,
          caption: "The last line is √−1, the imaginary unit's expression.",
        },
      },
      { h2: "Several names" },
      {
        example: {
          code: `${PKG}
#import modifiers: substitute
#show: math

$U := R I$, $I := U_0 / (R + R_i)$
$P := U I -> substitute(U, I)$`,
          caption: "Both formulas go in. Names without a formula (R, U₀, Rᵢ) stay as they are.",
        },
      },
      { h2: "Pitfalls" },
      { md: "- **The name must be defined** – `substitute(Q)` with an undefined Q is a problem.\n- **It must occur.** A definition puts in names that were defined *before* it. After `$U := R I$`, a later `$P := U^2/R$` already contains R I, not U – `$P -> substitute(U)$` then reports that U doesn't occur. Write the substitution where the formula is defined: `$P := U^2/R -> substitute(U)$`.\n- **One-letter constants** need their `#`: `substitute(#c)`, as in the math." },
      {
        example: {
          title: "U has already been put in",
          code: `${PKG}
#import modifiers: substitute
#show: math

$U := R I$
$P := U^2/R$
$P -> substitute(U)$`,
          expectError: true,
        },
      },
      {
        quiz: {
          question: "After `$R := 2 kΩ$, $I := 3 mA$, $U := R I$`, what does `$U -> substitute(U)$` show?",
          options: ["`U → 6 V`", "`U → I · R`", "`U → U`"],
          answer: 1,
          explain: "Only U's formula goes in; R and I are kept as names even though they have values. Bare `->` would give 6 V.",
        },
      },
    ],
    related: ["modifiers/simplify", "functions/constant", "symbolic", "constants"],
  },

  // ---------------------------------------------------------------- assume
  {
    slug: "modifiers/assume",
    title: "assume",
    group: "modifiers",
    summary: "After `->`, simplifies under assumptions you state – a name is real, positive, negative – for this one result only.",
    signature: "assume(..conditions)",
    import: "#import modifiers: assume",
    returns: "The symbolic result, valid under the assumptions.",
    keywords: ["assume", "real", "positive", "negative", "condition", "sign", "domain"],
    blocks: [
      { md: "solveng's names are complex until proven otherwise, so `sqrt(x^2)` can't just be x – it's −x for negative x, and something else for complex ones. `assume` tells it what you know: `x > 0`, `x in RR`. The assumption holds for this result only; nothing gets defined." },
      {
        example: {
          code: `${PKG}
#import modifiers: assume
#show: math

$sqrt(x^2) {{clause}}$
$abs(t) -> assume(t >= 0)$`,
          controls: [{ kind: "select", name: "clause", label: "clause", options: ["->", "-> assume(x > 0)", "-> assume(x < 0)"], value: "-> assume(x > 0)" }],
          caption: "Without an assumption √(x²) stays; with a sign, it becomes x or −x.",
        },
      },
      { h2: "Real names" },
      { md: "With `Re`, `Im`, `conj` and `abs`, solveng already shows the general result and – where it's simpler – the one for real names, with the fewest names it needs (`for a ∈ ℝ ∧ b ∈ ℝ`). That's a sufficient condition, not a complete list. `assume` makes it the only result." },
      {
        example: {
          code: `${PKG}
#import operations: Re
#import constants: e, i
#import modifiers: assume
#show: math

$Re(a z + b w) = a Re(z) + b Re(w) ->$
$Re(a z + b w) = a Re(z) + b Re(w) -> assume(a in RR, b in RR)$
$abs(1 + #e^(#i x))^2 -> assume(x in RR)$`,
        },
      },
      { h2: "Conditions" },
      {
        params: [
          { name: "..conditions", type: "content", md: "Real scalar names (`a in RR`) and signs (`x > 0`, `x >= 0`, `x < 0`, `x <= 0`, `x = 0`), separated by commas." },
        ],
      },
      { h2: "Pitfalls" },
      { md: "- **Only after `->`**, on expressions and equations – not with `solve` and not in a definition.\n- **Contradictions are errors**: assuming `x < 0` when x is 4, or a real value for a known complex one.\n- **Defined names**: if a name has a symbolic definition, assume things about the free names inside it instead.\n- **Local**: nothing carries over to the next line. Defined values shadow assumptions." },
      {
        example: {
          title: "A contradiction",
          code: `${PKG}
#import modifiers: assume
#show: math

$x := 4$
$sqrt(x^2) -> assume(x < 0)$`,
          expectError: true,
        },
      },
    ],
    related: ["complex", "symbolic", "operations/Re", "modifiers/simplify"],
  },

  // ---------------------------------------------------------------- simplify
  {
    slug: "modifiers/simplify",
    title: "simplify",
    group: "modifiers",
    summary: "Spells out what `->` does anyway: the simplified symbolic result, or for an equation, where it holds.",
    signature: "simplify",
    import: "#import modifiers: simplify",
    returns: "The symbolic result – identical to bare `->`.",
    keywords: ["simplify", "symbolic", "cancel", "where it holds", "true", "false"],
    blocks: [
      { md: "`-> simplify` and a bare `->` are the same. Use it when you want the intent visible in the source; otherwise prefer the shorter `->`.\n\n- An **expression** is simplified: like terms combined, fractions cancelled, exact numbers kept exact.\n- An **equation** shows where it holds: one side brought to zero, split into factors, each factor zero. `true` when it always holds, `false` when never." },
      {
        example: {
          code: `${PKG}
#import modifiers: simplify
#show: math

$f := (x^2 - 1)/(x - 1) -> simplify$
$x^2 = x -> simplify$
$x + 1 = x -> simplify$
$sin(x)^2 + cos(x)^2 = 1 ->$`,
          caption: "The last line is bare `->`: same thing. Supported identities are proven exactly, never by sampling.",
        },
      },
      { h2: "Related" },
      { md: "To multiply out instead, use [`expand`](/docs/modifiers/expand); to split into factors, [`factor`](/docs/modifiers/factor); to see the way there, [`steps`](/docs/modifiers/steps)." },
    ],
    related: ["modifiers/expand", "modifiers/factor", "modifiers/steps", "symbolic", "operators"],
  },

  // ---------------------------------------------------------------- expand
  {
    slug: "modifiers/expand",
    title: "expand",
    group: "modifiers",
    summary: "After `->`, multiplies the symbolic result out – and an equation side by side.",
    signature: "expand",
    import: "#import modifiers: expand",
    returns: "The result as a sum of products.",
    keywords: ["expand", "multiply out", "binomial", "polynomial", "distribute"],
    blocks: [
      {
        example: {
          code: `${PKG}
#import modifiers: expand
#show: math

$(a + b)^{{n}} -> expand$`,
          controls: [{ kind: "slider", name: "n", label: "n", min: 1, max: 7, step: 1, value: 3 }],
          caption: "Pascal's triangle in the coefficients – drag n.",
        },
      },
      { h2: "Equations" },
      { md: "An equation is expanded side by side, not solved. That keeps it recognisable." },
      {
        example: {
          code: `${PKG}
#import modifiers: expand
#show: math

$(x + 1)(x - 1) = 3 -> expand$
$(x + 1)^2 - (x - 1)^2 -> expand$`,
        },
      },
      { callout: "tip", md: "`expand` and [`factor`](/docs/modifiers/factor) are opposites: one multiplies out, the other splits into factors. Bare `->` does whatever simplifies best." },
    ],
    related: ["modifiers/factor", "modifiers/simplify", "symbolic"],
  },

  // ---------------------------------------------------------------- factor
  {
    slug: "modifiers/factor",
    title: "factor",
    group: "modifiers",
    summary: "After `->`, splits the symbolic result into factors – a fraction's numerator and denominator each, an equation side by side.",
    signature: "factor",
    import: "#import modifiers: factor",
    returns: "The result as a product of factors.",
    keywords: ["factor", "factorize", "roots", "polynomial", "common factor"],
    blocks: [
      {
        example: {
          code: `${PKG}
#import modifiers: factor
#show: math

$x^3 - x -> factor$
$x^4 - 1 -> factor$
$12 x^2 - 18 x -> factor$
$(x^2 - 1)/(x^2 - 2x + 1) -> factor$`,
          caption: "Common factors, factors of degree one, repeated factors. x² + 1 has no real factors and stays.",
        },
      },
      { h2: "Play: expand, then factor" },
      {
        example: {
          code: `${PKG}
#import modifiers: expand, factor
#show: math

$(x - {{a}})(x + 2)^2 -> expand$
$(x - {{a}})(x + 2)^2 -> factor$`,
          controls: [{ kind: "slider", name: "a", label: "a", min: -3, max: 3, step: 1, value: 1 }],
          caption: "Set a = −2 and watch the repeated factor become a cube.",
        },
      },
      { h2: "What it doesn't do" },
      { md: "- **Numbers aren't split into primes**: 12 stays 12, not 2²·3.\n- **Only factors in a name**: it splits off factors of degree one in a name, common factors and repeated factors – not every irreducible polynomial.\n- **Decimals** can't be factored the way exact numbers can; write fractions (`3/10`) when it matters." },
    ],
    related: ["modifiers/expand", "modifiers/simplify", "symbolic"],
  },

  // ---------------------------------------------------------------- steps
  {
    slug: "modifiers/steps",
    title: "steps",
    group: "modifiers",
    summary: "After `->`, shows the way to the symbolic result, a line each: definitions put in, one side zero, factors, where it holds.",
    signature: "steps",
    import: "#import modifiers: steps",
    returns: "A chain of equivalent forms, joined by ⇔ (equations) or = (expressions).",
    keywords: ["steps", "derivation", "show work", "explain", "intermediate"],
    blocks: [
      { md: "For homework, reviews, or just to trust a result: `steps` shows how `->` gets there. An **equation** goes from the definitions put in, to one side zero, to factors, to where it holds. An **expression** goes from the definitions put in to its simplified form." },
      {
        example: {
          code: `${PKG}
#import modifiers: steps
#show: math

$a^2 p = a p -> steps$
$(x - 2)(x + 3) = 0 -> steps$`,
        },
      },
      {
        example: {
          title: "An expression with definitions",
          code: `${PKG}
#import modifiers: steps
#show: math

$g(x) := (x + 1)^2 - 1$
$g(2 y) -> steps$`,
          caption: "g's body with 2y put in, then simplified.",
        },
      },
      { h2: "Pitfalls" },
      { md: "- **Not with `solve`**: `steps` shows how `->` simplifies, not how `solve` solves.\n- **Not in a definition**: `$U_2 := … -> steps$` is a problem – write the expression on its own line." },
      {
        example: {
          code: `${PKG}
#import modifiers: solve, steps
#show: math

$x^2 = 4 -> solve(x), steps$`,
          expectError: true,
        },
      },
      {
        quiz: {
          question: "Which of these works?",
          options: ["`$x^2 = 4 -> solve(x), steps$`", "`$y := (x + 1)^2 -> steps$`", "`$(x + 1)^2 = 1 -> steps$`"],
          answer: 2,
          explain: "`steps` explains a `->` on an expression or an equation. It can't follow `solve`, and a definition isn't the place for it.",
        },
      },
    ],
    related: ["modifiers/simplify", "modifiers/solve", "symbolic"],
  },

  // ---------------------------------------------------------------- digits
  {
    slug: "modifiers/digits",
    title: "digits",
    group: "modifiers",
    summary: "Shows this result with n significant digits, 1 to 15.",
    signature: "digits(n)",
    import: "#import modifiers: digits",
    returns: "The same value, rounded for display.",
    keywords: ["digits", "precision", "round", "significant", "decimals"],
    blocks: [
      { md: "The worksheet's option `digits` (default 4) rounds every result. `digits(n)` overrides it for one result – after `=>`, after `->` with `solve`, or in a [`value`](/docs/functions/value). Combine it with a unit or a notation, separated by commas." },
      {
        example: {
          code: `${PKG}
#import units: V, kΩ, mA, µA
#import modifiers: digits
#show: math

$U := 5 #V$, $R := 7.2 kΩ$
$I := U / R => mA, digits({{n}})$
$I => µA, digits({{n}})$`,
          controls: [{ kind: "slider", name: "n", label: "n", min: 1, max: 15, step: 1, value: 3 }],
          caption: "Significant digits, not decimal places: 0.694 mA and 694 µA both have three.",
        },
      },
      { h2: "Combinations" },
      {
        example: {
          code: `${PKG}
#import units: kΩ, nF, Hz
#import constants: pi
#import modifiers: digits, sci, eng, solve
#show: math

$R := 10 kΩ$, $C := 47 nF$
$f := 1 / (2 pi R C) => Hz, digits(2)$
$f => sci, digits(2)$
$f / x = 3 -> solve(x), digits(5)$`,
        },
      },
      { h2: "Everywhere at once" },
      { md: "For all results, set the option: `#show: math.with(digits: 3)`. `trailing-zeros: true` keeps zeros up to the digit count (2.50 V instead of 2.5 V)." },
      {
        example: {
          code: `${PKG}
#import units: V
#import modifiers: digits
#show: math

#math(trailing-zeros: {{tz}})[$5 #V / 2 => digits(3)$]`,
          controls: [{ kind: "toggle", name: "tz", label: "trailing-zeros", value: true, on: "true", off: "false" }],
        },
      },
      { h2: "Pitfalls" },
      { md: "- **1 to 15**: doubles carry about 15 significant digits; `digits(16)` is a problem.\n- **Rounding is display only**: the next calculation uses the full value.\n- A rounded result is marked with `≈` (option `show-approximation`)." },
      {
        example: {
          code: `${PKG}
#import modifiers: digits
#show: math

$1/3 => digits(16)$`,
          expectError: true,
        },
      },
      {
        quiz: {
          question: "What does `$1234.5678 dot 1 => digits(2)$` show?",
          options: ["`1234.57`", "`1200`", "`1.2`"],
          answer: 1,
          explain: "Two *significant* digits: 1.2 thousand, written 1200. For decimals after the point, choose n to match the magnitude.",
        },
      },
    ],
    related: ["modifiers/sci", "modifiers/eng", "formatting", "reference/options"],
  },

  // ---------------------------------------------------------------- sci
  {
    slug: "modifiers/sci",
    title: "sci",
    group: "modifiers",
    summary: "Shows this result in scientific notation: one digit before the point, times a power of ten, in the unit without prefix.",
    signature: "sci",
    import: "#import modifiers: sci",
    returns: "The value as m · 10ⁿ, 1 ≤ |m| < 10.",
    keywords: ["scientific", "notation", "exponent", "power of ten", "e-notation"],
    blocks: [
      {
        example: {
          code: `${PKG}
#import units: µF, kΩ, s
#import modifiers: {{mod}}, digits
#show: math

$C := 2.2 µF$, $R := 4.7 kΩ$
$tau := R C => {{mod}}$
$2 C => {{mod}}, digits(2)$`,
          controls: [{ kind: "select", name: "mod", label: "modifier", options: ["sci", "eng"], value: "sci" }],
          caption: "`sci`: 1.034 · 10⁻² s. Switch to [`eng`](/docs/modifiers/eng): 10.34 · 10⁻³ s. Either way the prefix goes into the power of ten.",
        },
      },
      { h2: "Notes" },
      { md: "- **Calculated results.** A given value on its own (`$C => sci$` right after `$C := 2.2 µF$`) keeps the form you typed. Anything calculated – even `2 C` – is written in the notation.\n- **When it switches by itself**: with the default notation, results switch to scientific when the exponent leaves the option `notation-thresholds`, `(-3, 6)`.\n- **For every result**: `#show: math.with(notation: \"scientific\")`.\n- **Input** has no e-notation: write `1.12 dot 10^4`, not `1.12e4` (that's 1.12 times a variable e4)." },
      {
        example: {
          title: "The automatic switch",
          code: `${PKG}
#show: math

$v := 2^{{p}} =>$`,
          controls: [{ kind: "slider", name: "p", label: "power", min: 1, max: 60, step: 1, value: 24 }],
          caption: "From 2²⁰ on (above a million) the result switches to scientific without being asked.",
        },
      },
    ],
    related: ["modifiers/eng", "modifiers/digits", "formatting", "reference/options"],
  },

  // ---------------------------------------------------------------- eng
  {
    slug: "modifiers/eng",
    title: "eng",
    group: "modifiers",
    summary: "Shows this result in engineering notation: a power of ten in steps of 3, matching the SI prefixes.",
    signature: "eng",
    import: "#import modifiers: eng",
    returns: "The value as m · 10³ᵏ, 1 ≤ |m| < 1000.",
    keywords: ["engineering notation", "exponent", "power of ten", "multiple of three"],
    blocks: [
      { md: "Engineering notation reads like prefixes without the letters: 10⁻³ is milli, 10³ kilo, 10⁶ mega. Useful when a reader should compare magnitudes at a glance, or when the unit has no sensible prefix." },
      {
        example: {
          code: `${PKG}
#import units: kΩ, nF, Hz
#import constants: pi
#import modifiers: eng, sci
#show: math

$R := {{r}} kΩ$, $C := 47 nF$
$f := 1 / (2 pi R C) => eng$
$f => sci$
$f =>$`,
          controls: [{ kind: "slider", name: "r", label: "R", min: 0.1, max: 100, step: 0.1, value: 1, suffix: " kΩ" }],
          caption: "Three ways to write the same cut-off frequency.",
        },
      },
      { h2: "Notes" },
      { md: "- **With digits**: `=> eng, digits(2)`.\n- **For every result**: `#show: math.with(notation: \"engineering\")`.\n- **Prefixes instead**: with no modifier, results already get an SI prefix that fits (option `prefixes`).\n- **Given values** keep the form you typed, as with [`sci`](/docs/modifiers/sci)." },
    ],
    related: ["modifiers/sci", "modifiers/digits", "formatting", "reference/options"],
  },

  // ---------------------------------------------------------------- fraction
  {
    slug: "modifiers/fraction",
    title: "fraction",
    group: "modifiers",
    summary: "Shows this result as an exact fraction, like ⅙ A, when its denominator is small enough.",
    signature: "fraction",
    import: "#import modifiers: fraction",
    returns: "The value as a fraction, or the usual decimal when no fraction with a denominator up to `max-denominator` fits.",
    keywords: ["fraction", "exact", "rational", "ratio", "denominator"],
    blocks: [
      { md: "solveng calculates exactly where it can – `1/3 + 1/6` is ½, not 0.5000000001. `=>` rounds to decimals for display; `=> fraction` shows the exact fraction instead." },
      {
        example: {
          code: `${PKG}
#import units: A, mA
#import modifiers: fraction
#show: math

$I := 1/2 #A$
$I / {{n}} => fraction$
$I / {{n}} =>$`,
          controls: [{ kind: "slider", name: "n", label: "divide by", min: 1, max: 12, step: 1, value: 3 }],
        },
      },
      { callout: "fun", title: "0.1 + 0.2", md: "In most programming languages `0.1 + 0.2` is 0.30000000000000004. solveng reads decimals you type as exact numbers: `$0.1 + 0.2 => fraction$` is exactly 3⁄10." },
      {
        example: {
          code: `${PKG}
#import modifiers: fraction
#show: math

$0.1 + 0.2 => fraction$
$1/1024 => fraction$
#math(max-denominator: 2000, fraction-style: "slash")[$1/1024 => fraction$]`,
          caption: "1024 is above the default `max-denominator` of 1000, so the second line stays decimal. The block raises the limit and writes fractions with a slash.",
        },
      },
      { h2: "Notes" },
      { md: "- **Irrational values** have no fraction: `sqrt(2) => fraction` shows a decimal.\n- **Exact symbolic results** come from `->`, which keeps roots and π too.\n- **For every result**: `#show: math.with(number-form: \"fraction\")`." },
      {
        quiz: {
          question: "Why does `$1/1024 => fraction$` show 9.766 · 10⁻⁴?",
          options: ["1/1024 isn't exact in floating point", "Its denominator is above `max-denominator` (1000)", "`fraction` only works with units"],
          answer: 1,
          explain: "1/1024 is perfectly exact, but fractions are only shown up to the option `max-denominator`, 1000 by default. Raise it: `#math(max-denominator: 2000)[…]`.",
        },
      },
    ],
    related: ["modifiers/digits", "symbolic", "formatting", "reference/options"],
  },
];
