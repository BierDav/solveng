import type { Page } from "../types";

export const pages: Page[] = [
  // ---------- Signals ----------
  {
    slug: "operations/rect",
    title: "rect",
    group: "operations",
    section: "Signals",
    summary: "The rectangular pulse: 1 for |x| < ½, ½ at x = ±½, and 0 otherwise.",
    signature: "rect(x)",
    import: "#import operations: rect",
    returns: "Unitless value: 0, ½ or 1.",
    keywords: ["rectangle", "pulse", "box", "window", "gate", "Π"],
    blocks: [
      { md: "`rect` is the unit pulse of signal theory: height 1, width 1, centered at 0, area 1. At its two edges it takes the mean of both sides, ½ – the value a Fourier series converges to, and what keeps `rect(t)` and `rect(-t)` the same.\n\nThe argument is unitless. Shift and stretch it to place a pulse: `rect((t - t_0)/T)` is centered at t₀ and T wide." },
      {
        example: {
          title: "Place a pulse",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: rect
#show: math

$t_0 := {{shift}}$, $T := {{width}}$
$p(t) := rect((t - t_0)/T)$
#graph($p(t)$, x-range: (-6, 6))
$integral_(-infinity)^infinity p(t) dif t ->$`,
          controls: [
            { kind: "slider", name: "shift", label: "Shift t₀", min: -3, max: 3, step: 0.5, value: 1, suffix: "" },
            { kind: "slider", name: "width", label: "Width T", min: 0.5, max: 4, step: 0.5, value: 2, suffix: "" },
          ],
          caption: "The pulse moves with t₀ and widens with T; its area is always T. The integral splits at the edges and is exact.",
        },
      },
      {
        example: {
          title: "At the edges",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: rect
#show: math

$rect(0) ->$, $rect(1/2) ->$, $rect(-1/2) ->$
$rect(0.49) =>$, $rect(0.51) =>$`,
        },
      },
      {
        quiz: {
          question: "What is `rect(1/2)`?",
          options: ["1", "½", "0"],
          answer: 1,
          explain: "At the edges `rect` takes the mean of the values on both sides: (1 + 0)/2 = ½. Graphs don't draw a dot there; integrals don't care either way.",
        },
      },
      { h2: "Its spectrum" },
      {
        example: {
          title: "Fourier transform of rect",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: rect
#import constants: pi, e, i
#show: math

$integral_(-infinity)^infinity rect(t) #e^(-#i 2 pi f t) dif t ->$`,
          caption: "The Fourier integral of a pulse is solved exactly: [sinc](/docs/operations/sinc)(πf). Narrow in time, wide in frequency.",
        },
      },
      { params: [{ name: "x", type: "expression", md: "Unitless position relative to the pulse center." }] },
    ],
    related: ["operations/tri", "operations/sinc", "operations/convolve", "signals"],
  },
  {
    slug: "operations/tri",
    title: "tri",
    group: "operations",
    section: "Signals",
    summary: "The triangular pulse: max(1 − |x|, 0), height 1 at 0, from −1 to 1.",
    signature: "tri(x)",
    import: "#import operations: tri",
    returns: "Unitless value from 0 to 1.",
    keywords: ["triangle", "Λ", "lambda", "hat", "pulse"],
    blocks: [
      { md: "`tri` is a tent: it rises from 0 at x = −1 to 1 at x = 0 and falls back to 0 at x = 1. Its area is 1, and it is what you get when you [convolve](/docs/operations/convolve) `rect` with itself.\n\nIt has three corners, at −1, 0 and 1. Integrals split there and stay exact. The first derivative is piecewise constant – 1, then −1 – and undefined at the corners. The argument is unitless: `tri((t - t_0)/T)` is centered at t₀ and reaches zero at t₀ ± T, so it is 2T wide at the base." },
      {
        example: {
          title: "Place a triangle",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: tri
#show: math

$t_0 := {{shift}}$, $T := {{width}}$
$p(t) := tri((t - t_0)/T)$
#graph($p(t)$, x-range: (-6, 6))
$integral_(-infinity)^infinity p(t) dif t ->$`,
          controls: [
            { kind: "slider", name: "shift", label: "Shift t₀", min: -3, max: 3, step: 0.5, value: 0, suffix: "" },
            { kind: "slider", name: "width", label: "Half-width T", min: 0.5, max: 3, step: 0.5, value: 2, suffix: "" },
          ],
          caption: "The base runs from t₀ − T to t₀ + T; the area is T.",
        },
      },
      {
        example: {
          title: "Values and slope",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: tri
#show: math

$tri(1/4) ->$, $tri(-1/2) ->$, $tri(1) =>$, $tri(3) =>$
$dif/(dif t) tri(t) ->$`,
          caption: "The slope is 1 on the way up and −1 on the way down; at the corners it isn't defined, so they are left out.",
        },
      },
      {
        example: {
          title: "Its spectrum",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: tri
#import constants: pi, e, i
#show: math

$integral_(-infinity)^infinity tri(t) #e^(-#i 2 pi f t) dif t ->$`,
          caption: "sinc(πf)² – the square of rect's spectrum, because tri is rect convolved with itself.",
        },
      },
      { params: [{ name: "x", type: "expression", md: "Unitless position relative to the pulse center." }] },
    ],
    related: ["operations/rect", "operations/convolve", "operations/sinc", "signals"],
  },
  {
    slug: "operations/sinc",
    title: "sinc",
    group: "operations",
    section: "Signals",
    summary: "The cardinal sine sin(x)/x, unnormalized, with sinc(0) = 1.",
    signature: "sinc(x)",
    import: "#import operations: sinc",
    returns: "Unitless number.",
    keywords: ["cardinal sine", "si", "spectrum", "unnormalized", "interpolation"],
    blocks: [
      { md: "`sinc(x)` is sin(x)/x, continued with its limit 1 at x = 0. solveng uses the **unnormalized** convention, without a factor π: its zeros are at ±π, ±2π, … Many signal-processing texts (and NumPy) mean sin(πx)/(πx) instead; write `sinc(pi x)` for that.\n\nThe argument and result are unitless; real and complex arguments work. Fourier integrals of pulses come out as `sinc`, and equalities involving it can be proven exactly." },
      {
        example: {
          title: "Values",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: sinc
#import constants: pi
#show: math

$sinc(0) ->$, $sinc(pi/2) ->$, $sinc(pi) ->$
$sinc(1) =>$`,
          caption: "Exact at multiples of π/2; zero at multiples of π.",
        },
      },
      {
        quiz: {
          question: "What does `$sinc(1) =>$` show?",
          options: ["0", "0.8415", "1"],
          answer: 1,
          explain: "sin(1)/1 = 0.8415. The normalized sinc of other tools, sin(πx)/(πx), would be 0 at x = 1 – in solveng that's `sinc(pi)`.",
        },
      },
      {
        example: {
          title: "The spectrum of a pulse",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: sinc
#import units: kHz, ms
#import constants: pi
#show: math

$T := {{T}} ms$
$S(f: kHz) := T sinc(pi f T)$
#graph($S(f)$, x-range: (-5, 5))`,
          controls: [{ kind: "slider", name: "T", label: "Pulse width T", min: 0.25, max: 2, step: 0.25, value: 1, suffix: " ms" }],
          caption: "A rectangular pulse T wide has this spectrum, with zeros at multiples of 1/T. Widen the pulse: the spectrum gets narrower and taller.",
        },
      },
      { params: [{ name: "x", type: "number", md: "Unitless argument." }] },
    ],
    related: ["operations/cosc", "operations/tanc", "operations/rect", "transforms"],
  },
  {
    slug: "operations/cosc",
    title: "cosc",
    group: "operations",
    section: "Signals",
    summary: "The cardinal cosine (1 − cos(x))/x, with cosc(0) = 0.",
    signature: "cosc(x)",
    import: "#import operations: cosc",
    returns: "Unitless number.",
    keywords: ["cardinal cosine", "odd", "spectrum"],
    blocks: [
      { md: "`cosc(x)` is (1 − cos(x))/x, continued with its limit 0 at x = 0. Where [sinc](/docs/operations/sinc) is even, `cosc` is odd: it appears in the spectra of odd pulses and in integrals of sines over an interval. Near zero it is evaluated without cancellation, so small arguments keep their digits.\n\nThe argument and result are unitless; real and complex arguments work." },
      {
        example: {
          title: "Values",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: cosc
#import constants: pi
#show: math

$cosc(0) =>$, $cosc(pi) =>$, $cosc(-pi) =>$
$cosc(10^(-6)) =>$`,
          caption: "cosc(π) = 2/π. Near zero it behaves like x/2: (1 − cos x)/x computed naively would lose every digit at 10⁻⁶.",
        },
      },
      {
        example: {
          title: "sinc and cosc",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: sinc, cosc
#show: math

#graph($sinc(x) \\ cosc(x)$, x-range: (-15, 15))`,
          caption: "sinc is even and peaks at 1; cosc is odd, passes through 0 and peaks near ±2.33.",
        },
      },
      { params: [{ name: "x", type: "number", md: "Unitless argument." }] },
    ],
    related: ["operations/sinc", "operations/tanc", "signals"],
  },
  {
    slug: "operations/tanc",
    title: "tanc",
    group: "operations",
    section: "Signals",
    summary: "The cardinal tangent tan(x)/x, with tanc(0) = 1.",
    signature: "tanc(x)",
    import: "#import operations: tanc",
    returns: "Unitless number.",
    keywords: ["cardinal tangent"],
    blocks: [
      { md: "`tanc(x)` is tan(x)/x, continued with its limit 1 at x = 0. The tangent's poles at π/2 + kπ stay poles: evaluating there is an error, and graphs break the curve.\n\nThe argument and result are unitless; real and complex arguments work." },
      {
        example: {
          title: "Values",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: tanc
#import constants: pi
#show: math

$tanc(0) =>$, $tanc(pi/4) =>$, $tanc(pi) =>$`,
          caption: "tanc(π/4) = 4/π; at π the tangent is 0 again.",
        },
      },
      {
        example: {
          title: "The graph",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: tanc
#show: math

#graph($tanc(x)$, x-range: (-5, 5), y-range: (-4, 4))`,
          caption: "Even, 1 at the origin, and a pole at ±π/2 and ±3π/2.",
        },
      },
      {
        example: {
          title: "A pole",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: tanc
#import constants: pi
#show: math

$tanc(pi/2) =>$`,
          expectError: true,
        },
      },
      { params: [{ name: "x", type: "number", md: "Unitless argument." }] },
    ],
    related: ["operations/sinc", "operations/cosc"],
  },
  {
    slug: "operations/dirac",
    title: "dirac",
    group: "operations",
    section: "Signals",
    summary: "The Dirac distribution δ(x): an impulse with unit area and no pointwise value.",
    signature: "dirac(x)",
    import: "#import operations: dirac",
    returns: "Distribution, in the reciprocal of x's unit.",
    keywords: ["delta", "δ", "impulse", "sifting", "echo"],
    blocks: [
      { md: "`dirac` is the ideal impulse: zero width, unit area. It isn't a function, so it has no value at a point – `dirac(0) =>` and plotting it are errors. It works where distributions make sense: in [convolve](/docs/operations/convolve) and in integrals over the whole line, through the **sifting rule** ∫ f(t) δ(t − a) dt = f(a).\n\nIts unit is the reciprocal of its argument's: `dirac(t)` with t in seconds is in 1/s, so its integral over time is 1.\n\n- Affine arguments work: `dirac(b t + c)` needs a known nonzero real b and contributes 1/|b|.\n- Shifts and weights can be symbolic; sums of weighted impulses and the convolution of two impulses work.\n- Nonlinear arguments, products of distributions and integrals outside convolution and the whole line are not supported." },
      {
        example: {
          title: "Shift by convolution",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: dirac, convolve, tri
#show: math

$a := {{a}}$
$y(t) := convolve(tri(tau), dirac(tau - a), tau, t) ->$
#graph($tri(t) \\ y(t)$, x-range: (-4, 5))`,
          controls: [{ kind: "slider", name: "a", label: "Delay a", min: -2, max: 3, step: 0.5, value: 2, suffix: "" }],
          caption: "Convolving with δ(t − a) delays a signal by a – exactly, with no integral to evaluate.",
        },
      },
      {
        example: {
          title: "An echo",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: dirac, convolve, rect
#show: math

$h(t) := dirac(t) + {{g}} dirac(t - 3)$
$y(t) := convolve(rect(tau), h(tau), tau, t) ->$
#graph($y(t)$, x-range: (-1, 5))`,
          controls: [{ kind: "slider", name: "g", label: "Echo gain", min: 0, max: 1, step: 0.1, value: 0.5, suffix: "" }],
          caption: "Two weighted impulses are a channel with an echo: the pulse arrives, then again 3 later, weaker.",
        },
      },
      {
        example: {
          title: "Sifting and scaling",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: dirac, convolve
#show: math

$integral_(-infinity)^infinity cos(t) dirac(t - 1) dif t ->$
$integral_(-infinity)^infinity dirac(2 t - 1) dif t ->$
$convolve(dirac(tau - 1), dirac(tau - 2), tau, t) ->$`,
          caption: "δ(2t − 1) has area ½, and two impulses convolve into one at the sum of their delays.",
        },
      },
      {
        example: {
          title: "No pointwise value",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: dirac
#show: math

$dirac(0) =>$`,
          expectError: true,
        },
      },
      { callout: "warning", md: "An infinity-valued `cases` function is *not* a Dirac distribution – solveng doesn't treat it as one. Use `dirac`." },
      { params: [{ name: "x", type: "expression", md: "Real impulse argument, optionally with units." }] },
    ],
    related: ["operations/convolve", "operations/sgn", "operations/rect", "signals"],
  },

  // ---------- Transforms ----------
  {
    slug: "operations/laplace",
    title: "laplace",
    group: "operations",
    section: "Transforms",
    summary: "The unilateral Laplace transform ℒ{f}(s) = ∫₀^∞ f(t) e^(−st) dt, symbolically.",
    signature: "laplace(expression, variable, result-variable)",
    import: "#import operations: laplace",
    returns: "Expression in the result variable.",
    keywords: ["ℒ", "transfer function", "s-domain", "transform"],
    blocks: [
      { md: "`laplace(f)` transforms a function of t into a function of s; give other names as the second and third arguments: `laplace(exp(-x), x, p)`. A worksheet shows it as ℒ{f}. The input variable is local – a worksheet value for t doesn't interfere.\n\nIt's **unilateral**: only t ≥ 0 counts. Supported are constants, polynomials (through degree 33), exponentials, sine, cosine, sinh and cosh of affine arguments, their linear combinations, exponential shifts, multiplication by powers of t (up to 20), and `rect`, `tri` and `sgn` with numeric affine arguments. Other names are constants; the result holds where the integral converges. Anything else is an error – no numerical transform is attempted." },
      {
        example: {
          title: "A table of transforms",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: laplace
#import constants: e
#show: math

$laplace(t^2) ->$
$laplace(#e^(-a t)) ->$
$laplace(sin(omega t)) ->$
$F(p) := laplace(exp(-x), x, p) ->$`,
        },
      },
      {
        example: {
          title: "Transfer function of an RC low-pass",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: laplace
#import constants: e
#show: math

$h(t) := #e^(-t/tau) / tau$
$H(s) := laplace(h(t)) ->$`,
          caption: "The impulse response e^(−t/τ)/τ becomes 1/(sτ + 1): the transfer function of a first-order low-pass.",
        },
      },
      {
        example: {
          title: "Pulses",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: laplace, rect, tri
#show: math

$laplace(rect(t)) ->$
$laplace(tri(2 t)) ->$`,
          caption: "Unilateral: only the half of `rect(t)` from 0 to ½ counts. To keep a whole pulse, shift it to start at 0, as in `rect(t - 1/2)`.",
        },
      },
      { callout: "note", md: "The result variable must differ from the input variable and must not occur in the expression. To use Λ for the triangle, define `$Lambda(t) := tri(t)$` first." },
      {
        params: [
          { name: "expression", type: "expression", md: "Function to transform." },
          { name: "variable", type: "name", default: "t", md: "Input variable." },
          { name: "result-variable", type: "name", default: "s", md: "Output variable." },
        ],
      },
    ],
    related: ["operations/invlaplace", "operations/convolve", "transforms"],
  },
  {
    slug: "operations/invlaplace",
    title: "invlaplace",
    group: "operations",
    section: "Transforms",
    summary: "The inverse unilateral Laplace transform: from F(s) back to f(t), for t ≥ 0.",
    signature: "invlaplace(expression, variable, result-variable)",
    import: "#import operations: invlaplace",
    returns: "Expression in the result variable.",
    keywords: ["ℒ⁻¹", "inverse laplace", "step response", "partial fractions"],
    blocks: [
      { md: "`invlaplace(F)` goes from s back to t; other names go in the second and third arguments: `invlaplace(1/(p + 1), p, x)`. A worksheet shows it as ℒ⁻¹{F}. Results describe the causal function for t ≥ 0.\n\nProper rational functions are inverted symbolically with partial fractions, repeated poles and quadratic factors. What can't be written in closed form – a polynomial with numeric coefficients whose roots aren't nice, delays e^(−ds) – stays written as `invlaplace(…)` and is still **evaluated and graphed** numerically, through a matrix exponential. Polynomial parts that would need Dirac impulses are not supported." },
      {
        example: {
          title: "Closed forms",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: invlaplace
#show: math

$invlaplace(1/s^2) ->$
$invlaplace(1/(s^2 + 2 s + 5)) ->$
$f(x) := invlaplace(1/(p + 1), p, x) ->$`,
          caption: "A ramp, a damped sine, an exponential.",
        },
      },
      {
        example: {
          title: "Step response of a second-order system",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: invlaplace
#show: math

$zeta := {{zeta}}$
$G(s) := 1/(s^2 + 2 zeta s + 1)$
$h(t) := invlaplace(G(s)/s, s, t)$
#graph($h(t)$, x-range: (0, 15))`,
          controls: [{ kind: "slider", name: "zeta", label: "Damping ζ", min: 0.1, max: 2, step: 0.1, value: 0.3, suffix: "" }],
          caption: "Dividing by s is the step input. Small ζ overshoots and rings, ζ = 1 is critically damped, larger ζ creeps up slowly.",
        },
      },
      {
        example: {
          title: "A system from its input and output",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: laplace, invlaplace, rect, tri
#show: math

$x(t) := rect(t - 1/2)$
$y(t) := tri(2 (t - 1/2))$
$H(s) := laplace(y(t), t, s) / laplace(x(t), t, s) ->$
$g(t) := invlaplace(H(s)/s, s, t)$
#graph($g(t)$, x-range: (0, 4))`,
          caption: "A system turns a unit pulse into a triangle. Its step response, with delays in H(s), is a triangle wave: 0 at whole numbers, 1 halfway.",
        },
      },
      { callout: "tip", title: "Write products with a dot", md: "`1/(s (s + 1))` is read as a call of s and isn't supported; write `1/(s dot (s + 1))` or multiply out, `1/(s^2 + s)`. Likewise call functions with plain names inside the transform – `G(s)`, not `H_2(s)`." },
      {
        params: [
          { name: "expression", type: "expression", md: "Expression in the transform domain." },
          { name: "variable", type: "name", default: "s", md: "Input variable." },
          { name: "result-variable", type: "name", default: "t", md: "Output variable." },
        ],
      },
    ],
    related: ["operations/laplace", "operations/convolve", "transforms", "graphs"],
  },
  {
    slug: "operations/convolve",
    title: "convolve",
    group: "operations",
    section: "Transforms",
    summary: "Convolution of two sequences, or of two functions over the whole line: (f ∗ g)(t) = ∫ f(τ) g(t − τ) dτ.",
    signature: "convolve(f, g, variable, at, step: none)",
    import: "#import operations: convolve",
    returns: "A vector (sequences) or an expression (functions).",
    keywords: ["convolution", "∗", "filter", "impulse response", "moving average"],
    blocks: [
      { md: "Convolution is what a linear, time-invariant system does to its input: the output is the input convolved with the impulse response.\n\n- **Sequences** (vectors): `convolve(x, h)` is the discrete convolution, (x ∗ h)ₖ = Σᵢ xᵢ hₖ₋ᵢ, with n + m − 1 elements. `step: Δt` multiplies it by the sampling step, the Riemann sum of the continuous convolution.\n- **Functions**: `convolve(f(τ), g(τ), τ, t)` integrates over the whole line, τ local. Without `at`, the result is in τ itself.\n\nPiecewise polynomial signals – `rect`, `tri`, `sgn`, `abs`, `cases` steps and polynomials, with numeric affine arguments – convolve **exactly** into a piecewise polynomial. Other signals (exponentials, sines, symbolic shifts) stay the integral and are evaluated numerically for `=>` and graphs. [Dirac impulses](/docs/operations/dirac) use the sifting rule. Units multiply: f's unit times g's times τ's." },
      {
        example: {
          title: "rect ∗ rect = tri",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: convolve, rect, tri
#show: math

$y(t) := convolve(rect(tau), rect(tau), tau, t) ->$
$y(1/2) ->$, $tri(1/2) ->$
#graph($rect(t) \\ y(t)$, x-range: (-2, 2))`,
          caption: "Slide one unit pulse across the other: the overlap grows linearly, then shrinks – the triangle, piece by piece.",
        },
      },
      {
        example: {
          title: "From triangle to trapezoid",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: convolve, rect
#show: math

$w := {{w}}$
$y(t) := convolve(rect(tau), rect(tau / w), tau, t) ->$
#graph($y(t)$, x-range: (-3, 3))`,
          controls: [{ kind: "slider", name: "w", label: "Second width w", min: 0.5, max: 4, step: 0.5, value: 2, suffix: "" }],
          caption: "Convolving pulses of different widths gives a trapezoid: its top is as long as the difference of the widths. At w = 1 it's the triangle again.",
        },
      },
      {
        example: {
          title: "tri ∗ tri",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: convolve, tri
#show: math

$y(t) := convolve(tri(tau), tri(tau), tau, t) ->$
$y(1/2) ->$`,
          caption: "Five cubic pieces – a cubic B-spline. Each convolution with a pulse smooths one more derivative.",
        },
      },
      {
        example: {
          title: "A pulse through an RC filter",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: convolve, rect, sgn
#import constants: e
#show: math

$h(t) := #e^(-t) (1 + sgn(t))/2$
$y(t) := convolve(rect(tau - 1/2), h(tau), tau, t)$
$y(1) =>$
#graph($rect(t - 1/2) \\ y(t)$, x-range: (-1, 5))`,
          caption: "The impulse response of a low-pass is a decaying exponential from t = 0. The exponential isn't piecewise polynomial, so the integral is evaluated numerically – for the value and for every point of the graph.",
        },
      },
      {
        example: {
          title: "Sequences: a moving average",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: convolve
#import units: ms
#show: math

$x := vec(1, 1, 1)$, $h := vec(1, 0.5, 0.25)$
$convolve(x, h) =>$
$convolve(x, h, step: 1 ms) =>$`,
          caption: "Three plus three samples give five. With a step, the sum becomes an integral's approximation and carries the step's unit.",
        },
      },
      {
        params: [
          { name: "f", type: "expression", md: "A sequence (vector), or a function of the variable." },
          { name: "g", type: "expression", md: "A sequence (vector), or a function of the variable." },
          { name: "variable", type: "name", md: "The integration variable of a convolution of functions." },
          { name: "at", type: "expression", default: "the variable", md: "Where the convolution of functions is taken, usually the result's variable." },
          { name: "step", type: "expression", md: "Named: the sampling step of sequences; the result is multiplied by it." },
        ],
      },
      { md: "Discrete convolution allows at most 1,000,000 coefficient products; larger requests report a work limit." },
    ],
    related: ["operations/rect", "operations/tri", "operations/dirac", "operations/laplace", "transforms"],
  },
];
