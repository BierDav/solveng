import type { Page } from "../types";

export const pages: Page[] = [
  // ---------------------------------------------------------------- complex
  {
    slug: "complex",
    title: "Complex numbers",
    group: "guides",
    icon: "ⅈ",
    summary: "Every value in solveng is a complex number: exact complex arithmetic, units on complex values, magnitudes and angles, and complex solutions.",
    keywords: ["imaginary", "j", "phasor", "impedance", "Re", "Im", "conj", "abs", "arg", "euler"],
    blocks: [
      { md: "You don't switch anything on for complex numbers. solveng calculates in the complex numbers all the time, so `sqrt(-4)` is `2i` and not an error. Where an operation has a real value it keeps it: the cube root of −8 is −2, not one of its complex cousins." },
      {
        example: {
          title: "Complex without asking",
          code: `#import "@preview/solveng:2.0.0": *
#show: math

$sqrt(-4) =>$, $(-8)^(1/3) =>$
$ln(-1) ->$`,
          caption: "No `i` was imported, but results still show it. `ln(-1)` is `iπ`, the principal value.",
        },
      },
      { h2: "Writing i and calculating exactly" },
      { md: "To *write* the imaginary unit, import it: `#import constants: i`. It's a single letter, so it needs a `#` in math: `3 + 4 #i`. Exact numbers stay exact, the same as real ones do, and `->` shows the exact result:" },
      {
        example: {
          title: "Exact complex arithmetic",
          code: `#import "@preview/solveng:2.0.0": *
#import constants: i, e, pi
#show: math

$(1 + 2 #i)(3 - #i) ->$
$1/(3 + 4 #i) ->$
$(1 + #i)^8 ->$
$#e^(#i pi) ->$`,
          caption: "Euler's identity comes out as exactly −1, not −1 + 1.2·10⁻¹⁶i.",
        },
      },
      {
        quiz: {
          question: "What does `$sqrt(-9) =>$` show?",
          options: ["An error: no square root of a negative number", "`3i`", "`NaN`"],
          answer: 1,
          explain: "Values are complex numbers, so the square root of −9 is `3i`. You don't need to import `i` for this; you only need the import to *write* i yourself.",
        },
      },
      { h2: "Magnitude, angle and parts" },
      { md: "`abs(z)` is the magnitude and `arg(z)` the angle in radians, from −π to π. Both are Typst's own math functions, so they need no import. `Re`, `Im` and `conj` come from `operations`:" },
      {
        example: {
          code: `#import "@preview/solveng:2.0.0": *
#import constants: i
#import operations: Re, Im, conj
#show: math

$z := 3 + 4 #i$
$abs(z) =>$, $arg(z) =>$
$Re(z) =>$, $Im(z) =>$, $conj(z) ->$
$z conj(z) ->$`,
          caption: "`z conj(z)` is `|z|²`, here 25, and it's real.",
        },
      },
      { table: { head: ["write", "means", "import"], rows: [
        ["`abs(z)`", "magnitude \\|z\\|", "none"],
        ["`arg(z)`", "angle in radians, −π … π", "none"],
        ["`Re(z)`, `Im(z)`", "real and imaginary part, both real", "`operations`"],
        ["`conj(z)`", "complex conjugate, shown with a bar", "`operations`"],
      ] } },
      { h2: "Complex values with units: impedances" },
      { md: "A complex value carries a unit like any other value: `(100 + 94.25i) Ω`. `Re` and `Im` keep the unit, so `Re(Z)` of an impedance is its resistance and `Im(Z)` its reactance.\n\nElectrical engineers write `j`. That's a question of display, not of meaning: the option `constant-displays` maps the identity `imaginary` to `j`, and every result follows." },
      {
        example: {
          title: "A series RLC circuit",
          code: `#import "@preview/solveng:2.0.0": *
#import units: Ω, kHz, mH, nF, deg
#import constants: i, pi
#show: math.with(constant-displays: (imaginary: $j$))

$f := {{f}} kHz$, $omega := 2 pi f$
$R := 50 #Ω$, $L := 10 mH$, $C := 100 nF$
$Z := R + #i omega L + 1/(#i omega C) =>$
$abs(Z) =>$, $arg(Z) => deg$`,
          controls: [{ kind: "slider", name: "f", label: "Frequency", min: 1, max: 10, step: 0.1, value: 2, suffix: " kHz" }],
          caption: "Below resonance the capacitor wins and the angle is negative; above it the coil wins.",
        },
      },
      { callout: "fun", title: "Find the resonance", md: "Drag the slider until `arg(Z)` is close to 0°. There the reactances cancel and `|Z|` is just R, 50 Ω. Check it with the formula `1/(2 pi sqrt(L C))`, about 5.03 kHz." },
      {
        example: {
          title: "Complex power",
          code: `#import "@preview/solveng:2.0.0": *
#import units: Ω, V
#import constants: i
#import operations: Re, Im, conj
#show: math.with(constant-displays: (imaginary: $j$))

$U := 230 #V$, $Z := (40 + 30 #i) #Ω$
$I := U / Z =>$
$S := U conj(I) =>$
$P := Re(S) =>$, $Q := Im(S) =>$`,
          caption: "The apparent power `S = U I*` splits into real power P and reactive power Q, each with its unit.",
        },
      },
      { h2: "Complex solutions" },
      { md: "Solving happens in the complex numbers too. A polynomial gets all of its roots, real or not:" },
      {
        example: {
          code: `#import "@preview/solveng:2.0.0": *
#import modifiers: solve
#show: math

$x^2 + 2x + 5 = 0 -> solve(x)$
$x^3 = 8 -> solve(x)$`,
          caption: "Exact roots up to degree 2; higher degrees are solved numerically, which is why −1 ± 1.732i are decimals. See [Solving](/docs/solving).",
        },
      },
      { h2: "Real names and assume" },
      { md: "A name without a value could be complex, so solveng doesn't simplify `abs(1 + e^(i x))^2` as if x were real. It shows the general result and, next to it, the simpler one that holds for real x. `assume` says the name is real and keeps only that one:" },
      {
        example: {
          code: `#import "@preview/solveng:2.0.0": *
#import constants: e, i
#import modifiers: assume
#show: math

$abs(1 + #e^(#i x))^2 ->$
$abs(1 + #e^(#i x))^2 -> assume(x in RR)$`,
        },
      },
      { h2: "Where values must be real" },
      { md: "Some things only make sense for real numbers. A complex value there is an error, not a silent guess:\n\n- **Comparisons** with `<`, `<=`, `>`, `>=`: complex numbers have no order. Compare `abs(z)`, `Re(z)` or `Im(z)` instead.\n- **Inequalities** with `solve`, and the **numeric search** for equations that can't be solved symbolically: they find real solutions.\n- **Graphs** and `compile`: plot `abs(…)`, `arg(…)`, `Re(…)` or `Im(…)` of a complex quantity." },
      {
        example: {
          title: "Plotting a complex transfer function",
          code: `#import "@preview/solveng:2.0.0": *
#import units: kHz, kΩ, nF
#import constants: i, pi
#show: math

$R := 1 kΩ$, $C := 100 nF$
#graph($H(f: kHz) = 1/(1 + #i 2 pi f R C)$, x-range: (0, 10))`,
          expectError: true,
          caption: "H is complex for every f > 0, so there's nothing real to draw. The error says what to do.",
        },
      },
      {
        example: {
          code: `#import "@preview/solveng:2.0.0": *
#import units: kHz, kΩ, nF
#import constants: i, pi
#show: math

$R := 1 kΩ$, $C := {{C}} nF$
$H(f) := 1/(1 + #i 2 pi f R C)$
$f_g := 1/(2 pi R C) =>$
#graph($G(f: kHz) = abs(H(f))$, x-range: (0, 10), xmark: $f_g$)`,
          controls: [{ kind: "slider", name: "C", label: "C", min: 20, max: 400, step: 10, value: 100, suffix: " nF" }],
          caption: "The magnitude is real. At the cutoff frequency it's always 1/√2 ≈ 0.707, whatever C is.",
        },
      },
    ],
    related: ["solving", "symbolic", "constants", "operations/Re", "operations/Im", "operations/conj", "modifiers/assume"],
  },

  // ---------------------------------------------------------------- symbolic
  {
    slug: "symbolic",
    title: "Symbolic results",
    group: "guides",
    icon: "→",
    summary: "`->` shows an exact result: defined values put in, fractions and roots kept, undefined names left as names, and equations reduced to where they hold.",
    keywords: ["exact", "simplify", "expand", "factor", "steps", "identity", "proof", "assume", "fraction", "radical", "cas"],
    blocks: [
      { md: "A trailing `=>` gives a *number*: it rounds to the worksheet's digits. A trailing `->` gives the *exact* result. Fractions stay fractions, roots stay roots, π stays π. Names that have no value stay names, so `->` works on formulas as well as on numbers." },
      {
        example: {
          title: "=> rounds, -> stays exact",
          code: `#import "@preview/solveng:2.0.0": *
#import constants: pi
#show: math

$1/3 + 1/6 =>$, $1/3 + 1/6 ->$
$sqrt({{n}}) =>$, $sqrt({{n}}) ->$
$sin(pi/3) ->$, $cos(pi/4)^2 ->$`,
          controls: [{ kind: "slider", name: "n", label: "n", min: 2, max: 200, step: 1, value: 72 }],
          caption: "Drag n: `->` pulls square factors out of the root (√72 = 6√2) and gives a whole number for perfect squares.",
        },
      },
      { h2: "What -> puts in" },
      { md: "Names defined by then are put in with their values, exactly. Names that aren't defined stay. In the voltage divider below, the resistors have values and `U_in` doesn't, so the result is a formula in `U_in` with an exact factor:" },
      {
        example: {
          code: `#import "@preview/solveng:2.0.0": *
#import units: kΩ
#show: math

$R_1 := 4 kΩ$, $R_2 := 6 kΩ$
$R_p := (R_1 R_2)/(R_1 + R_2) ->$
$U_"out" := U_"in" R_2/(R_1 + R_2) ->$`,
          caption: "3/5 of the input, exactly. If you define `U_in` later, `U_out` gets its value then.",
        },
      },
      { h2: "Simplification" },
      { md: "`->` simplifies on its own: it combines like terms, cancels common factors of polynomials and keeps exact numbers exact. You don't need the `simplify` modifier; it only spells out what `->` does anyway." },
      {
        example: {
          code: `#import "@preview/solveng:2.0.0": *
#show: math

$(x^2 - 1)/(x - 1) ->$
$(a + b)^2 - (a - b)^2 ->$
$x + 3/3 y ->$`,
        },
      },
      { md: "Two modifiers choose a different shape. `expand` multiplies out; `factor` splits into factors, the numerator and denominator of a fraction separately:" },
      {
        example: {
          code: `#import "@preview/solveng:2.0.0": *
#import modifiers: expand, factor
#show: math

$(x + 1)^3 -> expand$
$x^3 - x -> factor$
$(x^2 - 4)/(x^2 + 4x + 4) -> factor$`,
          caption: "`factor` splits off common and repeated factors and factors of degree one. It doesn't break numbers into primes, and it can't factor decimals.",
        },
      },
      { h2: "Equations show where they hold" },
      { md: "An equation after `->` isn't solved for one name. You get the conditions under which it's true: one side brought to zero, split into factors, each factor zero. When it always holds you get `true`, and when it never does, `false`." },
      {
        example: {
          code: `#import "@preview/solveng:2.0.0": *
#show: math

$x^2 = x ->$
$a^2 p = a p ->$
$x + 1 = 1 + x ->$
$x = x + 1 ->$`,
        },
      },
      {
        quiz: {
          question: "What does `$x^2 = x ->$` show?",
          options: ["`x = 1`", "`x = 0 ∨ x = 1`", "`true`"],
          answer: 1,
          explain: "Dividing both sides by x loses the solution x = 0. solveng doesn't divide: it writes `x² − x = 0`, factors it to `x (x − 1) = 0` and sets each factor to zero.",
        },
      },
      { md: "`steps` shows the way there, one line each: the equation with the definitions put in, one side zero, in factors, and where it holds." },
      {
        example: {
          code: `#import "@preview/solveng:2.0.0": *
#import modifiers: steps
#show: math

$x^2 = x -> steps$
$(x + 1)^2 - (x - 1)^2 -> steps$`,
        },
      },
      { h2: "Proving identities" },
      { md: "An equation without free values is checked for whether it *always* holds. solveng rewrites both sides in a common exponential form and tries a bounded exact proof. It never draws a conclusion from numeric samples, so `true` really means proven." },
      {
        example: {
          title: "Classic identities, proven",
          code: `#import "@preview/solveng:2.0.0": *
#import constants: e, i
#show: math

$sin(x)^2 + cos(x)^2 = 1 ->$
$#e^(#i x) = cos(x) + #i sin(x) ->$
$sin(a + b) = sin(a) cos(b) + cos(a) sin(b) ->$
$cosh(x)^2 - sinh(x)^2 = 1 ->$`,
        },
      },
      { callout: "note", title: "When there's no proof", md: "An identity solveng can't prove stays symbolic: you get the equation brought to zero, not `true` or `false`. That isn't a claim that it's wrong. The proof is bounded and works for exponentials, trigonometric and hyperbolic functions, and `sinc`." },
      { h2: "Assumptions" },
      { md: "Without a value, a name could be negative or even complex, so `sqrt(x^2)` isn't simply x. `assume` adds what you know, for this one result only. It changes no definitions." },
      {
        example: {
          code: `#import "@preview/solveng:2.0.0": *
#import modifiers: assume
#show: math

$sqrt(x^2) ->$
$sqrt(x^2) -> assume({{a}})$`,
          controls: [{ kind: "select", name: "a", label: "Assume", options: ["x > 0", "x < 0", "x >= 0"], value: "x > 0" }],
          caption: "With x < 0 the root is −x, which is positive, as a square root should be.",
        },
      },
      { md: "`assume` takes real names (`a in RR`) and signs (`x > 0`, `x >= 0`, `x < 0`, `x <= 0`). See [assume](/docs/modifiers/assume)." },
      { h2: "Putting formulas in" },
      { md: "A defined *formula* is normally kept by its name. `substitute` puts it in, and every other name stays:" },
      {
        example: {
          code: `#import "@preview/solveng:2.0.0": *
#import modifiers: substitute
#show: math

$U := R I$
$P := U^2/R -> substitute(U)$`,
          caption: "P = I² R: the power in a resistor in terms of the current.",
        },
      },
    ],
    related: ["solving", "modifiers/simplify", "modifiers/expand", "modifiers/factor", "modifiers/steps", "modifiers/assume", "modifiers/substitute", "operators"],
  },

  // ---------------------------------------------------------------- solving
  {
    slug: "solving",
    title: "Solving equations",
    group: "guides",
    icon: "x=",
    summary: "`-> solve(x)` isolates a name symbolically, or finds its value numerically, for single equations, polynomials, systems and inequalities, and shows the solutions without defining them.",
    keywords: ["solve", "roots", "equation", "system", "inequality", "sign chart", "numeric", "polynomial", "max-solutions", "at"],
    blocks: [
      { md: "Write the equation as it is, then `-> solve(x)` with the name you want. `solve` comes from `modifiers`. When the other names have no values, you get a formula:" },
      {
        example: {
          code: `#import "@preview/solveng:2.0.0": *
#import modifiers: solve
#show: math

$U = R I -> solve(R)$
$0 = 1 + 3/x + y^2 -> solve(x)$`,
        },
      },
      { md: "When the other names have values, you get a value, with the right unit. solveng works the unit of the unknown out from the equation:" },
      {
        example: {
          code: `#import "@preview/solveng:2.0.0": *
#import units: mA, mAh, h
#import modifiers: solve
#show: math

$Q_0 := {{Q}} mAh$, $I := 30 mA$
$Q_0/2 = I t -> solve(t)$`,
          controls: [{ kind: "slider", name: "Q", label: "Capacity", min: 500, max: 6000, step: 50, value: 3450, suffix: " mAh" }],
        },
      },
      { h2: "Symbolic first, numeric when needed" },
      { md: "solveng first tries to isolate the name: if the unknown occurs once it's unwrapped, polynomials are solved by formula up to degree 2, and so on. When that's not possible and every other name has a value, it searches numerically. `x = cos(x)` has no closed form, but it has a number:" },
      {
        example: {
          code: `#import "@preview/solveng:2.0.0": *
#import constants: e
#import modifiers: solve
#show: math

$x = cos(x) -> solve(x)$
$#e^x = {{c}} - x -> solve(x)$`,
          controls: [{ kind: "slider", name: "c", label: "c", min: 1, max: 10, step: 1, value: 3 }],
          caption: "The numeric search finds *real* solutions: it bisects sign changes and runs Newton's method from many starting points.",
        },
      },
      { h2: "Polynomials: every root" },
      { md: "A polynomial in the unknown gets all of its roots, complex ones included, with multiplicity. Roots of unity make a nice test:" },
      {
        example: {
          code: `#import "@preview/solveng:2.0.0": *
#import modifiers: solve
#show: math

$x^{{n}} = 1 -> solve(x)$`,
          controls: [{ kind: "slider", name: "n", label: "n", min: 2, max: 6, step: 1, value: 4 }],
          caption: "n roots for degree n, spaced evenly around the unit circle.",
        },
      },
      { md: "The option `max-solutions` (default 6) says how many are *listed*. All of them are still found:" },
      {
        example: {
          code: `#import "@preview/solveng:2.0.0": *
#import modifiers: solve
#show: math.with(max-solutions: 3)

$x^6 = 1 -> solve(x)$`,
        },
      },
      { h2: "Systems" },
      { md: "Put the equations in a `vec(…)` (or separate them by commas) and name every unknown in `solve`. Linear systems are solved by elimination, nonlinear ones numerically from many starting points. Here's a two-mesh circuit:" },
      {
        example: {
          code: `#import "@preview/solveng:2.0.0": *
#import units: V, Ω
#import modifiers: solve
#show: math

$U_0 := {{U}} #V$, $R_1 := 100 #Ω$, $R_2 := 200 #Ω$
$vec(U_0 = R_1 I_1 + R_2 dot (I_1 - I_2), 0 = R_2 dot (I_2 - I_1) + R_1 I_2) -> solve(I_1, I_2)$`,
          controls: [{ kind: "slider", name: "U", label: "U₀", min: 1, max: 24, step: 1, value: 12, suffix: " V" }],
        },
      },
      { callout: "warning", title: "A name before parentheses is a call", md: "`R_2 (I_1 - I_2)` reads as a call of a function `R_2`. Write a product with `dot`: `R_2 dot (I_1 - I_2)`, or put the factor in parentheses first: `(I_1 - I_2) R_2`." },
      {
        example: {
          title: "A circle and a line",
          code: `#import "@preview/solveng:2.0.0": *
#import modifiers: solve
#show: math

$vec(x^2 + y^2 = 25, y = x + 1) -> solve(x, y)$`,
          caption: "Two intersections, so two solutions, each a pair of values.",
        },
      },
      { h2: "Inequalities" },
      { md: "An inequality gives a *set*. solveng finds every real root and pole, checks the sign between them (a sign chart) and keeps track of open and closed ends:" },
      {
        example: {
          code: `#import "@preview/solveng:2.0.0": *
#import modifiers: solve
#show: math

$x^2 < {{c}} -> solve(x)$
$(x - 1)/(x + 2) >= 0 -> solve(x)$
$2x + 3 > 7 -> solve(x)$`,
          controls: [{ kind: "slider", name: "c", label: "c", min: -4, max: 16, step: 1, value: 9 }],
          caption: "Slide c below zero: no real x has a negative square.",
        },
      },
      {
        example: {
          title: "The sign chart, drawn",
          code: `#import "@preview/solveng:2.0.0": *
#show: math

#graph($(x - 1)/(x + 2)$, x-range: (-6, 4), y-range: (-5, 5), mark: "roots")`,
          caption: "Positive left of the pole at −2, negative between −2 and 1, positive again from 1 on. That's x < −2 ∨ x ≥ 1.",
        },
      },
      { h2: "Solutions are shown, not defined" },
      { md: "`solve` shows the solutions and **defines nothing**. An equation can have several solutions, or none, so solveng wouldn't know which one x should be. After solving, x is still undefined:" },
      {
        example: {
          code: `#import "@preview/solveng:2.0.0": *
#import modifiers: solve
#show: math

$x^2 = 4 -> solve(x)$
$x =>$`,
          expectError: true,
          caption: "The problem note tells you how to keep a solution.",
        },
      },
      {
        quiz: {
          question: "After `$I^2 R = P -> solve(I)$` with R and P defined, what is `I`?",
          options: ["The positive solution", "A vector of both solutions", "Still undefined"],
          answer: 2,
          explain: "`solve` only shows the solutions. To use one, keep them in a definition: `$I_s := I^2 R = P -> solve(I)$`, then pick one with `at(I_s, 1)`.",
        },
      },
      { md: "To keep the solutions, write a definition in front: `name := … -> solve(x)`. One solution gives a value; several give a vector, and `at(…, k)` picks one, counting from 0:" },
      {
        example: {
          code: `#import "@preview/solveng:2.0.0": *
#import operations: at
#import modifiers: solve
#show: math

$r := x^2 - x - 6 = 0 -> solve(x)$
$r_1 := at(r, 0) =>$, $r_2 := at(r, 1) =>$`,
          caption: "Use a name Typst knows in math: one letter, a subscript, or quoted text like `\"roots\"`. A bare `roots` is an unknown Typst variable.",
        },
      },
      { md: "A solved definition can be a function too. Its parameters stay open, and the solve variable is local. Graphing it draws every branch:" },
      {
        example: {
          title: "Both halves of a circle",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: at
#import modifiers: solve
#show: math

$b(x) := x^2 + b^2 = {{r2}} -> solve(b)$
$at(b(3), 1) =>$
#graph($b(x)$)`,
          controls: [{ kind: "slider", name: "r2", label: "r²", min: 1, max: 49, step: 1, value: 25 }],
          caption: "Solved definitions need a symbolic solution. Numeric search results are shown but can't be kept.",
        },
      },
      { h2: "What to expect" },
      { md: "- Isolating `sqrt(x) = 2` gives only x = 4. Periodic functions give the principal value: `tan(x) = 1` gives π/4, not every π/4 + kπ.\n- The numeric search covers about ±10¹⁵ (times the unit). It can miss roots squeezed between nearby poles or in very fast oscillations, and then says \"no real solution\".\n- Systems are solved numerically from a fixed set of starts, so solutions far from them can be missed.\n\nFor how solveng goes about it, see [How it solves](/docs/how-it-solves)." },
    ],
    related: ["modifiers/solve", "operations/at", "symbolic", "complex", "how-it-solves", "limits", "operations/polyroots"],
  },

  // ---------------------------------------------------------------- calculus
  {
    slug: "calculus",
    title: "Calculus",
    group: "guides",
    icon: "∫",
    summary: "Derivatives in Leibniz's notation, partial derivatives, antiderivatives and definite integrals with units and infinite bounds, Taylor polynomials, Jacobians and Hessians.",
    keywords: ["derivative", "diff", "dif", "partial", "integral", "integrate", "antiderivative", "quadrature", "taylor", "jacobian", "hessian", "leibniz"],
    blocks: [
      { md: "Calculus is written the way it's printed: `(dif y)/(dif x)` for a derivative, `integral_a^b … dif x` for an integral. Behind them are the operations `diff` and `integrate`, which you can also call by name (import them from `operations`)." },
      { h2: "Derivatives" },
      {
        example: {
          code: `#import "@preview/solveng:2.0.0": *
#import operations: diff
#show: math

$diff(x^3, x) ->$
$(dif x^3)/(dif x) ->$
$dif/(dif x) sin(x)^2 ->$`,
          caption: "Three spellings of differentiation. `dif/(dif x)` applies to the term right after it.",
        },
      },
      { md: "Derivatives of defined functions, higher orders and partial derivatives work the same way. Write `partial` for ∂:" },
      {
        example: {
          code: `#import "@preview/solveng:2.0.0": *
#show: math

$g(x, y) := x^2 y + sin(x y)$
$(partial g(x, y))/(partial x) ->$
$(partial^2 g(x, y))/(partial x partial y) ->$`,
        },
      },
      {
        example: {
          title: "From position to velocity, with units",
          code: `#import "@preview/solveng:2.0.0": *
#import units: m, s
#show: math

$a := 10 #m/#s^2$, $v_0 := 2 #m/#s$
$s(t) := a/2 t^2 + v_0 t$
$v(t) := (dif s(t))/(dif t) ->$
$v({{t}} #s) =>$`,
          controls: [{ kind: "slider", name: "t", label: "t", min: 0, max: 10, step: 1, value: 3, suffix: " s" }],
          caption: "The derivative's unit is the function's unit over the variable's: m/s.",
        },
      },
      { callout: "tip", title: "dif, not diff", md: "In math, `dif` is the upright d of a differential and `partial` is ∂. `diff` is the operation's name. The variable of a derivative is local, so a worksheet value for x doesn't get in the way." },
      { h2: "Antiderivatives" },
      { md: "An integral without bounds is an antiderivative, without the constant of integration. solveng finds them for polynomials, exponentials, sin, cos, sinh, cosh, ln and tan of a linear argument, for products that multiply out into those, and by parts for a polynomial times one of them:" },
      {
        example: {
          code: `#import "@preview/solveng:2.0.0": *
#import constants: e
#show: math

$integral x^2 sin(x) dif x ->$
$integral #e^(2x) dif x ->$
$integral sin(x^2) dif x ->$`,
          caption: "`sin(x²)` has no elementary antiderivative, so it stays an integral.",
        },
      },
      { h2: "Definite integrals" },
      { md: "With bounds you get a number. `->` shows the exact value, `=>` the decimal one:" },
      {
        example: {
          code: `#import "@preview/solveng:2.0.0": *
#import constants: pi
#show: math

$integral_0^3 x^2 dif x ->$
$integral_0^pi sin(x) dif x ->$
$integral_0^{{b}} sin(x) dif x ->$
$integral_0^{{b}} sin(x) dif x =>$`,
          controls: [{ kind: "slider", name: "b", label: "Upper bound", min: 1, max: 12, step: 1, value: 5 }],
          caption: "Without π in the bound the exact result is `1 − cos(b)`; `=>` gives its value.",
        },
      },
      { md: "Either bound may be infinite. Integrals over a whole line or half-line are evaluated, numerically when there's no closed form:" },
      {
        example: {
          code: `#import "@preview/solveng:2.0.0": *
#import constants: e
#show: math

$integral_0^infinity #e^(-x) dif x =>$
$integral_(-infinity)^infinity #e^(-x^2) dif x =>$`,
          caption: "The second one is √π ≈ 1.772.",
        },
      },
      { h3: "Units of the bounds" },
      { md: "Both bounds need the same unit, and the result has the integrand's unit times the variable's. A bound of 0 or ∞ fits any unit, so it takes the other bound's. When *both* are 0 or ∞, the variable gets no unit, and here that's a problem:" },
      {
        example: {
          code: `#import "@preview/solveng:2.0.0": *
#import units: A, ms, mC
#import constants: e
#show: math

$tau := 2 ms$
$Q := integral_0^infinity 3 #A #e^(-t/tau) dif t => mC$`,
          expectError: true,
          caption: "t has no unit, so `t/τ` has 1/s and can't go in an exponent.",
        },
      },
      {
        example: {
          code: `#import "@preview/solveng:2.0.0": *
#import units: A, s, ms, mC
#import constants: e
#show: math

$tau := {{tau}} ms$
$Q := integral_(0 #s)^infinity 3 #A #e^(-t/tau) dif t => mC$`,
          controls: [{ kind: "slider", name: "tau", label: "τ", min: 1, max: 10, step: 1, value: 2, suffix: " ms" }],
          caption: "Writing the unit on one bound fixes it: the charge of a discharging capacitor, 3 A · τ.",
        },
      },
      { h3: "How numbers are integrated" },
      { md: "Integrals without a closed form stay symbolic with `->` and are integrated numerically for `=>` and graphs. solveng uses adaptive Gauss–Kronrod quadrature and refines until the estimated error is at most 10⁻¹⁰ of the result. It splits integrals at the corners of `rect`, `tri` and other signals so narrow pulses aren't skipped. Samples avoid the endpoints, so integrable singularities like ln(x) at 0 are fine:" },
      {
        example: {
          code: `#import "@preview/solveng:2.0.0": *
#show: math

$integral_0^1 sin(x^2) dif x =>$
$integral_0^1 ln(x) dif x =>$`,
          caption: "The error estimate is numerical, not a proof. Very narrow features or wild oscillations can still slip between samples; an integral that doesn't converge in 20000 intervals reports an error.",
        },
      },
      { h2: "Taylor polynomials" },
      { md: "`taylor(expression, variable, center, degree)` is the Taylor polynomial through that degree (0 to 32). Drag the degree and watch it hug sin(x) further out:" },
      {
        example: {
          code: `#import "@preview/solveng:2.0.0": *
#import operations: taylor
#show: math

$p(x) := taylor(sin(x), x, 0, {{n}}) ->$
#graph($sin(x) \\ p(x)$, x-range: (-7, 7), y-range: (-2, 2))`,
          controls: [{ kind: "slider", name: "n", label: "Degree", min: 1, max: 15, step: 1, value: 5 }],
          caption: "sin is odd, so even degrees add nothing. The polynomial makes no claim about convergence.",
        },
      },
      { h2: "Jacobian and Hessian" },
      { md: "`jacobian(vector, variables)` is the matrix of first derivatives; `hessian(expression, variables)` the matrix of second derivatives. The variables vector sets the column order." },
      {
        example: {
          code: `#import "@preview/solveng:2.0.0": *
#import operations: jacobian, hessian
#show: math

$J := jacobian(vec(r cos(phi), r sin(phi)), vec(r, phi)) ->$
$det(J) ->$
$hessian(x^2 y + y^3, vec(x, y)) ->$`,
          caption: "The determinant of the polar-coordinates Jacobian is r, the r in `r dif r dif phi`.",
        },
      },
    ],
    related: ["operations/diff", "operations/integrate", "operations/taylor", "operations/jacobian", "operations/hessian", "signals", "transforms"],
  },

  // ---------------------------------------------------------------- signals
  {
    slug: "signals",
    title: "Signals",
    group: "guides",
    icon: "⎍",
    summary: "Pulses and steps (`rect`, `tri`, `sgn`), the cardinal functions `sinc`, `cosc` and `tanc`, the Dirac impulse, and exact Fourier integrals of pulses.",
    keywords: ["rect", "tri", "sgn", "sinc", "cosc", "tanc", "dirac", "pulse", "fourier", "spectrum", "step", "impulse"],
    blocks: [
      { md: "Signal operations come from `operations`: `#import operations: rect, tri, sgn, sinc`. Their arguments are unitless, so a pulse in time is written `rect(t/T)` with T in seconds." },
      {
        example: {
          code: `#import "@preview/solveng:2.0.0": *
#import operations: rect, tri, sgn
#show: math

$rect(0) =>$, $rect(1/2) =>$, $rect(1) =>$
$tri(1/4) =>$, $tri(2) =>$
$sgn(-3) =>$, $sgn(0) =>$`,
          caption: "At its edges rect is ½, halfway between the two sides.",
        },
      },
      { table: { head: ["operation", "is", "support"], rows: [
        ["`rect(x)`", "1 for \\|x\\| < ½, ½ at ±½, else 0", "width 1, centred at 0"],
        ["`tri(x)`", "max(1 − \\|x\\|, 0)", "from −1 to 1, height 1"],
        ["`sgn(x)`", "−1, 0 or 1", "everywhere"],
        ["`dirac(x)`", "unit impulse, no pointwise value", "at 0"],
      ] } },
      {
        example: {
          code: `#import "@preview/solveng:2.0.0": *
#import operations: rect, tri
#show: math

#graph($rect(t) \\ tri(t)$, x-range: (-2, 2))`,
        },
      },
      { h2: "Shifting and scaling" },
      { md: "`rect((t - t_0)/T)` is a pulse of width T centred at t₀. The same goes for `tri`, which then reaches zero at t₀ ± T. Drag the sliders:" },
      {
        example: {
          title: "A pulse you can move",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: rect, tri
#import units: ms
#show: math

$T := {{T}} ms$, $t_0 := {{t0}} ms$
#graph($p(t: ms) = rect((t - t_0)/T) \\ q(t: ms) = tri((t - t_0)/T)$, x-range: ($-10 ms$, $10 ms$))`,
          controls: [
            { kind: "slider", name: "T", label: "Width T", min: 1, max: 8, step: 0.5, value: 4, suffix: " ms" },
            { kind: "slider", name: "t0", label: "Shift t₀", min: -5, max: 5, step: 0.5, value: 2, suffix: " ms" },
          ],
          caption: "With units on the pulse, the argument `(t − t₀)/T` is unitless, as it must be.",
        },
      },
      { md: "Signals combine like any expression. Here are a rectangular and a triangular pulse in one signal:" },
      {
        example: {
          code: `#import "@preview/solveng:2.0.0": *
#import operations: rect, tri
#show: math

$x(t) := 2 rect(t - 1/2) - tri(t - {{d}})$
#graph($x(t)$, x-range: (-2, 6))
$integral_(-infinity)^infinity x(t) dif t ->$`,
          controls: [{ kind: "slider", name: "d", label: "Delay of the triangle", min: 0, max: 4, step: 0.5, value: 3 }],
          caption: "Areas add: 2 for the rectangle, −1 for the triangle, wherever it sits.",
        },
      },
      { md: "In graphs, the steps of `rect` and `sgn` break the line instead of drawing a vertical stroke. `joined: true` joins them:" },
      {
        example: {
          code: `#import "@preview/solveng:2.0.0": *
#import operations: sgn
#show: math

#graph($sgn(x)$, x-range: (-2, 2), joined: {{j}})`,
          controls: [{ kind: "toggle", name: "j", label: "joined", value: false, on: "true", off: "false" }],
        },
      },
      { h2: "sinc, cosc and tanc" },
      { md: "The cardinal functions divide by their argument and are continued at 0: `sinc(x) = sin(x)/x` with `sinc(0) = 1`, `cosc(x) = (1 − cos(x))/x` with `cosc(0) = 0`, `tanc(x) = tan(x)/x` with `tanc(0) = 1`. solveng uses the **unnormalized** sinc, without π inside." },
      {
        example: {
          code: `#import "@preview/solveng:2.0.0": *
#import operations: sinc
#show: math

#graph($sinc(x)$, x-range: (-20, 20), mark: "roots")`,
          caption: "The zeros are at multiples of π, not at the integers.",
        },
      },
      {
        quiz: {
          question: "Where is the first zero of `sinc(x)` for x > 0?",
          options: ["At x = 1", "At x = π", "sinc has no zeros"],
          answer: 1,
          explain: "solveng's `sinc(x)` is sin(x)/x, the unnormalized convention, so it's zero where sin is: π, 2π, … Some textbooks and NumPy define sinc(x) = sin(πx)/(πx), with zeros at the integers. In solveng that's `sinc(pi x)`.",
        },
      },
      {
        example: {
          code: `#import "@preview/solveng:2.0.0": *
#import operations: sinc, cosc, tanc
#show: math

$sinc(0) =>$, $cosc(0) =>$, $tanc(0) =>$
#graph($sinc(x) \\ cosc(x)$, x-range: (-15, 15))`,
        },
      },
      { h2: "Fourier integrals of pulses" },
      { md: "A Fourier integral of a pulse is solved exactly. solveng splits the integral at the pulse's corners and gives the compact `sinc` form:" },
      {
        example: {
          code: `#import "@preview/solveng:2.0.0": *
#import operations: rect, tri
#import constants: e, i, pi
#show: math

$integral_(-infinity)^infinity rect(t) #e^(-#i 2 pi f t) dif t ->$
$integral_(-infinity)^infinity tri(t) #e^(-#i 2 pi f t) dif t ->$`,
          caption: "rect becomes sinc(πf); tri, which is rect convolved with itself, becomes sinc(πf)².",
        },
      },
      {
        example: {
          title: "Short pulse, wide spectrum",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: rect
#import constants: e, i, pi
#show: math

$T := {{T}}$
$X(f) := integral_(-infinity)^infinity rect(t/T) #e^(-#i 2 pi f t) dif t ->$
#graph($X(f)$, x-range: (-4, 4))`,
          controls: [{ kind: "slider", name: "T", label: "Pulse width T", min: 0.5, max: 4, step: 0.5, value: 1 }],
          caption: "Make the pulse shorter and its spectrum gets lower and wider: the first zero is at f = 1/T.",
        },
      },
      { h2: "The Dirac impulse" },
      { md: "`dirac(x)` is a distribution with unit area and no value at a point. Asking for its value is an error, and so is plotting it:" },
      {
        example: {
          code: `#import "@preview/solveng:2.0.0": *
#import operations: dirac
#show: math

$dirac(0) =>$`,
          expectError: true,
        },
      },
      { md: "It earns its keep in convolutions, where it sifts: `convolve(f(t), dirac(t - a), t)` is `f(t - a)`. See [Transforms](/docs/transforms)." },
    ],
    related: ["operations/rect", "operations/tri", "operations/sgn", "operations/sinc", "operations/dirac", "transforms", "graphs", "calculus"],
  },

  // ---------------------------------------------------------------- transforms
  {
    slug: "transforms",
    title: "Transforms and convolution",
    group: "guides",
    icon: "ℒ",
    summary: "Laplace transforms and their inverses for step responses and transfer functions, and convolution of sequences and of functions, with Dirac sifting.",
    keywords: ["laplace", "invlaplace", "inverse laplace", "transfer function", "step response", "convolve", "convolution", "dirac", "sifting", "impulse response"],
    blocks: [
      { md: "`laplace`, `invlaplace` and `convolve` come from `operations`. All three work symbolically: they give a formula, not a table of numbers." },
      { h2: "Laplace transform" },
      { md: "`laplace(f)` is the unilateral transform of f(t) into F(s). Only the signal at t ≥ 0 counts. A worksheet shows it as ℒ{f}. Other variable names go in the second and third argument: `laplace(f, x, p)`." },
      {
        example: {
          code: `#import "@preview/solveng:2.0.0": *
#import operations: laplace
#import constants: e
#show: math

$laplace(t^2) ->$
$laplace(#e^(-a t)) ->$
$laplace(sin(omega t)) ->$
$F(p) := laplace(t #e^(-2 t), t, p) ->$`,
          caption: "Parameters such as a and ω stay formal constants. The result holds where the transform converges.",
        },
      },
      { md: "Constants, polynomials, exponentials, sine, cosine, sinh and cosh, linear combinations, exponential shifts, multiplication by powers of t, and `rect`, `tri` and `sgn` are supported. Anything else reports an error; there is no numerical transform." },
      { h2: "Inverse transform" },
      { md: "`invlaplace(F)` turns a rational function of s back into a function of t, using partial fractions where it can:" },
      {
        example: {
          code: `#import "@preview/solveng:2.0.0": *
#import operations: invlaplace
#show: math

$invlaplace(1/s^2) ->$
$invlaplace(s/(s^2 + 4)) ->$
$invlaplace(1/(s^2 + 2 s + 5)) ->$
$invlaplace(1/(s^2 dot (s + 1))) ->$`,
        },
      },
      { callout: "warning", title: "Write s dot (…)", md: "`s (s + 1)` reads as a call of a function s. Write the product as `s dot (s + 1)`, or as `(s + 1) s`." },
      { h2: "Step responses" },
      { md: "The step response of a transfer function G(s) is the inverse transform of G(s)/s. A first-order low-pass with time constant τ:" },
      {
        example: {
          title: "First order: 63 % after one τ",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: invlaplace
#show: math

$tau := {{tau}}$
$G(s) := 1/(1 + tau s)$
$h(t) := invlaplace(G(s)/s)$
$h(tau) =>$
#graph($h(t)$, x-range: (0, 10), xmark: $tau$)`,
          controls: [{ kind: "slider", name: "tau", label: "τ", min: 0.5, max: 5, step: 0.5, value: 2 }],
          caption: "Whatever τ is, after one time constant the output has reached 1 − 1/e ≈ 63 %.",
        },
      },
      {
        example: {
          title: "Second order: damping",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: invlaplace
#show: math

$zeta := {{zeta}}$
$h(t) := invlaplace(1/(s dot (s^2 + 2 zeta s + 1)))$
#graph($h(t)$, x-range: (0, 15))`,
          controls: [{ kind: "slider", name: "zeta", label: "Damping ζ", min: 0.1, max: 2, step: 0.1, value: 0.3 }],
          caption: "Below ζ = 1 the response overshoots and rings; at 1 it's critically damped; above, it creeps up. Numeric coefficients stay as ℒ⁻¹{…}, evaluated where needed.",
        },
      },
      { h3: "A transfer function from two signals" },
      { md: "Divide the transform of the output by the one of the input. Here a system turns a rectangular pulse into a triangular one. Both are shifted to start at 0, so the unilateral transform sees all of them:" },
      {
        example: {
          code: `#import "@preview/solveng:2.0.0": *
#import operations: laplace, invlaplace, rect, tri
#show: math

$x(t) := rect(t - 1/2)$, $y(t) := tri(2 dot (t - 1/2))$
$H(s) := laplace(y(t), t, s)/laplace(x(t), t, s) ->$
$g(t) := invlaplace(H(s)/s, s, t)$
#graph($g(t)$, x-range: (0, 4))`,
          caption: "The step response is a triangle wave: 0 at whole numbers, 1 at halves. The delays e^(−s/2) become causal shifts.",
        },
      },
      { h2: "Convolution of sequences" },
      { md: "`convolve(x, h)` of two vectors is the discrete convolution, with n + m − 1 elements. Here's a three-point moving sum:" },
      {
        example: {
          code: `#import "@preview/solveng:2.0.0": *
#import operations: convolve
#show: math

$x := vec(0, 0, 6, 6, 6, 0, 0)$
$y := convolve(x, vec(1, 1, 1)/3) =>$
#graph($y$)`,
          caption: "The edges of the block become ramps. `step: Δt` multiplies by the sampling step, the Riemann sum of the continuous convolution.",
        },
      },
      { h2: "Convolution of functions" },
      { md: "`convolve(f(τ), g(τ), τ, t)` is the integral of f(τ) g(t − τ) over the whole line. Piecewise polynomial signals (`rect`, `tri`, `sgn`, `abs`, steps in `cases`, polynomials) convolve exactly into a piecewise polynomial. The classic: a rectangle convolved with itself is a triangle." },
      {
        example: {
          code: `#import "@preview/solveng:2.0.0": *
#import operations: convolve, rect, tri
#show: math

$convolve(rect(t), rect(t), t) ->$
$y(t) := convolve(tri(tau), tri(tau), tau, t) ->$
$y(1/2) =>$`,
          caption: "rect ∗ rect is tri, written out piece by piece. tri ∗ tri has five pieces of degree 3.",
        },
      },
      {
        example: {
          title: "Two rectangles of different widths",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: convolve, rect
#show: math

$y(t) := convolve(rect(tau/{{a}}), rect(tau/{{b}}), tau, t)$
#graph($y(t)$, x-range: (-4, 4))`,
          controls: [
            { kind: "slider", name: "a", label: "Width a", min: 0.5, max: 3, step: 0.5, value: 2 },
            { kind: "slider", name: "b", label: "Width b", min: 0.5, max: 3, step: 0.5, value: 1 },
          ],
          caption: "Unequal widths give a trapezoid with a plateau as high as the narrower width. Equal widths make the plateau vanish into a triangle.",
        },
      },
      { md: "Other signals (exponentials, sines, symbolic shifts) stay the integral and are evaluated numerically for `=>` and graphs. Units multiply: f's unit times g's unit times τ's unit." },
      { h2: "Dirac sifting" },
      { md: "Convolving with a shifted impulse shifts the signal: `convolve(f(t), dirac(t - a), t)` is `f(t - a)`. Weighted and summed impulses work term by term:" },
      {
        example: {
          code: `#import "@preview/solveng:2.0.0": *
#import operations: convolve, dirac
#show: math

$convolve(sin(t), dirac(t - a), t) ->$
$convolve(t^2, 2 dirac(t - 1) + dirac(t + 1), t) ->$`,
          caption: "2 (t − 1)² + (t + 1)² = 3t² − 2t + 3. `dirac(b t + c)` adds the factor 1/|b|.",
        },
      },
      {
        quiz: {
          question: "What is `convolve(rect(t), dirac(t - 2), t)`?",
          options: ["`rect(2)`, which is 0", "`rect(t - 2)`: the pulse moved to t = 2", "`dirac(t - 2)`"],
          answer: 1,
          explain: "Convolution with an impulse at a doesn't evaluate f at a; it shifts f by a. Evaluating at a is what an *integral* of f times the impulse does.",
        },
      },
    ],
    related: ["operations/laplace", "operations/invlaplace", "operations/convolve", "operations/dirac", "signals", "calculus"],
  },

  // ---------------------------------------------------------------- numerics
  {
    slug: "numerics",
    title: "Numerical methods",
    group: "guides",
    icon: "≈",
    summary: "Numerical engineering operations: least squares, spectra and condition numbers, statistics, normal distributions, interpolation and fits, DFTs, minimization, ODEs, polynomial roots, special functions and temperatures, and their limits.",
    keywords: ["linear-solve", "least squares", "eigenvals", "singularvals", "cond", "mean", "median", "stdev", "variance", "pnorm", "qnorm", "interpolate", "fit", "regression", "dft", "fft", "minimize", "odesolve", "polyroots", "gamma", "erf", "besselj", "celsius", "fahrenheit"],
    blocks: [
      { md: "Most of solveng is exact. These operations are numerical on purpose: they work on measured data or problems without closed forms. They carry units, check them, and report an error instead of returning something doubtful. Import each from `operations`." },
      { table: { head: ["operation", "does", "limits"], rows: [
        ["`linear-solve(A, b)`", "A x = b, least squares for tall A", "full column rank, ≤ 256"],
        ["`eigenvals(A)`", "eigenvalues, complex too", "square, ≤ 256, unordered"],
        ["`singularvals(A)`", "all singular values, descending", "≤ 256"],
        ["`cond(A)`", "largest / smallest singular value", "singular matrices are an error"],
        ["`mean`, `median`, `variance`, `stdev`", "sample statistics (n − 1)", "matching units"],
        ["`pnorm(x, μ, σ)`, `qnorm(p, μ, σ)`", "normal distribution and its inverse", "σ > 0, 0 < p < 1"],
        ["`linear-interpolate(x, y, at)`", "straight lines between samples", "no extrapolation"],
        ["`linear-fit(x, y)`", "vec(intercept, slope) by least squares", "distinct x"],
        ["`dft`, `idft`", "discrete Fourier transform and inverse", "≤ 2048 samples"],
        ["`minimize(f, x, a, b)`", "where f is smallest in [a, b]", "unimodal f, not global"],
        ["`odesolve(f, y, t, y0, t0, t1)`", "y(t1) of dy/dt = f", "scalar, non-stiff"],
        ["`polyroots(c)`", "all complex roots, coefficients ascending", "degree ≤ 60"],
        ["`gamma`, `erf`, `besselj`", "special functions", "real arguments"],
        ["`celsius`, `fahrenheit`", "absolute temperature readings in K", "≥ absolute zero"],
      ] } },
      { callout: "note", title: "Hyphenated names in math", md: "Typst reads `linear-solve` in math as `linear` minus `solve`. Import it under a name without a hyphen: `#import operations: linear-solve as linsolve`. The worksheet shows the operation's own name." },
      { h2: "Least squares" },
      { md: "`linear-solve(A, b)` solves square systems and, for tall matrices, finds the x with the smallest residual. To fit a line u = c₀ + c₁ t to measurements, put a column of ones and a column of times side by side. Below, the matrix is built element by element over a range, as in Mathcad:" },
      {
        example: {
          code: `#import "@preview/solveng:2.0.0": *
#import operations: linear-solve as linsolve
#import units: s, V
#show: math

$t := vec(0, 1, 2, 3) #s$, $u := vec(0.1, 2.1, 3.9, 6.2) #V$
$i := 0..3$
$M[i, 0] := 1$, $M[i, 1] := t[i]$
$c := linsolve(M, u) =>$`,
          caption: "Each coefficient gets its own unit: the intercept in V, the slope in V/s.",
        },
      },
      { md: "For a straight line there's a shortcut, `linear-fit`, and `linear-interpolate` reads between the samples:" },
      {
        example: {
          code: `#import "@preview/solveng:2.0.0": *
#import operations: linear-fit as linfit, linear-interpolate as linterp
#import units: s, V
#show: math

$t := vec(0, 1, 2, 3) #s$, $u := vec(0.1, 2.1, 3.9, 6.2) #V$
$c := linfit(t, u) =>$
$linterp(t, u, {{at}} #s) =>$`,
          controls: [{ kind: "slider", name: "at", label: "at", min: 0, max: 3, step: 0.25, value: 1.5, suffix: " s" }],
          caption: "Interpolating outside the data, beyond 3 s, is an error: measured data aren't extended silently.",
        },
      },
      { h2: "Spectra and condition" },
      { md: "The Hilbert matrix, with elements 1/(i + j + 1), is the textbook example of an ill-conditioned matrix. Its condition number says how much relative errors in b can grow in x. Drag n and watch it explode:" },
      {
        example: {
          code: `#import "@preview/solveng:2.0.0": *
#import operations: cond, eigenvals, singularvals
#show: math

$n := {{n}}$
$i := 0..n-1$, $j := 0..n-1$
$H[i, j] := 1/(i + j + 1)$
$kappa := cond(H) =>$
$lambda := eigenvals(H) =>$`,
          controls: [{ kind: "slider", name: "n", label: "Size n", min: 2, max: 8, step: 1, value: 4 }],
          caption: "At n = 8 the condition is about 10¹⁰: of the roughly 15 digits floats have, only 5 survive a solve.",
        },
      },
      { md: "`eigenvals` also finds the complex eigenvalues of real matrices, in no particular order. `singularvals` gives all singular values, largest first. Entries must share one unit, and the results have it." },
      { h2: "Statistics" },
      {
        example: {
          title: "Five measured shafts",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: mean, median, stdev
#import units: mm, µm
#show: math

$d := vec(10.02, 9.98, 10.05, 9.97, 10.01) mm$
$mean(d) =>$, $median(d) =>$
$s := stdev(d) => µm$`,
          caption: "`variance` and `stdev` are sample statistics, with n − 1 in the denominator, and need at least two values.",
        },
      },
      { md: "`pnorm(x, μ, σ)` is the probability of a normal value being below x, and `qnorm(p, μ, σ)` the x below which a share p lies:" },
      {
        example: {
          code: `#import "@preview/solveng:2.0.0": *
#import operations: pnorm, qnorm
#import units: mm
#show: math

$mu := 10 mm$, $sigma := {{s}} mm$
$P := pnorm(10.1 mm, mu, sigma) - pnorm(9.9 mm, mu, sigma) =>$
$qnorm(0.975, 0, 1) =>$`,
          controls: [{ kind: "slider", name: "s", label: "σ", min: 0.01, max: 0.2, step: 0.01, value: 0.05, suffix: " mm" }],
          caption: "The share of parts within 10 ± 0.1 mm. At σ = 0.05 mm that's ±2σ, about 95 %.",
        },
      },
      { h2: "Discrete Fourier transform" },
      { md: "`dft(x)` is X[k] = Σ x[j] e^(−2πi jk/n), without normalization; `idft` divides by n and inverts it. Two cosines in 8 samples show up as two pairs of lines:" },
      {
        example: {
          code: `#import "@preview/solveng:2.0.0": *
#import operations: dft, idft
#import constants: pi
#show: math

$j := 0..7$
$x[j] := cos(2 pi j / 8) + {{a}} cos(6 pi j / 8)$
$X := dft(x) =>$
$A[j] := abs(X[j])$
#graph($A$)`,
          controls: [{ kind: "slider", name: "a", label: "Amplitude of the 3rd harmonic", min: 0, max: 2, step: 0.5, value: 0.5 }],
          caption: "A cosine of amplitude a gives two lines of height a·n/2, at k and n − k. The DFT is direct, O(n²), up to 2048 samples.",
        },
      },
      { h2: "Minimization" },
      { md: "`minimize(f, x, a, b)` returns the **position** of the minimum in [a, b], not the minimum value. To maximize, minimize −f. Cut squares of side x from the corners of a 30 cm sheet and fold up a box. Which x holds the most?" },
      {
        example: {
          code: `#import "@preview/solveng:2.0.0": *
#import operations: minimize
#import units: cm, l
#show: math

$a := {{a}} cm$
$V(x) := x dot (a - 2x)^2$
$x_"opt" := minimize(-V(x), x, 0 cm, a/2) => cm$
$V(x_"opt") => #l$`,
          controls: [{ kind: "slider", name: "a", label: "Sheet", min: 10, max: 60, step: 5, value: 30, suffix: " cm" }],
          caption: "Always a/6. Golden-section search needs a single minimum in the interval; it's not a global optimizer.",
        },
      },
      { h2: "Differential equations" },
      { md: "`odesolve(f, y, t, y0, t0, t1)` solves dy/dt = f from y(t0) = y0 and returns y(t1). y and t are local names. Here a capacitor charges through a resistor:" },
      {
        example: {
          code: `#import "@preview/solveng:2.0.0": *
#import operations: odesolve
#import units: V, s, kΩ, µF
#import constants: e
#show: math

$R := 10 kΩ$, $C := 100 µF$, $U_0 := 5 #V$
$t_1 := {{t1}} #s$
$u_1 := odesolve((U_0 - u)/(R C), u, t, 0 #V, 0 #s, t_1) =>$
$U_0 dot (1 - #e^(-t_1/(R C))) =>$`,
          controls: [{ kind: "slider", name: "t1", label: "t₁", min: 0.5, max: 5, step: 0.5, value: 1, suffix: " s" }],
          caption: "The numeric solution matches the exact one. Adaptive RK4 to a relative tolerance of 10⁻¹⁰; stiff problems and systems aren't supported.",
        },
      },
      { h2: "Polynomial roots" },
      { md: "`polyroots` takes the coefficients in **ascending** order, c₀ first, and returns all complex roots, sorted by real then imaginary part:" },
      {
        example: {
          code: `#import "@preview/solveng:2.0.0": *
#import operations: polyroots
#show: math

$polyroots(vec(-6, -1, 1)) =>$
$polyroots(vec(1, 0, 0, 1)) =>$`,
          caption: "x² − x − 6 has roots −2 and 3; x³ + 1 has −1 and two complex ones. Repeated roots are numerically ill-conditioned.",
        },
      },
      { h2: "Special functions" },
      {
        example: {
          code: `#import "@preview/solveng:2.0.0": *
#import operations: gamma, erf, besselj
#show: math

$gamma(5) =>$, $gamma(1/2) =>$
$erf(1) =>$
$besselj(0, 2.405) =>$`,
          caption: "Γ(5) = 4! and Γ(½) = √π. 2.405 is close to the first zero of J₀. All three take real, unitless arguments; `besselj` integer orders.",
        },
      },
      { h2: "Temperatures" },
      { md: "`degC` is a temperature **difference**: 20 °C is 20 K. A reading on a thermometer is converted with `celsius` or `fahrenheit`, which give an absolute temperature in kelvin:" },
      {
        example: {
          code: `#import "@preview/solveng:2.0.0": *
#import operations: celsius, fahrenheit
#import units: degC, K
#show: math

$T_1 := celsius(20) =>$, $T_2 := fahrenheit(98.6) =>$
$T_2 - T_1 => degC$
$20 degC => #K$`,
        },
      },
      {
        quiz: {
          question: "For the gas law `p V = n R T` at room temperature, which is right?",
          options: ["`T := 20 degC`", "`T := celsius(20)`", "Either, they're the same"],
          answer: 1,
          explain: "`20 degC` is a difference of 20 K. The gas law needs the absolute temperature, 293.15 K, which is what `celsius(20)` gives.",
        },
      },
    ],
    related: ["operations/linear-solve", "operations/eigenvals", "operations/cond", "operations/mean", "operations/pnorm", "operations/linear-fit", "operations/dft", "operations/minimize", "operations/odesolve", "operations/polyroots", "operations/celsius", "matrices", "limits"],
  },

  // ---------------------------------------------------------------- graphs
  {
    slug: "graphs",
    title: "Graphs",
    group: "guides",
    icon: "📈",
    summary: "`#graph` plots the math you write, picking ranges, ticks, axis labels and units itself, from one curve to data with marks, styles and several curves.",
    keywords: ["plot", "graph", "chart", "curve", "x-range", "y-range", "mark", "xmark", "ymark", "roots", "extrema", "crossings", "data", "scatter", "colors", "lines", "dots", "joined"],
    blocks: [
      { md: "`#graph($…$)` takes the same math as the rest of the worksheet. You don't have to choose anything: it finds where the curve is interesting (roots, extrema, poles, where curves cross) and rounds the range to nice ticks." },
      {
        example: {
          title: "One line",
          code: `#import "@preview/solveng:2.0.0": *
#show: math

#graph($x^3 - 3x$)`,
        },
      },
      { h2: "Units on the axes" },
      { md: "A parameter with a unit, `P(U: #V) = …`, gives the x axis its unit; the y axis takes the unit of what's plotted, with a fitting prefix. Axis labels read `U / V`:" },
      {
        example: {
          code: `#import "@preview/solveng:2.0.0": *
#import units: Ω, V
#show: math

$R := {{R}} #Ω$
#graph($P(U: #V) = U^2/R$, x-range: (0, 12))`,
          controls: [{ kind: "slider", name: "R", label: "R", min: 10, max: 200, step: 10, value: 50, suffix: " Ω" }],
          caption: "Make R small and the axis switches to a fitting unit by itself.",
        },
      },
      { md: "Defined values plot over their own scale. A charging capacitor picks a range around its time constant:" },
      {
        example: {
          code: `#import "@preview/solveng:2.0.0": *
#import units: ms, V
#import constants: e
#show: math

$tau := {{tau}} ms$
#graph($u(t: ms) = 5 #V dot (1 - #e^(-t/tau))$, x-range: (0, $5 tau$))`,
          controls: [{ kind: "slider", name: "tau", label: "τ", min: 1, max: 10, step: 1, value: 2, suffix: " ms" }],
          caption: "Ranges can be expressions of defined values, like 5τ.",
        },
      },
      { h2: "Several curves" },
      { md: "Separate curves with a backslash, `\\`. Curves of the same unit share the graph and get a legend:" },
      {
        example: {
          code: `#import "@preview/solveng:2.0.0": *
#show: math

#graph($y(x) = x^2 \\ y(x) = 2x + {{c}}$, x-range: (-3, 5), mark: auto)`,
          controls: [{ kind: "slider", name: "c", label: "Offset c", min: -1, max: 8, step: 1, value: 3 }],
          caption: "`mark: auto` marks where the curves cross. Below c = −1 they'd miss each other.",
        },
      },
      { h2: "Ranges" },
      { md: "`x-range` and `y-range` list values that must be visible. Two or more set the range from the smallest to the largest; a single one widens the range picked. Numbers are in the axis's unit, or write values with units: `x-range: ($10 #Ω$, $1 kΩ$)`." },
      {
        example: {
          code: `#import "@preview/solveng:2.0.0": *
#show: math

#graph($sin(x) \\ cos(x)$, x-range: (0, {{x1}}), y-range: (-1.5, 1.5))`,
          controls: [{ kind: "slider", name: "x1", label: "x up to", min: 2, max: 30, step: 1, value: 7 }],
        },
      },
      { h2: "Marks" },
      { md: "Marks put points on the curves, with guide lines and their coordinates on the axes:\n\n- `\"roots\"`: where a curve is zero\n- `\"extrema\"`: local maxima and minima\n- `\"crossings\"`: where curves cross each other, and `auto` for the same\n- x values (`$R$`, `2`, `$5 #V$`): a point on every curve there\n\n`mark` writes both coordinates, `xmark` only x, and `ymark` takes y values and marks every point where a curve reaches one. Combine marks in an array. The range always grows to show them." },
      {
        example: {
          code: `#import "@preview/solveng:2.0.0": *
#show: math

#graph($x^3 - 3x$, x-range: (-2.5, 2.5), mark: {{m}})`,
          controls: [{ kind: "select", name: "m", label: "mark", options: ["\"roots\"", "\"extrema\"", "(\"roots\", \"extrema\")", "1.5", "none"], value: "(\"roots\", \"extrema\")" }],
        },
      },
      {
        example: {
          title: "Where does the power reach 2 W?",
          code: `#import "@preview/solveng:2.0.0": *
#import units: Ω, W, V
#math($R := 50 #Ω$, $P_0 := {{P}} #W$, hidden: true)

#graph($P(U: #V) = U^2/R$, ymark: $P_0$)`,
          controls: [{ kind: "slider", name: "P", label: "P₀", min: 1, max: 8, step: 1, value: 2, suffix: " W" }],
          caption: "With `ymark` the y range is set around the marked value, so steep curves can leave the picture. The definitions are hidden with `hidden: true`.",
        },
      },
      {
        quiz: {
          question: "What does `#graph($cos(w) \\ w$)` mark without any `mark` option?",
          options: ["The crossing at w ≈ 0.739", "Nothing", "The roots of both curves"],
          answer: 1,
          explain: "Marks default to none. The range is still chosen around the crossing, because that's where the graph is interesting. `mark: auto` puts the point on it.",
        },
      },
      { h2: "Data" },
      { md: "A vector plots as points joined in order, with a dot at each when there are at most 30. Alone it's plotted against the index k, from 0; `y(t)` plots it over a vector t of the same length. Functions can go alongside, sampled over the same axis:" },
      {
        example: {
          title: "Measurements and a model",
          code: `#import "@preview/solveng:2.0.0": *
#import units: s, V
#import constants: e
#show: math

$t := vec(0, 1, 2, 3, 4) #s$, $u := vec(0, 3.1, 4.4, 4.7, 4.9) #V$
$U_0 := 5 #V$, $tau := {{T}} #s$
$c(t: #s) := U_0 dot (1 - #e^(-t/tau))$
#graph($u(t) \\ c(t)$, lines: (none, auto))`,
          controls: [{ kind: "slider", name: "T", label: "Model τ", min: 0.25, max: 3, step: 0.25, value: 1, suffix: " s" }],
          caption: "`lines: none` draws the measured points alone. Tune τ until the curve runs through them.",
        },
      },
      { h2: "Colours, lines, dots" },
      { md: "`colors`, `lines`, `dots` and `joined` take one value or an array, curve after curve and repeating; `auto` in the array keeps the default for that curve. The defaults (Petroff's colours; solid, dashed, dotted lines; circles, squares, triangles) stay apart for colour-blind readers and in grayscale prints." },
      {
        example: {
          code: `#import "@preview/solveng:2.0.0": *
#show: math

$x := vec(0, 1, 2, 3, 4, 5)$, $y := vec(0, 1.2, 3.8, 9.3, 15.8, 25.4)$
#graph($y(x) \\ x^2$, colors: (red, auto), dots: ("{{d}}", none), lines: (none, "{{l}}"))`,
          controls: [
            { kind: "select", name: "d", label: "dots", options: ["circle", "square", "triangle", "diamond", "star", "cross"], value: "star" },
            { kind: "select", name: "l", label: "line", options: ["solid", "dashed", "dotted", "dash-dotted"], value: "dashed" },
          ],
        },
      },
      { md: "For a whole document, set the options `graph-colors`, `graph-lines`, `graph-dots` and `graph-joined` on `math`: `#show: math.with(graph-colors: (blue, red))`. A graph's own options go over them." },
      { h2: "Piecewise functions" },
      { md: "`cases` break the line where a branch jumps, and a value that holds at a single point is drawn as a dot. `joined: true` draws through the jumps instead:" },
      {
        example: {
          code: `#import "@preview/solveng:2.0.0": *
#show: math

$f(x) := cases(x^2 & "if" x < 1, 2 & "for" x = 1, 3 - x & "otherwise")$
#graph($f(x)$, x-range: (-1, 3), joined: {{j}})`,
          controls: [{ kind: "toggle", name: "j", label: "joined", value: false, on: "true", off: "false" }],
        },
      },
      { h2: "Curves that define" },
      { md: "With `:=` a curve also defines its name, so you can calculate with it afterwards:" },
      {
        example: {
          code: `#import "@preview/solveng:2.0.0": *
#show: math

#graph($h(x) := x^2 - {{a}}$, mark: "roots")
$h(3) =>$`,
          controls: [{ kind: "slider", name: "a", label: "a", min: 1, max: 9, step: 1, value: 1 }],
        },
      },
      { h2: "More options" },
      { md: "- `width`, `height`: the whole graph, labels included (`12cm`, `100%`, `1fr`); `auto` makes the plot area 8 × 5 cm.\n- `samples`: sample points, 400 by default.\n- `x-include`: values the automatic range must contain, without a mark.\n- `variable`: which name is x, when the curves could mean more than one.\n- `x-unit`, `y-unit`: axis units, checked against the curves.\n- Every [option](/docs/reference/options) of `math` works for this graph alone, e.g. `digits: 2`.\n\nA curve that's complex where it should be drawn is an error; plot `abs(…)`, `arg(…)`, `Re(…)` or `Im(…)` of it (see [Complex numbers](/docs/complex)). All the details are on the [graph](/docs/functions/graph) page." },
    ],
    related: ["functions/graph", "signals", "solving", "complex", "reference/options", "piecewise", "vectors"],
  },
];
