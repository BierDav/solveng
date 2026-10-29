# Symbolic audit, 2026-10-29

All ten initial regression tests failed before fixes (`cargo test -p solveng-core --test audit_symbolic_20261029`). Each test states desired behavior. Related examples are variants, not additional counts.

| ID | Repro | Observed | Required |
|---|---|---|---|
| S01 | `sqrt(x^2)=1 -> solve(x)` | Only x=1 | Both x=-1 and x=1; nested inversions must retain preimages |
| S02 | `taylor(abs(x),x,0,2) ->` | Polynomial containing 0/0, no error | Reject undefined derivatives; sqrt, ln, reciprocal variants also reproduce |
| S03 | `integrate(1/(x+i),x,0,1)` | ln(2)/2 | ln(2)/2-iπ/4; reciprocal primitive incorrectly uses ln(abs) for complex values |
| S04 | `integrate(tan(x+i),x,0,1)` | Real 0.1765 | Imaginary component approximately 0.870327425; tangent primitive incorrectly takes modulus |
| S05 | Second derivative of sinc at0 | 0 | -1/3; differentiating the isolated continuation branch loses higher derivatives (also cosc, tanc) |
| S06 | derivative of `cases(x^2,x<0,x)` at0 | 1 | Undefined; branch derivatives are applied at a nonsmooth boundary |
| S07 | `sqrt(x)=-1/10000000000 -> solve(x)` | x=10^-20 | No solution; absolute residual tolerance accepts an extraneous inverse root |
| S08 | `x^3=10^(-60) -> solve(x)` | Only one real root | Three roots; unevaluated closed numeric coefficients force a real-only fallback |
| S09 | `(x-1)^3=10^(-30) -> solve(x)` | Four roots | Three roots; separately computed real and approximate complex roots are merged inconsistently |
| S10 | `integrate(ln(abs(x)),x,-1,1)` | Integrand undefined error | -2; an integrable interior logarithmic singularity needs interval splitting |

Complex `i` in test sources is the explicit imported constant token `#i`.


Seven additional confirmed defects (all seven desired-behavior tests failed before these fixes):

| ID | Repro | Observed | Required |
|---|---|---|---|
| S11 | `integrate(1/(i*x-1),x,-1,1)` and `integrate(tan(x+i),x,0,2*pi)` | Wrong endpoint-log phase/winding | -π/2 and 2πi respectively; principal log endpoint subtraction cannot measure path continuation |
| S12 | `f(x):=diff(abs(x^3),x)` at0 | Division by zero | 0; modulus is Lipschitz at a stationary inner path |
| S13 | `f(x):=diff(sqrt(x^4),x)` at0 | Division by zero | 0; extract complete real powers before a singular chain rule |
| S14 | `f(a,x):=integrate(exp(a*x),x)` at a=0,x=2 | Division by zero | A valid primitive, chosen as x, evaluates to2; parameter-degenerate indefinite primitives need guards |
| S15 | `f(a,x):=invlaplace(1/(s^2+a^2),s,x)` at a=0,x=2 | Division by zero | 2; the quadratic oscillatory inverse continues to the repeated-pole inverse |
| S16 | `f(a,x):=laplace(rect(a*t),t,x)` at a=0,x=2 | Division by zero | 1/2; the zero-scale pulse is the constant1 |
| S17 | `laplace(rect(i*t),t,s)` | A finite formula | An unsupported-domain error; rect requires real inputs (tri and sgn variants also reproduced) |

Implementation uses guarded parameter limits, exact algebra before floating-point root conversion,
local analytic limits at piecewise derivative joins, and numerical quadrature for complex logarithmic
primitives whose path may cross a principal branch cut. Unresolved nonanalytic piecewise limits stay
unsupported rather than using numerical samples as proof.

Validation after fixes: all17 symbolic audit tests pass, including a fourth-degree
sinc Taylor polynomial and tanc continuation. Existing calculus22, cardinal3,
cases6, transforms16 and solved-functions5 tests pass. The transform table's three
symbolic reference formulas now include their correct zero-parameter branches.
