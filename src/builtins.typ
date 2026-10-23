/// ## Built-in operations
///
/// Built-in operations work in worksheet math, calculation strings and graphs. Import solveng with `*`
/// to bring its additional math names into scope. These exported names are math content: write
/// `$tri(t)$` or `#worksheet("tri(0) =")`, rather than `#tri(0)`. Use a worksheet function such as
/// `$Lambda(t) := tri(t)$`, or a function overlay, for another name.
///
/// **Function overlays.** Names reach operations as units do, through layers: the underlying
/// operations, the built-in overlay, then user overlays in order. The built-in overlay gives the
/// standard names (`tri`, `convolve`, …) to plain lookup: names in code lines and quoted names in
/// math. `builtin-functions: false` disables it, never the operations. Each `functions: (…)` adds a
/// user overlay, scoped to its worksheet: `(conv: "convolve", Λ: "tri")` maps names to operations
/// (entries may refer to each other in any order, a self reference to the overlays below, cycles
/// are rejected), `(rect: none)` hides a name from plain lookup. A worksheet's own function of a
/// name takes precedence over plain lookup, so `tri(x) := …` in code replaces `tri` there.
///
/// An explicit reference is Typst's `math.op("name")`: what Typst's own `sin` and `det` are, and
/// what solveng exports for math (`tri`, `convolve`). It resolves through the overlays active where
/// it is used and always reaches an operation, even below a `none` or without the built-in overlay;
/// a `math.op` of a name that is no operation (`min`) is a name like any other. In Typst math,
/// named arguments (`step: 10 ms`) parse only for functions: `operation("name")` makes an explicit
/// reference that is one, `#let conv = operation("conv")`, as `convolve` and `range` are.
///
/// ```typ
/// #import "@preview/solveng:1.0.0": *
/// #show: worksheet.with(functions: (conv: "convolve", Λ: "tri"))
/// #let conv = operation("conv")
/// $y := conv(vec(1, 2), vec(1, 1), step: 2) =$
/// $a := "Λ"(1/2) =$
/// #worksheet("b := conv(vec(1), vec(3)) =")
/// ```
///
/// The following operations use Typst's existing math names or syntax:
///
/// | operation | behavior |
/// |---|---|
/// | `sqrt(x)` | Real square root; x must be nonnegative. Units are square-rooted too: `sqrt([9 m^2])` is 3 m. |
/// | `abs(x)` | Absolute value, preserving x's unit. |
/// | `exp(x)`, `ln(x)`, `log(x)` | Exponential, natural logarithm and base-10 logarithm. Arguments are unitless; logarithms require x > 0. |
/// | `sin(x)`, `cos(x)`, `tan(x)` | Trigonometric functions of a unitless angle in radians. |
/// | `arcsin(x)`, `arccos(x)`, `arctan(x)` | Inverse trigonometric functions; also spelled `asin`, `acos`, `atan`, documented below. |
/// | `sinh(x)`, `cosh(x)`, `tanh(x)` | Hyperbolic functions of a unitless argument. |
/// | `vec(a, b, …)` | Column vector. Use `at(vector, index)` to select a component, starting at 0. |
/// | `mat(a, b; c, d)` | Matrix, rows separated by semicolons, blocks allowed: `mat(A, b)`, `mat(A; B)`. `A B` multiplies, `A^(-1)` inverts, `A^T` transposes, `a times b` is the cross product. Use `at(matrix, row, column)` for an element. |
/// | `det(A)`, `tr(A)`, `norm(v)` | Determinant and trace of a square matrix; length of a vector (‖v‖). |
/// | `cases(value, condition, …, default)` | First matching branch; conditions are unitless, and only the selected value is evaluated. The final default is optional. In math use `cases(0 & t < 0, 1 & "otherwise")`. |
/// | `laplace(expression, variable, result-variable)`, `invlaplace(expression, variable, result-variable)` | Symbolic unilateral transforms, also used after `->`. Defaults: t → s and s → t. See their entries below for supported forms. |
/// | `sum(expression, index, lower, upper)` | Sum over inclusive integer bounds; the index is local and unitless. In math use `sum_(k=0)^10 k^2`. Small finite sums simplify exactly; infinite sums stay symbolic with `->` and are evaluated for `=` and graphs. |
///
/// For sums, the upper bound may be `infinity`. Piecewise linear signal sums are counted
/// directly between their corners; other numerical sums use a shrinking-tail convergence
/// estimate with a limit of 100000 terms. Nonconvergent or slower-converging series report an error.
///
/// ```typ
/// #import "@preview/solveng:1.0.0": *
/// #show: worksheet
/// $a := sqrt(9 m^2) =$
/// $b := sum_(k=0)^3 k^2 =$
/// $c := cases(0 & b < 0, b & "otherwise") =$
/// ```

