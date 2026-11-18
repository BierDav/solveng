import type { Page } from "../types";

export const pages: Page[] = [
  // ---------- Elementary ----------
  {
    slug: "operations/asin",
    title: "asin",
    group: "operations",
    section: "Elementary",
    summary: "Inverse sine: the angle in radians, from −π/2 to π/2, whose sine is x.",
    signature: "asin(x)",
    import: "#import operations: asin",
    returns: "Unitless angle in radians; the principal complex value outside [−1, 1].",
    keywords: ["arcsin", "inverse sine", "angle"],
    blocks: [
      { md: "`asin` undoes `sin` on its principal branch: the result always lies between −π/2 and π/2, so the angle of a right triangle from its opposite side and hypotenuse comes out directly. Typst's own spelling `arcsin` works too.\n\nAt the special values the result stays exact, as a multiple of π; `=>` gives the number." },
      {
        example: {
          title: "Exact where it can be",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: asin
#show: math

$asin(1/2) ->$, $asin(1) ->$, $arcsin(-1/2) ->$
$asin(1/3) =>$`,
          caption: "`->` keeps the exact angle, `=>` shows radians as a decimal.",
        },
      },
      {
        example: {
          title: "The angle of a ramp",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: asin
#import units: m, deg
#show: math

$h := {{h}} #m$, $L := 5 #m$
$alpha := asin(h/L) => deg$`,
          controls: [{ kind: "slider", name: "h", label: "Rise h", min: 0, max: 5, step: 0.1, value: 1.2, suffix: " m" }],
          caption: "A ramp of length L rising by h. Drag h up to L: the angle reaches 90°.",
        },
      },
      { h2: "Outside [−1, 1]" },
      { md: "No real angle has a sine of 2, but a complex one does. Like every operation in solveng, `asin` returns the principal complex value there instead of failing:" },
      {
        example: {
          code: `#import "@preview/solveng:2.0.0": *
#import operations: asin
#show: math

$asin(2) =>$`,
          caption: "π/2 − 1.317i: its sine is 2. A graph or a comparison would report the complex value as a problem.",
        },
      },
      { params: [{ name: "x", type: "expression", md: "Unitless sine value." }] },
    ],
    related: ["operations/acos", "operations/atan", "operations/atan2", "complex"],
  },
  {
    slug: "operations/acos",
    title: "acos",
    group: "operations",
    section: "Elementary",
    summary: "Inverse cosine: the angle in radians, from 0 to π, whose cosine is x.",
    signature: "acos(x)",
    import: "#import operations: acos",
    returns: "Unitless angle in radians; the principal complex value outside [−1, 1].",
    keywords: ["arccos", "inverse cosine", "angle", "power factor"],
    blocks: [
      { md: "`acos` undoes `cos` on its principal branch, from 0 to π – so unlike `asin` it tells an obtuse angle from an acute one. Typst's spelling `arccos` works too. Special values stay exact; outside [−1, 1] the result is complex." },
      {
        example: {
          title: "Exact angles",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: acos
#show: math

$acos(1/2) ->$, $acos(0) ->$, $acos(-1) ->$
$arccos(-1/2) ->$`,
        },
      },
      {
        example: {
          title: "Phase angle from the power factor",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: acos
#import units: deg, kW
#show: math

$P := 4.2 kW$, $S := {{S}} kW$
$lambda := P / S =>$
$phi := acos(lambda) => deg$`,
          controls: [{ kind: "slider", name: "S", label: "Apparent power S", min: 4.2, max: 12, step: 0.1, value: 5, suffix: " kVA" }],
          caption: "Real power P and apparent power S give the power factor λ = cos φ; `acos` turns it back into φ, the angle between voltage and current.",
        },
      },
      { params: [{ name: "x", type: "expression", md: "Unitless cosine value." }] },
    ],
    related: ["operations/asin", "operations/atan", "operations/atan2", "complex"],
  },
  {
    slug: "operations/atan",
    title: "atan",
    group: "operations",
    section: "Elementary",
    summary: "Inverse tangent: the angle in radians, between −π/2 and π/2, whose tangent is x.",
    signature: "atan(x)",
    import: "#import operations: atan",
    returns: "Unitless angle in radians.",
    keywords: ["arctan", "inverse tangent", "angle", "slope"],
    blocks: [
      { md: "`atan` takes a slope and gives its angle. It is defined for every real number and never reaches ±π/2. Typst's spelling `arctan` works too.\n\nA slope is a ratio, so `atan` can't tell (−3, −4) from (3, 4): both have the slope 4/3. For the angle of a vector or a phasor use [atan2](/docs/operations/atan2), which looks at both signs." },
      {
        example: {
          title: "Exact angles",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: atan
#show: math

$atan(1) ->$, $atan(-1) ->$, $atan(2) =>$`,
        },
      },
      {
        example: {
          title: "Phase of an RC low-pass",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: atan
#import units: kΩ, nF, kHz, deg
#import constants: pi
#show: math

$R := 10 kΩ$, $C := 10 nF$
$p(f: kHz) := -atan(2 pi f R C)$
$p(1 kHz) => deg$
#graph($p(f)$, x-range: (0, 10))`,
          caption: "The phase p of the output lags by 45° at the corner frequency 1/(2πRC) ≈ 1.6 kHz and approaches −90° above it.",
        },
      },
      { params: [{ name: "x", type: "expression", md: "Unitless tangent value." }] },
    ],
    related: ["operations/atan2", "operations/asin", "operations/acos", "graphs"],
  },
  {
    slug: "operations/atan2",
    title: "atan2",
    group: "operations",
    section: "Elementary",
    summary: "The quadrant-aware angle of the point (x, y): radians from −π to π.",
    signature: "atan2(y, x)",
    import: "#import operations: atan2",
    returns: "Unitless angle in radians, in [−π, π].",
    keywords: ["angle", "quadrant", "phase", "direction", "arctan2"],
    blocks: [
      { md: "`atan2(y, x)` is the angle from the positive x axis to the point (x, y). Where `atan(y/x)` only sees the ratio, `atan2` sees both signs and lands in the right quadrant. Mind the order: **y first**, as in most programming languages.\n\nBoth arguments must be real and have matching units – they cancel, so forces, voltages or lengths work alike. The origin (0, 0) has no direction and is an error." },
      {
        example: {
          title: "The direction of a force",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: atan, atan2
#import units: N, deg
#show: math

$F_x := {{fx}} #N$, $F_y := 4 #N$
$phi := atan2(F_y, F_x) => deg$
$atan(F_y / F_x) => deg$`,
          controls: [{ kind: "slider", name: "fx", label: "Fₓ", min: -6, max: 6, step: 1, value: -3, suffix: " N" }],
          caption: "With Fₓ negative the force points up and to the left: `atan2` says 126.9°, `atan` of the ratio says −53.13° – the opposite direction.",
        },
      },
      {
        example: {
          title: "Exact angles in every quadrant",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: atan2
#show: math

$atan2(1, 1) ->$, $atan2(1, -1) ->$
$atan2(-1, -1) ->$, $atan2(0, -1) ->$`,
        },
      },
      {
        quiz: {
          question: "A current phasor has the real part −2 A and the imaginary part −2 A. Which gives its angle?",
          options: ["`atan(-2/-2)`, that is π/4", "`atan2(-2, -2)`, that is −3π/4", "Both are the same"],
          answer: 1,
          explain: "The ratio −2/−2 is 1, so `atan` says π/4 – the direction of (2, 2). `atan2` sees both signs negative: the phasor points into the third quadrant, at −3π/4. (`arg(z)` of the complex number gives the same.)",
        },
      },
      {
        example: {
          title: "The origin has no angle",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: atan2
#show: math

$atan2(0, 0) =>$`,
          expectError: true,
        },
      },
      {
        params: [
          { name: "y", type: "expression", md: "Vertical component." },
          { name: "x", type: "expression", md: "Horizontal component, in the unit of y." },
        ],
      },
    ],
    related: ["operations/atan", "complex", "operations/Re", "operations/Im"],
  },
  {
    slug: "operations/min",
    title: "min",
    group: "operations",
    section: "Elementary",
    summary: "The smallest of one or more real values with matching units.",
    signature: "min(..values)",
    import: "#import operations: min",
    returns: "The smallest value, with its unit.",
    keywords: ["minimum", "smallest", "clamp", "limit"],
    blocks: [
      { md: "`min` compares any number of real values and returns the smallest. Units must match in dimension, not in prefix: `min(3 V, 500 mV)` compares 3 V with 0.5 V. Comparing volts with amperes is an error, as it should be. Complex values have no order and are an error too.\n\nTo pick the smallest element of a vector, write its elements out." },
      {
        example: {
          title: "Basics",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: min
#import units: V, mV
#show: math

$min(3, -2, 7) =>$
$min(3 #V, 500 mV) =>$`,
        },
      },
      {
        example: {
          title: "A clamp",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: min, max
#import units: V
#show: math

$U_"in" := {{u}} #V$
$U_"out" := max(0 #V, min(U_"in", 3.3 #V)) =>$`,
          controls: [{ kind: "slider", name: "u", label: "Input", min: -2, max: 6, step: 0.1, value: 2.5, suffix: " V" }],
          caption: "`min` caps at the supply, `max` at ground: the input of a 3.3 V ADC.",
        },
      },
      {
        example: {
          title: "Units must match",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: min
#import units: V, A
#show: math

$min(3 #V, 2 #A) =>$`,
          expectError: true,
        },
      },
      { params: [{ name: "..values", type: "expression", md: "One or more real values to compare, with matching units." }] },
    ],
    related: ["operations/max", "operations/floor", "comparisons", "piecewise"],
  },
  {
    slug: "operations/max",
    title: "max",
    group: "operations",
    section: "Elementary",
    summary: "The largest of one or more real values with matching units.",
    signature: "max(..values)",
    import: "#import operations: max",
    returns: "The largest value, with its unit.",
    keywords: ["maximum", "largest", "clamp", "rectifier"],
    blocks: [
      { md: "`max` compares any number of real values and returns the largest. Units must match in dimension; prefixes are fine. Complex values have no order and are an error.\n\n`max(x, 0)` is a half-wave rectifier, a ramp, a ReLU – one of the most useful shapes in engineering formulas." },
      {
        example: {
          title: "Basics",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: max
#import units: mA, A
#show: math

$max(3, -2, 7) =>$
$max(0.2 #A, 350 mA) =>$`,
        },
      },
      {
        example: {
          title: "A half-wave rectifier",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: max
#import units: V, ms
#import constants: pi
#show: math

$U_F := {{uf}} #V$, $U_m := 5 #V$, $T := 20 ms$, $U_0 := 0 #V$
$u(t: ms) := max(U_m sin(2 pi t / T) - U_F, U_0)$
#graph($u(t)$, x-range: (0, 40))`,
          controls: [{ kind: "slider", name: "uf", label: "Diode drop", min: 0, max: 2, step: 0.1, value: 0.7, suffix: " V" }],
          caption: "A 50 Hz sine through a diode with forward voltage U_F: only the part above U_F passes.",
        },
      },
      { params: [{ name: "..values", type: "expression", md: "One or more real values to compare, with matching units." }] },
    ],
    related: ["operations/min", "operations/sgn", "piecewise", "signals"],
  },
  {
    slug: "operations/floor",
    title: "floor",
    group: "operations",
    section: "Elementary",
    summary: "The greatest integer less than or equal to x – rounding down.",
    signature: "floor(x)",
    import: "#import operations: floor",
    returns: "Integer.",
    keywords: ["round down", "integer part", "staircase", "quantize"],
    blocks: [
      { md: "`floor` rounds down, towards −∞: `floor(2.7)` is 2 and `floor(-2.7)` is −3 (not −2). The argument is unitless – divide a quantity by a unit or by another quantity first, which also says what you count in. Typst's `⌊x⌋` is read as `floor` too.\n\nIts graph is a staircase: graphs break the line at each step. Its derivative stays symbolic." },
      {
        example: {
          title: "How many fit",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: floor
#import units: m
#show: math

$L := {{L}} #m$, $l := 2 #m$
$n := floor(L / l) =>$
$"rest" := L - n l =>$`,
          controls: [{ kind: "slider", name: "L", label: "Rod length L", min: 1, max: 12, step: 0.1, value: 7.3, suffix: " m" }],
          caption: "How many 2 m pieces can be cut from a rod, and what's left.",
        },
      },
      {
        example: {
          title: "A quantizer",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: floor
#import units: V, mV
#show: math

$q := {{q}} mV$
$Q(u: #V) := q floor(u / q)$
#graph($Q(u) \\ u$, x-range: (0, 2))`,
          controls: [{ kind: "slider", name: "q", label: "Step q", min: 100, max: 500, step: 50, value: 250, suffix: " mV" }],
          caption: "An ideal ADC rounds down to multiples of its step q. The staircase stays below the line u.",
        },
      },
      { params: [{ name: "x", type: "number", md: "Unitless real value." }] },
    ],
    related: ["operations/ceil", "operations/number", "sums", "piecewise"],
  },
  {
    slug: "operations/ceil",
    title: "ceil",
    group: "operations",
    section: "Elementary",
    summary: "The least integer greater than or equal to x – rounding up.",
    signature: "ceil(x)",
    import: "#import operations: ceil",
    returns: "Integer.",
    keywords: ["ceiling", "round up", "integer"],
    blocks: [
      { md: "`ceil` rounds up, towards +∞: `ceil(2.1)` is 3 and `ceil(-2.1)` is −2. The argument is unitless. Typst's math writes it `⌈x⌉`, which solveng reads as well. Its derivative stays symbolic.\n\nRounding up is what you need whenever *enough* matters: how many trucks, bolts, cable drums." },
      {
        example: {
          title: "Rounding up and down",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: ceil, floor
#show: math

$ceil(2.1) =>$, $ceil(-2.1) =>$, $ceil(3) =>$
$floor(2.1) =>$, $floor(-2.1) =>$`,
        },
      },
      {
        example: {
          title: "How many bolts",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: ceil
#import units: kN
#show: math

$F := {{F}} kN$, $F_"bolt" := 12 kN$
$n := ceil(F / F_"bolt") =>$`,
          controls: [{ kind: "slider", name: "F", label: "Load F", min: 5, max: 100, step: 1, value: 50, suffix: " kN" }],
          caption: "50 kN over bolts that carry 12 kN each needs 5 bolts, not 4.17.",
        },
      },
      { params: [{ name: "x", type: "number", md: "Unitless real value." }] },
    ],
    related: ["operations/floor", "operations/number"],
  },
  {
    slug: "operations/factorial",
    title: "factorial",
    group: "operations",
    section: "Elementary",
    summary: "The factorial k! = 1 · 2 · … · k of a nonnegative integer.",
    signature: "factorial(k)",
    import: "#import operations: factorial",
    returns: "Number: exact through 33!, a float through 170!.",
    keywords: ["k!", "permutations", "combinations", "binomial"],
    blocks: [
      { md: "`factorial(k)` multiplies the integers from 1 to k; `factorial(0)` is 1, the empty product. Write it as a call – the `!` sign isn't read as factorial.\n\nValues through 33! are exact integers; above that they are floats with about 15 digits, up to 170! ≈ 7.3 · 10³⁰⁶, the largest that fits in a float. For non-integers use [gamma](/docs/operations/gamma): `gamma(k + 1)` is `factorial(k)`." },
      {
        example: {
          title: "Counting",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: factorial
#show: math

$factorial(0) =>$, $factorial(5) =>$, $factorial(20) =>$
$n := {{n}}$, $k := 5$
$factorial(n) / (factorial(k) factorial(n - k)) =>$`,
          controls: [{ kind: "slider", name: "n", label: "n", min: 5, max: 60, step: 1, value: 52, suffix: "" }],
          caption: "The binomial coefficient: there are 2 598 960 ways to pick 5 cards from 52.",
        },
      },
      {
        example: {
          title: "Taylor coefficients",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: factorial
#show: math

$1/factorial(1) - 1/factorial(3) + 1/factorial(5) - 1/factorial(7) =>$
$sin(1) =>$`,
          caption: "The factorials of the sine's series: four terms already agree to five digits.",
        },
      },
      { params: [{ name: "k", type: "integer", md: "Nonnegative unitless integer." }] },
    ],
    related: ["operations/gamma", "operations/prod", "operations/taylor", "sums"],
  },
  {
    slug: "operations/sgn",
    title: "sgn",
    group: "operations",
    section: "Elementary",
    summary: "The sign function: −1 for negative x, 0 at zero, 1 for positive x.",
    signature: "sgn(x)",
    import: "#import operations: sgn",
    returns: "Unitless −1, 0 or 1.",
    keywords: ["sign", "signum", "step", "friction"],
    blocks: [
      { md: "`sgn(x)` is the sign of x: −1, 0 or 1. The argument is unitless, so divide a quantity by its unit first – or use the sign of a ratio. Graphs break the line at the jump, and integrals split there exactly. Its derivative (2 δ(x)) stays symbolic.\n\nA unit step is `(1 + sgn(t))/2`: 0 before, 1 after, ½ right at 0." },
      {
        example: {
          title: "Values",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: sgn
#show: math

$sgn(-3) =>$, $sgn(0) =>$, $sgn(1/1000) =>$
$integral_(-2)^3 sgn(x) dif x ->$`,
          caption: "The integral splits at 0: −2 + 3 = 1.",
        },
      },
      {
        example: {
          title: "A square wave from a sine",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: sgn
#import units: V, ms
#import constants: pi
#show: math

$a := {{a}}$, $U_m := 5 #V$, $T := 10 ms$
$u(t: ms) := U_m sgn(sin(2 pi t / T) - a)$
#graph($u(t)$, x-range: (0, 20))`,
          controls: [{ kind: "slider", name: "a", label: "Threshold a", min: -0.9, max: 0.9, step: 0.1, value: 0, suffix: "" }],
          caption: "The sign of a sine is a square wave. Comparing against a threshold a instead of 0 changes the duty cycle – a comparator in one line.",
        },
      },
      { params: [{ name: "x", type: "expression", md: "Unitless value whose sign to take." }] },
    ],
    related: ["operations/rect", "operations/dirac", "piecewise", "signals"],
  },
  {
    slug: "operations/number",
    title: "number",
    group: "operations",
    section: "Elementary",
    summary: "A value as a plain number in a given unit: `number(U, mV)` is U in millivolts, without a unit.",
    signature: "number(value, unit)",
    import: "#import operations: number",
    returns: "Unitless number.",
    keywords: ["strip unit", "magnitude", "empirical formula", "numeric value"],
    blocks: [
      { md: "Empirical formulas are written for numbers in particular units: *d in millimetres*, *v in km/h*. `number(d, mm)` is exactly that number. The value must have the unit's dimension, so a length can't sneak in where a mass was meant.\n\nDividing by the unit, `U / mV`, gives the same number but lets anything through: `U / mV` of a current is a quiet mistake, `number(I, mV)` is an error." },
      {
        example: {
          title: "An empirical formula",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: number
#import units: mm, N, cm
#show: math

$d := {{d}} mm$
$F := 3.2 number(d, mm)^0.6 #N =>$
$number(d, #cm) =>$`,
          controls: [{ kind: "slider", name: "d", label: "d", min: 1, max: 40, step: 1, value: 12, suffix: " mm" }],
          caption: "The formula wants d in millimetres, whatever unit d was given in: `number` converts first.",
        },
      },
      {
        example: {
          title: "The dimension must fit",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: number
#import units: V, mV, A
#show: math

$U := 2.5 #V$
$number(U, mV) =>$
$number(U, #A) =>$`,
          expectError: true,
          caption: "A voltage has no number in amperes.",
        },
      },
      {
        params: [
          { name: "value", type: "expression", md: "The value, with a unit." },
          { name: "unit", type: "expression", md: "The unit to express it in: a unit binding or an expression of them (`#m / #s`)." },
        ],
      },
    ],
    related: ["units", "custom-units", "operations/celsius"],
  },
  {
    slug: "operations/celsius",
    title: "celsius",
    group: "operations",
    section: "Elementary",
    summary: "An absolute temperature from a Celsius reading, in kelvin: `celsius(20)` is 293.15 K.",
    signature: "celsius(x)",
    import: "#import operations: celsius",
    returns: "Temperature in kelvin.",
    keywords: ["temperature", "degC", "°C", "kelvin", "absolute"],
    blocks: [
      { md: "A thermometer reading in °C is not a quantity you can multiply: 20 °C is not twice 10 °C. `celsius(reading)` turns the reading into the absolute temperature, reading + 273.15 K, which gas laws and radiation formulas need.\n\nThe unit `degC` is something else: a temperature *difference*, the same size as a kelvin. `20 degC` is 20 K, not 293.15 K. Subtracting two `celsius` values gives a difference too. Readings below −273.15 are rejected." },
      {
        example: {
          title: "Ideal gas",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: celsius
#import units: J, mol, K, L, kPa
#show: math

$R := 8.314 #J / (mol #K)$
$n := 1 mol$, $V := 22.4 #L$
$p := n R celsius({{T}}) / V => kPa$`,
          controls: [{ kind: "slider", name: "T", label: "Reading", min: -50, max: 200, step: 5, value: 0, suffix: " °C" }],
          caption: "One mole in 22.4 L at 0 °C is about one atmosphere.",
        },
      },
      {
        example: {
          title: "Absolute and difference",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: celsius
#import units: K, degC
#show: math

$celsius(20) => #K$
$celsius(80) - celsius(20) => #K$
$20 degC => #K$`,
          caption: "A reading becomes 293.15 K; a difference of readings and `degC` stay differences.",
        },
      },
      {
        quiz: {
          question: "Air at 20 °C is heated at constant volume until its pressure doubles. Which temperature does it reach?",
          options: ["40 °C", "313.15 °C", "293.15 °C"],
          answer: 1,
          explain: "Pressure is proportional to the *absolute* temperature: 2 · `celsius(20)` = 586.3 K, which is 313.15 °C. Doubling the reading would be wrong – that's why the reading goes through `celsius` first.",
        },
      },
      { params: [{ name: "x", type: "expression", md: "Unitless reading on the Celsius scale." }] },
    ],
    related: ["operations/fahrenheit", "units", "operations/number"],
  },
  {
    slug: "operations/fahrenheit",
    title: "fahrenheit",
    group: "operations",
    section: "Elementary",
    summary: "An absolute temperature from a Fahrenheit reading, in kelvin: `fahrenheit(32)` is 273.15 K.",
    signature: "fahrenheit(x)",
    import: "#import operations: fahrenheit",
    returns: "Temperature in kelvin.",
    keywords: ["temperature", "°F", "kelvin", "absolute"],
    blocks: [
      { md: "`fahrenheit(reading)` is the absolute temperature (reading + 459.67) · 5/9 K. Like [celsius](/docs/operations/celsius) it takes a plain number – the reading on the scale – and returns kelvin, ready for physics. Readings below absolute zero (−459.67 °F) are rejected." },
      {
        example: {
          title: "Fixed points",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: fahrenheit, celsius
#import units: K
#show: math

$fahrenheit(32) => #K$, $fahrenheit(212) => #K$
$fahrenheit(-40) - celsius(-40) => #K$`,
          caption: "Water freezes at 32 °F and boils at 212 °F; −40 is the same temperature on both scales.",
        },
      },
      {
        example: {
          title: "Radiated power",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: fahrenheit
#import units: W, m, K
#show: math

$sigma := 5.670 dot 10^(-8) #W / (#m^2 #K^4)$
$q := sigma fahrenheit({{T}})^4 => #W / #m^2$`,
          controls: [{ kind: "slider", name: "T", label: "Surface", min: 0, max: 500, step: 10, value: 98, suffix: " °F" }],
          caption: "Stefan–Boltzmann needs the absolute temperature, to the fourth power.",
        },
      },
      { params: [{ name: "x", type: "expression", md: "Unitless reading on the Fahrenheit scale." }] },
    ],
    related: ["operations/celsius", "units"],
  },
  {
    slug: "operations/gamma",
    title: "gamma",
    group: "operations",
    section: "Elementary",
    summary: "The real gamma function Γ(x), the factorial extended to real numbers: Γ(k + 1) = k!.",
    signature: "gamma(x)",
    import: "#import operations: gamma",
    returns: "Unitless number.",
    keywords: ["Γ", "gamma function", "factorial", "special function"],
    blocks: [
      { md: "Γ(x) = ∫₀^∞ t^(x−1) e^(−t) dt interpolates the factorial: Γ(k + 1) = k! for every nonnegative integer k, and Γ(x + 1) = x Γ(x) everywhere. It has poles at 0, −1, −2, …, where it's an error, and grows so fast that Γ(172) no longer fits in a float.\n\nThe argument is real and unitless. Half-integers stay exact: Γ(½) = √π." },
      {
        example: {
          title: "The factorial, and between",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: gamma
#show: math

$gamma(5) ->$, $gamma(1/2) ->$, $gamma(-1/2) ->$
$gamma(4.5) =>$`,
          caption: "Γ(5) = 4! = 24, and Γ(½)² = π.",
        },
      },
      {
        example: {
          title: "The graph",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: gamma
#show: math

#graph($gamma(x)$, x-range: (-3, 5), y-range: (-6, 12))`,
          caption: "Poles at the nonpositive integers, a minimum near x ≈ 1.46, and from there it grows faster than any exponential.",
        },
      },
      {
        example: {
          title: "A pole",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: gamma
#show: math

$gamma(-2) =>$`,
          expectError: true,
        },
      },
      { callout: "fun", title: "Half a factorial", md: "What is (½)!? It's Γ(3/2) = √π/2 ≈ 0.886. Try `$gamma(3/2) ->$` in the first example." },
      { params: [{ name: "x", type: "expression", md: "Real unitless argument away from a pole." }] },
    ],
    related: ["operations/factorial", "operations/erf", "operations/besselj", "graphs"],
  },
  {
    slug: "operations/erf",
    title: "erf",
    group: "operations",
    section: "Elementary",
    summary: "The real error function erf(x) = 2/√π ∫₀ˣ e^(−t²) dt.",
    signature: "erf(x)",
    import: "#import operations: erf",
    returns: "Unitless number between −1 and 1.",
    keywords: ["error function", "gaussian", "normal distribution", "diffusion"],
    blocks: [
      { md: "The error function is the integral of the Gaussian bell, scaled so it runs from −1 to 1. It shows up wherever something diffuses – heat into a wall, dopants into silicon – and in probability: the share of a normal distribution within ±k standard deviations is erf(k/√2).\n\nThe argument is real and unitless. For normal probabilities directly, there's [pnorm](/docs/operations/pnorm)." },
      {
        example: {
          title: "The definition, checked",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: erf
#import constants: pi, e
#show: math

$x := {{x}}$
$erf(x) =>$
$2/sqrt(pi) integral_0^x #e^(-t^2) dif t =>$`,
          controls: [{ kind: "slider", name: "x", label: "x", min: -3, max: 3, step: 0.1, value: 1, suffix: "" }],
          caption: "The integral of the bell, numerically, agrees with `erf`.",
        },
      },
      {
        example: {
          title: "The graph",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: erf
#show: math

#graph($erf(x)$, x-range: (-3, 3))`,
          caption: "An S-curve: steepest at 0, at ±2 already within 0.5 % of ±1.",
        },
      },
      {
        example: {
          title: "Within one sigma",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: erf
#show: math

$k := {{k}}$
$P := erf(k / sqrt(2)) =>$`,
          controls: [{ kind: "slider", name: "k", label: "± k σ", min: 1, max: 6, step: 1, value: 1, suffix: "" }],
          caption: "68 % within ±1σ, 95 % within ±2σ, 99.7 % within ±3σ. Six sigma leaves about two in a billion outside.",
        },
      },
      { params: [{ name: "x", type: "expression", md: "Real unitless argument." }] },
    ],
    related: ["operations/gamma", "operations/pnorm", "operations/integrate", "graphs"],
  },
  {
    slug: "operations/besselj",
    title: "besselj",
    group: "operations",
    section: "Elementary",
    summary: "The Bessel function of the first kind Jₙ(x), for integer order n.",
    signature: "besselj(order, x)",
    import: "#import operations: besselj",
    returns: "Unitless number.",
    keywords: ["bessel", "J0", "J1", "FM", "sidebands", "vibration", "special function"],
    blocks: [
      { md: "Bessel functions are the waves of round things: the modes of a drum, the field in a coaxial cable, the light through a circular aperture. Jₙ(x) oscillates like a damped cosine, with zeros that are *not* evenly spaced.\n\nThe order must be an integer from −10000 to 10000; noninteger orders are an error. Both arguments are real and unitless." },
      {
        example: {
          title: "The first three",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: besselj
#show: math

#graph($J_0(x) = besselj(0, x) \\ J_1(x) = besselj(1, x) \\ J_2(x) = besselj(2, x)$, x-range: (0, 15))`,
          caption: "J₀ starts at 1, every higher order at 0 – and starts later the higher it is.",
        },
      },
      {
        example: {
          title: "FM sidebands",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: besselj
#show: math

$beta := {{beta}}$
$A_0 := besselj(0, beta) =>$
$A_1 := besselj(1, beta) =>$
$A_2 := besselj(2, beta) =>$`,
          controls: [{ kind: "slider", name: "beta", label: "Modulation index β", min: 0, max: 5, step: 0.05, value: 1, suffix: "" }],
          caption: "A frequency-modulated carrier splits into lines at f_c ± k f_m with amplitudes J_k(β). Find β where the carrier A₀ vanishes – near 2.405, the first zero of J₀.",
        },
      },
      {
        params: [
          { name: "order", type: "int", md: "Integer order, from −10000 through 10000." },
          { name: "x", type: "expression", md: "Real unitless argument." },
        ],
      },
    ],
    related: ["operations/gamma", "operations/erf", "graphs", "signals"],
  },

  // ---------- Complex ----------
  {
    slug: "operations/Re",
    title: "Re",
    group: "operations",
    section: "Complex",
    summary: "The real part of a complex value, with its unit: `Re(Z)` of an impedance is its resistance.",
    signature: "Re(z)",
    import: "#import operations: Re",
    returns: "A real value, in z's unit.",
    keywords: ["real part", "resistance", "complex", "phasor"],
    blocks: [
      { md: "`Re(z)` is the real part of z and keeps its unit. A real number is its own real part.\n\nSymbolically, `Re` distributes over sums and expands products with complex arithmetic: `Re(z w)` is `Re(z) Re(w) − Im(z) Im(w)`. A name without a value may be complex, so `Re(a z) = a Re(z)` holds only for real a – solveng says so, or use [assume](/docs/modifiers/assume)." },
      {
        example: {
          title: "Resistance and reactance",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: Re, Im
#import units: Ω, Hz, mH
#import constants: pi, i
#show: math.with(constant-displays: (imaginary: $j$))

$Z := 100 #Ω + #i 2 pi {{f}} Hz dot 300 mH =>$
$R := Re(Z) =>$, $X := Im(Z) =>$`,
          controls: [{ kind: "slider", name: "f", label: "f", min: 10, max: 500, step: 10, value: 50, suffix: " Hz" }],
          caption: "The resistance stays put as the frequency changes; the reactance grows with it.",
        },
      },
      {
        example: {
          title: "Symbolic",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: Re
#import modifiers: assume
#import constants: i
#show: math

$Re(3 + 4 #i) ->$
$Re(a z) = a Re(z) ->$
$Re(a z) = a Re(z) -> assume(a in RR)$`,
          caption: "Without assumptions solveng says *when* the equation holds; `assume` makes a real and gets a plain `true`.",
        },
      },
      {
        example: {
          title: "Impedance of a series RC, over frequency",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: Re, Im
#import units: Ω, kHz, µF
#import constants: pi, i
#show: math

$R := 100 #Ω$, $C := 1 µF$
$Z(f: kHz) := R + 1/(#i 2 pi f C)$
#graph($Re(Z(f)) \\ Im(Z(f))$, x-range: (0.5, 10))`,
          caption: "Graphs need real values: plot the parts (or `abs` and `arg`), not Z itself.",
        },
      },
      { params: [{ name: "z", type: "expression", md: "A complex or real value." }] },
    ],
    related: ["operations/Im", "operations/conj", "complex", "modifiers/assume"],
  },
  {
    slug: "operations/Im",
    title: "Im",
    group: "operations",
    section: "Complex",
    summary: "The imaginary part of a complex value, as a real number with its unit: `Im(3 + 4i)` is 4.",
    signature: "Im(z)",
    import: "#import operations: Im",
    returns: "A real value, in z's unit.",
    keywords: ["imaginary part", "reactance", "complex", "phasor"],
    blocks: [
      { md: "`Im(z)` is the imaginary part of z – a *real* number, without the i – and keeps the unit: `Im(Z)` of an impedance is its reactance in ohms. Real numbers have the imaginary part 0. Symbolically it follows the same rules as [Re](/docs/operations/Re)." },
      {
        example: {
          title: "Parts of a number",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: Re, Im
#import constants: i, e, pi
#show: math

$Im(3 + 4 #i) ->$, $Im(5) ->$
$z := 2 #e^(#i pi / 6)$
$Re(z) ->$, $Im(z) ->$`,
          caption: "A phasor of length 2 at 30°: √3 along, 1 up.",
        },
      },
      {
        example: {
          title: "Reactance of an LC branch",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: Im
#import units: Ω, kHz, mH, nF
#import constants: pi, i
#show: math

$L := 1 mH$, $C := {{C}} nF$
$Z(f: kHz) := #i 2 pi f L + 1/(#i 2 pi f C)$
#graph($Im(Z(f))$, x-range: (10, 100), mark: "roots")`,
          controls: [{ kind: "slider", name: "C", label: "C", min: 10, max: 100, step: 5, value: 25, suffix: " nF" }],
          caption: "The reactance crosses zero at resonance, 1/(2π√(LC)). Drag C and watch the root move.",
        },
      },
      { params: [{ name: "z", type: "expression", md: "A complex or real value." }] },
    ],
    related: ["operations/Re", "operations/conj", "complex"],
  },
  {
    slug: "operations/conj",
    title: "conj",
    group: "operations",
    section: "Complex",
    summary: "The complex conjugate: `conj(3 + 4i)` is 3 − 4i, shown with a bar.",
    signature: "conj(z)",
    import: "#import operations: conj",
    returns: "A value in z's unit.",
    keywords: ["conjugate", "complex", "bar", "apparent power"],
    blocks: [
      { md: "`conj(z)` flips the sign of the imaginary part and keeps the unit; a worksheet shows it as z̄. It distributes over sums, products and integer powers, cancels when applied twice, and leaves provably real expressions alone. A name without a value may be complex, so `conj(z) z` is |z|², not z².\n\nThe magnitude is `abs(z)` and the angle `arg(z)`; both need no import." },
      {
        example: {
          title: "Complex power",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: conj, Re, Im
#import units: V, A, W
#import constants: i
#show: math.with(constant-displays: (imaginary: $j$))

$U := 230 #V$, $I := (4 - 3 #i) #A$
$S := U conj(I) =>$
$P := Re(S) => #W$, $Q := Im(S) =>$`,
          caption: "Apparent power is U times the conjugate current: its real part is the real power, its imaginary part the reactive power.",
        },
      },
      {
        example: {
          title: "Symbolic rules",
          code: `#import "@preview/solveng:2.0.0": *
#import operations: conj
#import constants: i
#show: math

$conj(3 + 4 #i) ->$
$conj(conj(w)) ->$
$z conj(z) ->$`,
          caption: "z z̄ equals z² only when z is real – solveng says so instead of simplifying it away.",
        },
      },
      { params: [{ name: "z", type: "expression", md: "A complex or real value." }] },
    ],
    related: ["operations/Re", "operations/Im", "complex"],
  },
];
