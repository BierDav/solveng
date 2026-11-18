import type { Page } from "../types";

const S = "Vectors & matrices";

export const pages: Page[] = [
  // ---------- range ----------
  {
    slug: "operations/range",
    title: "range",
    group: "operations",
    section: S,
    summary: "Evenly spaced values from start up to, but not including, end – a vector of sample points, with units if you like.",
    signature: "range(end) · range(start, end, step: 1)",
    import: "#import operations: range",
    returns: "Vector of the values, in the unit of start and step.",
    keywords: ["linspace", "sequence", "samples", "sample times", "range variable", "seq"],
    blocks: [
      { md: "`range` counts like Typst's own `range`: from `start` (0 when you give only one argument) in steps of `step`, stopping *before* `end`. The result is a column vector, ready to index with [at](/docs/operations/at) or to plot as data with [graph](/docs/functions/graph).\n\nValues may carry a unit – then `start`, `end` and `step` all need one of the same dimension. That makes sample times read like a lab notebook: `range(0 #s, 1 #s, step: 250 ms)`." },
      {
        example: {
          title: "Sample times with units",
          code: `#import "@preview/solveng:3.0.0": *
#import operations: range
#import units: s, ms
#show: math

$t := range(0 #s, 1 #s, step: {{step}} ms) =>$`,
          controls: [{ kind: "slider", name: "step", label: "Sample period", min: 100, max: 500, step: 50, value: 250, suffix: " ms" }],
          caption: "Drag the period: 1 s itself never appears – the end is left out, as in Typst.",
        },
      },
      {
        example: {
          title: "A sampled sine, plotted as data",
          code: `#import "@preview/solveng:3.0.0": *
#import operations: range, at
#import units: s, ms, V, Hz
#import constants: pi
#show: math

$f := {{f}} #Hz$, $N := 20$
$t := range(0 #s, 20 ms, step: 1 ms)$
$k := 0 .. N - 1$
$u[k] := 5 #V sin(2 pi f at(t, k))$
#graph($u(t)$)`,
          controls: [{ kind: "slider", name: "f", label: "Signal frequency", min: 25, max: 950, step: 25, value: 50, suffix: " Hz" }],
          caption: "20 samples at 1 kHz. Push the frequency toward 500 Hz – half the sample rate – and the dots stop looking like a sine. At 950 Hz they trace a slow 50 Hz wave, upside down: that's aliasing.",
        },
      },
      { md: "`k := 0 .. N - 1` is a *range variable*, as Mathcad writes it: unlike `range`, it **includes** its end. Used as an index, `u[k] := …` defines one element for each value of k." },
      {
        params: [
          { name: "start", type: "expression", default: "0", md: "The first value; with a single argument, that argument is `end` and the range starts at 0." },
          { name: "end", type: "expression", md: "The bound, **not** included." },
          { name: "step", type: "expression", default: "1", md: "Named: the distance between values. Negative counts down. Needs the unit of start and end." },
        ],
      },
      { h2: "Conventions and limits" },
      { md: "- The bounds and the step need values when the range is defined – a free name can't be counted.\n- `range(0, 1, step: 1/4)` stays exact: `vec(0, 1/4, 1/2, 3/4)`.\n- In Typst code (`#for i in range(5)`), `range` is still Typst's own; only math uses solveng's." },
      {
        quiz: {
          question: "How many elements does `range(0, 10, step: 2)` have?",
          options: ["5", "6", "10"],
          answer: 0,
          explain: "0, 2, 4, 6, 8 – the end, 10, is left out. A range variable `0 .. 10` would include it.",
        },
      },
    ],
    related: ["operations/at", "operations/rows", "vectors", "functions/graph"],
  },

  // ---------- at ----------
  {
    slug: "operations/at",
    title: "at",
    group: "operations",
    section: S,
    summary: "One element of a vector or matrix, counted from 0: `at(v, i)` or `at(A, row, column)`.",
    signature: "at(vector, index) · at(matrix, row, column)",
    import: "#import operations: at",
    returns: "The element, with its unit.",
    keywords: ["index", "element", "component", "subscript", "select solution", "zero-based"],
    blocks: [
      { md: "`at` picks one element out of a vector or matrix. Indices start at **0**, like Typst arrays and most programming languages – `at(v, 0)` is the first element. The element keeps its unit.\n\nIt also picks one solution from anything that returns several: the roots of [polyroots](/docs/operations/polyroots), the eigenvalues of [eigenvals](/docs/operations/eigenvals), the coefficients of [linear-fit](/docs/operations/linear-fit)." },
      {
        example: {
          title: "A reading from a log",
          code: `#import "@preview/solveng:3.0.0": *
#import operations: at
#import units: V
#show: math

$u := vec(4.98, 5.02, 5.11, 4.95) #V$
$u_0 := at(u, 0) =>$
$d := at(u, {{i}}) - u_0 =>$`,
          controls: [{ kind: "slider", name: "i", label: "Index", min: 0, max: 3, step: 1, value: 2 }],
          caption: "Four readings, indices 0 to 3. The difference keeps volts.",
        },
      },
      {
        example: {
          title: "A matrix element, and a chosen root",
          code: `#import "@preview/solveng:3.0.0": *
#import operations: at, polyroots
#show: math

$A := mat(2, -1; -1, 2)$
$a_01 := at(A, 0, 1) =>$
$r := polyroots(vec(-6, 1, 1)) =>$
$r_1 := at(r, 1) =>$`,
          caption: "Row 0, column 1 is the upper right element. The roots of x² + x − 6 come sorted, so index 1 is the positive one.",
        },
      },
      {
        params: [
          { name: "vector", type: "expression", md: "A vector or matrix, or the result of a function that returns several values." },
          { name: "index", type: "int", md: "The component – or, with three arguments, the row – starting at 0." },
          { name: "column", type: "int", md: "Optional: the column of a matrix element, starting at 0." },
        ],
      },
      { h2: "Conventions and limits" },
      { md: "- The index must be a nonnegative integer smaller than the number of elements; anything else is reported.\n- In plain Typst math, `at` alone is the symbol @ – import it from `operations` to get the operation.\n- To define every element at once, use a range variable as an index: `x[k] := …` (see [range](/docs/operations/range))." },
      {
        example: {
          title: "One past the end",
          code: `#import "@preview/solveng:3.0.0": *
#import operations: at
#show: math

$v := vec(3, 5, 7)$
$at(v, 3) =>$`,
          expectError: true,
          caption: "Three elements have the indices 0, 1 and 2 – index 3 doesn't exist.",
        },
      },
      {
        quiz: {
          question: "What does `$at(vec(3, 5, 7), 1) =>$` show?",
          options: ["3", "5", "7"],
          answer: 1,
          explain: "Indices start at 0: index 0 is 3, index 1 is 5.",
        },
      },
    ],
    related: ["operations/range", "operations/rows", "vectors", "matrices"],
  },

  // ---------- rank ----------
  {
    slug: "operations/rank",
    title: "rank",
    group: "operations",
    section: S,
    summary: "The number of independent rows of a matrix – how many of your equations actually say something new.",
    signature: "rank(A)",
    import: "#import operations: rank",
    returns: "Integer.",
    keywords: ["linear independence", "redundant equations", "singular", "degrees of freedom"],
    blocks: [
      { md: "A system of linear equations can look bigger than it is: in a circuit, the current law at the last node follows from all the others. `rank` counts the rows that are really independent. When the rank of a square matrix is smaller than its size, the matrix is singular and the system has no unique solution." },
      {
        example: {
          title: "Kirchhoff's current law, node by node",
          code: `#import "@preview/solveng:3.0.0": *
#import operations: rank
#show: math

$K := mat(
  1, 1, 0;
  0, -1, 1;
  -1, 0, -1)$
$r := rank(K) =>$`,
          caption: "Three branches between three nodes, one row per node. The rows add up to zero – the third is minus the sum of the others – so only two equations count.",
        },
      },
      {
        example: {
          title: "When does the redundancy go away?",
          code: `#import "@preview/solveng:3.0.0": *
#import operations: rank
#show: math

$A := mat(1, 2; 2, {{a}})$
$r := rank(A) =>$`,
          controls: [{ kind: "slider", name: "a", label: "Element a₁₁", min: 1, max: 7, step: 1, value: 3 }],
          caption: "The second row is twice the first exactly when a₁₁ = 4. Drag there and the rank drops to 1.",
        },
      },
      { params: [{ name: "A", type: "expression", md: "A vector or matrix." }] },
      { h2: "Conventions and limits" },
      { md: "- Exact numbers are counted exactly – `1/3` is not rounded.\n- Decimals are floats: an element counts as zero when it is within rounding of the largest element. Nearly dependent rows can therefore count as independent; [cond](/docs/operations/cond) tells you *how* nearly." },
    ],
    related: ["operations/cond", "operations/linear-solve", "matrices"],
  },

  // ---------- identity ----------
  {
    slug: "operations/identity",
    title: "identity",
    group: "operations",
    section: S,
    summary: "The n × n identity matrix: ones on the diagonal, zeros elsewhere.",
    signature: "identity(n)",
    import: "#import operations: identity",
    returns: "Square matrix.",
    keywords: ["unit matrix", "eye", "diagonal", "I"],
    blocks: [
      { md: "`identity(n)` is the matrix that changes nothing: `A identity(n)` is A. You'll need it whenever a number has to act on a whole matrix – most famously in the eigenvalue condition `det(A − λ I) = 0`." },
      {
        example: {
          code: `#import "@preview/solveng:3.0.0": *
#import operations: identity
#show: math

$I_3 := identity(3) =>$`,
        },
      },
      {
        example: {
          title: "Checking an eigenvalue",
          code: `#import "@preview/solveng:3.0.0": *
#import operations: identity, eigenvals, at
#show: math

$A := mat(4, 1; 2, 3)$
$λ := eigenvals(A) =>$
$det(A - at(λ, {{i}}) identity(2)) =>$`,
          controls: [{ kind: "slider", name: "i", label: "Which eigenvalue", min: 0, max: 1, step: 1, value: 0 }],
          caption: "Subtracting an eigenvalue from the diagonal makes the matrix singular: its determinant is 0 for both.",
        },
      },
      { params: [{ name: "n", type: "int", md: "Number of rows and columns, 1 or more." }] },
      { h2: "Conventions and limits" },
      { md: "- The elements are unitless. To subtract a value with a unit, multiply: `K - ω^2 M` with a mass matrix, or `A - λ identity(2)` with λ in A's unit." },
    ],
    related: ["operations/eigenvals", "matrices"],
  },

  // ---------- rows ----------
  {
    slug: "operations/rows",
    title: "rows",
    group: "operations",
    section: S,
    summary: "The number of rows of a vector or matrix – for a vector of measurements, how many there are.",
    signature: "rows(A)",
    import: "#import operations: rows",
    returns: "Integer.",
    keywords: ["length", "size", "count", "number of samples", "dimension"],
    blocks: [
      { md: "A vector is a single column, so `rows(v)` is its length. Use it instead of counting by hand, so the worksheet stays right when you add a measurement." },
      {
        example: {
          title: "Standard error of a mean",
          code: `#import "@preview/solveng:3.0.0": *
#import operations: rows, stdev
#import units: mm, μm
#show: math

$d := vec(12.03, 11.98, 12.01, 12.05, 11.99, 12.02) mm$
$n := rows(d) =>$
$s_m := stdev(d) / sqrt(n) => μm$`,
          caption: "Six diameters: the uncertainty of their mean shrinks with √n.",
        },
      },
      {
        example: {
          code: `#import "@preview/solveng:3.0.0": *
#import operations: rows, cols
#show: math

$A := mat(1, 2, 3; 4, 5, 6)$
$rows(A) =>$, $cols(A) =>$`,
          caption: "2 rows, 3 columns.",
        },
      },
      { params: [{ name: "A", type: "expression", md: "A vector or matrix." }] },
    ],
    related: ["operations/cols", "operations/at", "operations/mean"],
  },

  // ---------- cols ----------
  {
    slug: "operations/cols",
    title: "cols",
    group: "operations",
    section: S,
    summary: "The number of columns of a vector or matrix; a vector has one.",
    signature: "cols(A)",
    import: "#import operations: cols",
    returns: "Integer.",
    keywords: ["columns", "size", "dimension", "width"],
    blocks: [
      { md: "`cols` is the partner of [rows](/docs/operations/rows). solveng's vectors are columns, so `cols(vec(…))` is always 1 – handy as a check that a calculation really gave a matrix." },
      {
        example: {
          title: "Unknowns versus equations",
          code: `#import "@preview/solveng:3.0.0": *
#import operations: rows, cols, rank
#show: math

$A := mat(1, 0; 1, 1; 1, 2)$
$m := rows(A) =>$, $n := cols(A) =>$
$rank(A) =>$`,
          caption: "Three equations, two unknowns and rank 2: an overdetermined system, which [linear-solve](/docs/operations/linear-solve) answers in the least-squares sense.",
        },
      },
      {
        example: {
          code: `#import "@preview/solveng:3.0.0": *
#import operations: cols
#show: math

$cols(vec(1, 2, 3)) =>$, $cols(vec(1, 2, 3)^T) =>$`,
          caption: "A vector is one column; transposed, it is one row of three columns.",
        },
      },
      { params: [{ name: "A", type: "expression", md: "A vector or matrix." }] },
    ],
    related: ["operations/rows", "matrices"],
  },

  // ---------- linear-solve ----------
  {
    slug: "operations/linear-solve",
    title: "linear-solve",
    group: "operations",
    section: S,
    summary: "Solves A x = b for x – exactly for a square system, in the least-squares sense for a tall one – with units on every coefficient.",
    signature: "linear-solve(A, b)",
    import: "#import operations: linear-solve",
    returns: "Vector (or matrix, when b has several columns) x, each element in the unit of b over the unit of A's column.",
    keywords: ["lsolve", "linear system", "least squares", "nodal analysis", "Ax = b", "regression", "matrix solve"],
    blocks: [
      { md: "`linear-solve(A, b)` finds the x that makes `A x` closest to `b`. For a square, regular A that is the exact solution; for a tall A – more equations than unknowns, as with measurements – it is the least-squares solution, the one that minimizes ‖A x − b‖.\n\nThe name has a hyphen, which Typst math would read as a minus. Call it with `#` and pass the arguments as math: `#linear-solve($A$, $b$)`. The same operation is also available as `lsolve`, without the hyphen, for writing `lsolve(A, b)` directly." },
      {
        example: {
          title: "Nodal analysis of a resistor network",
          code: `#import "@preview/solveng:3.0.0": *
#import operations: linear-solve
#import units: kΩ, mA, V
#show: math

$I_0 := 2 mA$, $R_1 := 1 kΩ$, $R_2 := 2 kΩ$, $R_3 := {{r3}} kΩ$
$G := mat(1/R_1 + 1/R_2, -1/R_2; -1/R_2, 1/R_2 + 1/R_3)$
$i := vec(I_0, 0 mA)$
$u := #linear-solve($G$, $i$) => #V$`,
          controls: [{ kind: "slider", name: "r3", label: "R₃", min: 0.5, max: 20, step: 0.5, value: 3, suffix: " kΩ" }],
          caption: "A current source feeds node 1; R₁ and R₃ go to ground, R₂ links the nodes. G is in siemens, i in amperes – so the node voltages come out in volts. Make R₃ large and both nodes approach I₀ R₁ = 2 V.",
        },
      },
      {
        example: {
          title: "Least squares: a line through three readings",
          code: `#import "@preview/solveng:3.0.0": *
#import operations: lsolve
#import units: s, V
#show: math

$A := mat(1, 0 #s ; 1, 1 #s ; 1, 2 #s)$
$u := vec(1, 2, 2) #V$
$c := lsolve(A, u) =>$`,
          caption: "Three equations u = c₀ + c₁ t for two unknowns. The first coefficient is in volts, the second in volts per second – each column of A divides b's unit by its own. Note the space in `#s ;`: without it, Typst reads the semicolon as the end of `#s` and the rows run together.",
        },
      },
      {
        params: [
          { name: "A", type: "expression", md: "Square or tall coefficient matrix with full column rank. Each column has one unit down its rows." },
          { name: "b", type: "expression", md: "Right side with as many rows as A: a vector, or a matrix to solve for several right sides at once." },
        ],
      },
      { h2: "Conventions and limits" },
      { md: "- **Units:** `x[i, j]` has the unit of b's column j divided by the unit of A's column i. Residuals are weighed equally in SI values.\n- **Shape:** A needs at least as many rows as columns, and full column rank. A rank-deficient A is reported as an error rather than answered with arbitrary coefficients – check with [rank](/docs/operations/rank).\n- **Method:** a singular-value decomposition, without forming AᵀA, so it stays accurate for poorly scaled problems. Complex values work. At most 256 rows and columns.\n- For a straight-line fit, [linear-fit](/docs/operations/linear-fit) does the setup for you." },
      {
        example: {
          title: "Two equations that are really one",
          code: `#import "@preview/solveng:3.0.0": *
#import operations: lsolve
#show: math

$A := mat(1, 1; 2, 2)$
$lsolve(A, vec(1, 2)) =>$`,
          expectError: true,
          caption: "The second row is twice the first: infinitely many x fit. solveng says so instead of picking one.",
        },
      },
    ],
    related: ["operations/linear-fit", "operations/cond", "operations/rank", "matrices", "solving"],
  },

  // ---------- eigenvals ----------
  {
    slug: "operations/eigenvals",
    title: "eigenvals",
    group: "operations",
    section: S,
    summary: "All eigenvalues of a square matrix – the natural frequencies of a vibrating system, the poles of a state-space model – complex ones included.",
    signature: "eigenvals(A)",
    import: "#import operations: eigenvals",
    returns: "Vector of the eigenvalues, in the unit of A's entries; complex where needed.",
    keywords: ["eigenvalues", "natural frequency", "modes", "spectrum", "characteristic polynomial", "vibration", "stability"],
    blocks: [
      { md: "An eigenvalue λ of A is a number for which `A v = λ v` has a solution v ≠ 0 – a direction A only stretches. In mechanics, the eigenvalues of the stiffness matrix divided by the mass are the squared natural angular frequencies ω²; in control, the eigenvalues of the system matrix are its poles." },
      {
        example: {
          title: "Two masses, three springs",
          code: `#import "@preview/solveng:3.0.0": *
#import operations: eigenvals, at
#import units: kg, N, m, Hz
#import constants: pi
#show: math

$m := 2 #kg$, $k := 800 #N / #m$, $k_c := {{kc}} #N / #m$
$K := mat(k + k_c, -k_c; -k_c, k + k_c) / m$
$λ := eigenvals(K) =>$
$f_1 := sqrt(at(λ, 0)) / (2 pi) => #Hz$
$f_2 := sqrt(at(λ, 1)) / (2 pi) => #Hz$`,
          controls: [{ kind: "slider", name: "kc", label: "Coupling spring k_c", min: 0, max: 1000, step: 50, value: 200, suffix: " N/m" }],
          caption: "Each mass hangs on its own spring k; a spring k_c couples them. With k_c = 0 both oscillate alone at the same frequency. Couple them and the mode where they swing *against* each other gets stiffer and faster.",
        },
      },
      {
        example: {
          title: "A rotation has no real direction",
          code: `#import "@preview/solveng:3.0.0": *
#import operations: eigenvals
#show: math

$R := mat(0, -1; 1, 0)$
$eigenvals(R) =>$`,
          caption: "A quarter turn leaves no direction in place, so its eigenvalues are ±i – solveng returns complex eigenvalues of real matrices.",
        },
      },
      { params: [{ name: "A", type: "expression", md: "Square real or complex matrix, every entry in the same unit." }] },
      { h2: "Conventions and limits" },
      { md: "- The eigenvalues have the unit of A's entries – `1/s²` above, so `sqrt` gives an angular frequency.\n- Their order is **unspecified**; sort or pick with [at](/docs/operations/at) knowing that. Repeated eigenvalues appear as often as their multiplicity.\n- Uses a complex Schur decomposition with a bounded number of iterations; at most 256 rows. Results are floating-point estimates.\n- Only the eigenvalues – not the eigenvectors." },
      { callout: "fun", title: "Try it", md: "In the two-mass example, set k_c to 800 N/m, as stiff as k: the second frequency is then exactly √3 times the first. Why? The first mode swings both masses together, so k_c never stretches: ω₁² = k/m. The second swings them against each other: ω₂² = (k + 2k_c)/m = 3k/m." },
    ],
    related: ["operations/singularvals", "operations/identity", "operations/polyroots", "matrices"],
  },

  // ---------- singularvals ----------
  {
    slug: "operations/singularvals",
    title: "singularvals",
    group: "operations",
    section: S,
    summary: "All singular values of a matrix, largest first – how much it can stretch a vector in each direction, for any shape.",
    signature: "singularvals(A)",
    import: "#import operations: singularvals",
    returns: "Vector of min(rows, cols) nonnegative values in descending order, in the unit of A's entries.",
    keywords: ["SVD", "singular value decomposition", "gain", "norm", "spectrum", "principal"],
    blocks: [
      { md: "Every matrix maps the unit sphere to an ellipsoid; the singular values are the lengths of its half-axes. The largest is the most A can amplify any input – the gain of a multi-input system in its worst direction – and the smallest tells how close A is to losing a dimension. Unlike eigenvalues, singular values exist for rectangular matrices too and are always real and nonnegative." },
      {
        example: {
          title: "Gains of a 3 × 2 system",
          code: `#import "@preview/solveng:3.0.0": *
#import operations: singularvals
#import units: m
#show: math

$A := mat(3 #m, 0; 0, 4 #m ; 0, 0)$
$σ := singularvals(A) =>$`,
          caption: "Two inputs, three outputs: two singular values, sorted 4 m, 3 m – with the entries' unit.",
        },
      },
      {
        example: {
          title: "Squeezing a matrix flat",
          code: `#import "@preview/solveng:3.0.0": *
#import operations: singularvals, cond
#show: math

$A := mat(1, 1; 1, 1 + {{e}})$
$σ := singularvals(A) =>$`,
          controls: [{ kind: "slider", name: "e", label: "Difference ε", min: 0.01, max: 1, step: 0.01, value: 0.5 }],
          caption: "As the rows become alike, the smaller singular value shrinks toward 0 while the larger stays near 2: the matrix flattens the ellipse into a line.",
        },
      },
      { params: [{ name: "A", type: "expression", md: "Real or complex matrix of any shape, every entry in the same unit." }] },
      { h2: "Conventions and limits" },
      { md: "- Returns the complete spectrum, min(rows, cols) values – not a truncated or sparse decomposition – and only the values, not the singular vectors.\n- The largest singular value is the matrix's 2-norm; largest over smallest is [cond](/docs/operations/cond).\n- At most 256 rows and columns; a bounded number of iterations." },
    ],
    related: ["operations/cond", "operations/eigenvals", "operations/rank", "matrices"],
  },

  // ---------- cond ----------
  {
    slug: "operations/cond",
    title: "cond",
    group: "operations",
    section: S,
    summary: "The condition number of a matrix: by how much relative errors in your data can grow in the solution.",
    signature: "cond(A)",
    import: "#import operations: cond",
    returns: "Unitless number ≥ 1.",
    keywords: ["condition number", "sensitivity", "ill-conditioned", "numerical stability", "accuracy"],
    blocks: [
      { md: "`cond(A)` is the largest singular value divided by the smallest. When you solve `A x = b`, a relative error in b – a measurement's tolerance, a rounded constant – can turn into an error up to `cond(A)` times as large in x. A condition number of 1000 means three of your significant digits may be gone.\n\nSo it's the number to check before you trust a solution: near 1 is excellent, in the thousands deserves a look, and around 10¹⁵ means the matrix is singular as far as floating-point numbers can tell." },
      {
        example: {
          title: "Two sensors that see almost the same thing",
          code: `#import "@preview/solveng:3.0.0": *
#import operations: cond
#show: math

$A := mat(1, 1; 1, 1 + {{e}})$
$κ := cond(A) =>$`,
          controls: [{ kind: "slider", name: "e", label: "Difference ε", min: 0.001, max: 1, step: 0.001, value: 0.1 }],
          caption: "Two equations whose rows differ by ε. Drag ε toward 0: the condition number grows like 4/ε, and the system becomes hopeless to solve accurately.",
        },
      },
      {
        example: {
          title: "Scaling matters",
          code: `#import "@preview/solveng:3.0.0": *
#import operations: cond
#show: math

$cond(mat(1, 0; 0, 1)) =>$
$cond(mat(1, 0; 0, 1/100000000)) =>$`,
          caption: "The identity is perfectly conditioned. A diagonal matrix whose entries span eight orders of magnitude has cond = 10⁸ – even though it's trivially invertible on paper.",
        },
      },
      { params: [{ name: "A", type: "expression", md: "Nonempty real or complex matrix, every entry in the same unit." }] },
      { h2: "Conventions and limits" },
      { md: "- The spectral (2-norm) condition number, from [singularvals](/docs/operations/singularvals); unitless, whatever A's unit.\n- A numerically singular matrix – smallest singular value zero within rounding – is reported as an error rather than given an enormous number.\n- A finite result is itself a floating-point estimate: meaningful to about cond × 10⁻¹⁶. At most 256 rows and columns." },
      {
        quiz: {
          question: "You solve a system with `cond(A) ≈ 10^4` from data that is accurate to 5 significant digits. How many digits of the solution can you trust in the worst case?",
          options: ["About 1", "About 5", "About 11"],
          answer: 0,
          explain: "The relative error can grow by up to 10⁴ – four digits lost from five. Computing with more digits doesn't help: the data's error is what gets amplified.",
        },
      },
    ],
    related: ["operations/singularvals", "operations/linear-solve", "operations/rank", "matrices"],
  },
];
