# Solveng: 50 bugs and feature gaps

Audited on 2026-10-29. There are **22 reproduced bugs (B01–B22)** and
**28 feature gaps (F23–F50)**. The feature entries describe proposed behavior,
including extensions to documented limitations; they do not claim that the
current documentation promises those features or spellings. The original audit made no application changes; see the implementation status below.

Each entry has one independently selectable test in
[`core/tests/gap_audit_50.rs`](../core/tests/gap_audit_50.rs). Test names start
with the lowercased finding ID. Related input variations share one test rather
than being counted as additional findings.

## Implementation status

The repair pass resolves B01–B22 and F23–F25, F27, F33–F35, F50.
Their 30 acceptance cases now run in normal CI. The remaining 20 feature cases
stay explicitly ignored until implemented; the original observations below are
retained as the audit record, rather than descriptions of the repaired version.

Remaining: F26, F28–F32, F36–F49. These include special-function continuations,
limits, matrix decompositions, bivariate statistics, splines, extended ODE and
optimization APIs, solve assumptions, and dimensioned polynomial coefficients.

Function definitions retain explicit division and negative-power restrictions
through simplification. Calculus-generated closed forms retain their existing
local guards. Piecewise differentiation remains conservative for piecewise
constant expressions, whose derivatives can require distributions.

## Running the cases

```sh
# Run resolved cases in ordinary CI.
cargo test -p solveng-core --test gap_audit_50

# Exercise proposed capabilities that are still pending (expected to fail).
cargo test -p solveng-core --test gap_audit_50 -- --ignored
```

Examples below use the readable worksheet syntax supported by the existing test
harness: `#i` is the imaginary constant and unit names are explicit bindings.
`Z` and `Rat` examples target the public Rust API directly. B02 and B03 exercise
finite `f64` values near its upper boundary, beyond the worksheet documentation's
approximate 10^300 magnitude range; their issue is in the scaling of the Rust
numeric primitives. Comparisons use numerical values, shapes and dimensions,
rather than the worksheet's rounded display text.

## Reproduced bugs

Priority is suggested triage order: P1 for a panic on worksheet input or a wrong
symbolic branch; P2 for incorrect results or invalid accepted inputs; P3 for
numeric-library boundary behavior. Code links identify the relevant area, not a
claim that the final repair must be confined to that function.