#import "operation.typ": operation, _op-call

/// Rectangular pulse: `rect(x)` is 1 for |x| < ½, ½ at x = ±½, and 0 otherwise.
///
/// The argument and result are unitless. Shift and scale the argument to place a pulse:
/// `$p(t) := rect((t-2)/4)$` has width 4 and is centered at 2.
/// `$rect(1/2) =$` gives ½. Integrals split at the pulse boundaries; derivatives stay symbolic.
///
/// - x (expression): Unitless position relative to the pulse center.
/// -> unitless value
#let rect = math.op("rect")

/// Triangular pulse: `tri(x)` is max(1 − |x|, 0), with height 1 at 0 and support from −1 to 1.
///
/// The argument and result are unitless. `$p(t) := tri((t-2)/4)$` is centered at 2 and
/// reaches zero at −2 and 6. `$tri(1/4) =$` gives ¾. Integrals split at its corners;
/// the first derivative is piecewise constant and undefined at the three corners.
///
/// - x (expression): Unitless position relative to the pulse center.
/// -> unitless value
#let tri = math.op("tri")

/// Greatest integer less than or equal to x: `floor(x)`.
///
/// The argument and result are unitless. Used for finite time-delay sums.
/// Its derivative stays symbolic.
///
/// - x (number): Unitless real value.
/// -> integer
#let floor = math.op("floor")

/// Factorial of a nonnegative integer: `factorial(k)`.
///
/// Zero factorial is 1. Values through 33! are exact; larger values can be
/// evaluated numerically through 170!. Its derivative stays symbolic.
///
/// - k (integer): Nonnegative unitless integer.
/// -> number
#let factorial = math.op("factorial")

/// Sign function: `sgn(x)` is −1 for x < 0, 0 for x = 0, and 1 for x > 0.
///
/// The argument and result are unitless. `$sgn(-3) =$` gives −1.
/// Its distributional derivative stays symbolic.
///
/// - x (expression): Unitless value whose sign to take.
/// -> unitless value
#let sgn = math.op("sgn")

/// Symbolically differentiates an expression with respect to a name: `diff(expression, variable)`.
///
/// `$diff(x^3, x) ->$` gives 3x². The variable is local to the operation, so an existing
/// worksheet value for it does not replace it while differentiating. Unsupported piecewise
/// or distributional derivatives stay symbolic and cannot be evaluated numerically.
///
/// - expression (expression): Expression to differentiate.
/// - variable (name): Differentiation variable, written as a name rather than a value.
/// -> expression
#let diff = math.op("diff")

/// Definite integral: `integrate(expression, variable, lower, upper)`.
///
/// `$integrate(x^2, x, 0, 3) =$` gives 9. The variable is local to the operation.
/// Bounds must have matching units; the result has the integrand's unit times the
/// variable's unit. Either bound may be infinite. Recognized antiderivatives simplify
/// exactly; remaining integrals stay symbolic with `->` and use adaptive quadrature for
/// `=` or graphs. Affine signal arguments are split at their corners so narrow pulses
/// are not skipped. Undefined integrands or failure to converge report an error.
///
/// - expression (expression): Integrand.
/// - variable (name): Integration variable, written as a name rather than a value.
/// - lower (expression): Lower bound, including `-infinity`.
/// - upper (expression): Upper bound, including `infinity`.
/// -> expression or value
#let integrate = math.op("integrate")

