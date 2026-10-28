# Adversarial regression cases

`core/tests/adversarial.rs` covers the 19 original audit cases: 11 correctness cases,
6 common operations, and 2 resource/robustness cases. All are enabled regression tests.
Four additional tests cover related errors, units, operation boundaries, and convergent integrals.
Only the internal subprocess entry point remains ignored; the resource tests invoke it directly.

Run the regressions, including serial release-mode resource checks:

```sh
cargo test -p solveng-core --release --test adversarial -- --nocapture --test-threads=1
```

Run a single case:

```sh
cargo test -p solveng-core --release --test adversarial negative_square_root_target_has_no_solution -- --exact --nocapture
```

Run normal checks:

```sh
cargo test --workspace
```

The tables below retain the original reproducers and audited behavior for context.
Existing failures recorded in `core/tests/oracle/known.txt` are separate from these regressions.

## Correctness

Expressions below use the readable fixture notation of `core/tests/support/mod.rs`.
For example, `#i` is the imported imaginary constant and `[m]` is the meter binding.

| Test | Reproducer | Audited behavior | Expected behavior |
| --- | --- | --- | --- |
| `zero_divided_by_zero_is_undefined` | `0/0 =` | Returns 0 | Domain error |
| `multiplying_by_zero_preserves_domain_errors` | `0 * (1/0) =` | Returns 0 | Domain error |
| `power_zero_preserves_domain_errors` | `(1/0)^0 =` | Returns 1 | Domain error: the base is undefined |
| `reciprocal_preserves_domain_errors` | `1/(1/0) =` | Returns 0 | Domain error |
| `negative_square_root_target_has_no_solution` | `sqrt(x) = -1 -> solve(x)` | Returns and stores x = 1 | No solution; do not store an extraneous root. Also covers target -2 |
| `divergent_integral_at_midpoint_is_rejected` | `integrate(1/x^2, x, -1, 1) =` | Returns -2 | Reject divergent improper integral |
| `divergent_integral_away_from_midpoint_is_rejected` | `integrate(1/(x - 1/3)^2, x, 0, 1) =` | Returns -4.5 | Reject divergent improper integral, including poles away from sample midpoints |
| `complex_vector_norm_is_positive_definite` | `norm(vec(3 #i, 4))` | Returns sqrt(7) | 5; sum squared magnitudes. Also checks `norm(vec(#i, 1))`, which returned 0 in the audit |
| `complex_matrix_norm_is_positive_definite` | `norm(mat(#i, 0; 0, 1))` | Returns 0 for a nonzero matrix | Frobenius norm sqrt(2) |
| `simplified_log_exp_matches_direct_complex_evaluation` | Define `f(x) := ln(exp(x)) ->`, then evaluate `f(4 pi #i) =` | Returns approximately 12.57i; direct evaluation returns 0 | Preserve the principal logarithm's branch when simplifying a function of an unrestricted argument |
| `zero_with_a_unit_cannot_be_added_to_a_different_dimension` | `0 [m] + 1 [s] =` | Returns 1 s | Unit mismatch; an explicitly dimensioned zero still has a dimension |

The four undefined-expression cases exercise different simplification paths. The integrals
are positive on their domains and diverge to positive infinity, so the negative finite answers
are unambiguously wrong; no numerical reference library is needed. The norm tests use the
standard complex Euclidean/Frobenius norm.

The fixes preserve known undefined operands and dimensioned zeros through simplification,
validate numeric roots introduced by isolation, and check polynomial denominator roots before
using a definite integral's closed form. Integrable affine algebraic singularities remain supported.
Complex norms sum squared magnitudes. Symbolic `ln(exp(x))` retains the principal branch.
The oracle's real-valued samples of `ln(exp(x + y)) - x - y` still evaluate to zero; they no
longer require the expression to simplify to a universal constant for unrestricted complex inputs.

## Common feature gaps

These operations are implemented and exported through Typst's `operations` module and editor
completion. Explicit operation tokens (`⦇name⦈`) exercise the native operation contract.

| Test | Expression | Expected value | Audited behavior |
| --- | --- | --- | --- |
| `quadrant_aware_atan2` | `atan2(1, -1)` and `atan2(-1, -1)` | 3π/4 and -3π/4 | Undefined operation |
| `minimum_of_scalar_values` | `min(3, -2, 7)` | -2 | Undefined operation |
| `maximum_of_scalar_values` | `max(3, -2, 7)` | 7 | Undefined operation |
| `logarithm_with_explicit_base` | `log(8, 2)` | 3 | Rejects the second argument |
| `finite_product` | `prod(k, k, 1, 5)` | 120 | Undefined operation; proposed bound-variable syntax mirrors `sum` |
| `arithmetic_mean_of_vector` | `mean(vec(1, 2, 6))` | 3 | Undefined operation |

`min` and `max` take one or more real scalars with matching units. `atan2(y, x)` takes
real components with matching units and rejects (0, 0). `mean` takes a nonempty vector
with compatible element units and supports complex elements. `log(x, base)` uses the
quotient of principal logarithms; zero arguments and base 1 are undefined.
`prod` uses a local index and known unitless integer bounds, including both endpoints;
an empty product is 1, and more than 10,000 factors reports a limit.

## Resource and robustness failures

`deeply_nested_input_returns_or_reports_a_limit` tests 2,000 matched parentheses, unary
signs, and chained powers through the worksheet API. Excessive delimiter nesting or parser
recursion now returns an explicit limit diagnostic (64 levels/recursive parser frames).

`large_convolution_completes_or_reports_a_limit` evaluates
`convolve(range(50000), range(50000))`. Direct convolution now checks its work before
computing coefficients: more than 1,000,000 coefficient products reports a resource limit.
This applies to numeric and symbolic sequences. Smaller convolutions retain their exact
integer arithmetic and unit handling.

Both tests launch the affected expression in a child process, enforce a three-second
wall-clock deadline, and kill/reap a timed-out child. Compilation is outside the deadline.
A resource diagnostic is accepted; unrelated errors and process crashes fail the test.
Native timing does not establish WASM timing.