| ID | Priority | Reproduction | Desired behavior | Observed behavior and relevant code |
| --- | --- | --- | --- | --- |
| B01 | P2 | `Z::new(1.0, 1e-12).sqrt()`; worksheet `Im(sqrt(1+#i*10^(-20)))` | Preserve imaginary parts approximately `5e-13` and `5e-21`, respectively; also preserve the sign below the real axis. | Imaginary part becomes exactly zero through cancellation in `abs(z)-Re(z)`. [Complex square root](../core/src/complex.rs). |
| B02 | P3 | `Z::new(1e308,1e308).sqrt()` | Both components finite, approximately `1.09868411346781e154` and `4.55089860562227e153`. | Real component is infinity because `abs(z)+Re(z)` overflows before division by two. Distinct from B01's loss of a small component. [Complex square root](../core/src/complex.rs). |
| B03 | P3 | For `z=Z::new(1e308,1e308)`, evaluate `z.div(z)`. | `1+0i`, also for the conjugate input. | `infinity+0i`: denominator scaling leaves an overflowing numerator sum. [Complex division](../core/src/complex.rs). |
| B04 | P2 | `arcsin(10^300)` | Principal continuation `pi/2 - i*acosh(10^300)`, imaginary part approximately `-691.4686750787736`. | Reports an argument outside −1…1 because the complex continuation squares the input and overflows. [Inverse trig](../core/src/complex.rs), [evaluation](../core/src/eval.rs). |
| B05 | P2 | `tan(1000 #i)` | Approximately `i`, by `tan(i*y)=i*tanh(y)`. | Reports that tangent takes a real argument: the complex path forms overflowing sine/cosine components although the quotient is finite. [Complex function evaluation](../core/src/eval.rs). |
| B06 | P2 | `norm(vec(10^300,10^300))`; tiny variant with `10^(-300)` | `sqrt(2)*10^300` and `sqrt(2)*10^(-300)`. | Large variant reports overflow; tiny variant returns zero. Squaring before scaling loses representable norms. [Array norm](../core/src/linalg/mod.rs). |
| B07 | P2 | Let `n=10^30`, `a=(n+1)/n`, `b=(n+2)/(n+1)` as `Rat`; compare `a.cmp(&b)`. | `Greater`, because `a-b=1/(n*(n+1))>0`. | `Equal`, although exact equality says they differ. Overflowing cross products fall back to identical floats, violating `Ord`/`Eq` consistency. [Rational ordering](../core/src/num.rs). |
| B08 | P2 | `Rat::int(1234567890123456789_i128.pow(2)).root(2)` | `Some(Rat::int(1234567890123456789))`; the square fits in i128. | `None`. A floating estimate plus only its two immediate neighbors does not cover the estimate's integer error. [Exact roots](../core/src/num.rs). |
| B09 | P2 | `Rat::new(i128::MIN,1)` in the default debug test build | A valid rational with that numerator and denominator 1. | Panics on signed absolute-value overflow inside the GCD. The test expects a result, not a panic. [Rational construction/GCD](../core/src/num.rs). |
| B10 | P2 | `(-8)^(1/65)` | Real root approximately `-1.032508634297203`, consistent with the documented real odd-root convention. | Approximately `1.03130289841868+0.04988398151216i`. The rational exponent leaves the bounded fast path and loses the real branch. [Numeric powers](../core/src/num.rs), [real evaluation](../core/src/eval.rs). |
| B11 | P2 | `f(x):=diff(abs(x+#i),x)`; evaluate `f(1)`. | Real `1/sqrt(2)`, since `abs(x+i)=sqrt(x^2+1)` for real x. | `(1+i)/sqrt(2)`. The real-argument `a/abs(a)` derivative is applied to a complex inner expression. [Derivative rules](../core/src/expr.rs). |
| B12 | P2 | `f(x):=diff(abs(x^2),x)`; evaluate `f(0)`. | `0`; the real function is `x^2` and is smooth at zero. | Division-by-zero error from the unsmoothed absolute-value derivative. Distinct from B11's complex inner expression. [Derivative rules](../core/src/expr.rs). |
| B13 | P2 | `lambdify` request with `params=["x","x"]`, also `["alpha","α"]` | Reject duplicate names after normalization, preventing ambiguous argument binding. | Both parameter lists are accepted; evaluation uses only the first matching argument. [Lambda construction](../core/src/api.rs). |
| B14 | P2 | `lambdify("x+1 s", params=["x"], units={x:m})` with no output unit; also `sin(x)` with x in metres. | Reject the known unit mismatch or non-unitless function argument. | Accepts the request and numerically returns `3` for the first expression at x=2. Dimensional checking is only reached inside the optional output-unit branch. [Lambda construction/evaluation](../core/src/api.rs). |
| B15 | P2 | `0 m = 0 s ->` | Reject incompatible equation units or return false, consistent with checking the same equation without `->`. | Returns `true`, although the plain equation reports incompatible units. [Symbolic equation handling](../core/src/sheet.rs). |
| B16 | P2 | `odesolve(cos(256*pi*t),y,t,0,0,1)` | Approximately `0`; the exact solution is `sin(256*pi*t)/(256*pi)`. | Returns `1`. Step-doubling samples alias the forcing and falsely accept convergence. [ODE integration](../core/src/numerics.rs). |
| B17 | P2 | `minimize((x-1000000000000.25)^2,x,1000000000000,1000000000001)` | `1000000000000.25`, within `1e-3`, a resolvable point inside the interval. | Returns the lower endpoint `1000000000000`. Translating a simple unimodal quadratic changes the numerical answer. [Minimization](../core/src/numerics.rs), [expression evaluation](../core/src/eval.rs). |
| B18 | P2 | `f(x):=x/x`; evaluate `f(0)`. | Report the original denominator's excluded input, consistently with direct `0/0`. | Returns `1`: simplifying the stored function discards its domain restriction. [Product/power simplification](../core/src/expr.rs). |
| B19 | P2 | `0^#i`, also `0^(-#i)` | Domain error: a purely imaginary exponent has no limiting value at base zero. | Returns `0`. [Zero-base complex power](../core/src/complex.rs), [power evaluation](../core/src/eval.rs). |
| B20 | P2 | `tan(pi/2)`, `tan(3*pi/2)`, `tanc(pi/2)` | Domain error at the exact symbolic poles. | The first returns approximately `1.633e16`, and cardinal tangent also returns a finite value. [Function simplification](../core/src/expr.rs), [function evaluation](../core/src/eval.rs). |
| B21 | P1 | `f(x):=sqrt(x^3)`; evaluate `f(-1)`. | `i`, matching direct `sqrt((-1)^3)`. | `-i`. Flattening the nested power changes the principal square-root branch. [Nested powers](../core/src/expr.rs). |
| B22 | P1 | `(x^10000000000000000)^(1/2) ->` | A symbolic result or an explicit calculation-limit error. | Panics by unwrapping `as_int()` after checking only `is_int()`; this integer exceeds the conversion's permitted range. [Nested powers](../core/src/expr.rs). |

## Feature gaps and proposed acceptance contracts