/// Convolution: `convolve(x, h)` of two sequences, `convolve(f(τ), g(τ), τ, t)` of two functions.
///
/// Of sequences (vectors) it is the discrete convolution, (x ∗ h)ₖ = Σᵢ xᵢ hₖ₋ᵢ, with n + m − 1
/// elements; `step: Δt` multiplies it by the sampling step, the Riemann sum of the continuous convolution of
/// the sampled signals. Of functions it is (f ∗ g)(t) = ∫ f(τ) g(t − τ) dτ over the whole line, τ local to
/// the operation. Without `at`, the result is in the variable itself: `convolve(rect(t), rect(t), t)` is
/// `tri(t)`. Piecewise polynomial signals – `rect`, `tri`, `sgn`, `abs`, `cases` steps, polynomials, with
/// numeric affine arguments such as `tri((τ - 2 s)/(4 s))` – convolve exactly into a piecewise polynomial;
/// `$y(t) := convolve(tri(tau), tri(tau), tau, t) ->$` shows the five pieces, `$y(1/2) =$` gives 23/48.
/// Other signals (exp, sin, symbolic shifts) stay the integral, evaluated numerically for `=` and graphs.
/// Units multiply: the result has f's unit times g's unit (times τ's unit, or the step's).
///
/// - f (expression): A sequence (vector), or a function of the variable.
/// - g (expression): A sequence (vector), or a function of the variable.
/// - variable (name): The integration variable of a convolution of functions.
/// - at (expression): Where the convolution of functions is taken, usually the result's variable; default: the variable.
/// - step (expression): Named: the sampling step of sequences, the result is multiplied by it.
/// -> vector or expression
#let convolve = operation("convolve")

/// Values from start to end, as Typst's range: `range(end)`, `range(start, end, step: 1)`, the end left out.
///
/// `$range(0, 1, step: 1/4)$` is `vec(0, 1/4, 1/2, 3/4)`. Values may have a unit, then the step needs it too:
/// `$range(0 s, 1 s, step: 250 ms)$`. The bounds and the step need values when the range is defined. A
/// range variable, as Mathcad writes it, includes the end: `$i := 0 .. N$` (or `0...N`) counts from 0 to N in
/// steps of 1. Used as an index, `$x[i] := tri(t[i])$` defines an element for each value. In Typst code
/// (`#for i in range(5)`), `range` stays Typst's own.
///
/// - start (expression): The first value; default 0 with one argument.
/// - end (expression): The bound, not included.
/// - step (expression): Named: the distance between values, default 1; negative counts down.
/// -> vector
#let range = (..args) => if args.pos().all(a => type(a) in (int, float)) and args.named().values().all(a => type(a) in (int, float)) { std.range(..args) } else { _op-call("range", args) }

/// Selects a vector component with a zero-based index: `at(vector, index)`, or a matrix
/// element: `at(matrix, row, column)`.
///
/// `$at(vec(3, 5), 0) =$` gives 3. The index must be a nonnegative integer smaller
/// than the number of components. This also selects a solution from a function that
/// returns several solutions as a vector. The selected component keeps its unit.
///
/// - vector (expression): A vector or matrix, including the result of a function returning a vector.
/// - index (int): Component number, or the row of a matrix, starting at 0.
/// - column (int): Column of a matrix element, starting at 0.
/// -> expression or value
#let at = math.op("at")

