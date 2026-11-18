import type { Page } from "../types";

const STATS = "Statistics & data";
const NUM = "Numerical";

export const pages: Page[] = [
  // ---------- mean ----------
  {
    slug: "operations/mean",
    title: "mean",
    group: "operations",
    section: STATS,
    summary: "The arithmetic mean of a vector: the sum of its elements over their number, in their unit.",
    signature: "mean(values)",
    import: "#import operations: mean",
    returns: "A value in the elements' unit; complex if they are.",
    keywords: ["average", "arithmetic mean", "expected value", "nominal"],
    blocks: [
      { md: "`mean` adds up the elements of a vector and divides by their number. The elements need one unit, and the mean keeps it – the average of resistances is a resistance. Exact numbers stay exact: `mean(vec(1, 2, 6))` is 3." },
      {
        example: {
          title: "A batch of 100 Ω resistors",
          code: `#import "@preview/solveng:3.0.0": *
#import operations: mean
#import units: Ω
#show: math

$R := vec(99.2, 100.4, 100.1, 99.7, 100.9, 99.8, 100.3, 99.6) #Ω$
$μ := mean(R) =>$
#graph($R$, ymark: $μ$, y-range: (98, 102))`,
          caption: "Eight measurements, plotted over their index k, with the mean marked: about half the points lie on each side.",
        },
      },
      {
        example: {
          title: "Three phases cancel",
          code: `#import "@preview/solveng:3.0.0": *
#import operations: mean
#import units: A
#import constants: i
#show: math

$I := vec(1, -1/2 - sqrt(3)/2 #i, -1/2 + sqrt(3)/2 #i) 10 #A$
$mean(I) =>$`,
          caption: "Complex values work too. The currents of a balanced three-phase system are 120° apart, written exactly as 1 and −½ ± (√3/2)i. Their mean – and so the neutral current – is exactly zero.",
        },
      },
      { params: [{ name: "values", type: "expression", md: "Nonempty vector, every element in the same unit. Complex values are allowed." }] },
      { h2: "Conventions and limits" },
      { md: "- An empty vector, or elements of different units, are reported.\n- A single outlier pulls the mean with it; [median](/docs/operations/median) doesn't budge." },
    ],
    related: ["operations/median", "operations/stdev", "operations/variance", "vectors"],
  },

  // ---------- median ----------
  {
    slug: "operations/median",
    title: "median",
    group: "operations",
    section: STATS,
    summary: "The middle value of a vector once sorted – a robust average that a single bad reading can't drag away.",
    signature: "median(values)",
    import: "#import operations: median",
    returns: "A value in the elements' unit.",
    keywords: ["middle", "robust", "outlier", "percentile", "50 %"],
    blocks: [
      { md: "Sort the values and take the one in the middle; for an even number, the mean of the two middle ones. Because only the order matters, an outlier – a loose probe, a typo in the log – changes the median by at most one position, while it can move the [mean](/docs/operations/mean) anywhere." },
      {
        example: {
          title: "One reading goes wrong",
          code: `#import "@preview/solveng:3.0.0": *
#import operations: mean, median
#import units: V
#show: math

$u := vec(4.98, 5.01, 5.03, 4.99, {{bad}}) #V$
$mean(u) =>$
$median(u) =>$`,
          controls: [{ kind: "slider", name: "bad", label: "Last reading", min: 0, max: 50, step: 0.5, value: 5, suffix: " V" }],
          caption: "Drag the last reading from 0 V to 50 V: the mean follows it, the median stays near 5 V.",
        },
      },
      {
        example: {
          title: "An even number of values",
          code: `#import "@preview/solveng:3.0.0": *
#import operations: median
#import units: m
#show: math

$median(vec(9, 1, 3, 7) #m) =>$`,
          caption: "Sorted: 1, 3, 7, 9. The middle two average to 5 m.",
        },
      },
      { params: [{ name: "values", type: "expression", md: "Nonempty real vector, every element in the same unit." }] },
      { h2: "Conventions and limits" },
      { md: "- Real values only: complex numbers have no order.\n- The vector doesn't need to be sorted; `median` sorts a copy." },
    ],
    related: ["operations/mean", "operations/stdev"],
  },

  // ---------- variance ----------
  {
    slug: "operations/variance",
    title: "variance",
    group: "operations",
    section: STATS,
    summary: "The sample variance of a vector: the mean squared deviation from the mean, with n − 1 in the denominator.",
    signature: "variance(values)",
    import: "#import operations: variance",
    returns: "A value in the square of the elements' unit.",
    keywords: ["sample variance", "spread", "scatter", "n-1", "Bessel's correction", "s²"],
    blocks: [
      { md: "`variance` measures how far measurements scatter around their mean: `s² = Σ (xₖ − x̄)² / (n − 1)`. Its unit is the square of the data's – mm² for diameters – which is why you'll usually want its square root, [stdev](/docs/operations/stdev).\n\nWhy n − 1 and not n? The deviations are taken from the *sample's own* mean, which sits closer to the data than the true mean does. Dividing by n would underestimate the scatter, so the sample variance divides by n − 1 (Bessel's correction). It is what calculators label s² and spreadsheets `VAR.S`." },
      {
        example: {
          title: "Repeatability of a gauge",
          code: `#import "@preview/solveng:3.0.0": *
#import operations: variance
#import units: mm, μm
#show: math

$d := vec(12.03, 11.98, 12.01, 12.05, 11.99) mm$
$variance(d) => μm^2$`,
          caption: "Five readings of the same part. The variance comes out in an area unit – here μm².",
        },
      },
      {
        example: {
          title: "Sample versus population",
          code: `#import "@preview/solveng:3.0.0": *
#import operations: variance, rows
#show: math

$x := vec(1, 2, 3)$, $n := rows(x)$
$variance(x) ->$
$variance(x) (n - 1)/n ->$`,
          caption: "The deviations are −1, 0, 1. Their squares add to 2: over n − 1 that's 1, the sample variance; over n it's 2/3 ≈ 0.667, the population variance. Multiply by (n − 1)/n when your data is the whole population.",
        },
      },
      {
        quiz: {
          question: "Two readings, 9 V and 11 V. What is `variance(vec(9, 11) #V)`?",
          options: ["1 V²", "2 V²", "4 V²"],
          answer: 1,
          explain: "The mean is 10 V and both deviations are ±1 V, so the squares add to 2 V². Divided by n − 1 = 1, that's 2 V². Dividing by n would give 1 V² – the population variance, too small for a sample.",
        },
      },
      { params: [{ name: "values", type: "expression", md: "Real vector of at least two values, every element in the same unit." }] },
      { h2: "Conventions and limits" },
      { md: "- At least two values: a single reading has no scatter to estimate, and n − 1 would be 0.\n- Real values only.\n- `s^2 := …` isn't a valid name – a superscript isn't part of a name. Call it `s_2`, `σ_x`, or show `variance(d)` directly." },
    ],
    related: ["operations/stdev", "operations/mean", "operations/pnorm"],
  },

  // ---------- stdev ----------
  {
    slug: "operations/stdev",
    title: "stdev",
    group: "operations",
    section: STATS,
    summary: "The sample standard deviation of a vector: the square root of its variance, in the data's own unit.",
    signature: "stdev(values)",
    import: "#import operations: stdev",
    returns: "A nonnegative value in the elements' unit.",
    keywords: ["standard deviation", "sigma", "s", "spread", "scatter", "tolerance", "n-1", "STDEV.S"],
    blocks: [
      { md: "`stdev(x)` is `sqrt(variance(x))`: the typical distance of a measurement from the mean, back in the measurement's unit. Like [variance](/docs/operations/variance), it divides by n − 1 – the *sample* standard deviation s, not the population's σ." },
      {
        example: {
          title: "Scatter around the mean",
          code: `#import "@preview/solveng:3.0.0": *
#import operations: mean, stdev
#import units: mm
#show: math

$d := vec(12.03, 11.98, 12.01, 12.05, 11.99, 12.02, 11.96, 12.04) mm$
$μ := mean(d) =>$, $s := stdev(d) =>$
#graph($d$, ymark: ($μ - s$, $μ + s$))`,
          caption: "Eight shaft diameters with the band μ ± s marked. For roughly normal data, about two thirds of the points fall inside it.",
        },
      },
      {
        example: {
          title: "A process capability check",
          code: `#import "@preview/solveng:3.0.0": *
#import operations: mean, stdev
#import units: mm
#show: math

$d := vec(12.03, 11.98, 12.01, 12.05, 11.99, 12.02, 11.96, 12.04) mm$
$T := {{tol}} mm$
$C_p := (2 T) / (6 stdev(d)) =>$`,
          controls: [{ kind: "slider", name: "tol", label: "Tolerance ±T", min: 0.05, max: 0.3, step: 0.01, value: 0.1, suffix: " mm" }],
          caption: "C_p compares the tolerance band 2T with the process spread 6s. Industry often asks for C_p ≥ 1.33 – find the tolerance where this process gets there.",
        },
      },
      { params: [{ name: "values", type: "expression", md: "Real vector of at least two values, every element in the same unit." }] },
      { h2: "Conventions and limits" },
      { md: "- At least two values; real values only.\n- For the uncertainty of the *mean* rather than of a single reading, divide by √n: `stdev(d) / sqrt(rows(d))` (see [rows](/docs/operations/rows))." },
    ],
    related: ["operations/variance", "operations/mean", "operations/pnorm", "operations/qnorm"],
  },

  // ---------- pnorm ----------
  {
    slug: "operations/pnorm",
    title: "pnorm",
    group: "operations",
    section: STATS,
    summary: "The normal distribution's cumulative probability: how likely a value at or below x is, given a mean and a standard deviation.",
    signature: "pnorm(x, mean, sigma)",
    import: "#import operations: pnorm",
    returns: "Unitless probability between 0 and 1.",
    keywords: ["normal distribution", "Gaussian", "CDF", "cumulative", "probability", "yield", "Phi", "NORM.DIST"],
    blocks: [
      { md: "`pnorm(x, μ, σ)` is the area under the bell curve to the left of x – the share of parts, readings or days that fall at or below x when they scatter normally around μ with standard deviation σ. The difference of two `pnorm`s is the share between two limits: the yield of a tolerance." },
      {
        example: {
          title: "How many shafts are within tolerance?",
          code: `#import "@preview/solveng:3.0.0": *
#import operations: pnorm
#import units: mm
#show: math

$μ := 10 mm$, $σ := {{sigma}} mm$
$L := 9.97 mm$, $U := 10.03 mm$
$"yield" := pnorm(U, μ, σ) - pnorm(L, μ, σ) =>$`,
          controls: [{ kind: "slider", name: "sigma", label: "Process σ", min: 0.005, max: 0.05, step: 0.005, value: 0.02, suffix: " mm" }],
          caption: "A ±0.03 mm tolerance. At σ = 0.01 mm the limits are 3σ away and 99.7 % pass; at σ = 0.02 mm only 87 %.",
        },
      },
      {
        example: {
          title: "The S-curve",
          code: `#import "@preview/solveng:3.0.0": *
#import operations: pnorm
#import units: mm
#show: math

$μ := 10 mm$, $σ := 0.02 mm$
$F(x: mm) := pnorm(x, μ, σ)$
#graph($F(x)$, x-range: (9.94, 10.06), ymark: (0.5))`,
          caption: "The cumulative probability rises from 0 to 1, passing ½ at the mean.",
        },
      },
      {
        params: [
          { name: "x", type: "expression", md: "The real value whose cumulative probability you want." },
          { name: "mean", type: "expression", md: "The distribution's mean μ, in x's unit." },
          { name: "sigma", type: "expression", md: "The standard deviation σ, strictly positive, in x's unit." },
        ],
      },
      { h2: "Conventions and limits" },
      { md: "- x, μ and σ share a unit; the result is unitless.\n- Computed through a stable complementary error function, so far tails stay accurate: `pnorm(μ − 7σ, μ, σ)` is about 1.3 · 10⁻¹², not 0.\n- The inverse is [qnorm](/docs/operations/qnorm): `qnorm(pnorm(x, μ, σ), μ, σ)` is x again." },
    ],
    related: ["operations/qnorm", "operations/stdev", "operations/erf"],
  },

  // ---------- qnorm ----------
  {
    slug: "operations/qnorm",
    title: "qnorm",
    group: "operations",
    section: STATS,
    summary: "The inverse of pnorm: the value below which a given share of a normal distribution lies.",
    signature: "qnorm(probability, mean, sigma)",
    import: "#import operations: qnorm",
    returns: "A value in the unit of mean and sigma.",
    keywords: ["quantile", "inverse normal", "percentile", "z-score", "NORM.INV", "probit", "design value"],
    blocks: [
      { md: "`qnorm(p, μ, σ)` answers the question backwards: *which* value is exceeded only with probability 1 − p? That's how design loads, guard bands and confidence limits are set. With μ = 0 and σ = 1 it gives the familiar z-values: 1.645 for 95 %, 1.960 for 97.5 %, 2.326 for 99 %." },
      {
        example: {
          title: "A quantile, live",
          code: `#import "@preview/solveng:3.0.0": *
#import operations: qnorm
#import units: mm
#show: math

$μ := 25 mm$, $σ := 0.05 mm$
$x := qnorm({{p}}, μ, σ) =>$
$z := (x - μ)/σ =>$`,
          controls: [{ kind: "slider", name: "p", label: "Probability p", min: 0.5, max: 0.999, step: 0.001, value: 0.95 }],
          caption: "Drag p: at 0.5 you get the mean, at 0.975 the 1.96σ of a two-sided 95 % interval, and near 0.999 it climbs past 3σ.",
        },
      },
      {
        example: {
          title: "A design load",
          code: `#import "@preview/solveng:3.0.0": *
#import operations: qnorm, pnorm
#import units: kN
#show: math

$μ := 12 kN$, $σ := 1.5 kN$
$F_d := qnorm(0.999, μ, σ) =>$
$pnorm(F_d, μ, σ) =>$`,
          caption: "A load exceeded only once in a thousand cases – and pnorm takes it straight back to 0.999.",
        },
      },
      {
        params: [
          { name: "probability", type: "expression", md: "Unitless, strictly between 0 and 1." },
          { name: "mean", type: "expression", md: "The distribution's mean μ." },
          { name: "sigma", type: "expression", md: "The standard deviation σ, strictly positive, in μ's unit." },
        ],
      },
      { h2: "Conventions and limits" },
      { md: "- p = 0 and p = 1 have no finite answer and are reported.\n- Near p = 1 it inverts the smaller tail, so `qnorm(1 - 10^(-12), 0, 1)` is accurate instead of lost to cancellation." },
    ],
    related: ["operations/pnorm", "operations/stdev"],
  },

  // ---------- linear-interpolate ----------
  {
    slug: "operations/linear-interpolate",
    title: "linear-interpolate",
    group: "operations",
    section: STATS,
    summary: "Reads a value between table points by joining neighbouring samples with straight lines – and refuses to extrapolate.",
    signature: "linear-interpolate(x, y, at)",
    import: "#import operations: linear-interpolate",
    returns: "A value in y's unit.",
    keywords: ["linterp", "interpolation", "lookup table", "datasheet", "table", "between points"],
    blocks: [
      { md: "Datasheets give curves as tables. `linear-interpolate(x, y, at)` finds the two neighbouring points `at` lies between and reads the straight line joining them. At a table point it returns that point's y exactly.\n\nThe hyphen in its name would read as a minus in Typst math, so call it with `#` and pass math: `#linear-interpolate($T$, $R$, $T_x$)`. It is also available as `linterp`, without the hyphen: `linterp(T, R, T_x)` – which is also how results show it." },
      {
        example: {
          title: "An NTC thermistor's datasheet",
          code: `#import "@preview/solveng:3.0.0": *
#import operations: linear-interpolate
#import units: kΩ
#show: math

$T := vec(0, 25, 50, 75, 100)$
$R := vec(32.65, 10, 3.6, 1.47, 0.68) kΩ$
$T_x := {{t}}$
$R_x := #linear-interpolate($T$, $R$, $T_x$) =>$
#graph($R(T)$, mark: $T_x$)`,
          controls: [{ kind: "slider", name: "t", label: "Temperature reading", min: 0, max: 100, step: 1, value: 37 }],
          caption: "Temperatures as plain °C readings, resistances in kΩ. Drag the reading: the point slides along the straight segments. Between 0 °C and 25 °C the real curve sags below the line – a reminder to keep tables dense where a curve bends.",
        },
      },
      {
        example: {
          title: "Outside the table",
          code: `#import "@preview/solveng:3.0.0": *
#import operations: linterp
#import units: s, V
#show: math

$t := vec(0, 2, 5) #s$
$u := vec(1, 5, 11) #V$
$linterp(t, u, 3 #s) =>$
$linterp(t, u, 6 #s) =>$`,
          expectError: true,
          caption: "3 s lies between 2 s and 5 s: 7 V. 6 s is past the last sample – solveng reports it rather than silently extending your data.",
        },
      },
      {
        params: [
          { name: "x", type: "expression", md: "At least two real knots, strictly increasing, in one unit." },
          { name: "y", type: "expression", md: "The samples at the knots: a real vector of the same length, in one unit." },
          { name: "at", type: "expression", md: "Where to read, in x's unit, within the closed range from the first to the last knot." },
        ],
      },
      { h2: "Conventions and limits" },
      { md: "- x must be strictly increasing – sort your table first; repeated knots are reported.\n- No extrapolation: a position outside [x₀, xₙ] is an error.\n- Exact numbers stay exact, so a table of fractions interpolates to fractions." },
    ],
    related: ["operations/linear-fit", "functions/graph", "vectors"],
  },

  // ---------- linear-fit ----------
  {
    slug: "operations/linear-fit",
    title: "linear-fit",
    group: "operations",
    section: STATS,
    summary: "The least-squares straight line through data points: returns vec(intercept, slope), with units.",
    signature: "linear-fit(x, y)",
    import: "#import operations: linear-fit",
    returns: "vec(intercept, slope): the intercept in y's unit, the slope in y's unit over x's.",
    keywords: ["linfit", "regression", "least squares", "calibration", "trend line", "slope", "intercept", "sensitivity"],
    blocks: [
      { md: "`linear-fit(x, y)` finds the line `y ≈ c₀ + c₁ x` that minimizes the sum of squared vertical distances to the points. It returns both coefficients as a vector, **intercept first**: pick them with [at](/docs/operations/at).\n\nLike its siblings, the name has a hyphen, so in math write `#linear-fit($x$, $y$)` – or use the alias `linfit(x, y)`." },
      {
        example: {
          title: "Calibrating a temperature sensor",
          code: `#import "@preview/solveng:3.0.0": *
#import operations: linear-fit, at
#import units: K, V, mV
#show: math

$T := vec(20, 40, 60, 80) #K$
$u := vec(0.81, 1.62, 2.38, {{u3}}) #V$
$c := #linear-fit($T$, $u$) =>$
$g(x) := at(c, 0) + at(c, 1) x$
#graph($u(T) \\ g(T)$, lines: (none, auto))`,
          controls: [{ kind: "slider", name: "u3", label: "Last reading", min: 2.8, max: 3.6, step: 0.01, value: 3.21, suffix: " V" }],
          caption: "Four calibration points above ambient, the measured points drawn alone and the fitted line through them. The slope is the sensor's sensitivity, about 40 mV/K. Drag the last reading and watch the line tilt.",
        },
      },
      {
        example: {
          title: "Exact data, exact fit",
          code: `#import "@preview/solveng:3.0.0": *
#import operations: linfit
#show: math

$linfit(vec(-1, 0, 1), vec(5, 3, 1)) =>$`,
          caption: "The points lie on y = 3 − 2x: intercept 3, slope −2.",
        },
      },
      {
        quiz: {
          question: "After `$c := linfit(x, y)$`, which is the slope?",
          options: ["`at(c, 0)`", "`at(c, 1)`", "`c` itself – it's a single number"],
          answer: 1,
          explain: "linfit returns vec(intercept, slope), the coefficients in ascending power of x – like polyroots' coefficients. Some tools return the slope first; solveng doesn't.",
        },
      },
      {
        params: [
          { name: "x", type: "expression", md: "Real inputs, at least two, not all equal, in one unit." },
          { name: "y", type: "expression", md: "Real responses, same length, in one unit." },
        ],
      },
      { h2: "Conventions and limits" },
      { md: "- Ordinary least squares: only y is assumed to carry error, and every point counts equally.\n- The data is centered internally, so a large offset in x (timestamps, absolute temperatures) doesn't cost accuracy.\n- For more than two coefficients – a parabola, a sum of known shapes – set up the design matrix and use [linear-solve](/docs/operations/linear-solve)." },
    ],
    related: ["operations/linear-solve", "operations/linear-interpolate", "operations/at", "functions/graph"],
  },

  // ---------- dft ----------
  {
    slug: "operations/dft",
    title: "dft",
    group: "operations",
    section: STATS,
    summary: "The discrete Fourier transform of a vector of samples: which frequencies it contains, and how strongly.",
    signature: "dft(values)",
    import: "#import operations: dft",
    returns: "Complex vector of the same length, in the samples' unit.",
    keywords: ["DFT", "FFT", "Fourier", "spectrum", "frequency", "harmonics", "bins", "leakage"],
    blocks: [
      { md: "`dft(x)` computes `X[k] = Σ x[j] e^(−2πi j k/n)` for k = 0 … n − 1. Bin k stands for k cycles per record: with a sample rate f_s, the frequency `k f_s / n`. For real signals the upper half mirrors the lower half – bin n − k is the conjugate of bin k.\n\nThere is **no normalization** on the way forward: a constant 1 over n samples gives X[0] = n, not 1. A cosine of amplitude A that fits a whole number of cycles shows up as A n/2 in its bin and its mirror. So to read amplitudes, scale by 2/n." },
      {
        example: {
          title: "Two tones, and leakage",
          code: `#import "@preview/solveng:3.0.0": *
#import operations: dft, at
#import constants: pi
#show: math

$N := 16$, $f := {{f}}$
$j := 0 .. N - 1$
$x[j] := cos(2 pi f j / N) + 1/2 sin(2 pi 6 j / N)$
$X := dft(x)$
$A[j] := 2/N abs(at(X, j))$
#graph($A$)`,
          controls: [{ kind: "slider", name: "f", label: "First tone (cycles per record)", min: 1, max: 5, step: 0.25, value: 3 }],
          caption: "Amplitudes 2|X[k]|/N over the bin k: 1 at the first tone, ½ at bin 6, mirrored above N/2. Set the first tone between two bins – 3.5 – and its energy smears over its neighbours: spectral leakage, because the record no longer holds a whole number of cycles.",
        },
      },
      {
        example: {
          title: "No normalization",
          code: `#import "@preview/solveng:3.0.0": *
#import operations: dft
#show: math

$dft(vec(1, 1, 1, 1)) =>$
$dft(vec(1, 0, 0, 0)) =>$`,
          caption: "A constant concentrates in bin 0 with the sum of its samples, 4. A single impulse is spread flat over every bin.",
        },
      },
      {
        quiz: {
          question: "A signal of 8 samples is a constant 2 V. What is the first element of its `dft`?",
          options: ["2 V", "16 V", "0.25 V"],
          answer: 1,
          explain: "Bin 0 is the plain sum of the samples: 8 · 2 V = 16 V. The DFT has no 1/n in front – idft carries it. Divide by n to get the mean.",
        },
      },
      { params: [{ name: "values", type: "expression", md: "Nonempty column vector of samples, real or complex, in one unit." }] },
      { h2: "Conventions and limits" },
      { md: "- Sign convention e^(−2πi jk/n) forward; [idft](/docs/operations/idft) is the inverse with 1/n.\n- The spectrum keeps the samples' unit (volts stay volts).\n- A direct O(n²) sum, not an FFT: any length works, up to 2048 samples.\n- Imaginary parts that are only rounding noise are removed – within an error estimate per element, so genuinely tiny imaginary parts survive." },
    ],
    related: ["operations/idft", "operations/range", "signals", "transforms"],
  },

  // ---------- idft ----------
  {
    slug: "operations/idft",
    title: "idft",
    group: "operations",
    section: STATS,
    summary: "The inverse discrete Fourier transform: from a spectrum back to samples, with the 1/n that dft leaves out.",
    signature: "idft(values)",
    import: "#import operations: idft",
    returns: "Complex vector of the same length, in the spectrum's unit.",
    keywords: ["inverse DFT", "IFFT", "Fourier", "synthesis", "filter", "reconstruction"],
    blocks: [
      { md: "`idft(X)` computes `x[j] = Σ X[k] e^(2πi j k/n) / n`. Together with [dft](/docs/operations/dft) it round-trips: `idft(dft(x))` is x to rounding. In between you can edit the spectrum – drop bins to filter, scale them to equalize." },
      {
        example: {
          title: "There and back",
          code: `#import "@preview/solveng:3.0.0": *
#import operations: dft, idft
#show: math

$x := vec(1, 2, 3, 4)$
$X := dft(x) =>$
$idft(X) =>$`,
          caption: "The complex spectrum carries every bit of the signal; the inverse divides by n = 4 and returns the samples.",
        },
      },
      {
        example: {
          title: "A brick-wall low-pass",
          code: `#import "@preview/solveng:3.0.0": *
#import operations: dft, idft, at, Re
#import constants: pi
#show: math

$N := 16$, $k := 0 .. N - 1$
$x[k] := sin(2 pi k / N) + 1/3 sin(2 pi 6 k / N)$
$m := vec(1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1)$
$X := dft(x)$
$Y[k] := at(X, k) at(m, k)$
$Z := idft(Y)$
$y[k] := Re(at(Z, k))$
#graph($x(k) \\ y(k)$)`,
          caption: "The mask m keeps bins 0–2 and their mirrors 14–15 and zeroes the rest. The ripple at bin 6 is gone; the slow sine stays. Keeping each kept bin's mirror keeps the result real – `Re` only drops rounding noise.",
        },
      },
      { params: [{ name: "values", type: "expression", md: "Nonempty column vector of spectral values, real or complex, in one unit." }] },
      { h2: "Conventions and limits" },
      { md: "- Sign convention e^(+2πi jk/n) with 1/n; the forward [dft](/docs/operations/dft) has neither.\n- Same unit handling and 2048-sample limit as dft.\n- The result is complex in general; a spectrum with conjugate-symmetric bins gives real samples." },
    ],
    related: ["operations/dft", "operations/Re", "transforms"],
  },

  // ---------- minimize ----------
  {
    slug: "operations/minimize",
    title: "minimize",
    group: "operations",
    section: NUM,
    summary: "Where an expression is smallest between two bounds: returns the position, not the minimum value.",
    signature: "minimize(expression, variable, lower, upper)",
    import: "#import operations: minimize",
    returns: "The minimizing value of the variable, in the bounds' unit.",
    keywords: ["optimization", "optimum", "golden section", "minimum", "maximum", "argmin", "best"],
    blocks: [
      { md: "`minimize(f, x, a, b)` searches the interval from a to b for the x where f is lowest, and returns that x. Put f(x) back in to get the minimum itself. To find a **maximum**, minimize the negative.\n\nThe variable is local: an x defined elsewhere doesn't interfere. The bounds can carry units, and the result has theirs." },
      {
        example: {
          title: "Maximum power transfer",
          code: `#import "@preview/solveng:3.0.0": *
#import operations: minimize
#import units: Ω, V, W
#show: math

$U_0 := 12 #V$, $R_i := 4 #Ω$
$P(R) := U_0^2 R / (R_i + R)^2$
$R_L := minimize(-P(R), R, 1 #Ω, {{upper}} #Ω) =>$
$P(R_L) => #W$`,
          controls: [{ kind: "slider", name: "upper", label: "Largest load available", min: 2, max: 20, step: 0.5, value: 10, suffix: " Ω" }],
          caption: "A source with internal resistance R_i delivers most power into a load equal to R_i. Pull the upper bound below 4 Ω: the best you can do is then the bound itself – minimize compares the endpoints too.",
        },
      },
      {
        example: {
          title: "The cheapest can",
          code: `#import "@preview/solveng:3.0.0": *
#import operations: minimize
#import units: cm, l
#import constants: pi
#show: math

$V := 0.33 #l$
$S(r) := 2 pi r^2 + 2 V / r$
$r_0 := minimize(S(r), r, 1 #cm, 10 #cm) => #cm$
$h := V / (pi r_0^2) => #cm$`,
          caption: "The surface of a closed cylinder holding V, as a function of its radius. The least sheet metal is used when the height equals the diameter.",
        },
      },
      {
        params: [
          { name: "expression", type: "expression", md: "The real objective, continuous, with a single minimum in the interval." },
          { name: "variable", type: "name", md: "The name to vary; local to this call." },
          { name: "lower", type: "expression", md: "Finite lower bound." },
          { name: "upper", type: "expression", md: "Finite upper bound, greater than lower, in its unit." },
        ],
      },
      { h2: "Conventions and limits" },
      { md: "- Golden-section search: reliable for a **unimodal** objective – one valley in the interval. With several valleys it finds *a* minimum, not necessarily the lowest; narrow the bounds around the one you want, or [graph](/docs/graphs) it first.\n- The endpoints are candidates, so a minimum at a bound is found.\n- One variable only; no constraints beyond the bounds. A result is a numerical estimate, not a proof of optimality." },
      {
        quiz: {
          question: "`$minimize((x - 3)^2 + 1, x, 0, 10) =>$` shows…",
          options: ["1", "3", "(3, 1)"],
          answer: 1,
          explain: "minimize returns where the minimum is, x = 3. The minimum value, 1, is the expression evaluated there.",
        },
      },
    ],
    related: ["operations/polyroots", "solving", "numerics", "functions/graph"],
  },

  // ---------- odesolve ----------
  {
    slug: "operations/odesolve",
    title: "odesolve",
    group: "operations",
    section: NUM,
    summary: "Integrates a first-order differential equation dy/dt = f(y, t) from an initial value and returns y at the end time.",
    signature: "odesolve(expression, state, time, initial, start, end)",
    import: "#import operations: odesolve",
    returns: "The state at the end time, in the initial value's unit.",
    keywords: ["ODE", "differential equation", "initial value problem", "Runge-Kutta", "RK4", "integration", "transient", "simulation"],
    blocks: [
      { md: "`odesolve(f, y, t, y₀, t₀, t₁)` solves `dy/dt = f` with `y(t₀) = y₀` and returns `y(t₁)`. The expression may use both the state y and the time t; both names are local, so they don't clash with your definitions.\n\nUnits are checked: the right side must be a unit of y per unit of t. To see the whole trajectory, wrap it in a function of the end time and [graph](/docs/functions/graph) that." },
      {
        example: {
          title: "A cup of coffee cools",
          code: `#import "@preview/solveng:3.0.0": *
#import operations: odesolve, celsius
#import units: min, degC
#show: math

$T_a := celsius(20)$, $τ := 8 #min$
$T_1 := odesolve(-(T - T_a)/τ, T, t, celsius(90), 0 #min, {{end}} #min)$
$T_1 - celsius(0) => degC$`,
          controls: [{ kind: "slider", name: "end", label: "Time", min: 0, max: 40, step: 1, value: 10, suffix: " min" }],
          caption: "Newton's law of cooling: the excess over room temperature decays with time constant τ. Drag the time: after 8 min (one τ) the 70 K excess has fallen to 37 %: 45.8 °C.",
        },
      },
      {
        example: {
          title: "The whole curve",
          code: `#import "@preview/solveng:3.0.0": *
#import operations: odesolve
#import units: min, K
#show: math

$τ := 8 #min$, $θ_0 := 70 #K$, $t_0 := 0 #min$
$θ(t_e: #min) := odesolve(-θ/τ, θ, t, θ_0, t_0, t_e)$
#graph($θ(t_e)$, x-range: (0, 30))`,
          caption: "The excess temperature θ as a function of the end time, plotted. Each point is one integration.",
        },
      },
      {
        example: {
          title: "A tank drains – nonlinear",
          code: `#import "@preview/solveng:3.0.0": *
#import operations: odesolve
#import units: m, s, cm
#show: math

$g := 9.81 #m / #s^2$
$h_1 := odesolve(-1/400 sqrt(2 g h), h, t, 2 #m, 0 #s, {{end}} #s) => #cm$`,
          controls: [{ kind: "slider", name: "end", label: "Time", min: 0, max: 250, step: 10, value: 100, suffix: " s" }],
          caption: "Torricelli: water leaves through an outlet 1/400 of the tank's cross-section at speed √(2gh). The level falls ever more slowly, yet the tank is empty after about 255 s – past that, √h would turn complex, so the slider stops at 250 s.",
        },
      },
      {
        params: [
          { name: "expression", type: "expression", md: "The real right side dy/dt, in the state's unit per the time's unit." },
          { name: "state", type: "name", md: "The local name of the state y." },
          { name: "time", type: "name", md: "The local name of the time t, different from the state." },
          { name: "initial", type: "expression", md: "The state at the start time, y(t₀)." },
          { name: "start", type: "expression", md: "Finite start time t₀." },
          { name: "end", type: "expression", md: "Finite end time t₁, in t₀'s unit. May be before t₀: then it integrates backwards." },
        ],
      },
      { h2: "Conventions and limits" },
      { md: "- **Method:** classic Runge-Kutta (RK4) with adaptive step doubling; relative tolerance 10⁻¹⁰, absolute 10⁻¹² · max(|y₀|, 10⁻¹²) in SI values; at most 100 000 steps.\n- **Scalar only:** one state. A second-order equation – a mass on a spring – is a system of two and isn't supported, nor are events or boundary-value problems.\n- **Not for stiff problems:** when one time constant is much shorter than the time span, the step budget runs out and the error is reported.\n- Outer definitions of the state or time name don't enter the right side – other names (τ, T_a) do." },
      { callout: "warning", title: "Temperatures", md: "`degC` is a temperature *difference*. `T_1 => degC` would show 313 K as \"313 °C\". Subtract `celsius(0)` first, as above, to show a reading on the Celsius scale." },
    ],
    related: ["operations/celsius", "calculus", "numerics", "functions/graph"],
  },

  // ---------- polyroots ----------
  {
    slug: "operations/polyroots",
    title: "polyroots",
    group: "operations",
    section: NUM,
    summary: "All roots of a polynomial, real and complex, from its coefficients in ascending order.",
    signature: "polyroots(coefficients)",
    import: "#import operations: polyroots",
    returns: "Complex vector of the n roots of a degree-n polynomial, sorted by real part, then imaginary part.",
    keywords: ["roots", "zeros", "polynomial", "characteristic equation", "poles", "complex roots", "companion"],
    blocks: [
      { md: "`polyroots(vec(c₀, c₁, …, cₙ))` returns every root of `c₀ + c₁ x + … + cₙ xⁿ` – n of them, counted with multiplicity, complex ones included. Note the order: **constant term first**, ascending powers, the same order [linear-fit](/docs/operations/linear-fit) returns its coefficients in." },
      {
        example: {
          title: "Damping decides",
          code: `#import "@preview/solveng:3.0.0": *
#import operations: polyroots
#show: math

$ζ := {{zeta}}$
$s := polyroots(vec(1, 2 ζ, 1)) =>$`,
          controls: [{ kind: "slider", name: "zeta", label: "Damping ratio ζ", min: 0, max: 2, step: 0.05, value: 0.5 }],
          caption: "The characteristic polynomial s² + 2ζs + 1 of a normalized mass-spring-damper. Below ζ = 1 the roots are a complex pair – it oscillates; at 1 they meet; above, two real roots – it creeps back without overshoot. At ζ = 0 they sit on the imaginary axis: ±i, undamped.",
        },
      },
      {
        example: {
          title: "Picking a root",
          code: `#import "@preview/solveng:3.0.0": *
#import operations: polyroots, at
#show: math

$r := polyroots(vec(-6, 11, -6, 1)) =>$
$x_3 := at(r, 2) =>$`,
          caption: "x³ − 6x² + 11x − 6 = (x − 1)(x − 2)(x − 3). Sorted by real part, the largest root is last.",
        },
      },
      {
        quiz: {
          question: "Which call gives the roots of 2x² − 3x + 1?",
          options: ["`polyroots(vec(2, -3, 1))`", "`polyroots(vec(1, -3, 2))`"],
          answer: 1,
          explain: "Coefficients go constant first: c₀ = 1, c₁ = −3, c₂ = 2. The roots are ½ and 1. The other order – common in other tools – gives the roots of x² − 3x + 2 instead, 1 and 2.",
        },
      },
      { params: [{ name: "coefficients", type: "expression", md: "Vector of 2 to 61 unitless coefficients, constant term first; the last must not be zero." }] },
      { h2: "Conventions and limits" },
      { md: "- Degree 1 to 60. Coefficients are unitless – divide physical ones by a reference first, as with the normalized damper above.\n- Repeated roots appear once per multiplicity, but numerically they are ill-conditioned: a double root may come out as two nearby roots.\n- When you `solve` a polynomial of degree three or more whose coefficients aren't known yet, the solution keeps a `polyroots` that is evaluated once they have values." },
    ],
    related: ["operations/at", "operations/eigenvals", "solving", "complex"],
  },
];