These are separate capabilities to consider, not implementation requests in this
change. Several are explicitly outside today's contracts, especially F28,
F38 and F44–F49. Names marked **new API** are suggestions. Tests use explicit
`⦇operation⦈` tokens so an unsupported name cannot be confused with an imported
operation or a test-harness translation issue.

| ID | Missing capability and proposed test input | Acceptance criterion | Current evidence |
| --- | --- | --- | --- |
| F23 | Vector extrema: `min(vec(3,-2,7))`, `max(vec(3,-2,7))` | Scalar results `-2` and `7`. | Scalar variadic extrema exist, but a vector argument reports that one component must be selected. [Operations](../core/src/linalg/mod.rs). |
| F24 | Ordinary piecewise differentiation: `f(x):=diff(cases(x^2,x<0,x^3),x)` | `f(-2)=-4`, `f(2)=12`; this contract makes no assertion at a boundary. | Derivative stays symbolic and cannot be evaluated even inside a smooth branch. [Derivative dispatch](../core/src/expr.rs). |
| F25 | Error-function differentiation: `f(x):=diff(erf(x),x)` | `f(0)=2/sqrt(pi)`. | Numeric erf exists, but its derivative is deferred as unsupported calculus. [Derivative dispatch](../core/src/expr.rs). |
| F26 | **New API** `digamma(x)` | `digamma(2)-digamma(1)=1`, the recurrence needed in gamma derivatives and parameter estimation. | Absent from the builtin registry; calls remain undefined. [Registry](../core/src/builtins.rs). |
| F27 | Bessel differentiation: `f(x):=diff(besselj(0,x),x)+besselj(1,x)` | `f(1)=0`, checking `J0'=-J1`. | Both Bessel values exist, but symbolic differentiation remains unsupported. [Derivative dispatch](../core/src/expr.rs). |
| F28 | Half-integer Bessel orders: `besselj(1/2,pi/2)` | `2/pi`, from the elementary half-integer formula. | Explicitly rejects noninteger orders. This extends the documented integer-order contract. [Special functions](../core/src/numerics.rs). |
| F29 | Complex error function: `erf(#i)` | Purely imaginary `2/sqrt(pi)*sum(1/(k!*(2k+1)),k=0..infinity)`; the test computes an independent convergent series. | Rejects complex arguments despite an entire continuation. [Special functions](../core/src/numerics.rs). |
| F30 | Complex gamma: `abs(gamma(1+#i))^2` | `pi/sinh(pi)`. | Gamma currently requires a real argument. [Special functions](../core/src/numerics.rs). |
| F31 | **New APIs** inverse hyperbolic functions `asinh`, `acosh`, `atanh` | Inverting `sinh(1/2)`, `cosh(1/2)`, `tanh(1/2)` gives `1/2` on their real principal branches. | Forward functions exist; these inverse operations are absent. [Registry](../core/src/builtins.rs). |
| F32 | **New API** `limit(expression,variable,point)` | `limit(sin(x)/x,x,0)=1`, without changing the quotient's value at zero. | There is no general limit operation; ordinary point substitution is insufficient. [Registry](../core/src/builtins.rs). |
| F33 | **New API** `taylor(expression,variable,center,degree)` | `taylor(exp(x),x,0,4)=1+x+x^2/2+x^3/6+x^4/24`. Degree includes the highest retained power. | No Taylor operation; the proposed call stays unresolved in the symbolic equation. [Registry](../core/src/builtins.rs). |
| F34 | **New API** `jacobian(vector_expression,variables)` | Jacobian of `(x*y,x+y)` at `(2,3)` is `mat(3,2;1,1)`, outputs by row and variables by column. | Call stays unresolved rather than producing a matrix. [Registry](../core/src/builtins.rs). |
| F35 | **New API** `hessian(expression,variables)` | Hessian of `x^2+x*y+3*y^2` in `(x,y)` is `mat(2,1;1,6)`. | No Hessian operation. [Registry](../core/src/builtins.rs). |
| F36 | **New API** `eigenvecs(A)` | A normalized column eigenbasis of `diag(2,3)`; either column order and arbitrary signs/complex phases are accepted by the test. | Eigenvalues are available, eigenvectors are not. [Matrix operations](../core/src/numerics.rs). |
| F37 | **New API** `pinv(A)` | Moore–Penrose inverse of `mat(1,2;2,4)` is `mat(1,2;2,4)/25`. | Singular-value decomposition exists, but no public pseudoinverse operation. [Matrix operations](../core/src/numerics.rs). |
| F38 | Minimum-norm linear systems through `lsolve` | `lsolve(mat(1,1),vec(2))=vec(1,1)`; rank-deficient `lsolve(mat(1,1;2,2),vec(1,2))=vec(1/2,1/2)`. | Requires at least as many rows as columns and full column rank. This proposes extending that documented contract. [Linear solver](../core/src/numerics.rs). |
| F39 | **New API** sample `covariance(x,y)` | Vectors `(1,2,3)` and `(2,4,6)` give `2`, using denominator `n-1`. | Individual variance is supported, covariance is absent. [Registry](../core/src/builtins.rs). |
| F40 | **New API** Pearson `correlation(x,y)` | Vectors `(1,2,3)` and `(-2,-4,-6)` give `-1`. | No bivariate correlation operation. [Registry](../core/src/builtins.rs). |
| F41 | **New API** `weightedmean(values,weights)` | Values `(1,3)` and weights `(1,3)` give `5/2`; weights normalize by their sum. | Only unweighted mean is exposed. [Registry](../core/src/builtins.rs). |
| F42 | **New API** empirical `quantile(values,p)` | For unsorted `(20,0,10)` at `p=1/4`, return `5`. Proposed convention: sort and linearly interpolate index `(n-1)*p`. | Median exists; general quantiles do not. The interpolation convention is a proposal, not a unique mathematical requirement. [Registry](../core/src/builtins.rs). |
| F43 | **New API** natural cubic `spline(x,y,at)` | Knots `(0,1,2)`, values `(0,1,0)`, at `1/2` give `11/16`, with zero endpoint second derivatives. | Only piecewise linear interpolation is exposed. [Measured-data operations](../core/src/numerics.rs). |
| F44 | Vector ODE state: `odesolve(vec(v,-u),vec(u,v),t,vec(1,0),0,pi/2)` | State `(0,-1)`, preserving vector shape and variable order. | Rejects non-name local state arguments; ODE systems are a documented exclusion. [ODE parsing](../core/src/parse.rs), [integration](../core/src/numerics.rs). |
| F45 | Sampled ODE output: `odesolve(-y,y,t,1,0,vec(0,1,2))` | Column vector `(1,exp(-1),exp(-2))` at the requested times. | The current API provides a scalar endpoint only; the vector-end request stays unevaluated and produces no trajectory. [ODE evaluation](../core/src/eval.rs). |
| F46 | Complex ODE state: `odesolve(#i*y,y,t,1,0,1)` | `cos(1)+i*sin(1)`. | Complex right side is rejected as undefined/overflowing; the solver samples only real states. [ODE evaluation](../core/src/eval.rs). |
| F47 | Multivariate box minimization: `minimize((x-1)^2+(y+2)^2,vec(x,y),vec(-3,-3),vec(3,3))` | Minimizer `(1,-2)`. | Rejects vector variables. This extends the documented scalar unimodal optimizer. [Numerical binders](../core/src/parse.rs), [minimizer](../core/src/numerics.rs). |
| F48 | Solve with assumptions: `x^2=1 -> assume(x>0),solve(x)` | Exactly the root `1`, assigned to x; the negative root is excluded. | Explicitly prohibits combining `assume` and `solve`. [Clause handling](../core/src/sheet.rs). |
| F49 | Physical polynomial coefficients: `polyroots(vec(-4 m^2,0,1))` | Two real roots `-2 m`, `2 m`, with length dimension on each. | Explicitly accepts only unitless coefficients. This proposes dimension inference across coefficient powers. [Polynomial roots](../core/src/numerics.rs). |
| F50 | Branch-safe inverse-trig composition: `sin(arcsin(x))=x ->` | Prove `true` for the principal complex continuation; do not infer the invalid reverse identity `arcsin(sin(x))=x`. | Leaves a symbolic equation instead of proving this identity. [Function simplification](../core/src/expr.rs), [identity prover](../core/src/identity.rs). |

## Repair verification

`cargo test --workspace`: **559 passed, 0 failed, 21 ignored**. The ignored
cases are the 20 pending feature proposals and the existing subprocess worker.
Three additional validation tests cover derivative binders, Taylor degrees,
and negative-power domain restrictions. The WebAssembly plugin was rebuilt;
the new calculus APIs compile through Typst, all documented examples compile,
and generated unit exports pass their consistency check.

## Original audit verification

`cargo test --workspace` passed: **526 passed, 0 failed, 51 ignored** across
the workspace's test binaries and doc tests. The ignored count includes these
50 new known failures and one pre-existing subprocess worker. The explicit audit
run above separately demonstrated all 50 new failures.

The cases exercise the worksheet harness and public numerical APIs. They do not
claim exhaustive UI, language-server, or Typst rendering coverage. No new
dependencies, application implementations, fixes, or changes to existing tests
are required by this audit. The opt-in tests intentionally remain red until the
individual bugs are repaired or the proposed features are accepted and added.