/// Inverse sine, also spelled `arcsin`: `asin(x)` returns an angle in radians from −π/2 to π/2.
///
/// The argument is unitless and must lie in [−1, 1]; the result is unitless.
/// `$asin(1) =$` gives π/2.
///
/// - x (expression): Unitless sine value in [−1, 1].
/// -> unitless angle in radians
#let asin = math.op("asin")

/// Inverse cosine, also spelled `arccos`: `acos(x)` returns an angle in radians from 0 to π.
///
/// The argument is unitless and must lie in [−1, 1]; the result is unitless.
/// `$acos(1) =$` gives 0.
///
/// - x (expression): Unitless cosine value in [−1, 1].
/// -> unitless angle in radians
#let acos = math.op("acos")

/// Inverse tangent, also spelled `arctan`: `atan(x)` returns an angle in radians between −π/2 and π/2.
///
/// The argument and result are unitless. `$atan(1) =$` gives π/4.
///
/// - x (expression): Unitless tangent value.
/// -> unitless angle in radians
#let atan = math.op("atan")

// Native operation documentation is also read by the IDE. Each @builtin marker
// names the operation documented by the preceding /// comment.

/// Real square root: `sqrt(x)` returns the nonnegative square root of x.
///
/// x must be nonnegative. Units are square-rooted too: `sqrt([9 m^2])` is 3 m.
/// - x (expression): Nonnegative value, optionally with units.
// @builtin sqrt

/// Absolute value: `abs(x)` removes the sign and preserves x's unit.
///
/// `abs([-3 V])` is 3 V.
/// - x (expression): Value whose magnitude to take.
// @builtin abs

/// Exponential: `exp(x)` is e raised to the power x.
///
/// The argument and result are unitless. `exp(0)` is 1.
/// - x (expression): Unitless exponent.
// @builtin exp

/// Natural logarithm: `ln(x)` is the inverse of `exp(x)`.
///
/// The argument must be positive and unitless. `ln(e)` is 1.
/// - x (expression): Positive unitless value.
// @builtin ln

/// Base-10 logarithm: `log(x)` is the exponent giving x as a power of 10.
///
/// The argument must be positive and unitless. `log(100)` is 2.
/// - x (expression): Positive unitless value.
// @builtin log

/// Sine of a unitless angle in radians: `sin(x)`.
///
/// `sin(pi/2)` is 1. The result is unitless and lies in [−1, 1].
/// - x (expression): Unitless angle in radians.
// @builtin sin

/// Cosine of a unitless angle in radians: `cos(x)`.
///
/// `cos(0)` is 1. The result is unitless and lies in [−1, 1].
/// - x (expression): Unitless angle in radians.
// @builtin cos

/// Tangent of a unitless angle in radians: `tan(x) = sin(x)/cos(x)`.
///
/// `tan(0)` is 0. The function has poles at π/2 + kπ; the result is unitless.
/// - x (expression): Unitless angle in radians, away from a pole.
// @builtin tan

/// Hyperbolic sine: `sinh(x) = (exp(x) - exp(-x))/2`.
///
/// The argument and result are unitless. `sinh(0)` is 0.
/// - x (expression): Unitless value.
// @builtin sinh

/// Hyperbolic cosine: `cosh(x) = (exp(x) + exp(-x))/2`.
///
/// The argument and result are unitless. `cosh(0)` is 1.
/// - x (expression): Unitless value.
// @builtin cosh

/// Hyperbolic tangent: `tanh(x) = sinh(x)/cosh(x)`.
///
/// The argument and result are unitless. `tanh(0)` is 0.
/// - x (expression): Unitless value.
// @builtin tanh

/// Column vector: `vec(a, b, …)` keeps the components in the written order.
///
/// Components can carry units. Use `at(vector, index)` to select a component,
/// starting at 0: `at(vec(3, 5), 1)` is 5.
/// - ..components (expression): Vector components, in order.
// @builtin vec

/// Piecewise expression: `cases(value, condition, …, default)` selects the first matching branch.
///
/// Conditions must be unitless; nonzero means true. Only the selected value is evaluated.
/// The final default is optional; without a matching branch the value is undefined.
/// In math use `cases(0 & t < 0, 1 & "otherwise")`.
/// - ..branches (expression): Alternating values and conditions, optionally followed by a default value.
// @builtin cases

/// Inclusive integer sum: `sum(expression, index, lower, upper)`.
///
/// The index is local and unitless. Bounds are unitless integers; the upper bound may be
/// `infinity`. In math use `sum_(k=0)^10 k^2`. Small finite sums simplify exactly;
/// infinite sums stay symbolic with `->` and are evaluated for `=` and graphs.
/// Piecewise linear signals are counted directly between their corners. Other numerical
/// sums use a shrinking-tail convergence estimate and report an error after 100000 terms.
/// - expression (expression): Summand; it may carry units.
/// - index (name): Local summation index, written as a name.
/// - lower (expression): Inclusive lower integer bound.
/// - upper (expression): Inclusive upper integer bound, or `infinity`.
// @builtin sum

/// Unilateral Laplace transform: `laplace(expression, variable, result-variable)`.
///
/// The variables default to t and s. Use `$t^2 -> laplace$`,
/// `$exp(-x) -> laplace(x, p)$`, or `$laplace(t^2) ->$`.
/// The input variable is local, so a worksheet value for it is ignored.
/// Supports constants, polynomials through degree 33, affine exponentials,
/// sine, cosine, hyperbolic sine and cosine, linear combinations, exponential
/// shifts and multiplication by powers of the input variable (up to 20).
/// Also supports `rect(a*t)`, `tri(a*t)` and `sgn(a*t)` for nonzero a,
/// and shifted affine signal arguments with known real numeric coefficients.
/// These are unilateral transforms: only the signal at t >= 0 contributes.
/// For example, `laplace(rect(t))` is (1 − exp(−s/2))/s and
/// `laplace(tri(2*t))` is 1/s − 2(1 − exp(−s/2))/s².
/// Define `$Lambda(t) := tri(t)$` before using Λ as the triangular signal.
/// Parameters are formal constants; results apply in the half-plane of convergence.
/// The result variable must differ from the input and must not occur in the input
/// expression. Unsupported transforms report an error; no numerical transform is used.
///
/// - expression (expression): Function to transform.
/// - variable (name): Input variable, t by default.
/// - result-variable (name): Output variable, s by default.
/// -> expression
#let laplace = math.op("laplace")

/// Inverse unilateral Laplace transform: `invlaplace(expression, variable, result-variable)`.
///
/// The variables default to s and t. Use `$1/(s+1) -> invlaplace$`,
/// `$1/(p+1) -> invlaplace(p, x)$`, or `$invlaplace(1/s^2) ->$`.
/// Proper rational functions are inverted symbolically using repeated poles,
/// quadratic factors and exact partial fractions where available. The general
/// numerical evaluator uses a state-space matrix exponential, including complex
/// poles and repeated irreducible factors; it does not require numerical roots.
///
/// Rational delay expressions `N(s,q)/D(s,q)`, with `q = exp(-d*s)`, are handled
/// by formal power-series division and causal shifts. Delays must be commensurate
/// rational numbers. Products, powers, `e^x` and hyperbolic functions are normalized
/// before inversion. The constant delay coefficient D_0(s) must be nonzero;
/// each numerator coefficient must have lower degree in s than D_0, and no
/// denominator coefficient may have higher degree. Polynomial parts requiring
/// Dirac distributions remain unsupported.
///
/// Separable delay denominators use generating-function partial fractions to
/// produce exact finite sums when their delay poles can be determined exactly.
/// Other supported inverses remain written as `invlaplace(...)` and can be
/// evaluated or graphed. Numerical evaluation uses at most 256 delay intervals
/// and a state dimension of 128; exceeding these limits reports an error.
///
/// For a system taking `rect(t)` to `tri(2*t)`, shift both signals to preserve
/// their negative-time parts under the unilateral transform:
/// `$x(t) := rect(t-1/2)$`, `$y(t) := tri(2*(t-1/2))$`,
/// `$H(s) := laplace(y(t), t, s)/laplace(x(t), t, s) ->$`, then
/// `$g_2(t) := invlaplace(H(s)/s, s, t) ->$` gives the step response.
/// It is a periodic triangular wave: 0 at integer times and 1 at half-integers.
/// Results describe the causal function for t >= 0.
/// The input variable is local. The output name must be distinct and absent from
/// the input expression. Unsupported forms report an error.
/// Arrow transforms cannot be combined with `solve` or a second transform keyword.
///
/// - expression (expression): Expression in the transform domain.
/// - variable (name): Input variable, s by default.
/// - result-variable (name): Output variable, t by default.
/// -> expression
#let invlaplace = math.op("invlaplace")

/// Matrix: `mat(a, b; c, d)`, commas between the elements of a row and semicolons between rows.
///
/// Elements can carry units and names. A matrix of one column is a vector, the same as `vec(…)`.
/// Elements that are matrices are blocks: `mat(A, b)` puts A and b side by side, `mat(A; B)`
/// puts B below A, `mat(A, 0; 0, B)` builds a block diagonal from scalars and blocks.
///
/// Everything else is Typst's math notation. `+` and `−` work element by element (a scalar is
/// added to every element); products follow the written order: `A B` is the matrix product,
/// `A v` a matrix times a vector, `v dot w` or `v w` the dot product of two vectors, `v^T A` a row
/// times a matrix, and `a times b` the cross product of two vectors of 3 components. `A^n` is a
/// whole power of a square matrix, `A^(-1)` its inverse, `A^T` or `A^top` its transpose, `|A|` its
/// determinant; `|v|` and `norm(v)` are the length of a vector. `A^(-1) b` solves the system
/// A x = b without forming the inverse, and `A vec(x, y) = b -> solve(x, y)` solves a matrix
/// equation element by element.
///
/// Determinants, inverses and systems are exact for exact numbers and formulas for names. With
/// units, every row and column needs a unit of its own (element = row unit · column unit), as in
/// physical matrices: a resistance matrix inverts to conductances. Names without a value are
/// scalars until they get one.
/// - ..rows (expression): Elements or blocks, row by row.
// @builtin mat

/// Determinant of a square matrix: `det(A)`, also written `|A|`.
///
/// Exact for exact numbers, a formula for names: `$det(mat(a, b; c, d)) ->$` gives a d − b c.
/// With units, the determinant has the product of the row and column units.
/// - matrix (expression): A square matrix.
// @builtin det

/// Trace of a square matrix: `tr(A)`, the sum of its diagonal.
/// - matrix (expression): A square matrix.
// @builtin tr

/// Norm: `norm(v)`, written ‖v‖, is the length of a vector and the Frobenius norm of a matrix
/// (the root of the sum of its squared elements); of a number, its absolute value.
/// - x (expression): A vector, matrix or number.
// @builtin norm

/// Rank of a matrix: `rank(A)`, the number of independent rows.
///
/// Exact for exact numbers; floats count as zero within rounding of the largest element.
/// - matrix (expression): A vector or matrix.
/// -> integer
#let rank = math.op("rank")

/// Identity matrix: `identity(n)` has n rows and columns, ones on the diagonal.
/// - n (int): Number of rows and columns, 1 or more.
/// -> matrix
#let identity = math.op("identity")

/// Number of rows of a vector or matrix: `rows(A)`.
/// - matrix (expression): A vector or matrix.
/// -> integer
#let rows = math.op("rows")

/// Number of columns of a vector or matrix: `cols(A)`; a vector has 1.
/// - matrix (expression): A vector or matrix.
/// -> integer
#let cols = math.op("cols")
