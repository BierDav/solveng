# Additional numerical audit

Each N01–N10 is reproduced by the matching test in
`core/tests/audit_numeric_20261029.rs`. All ten tests failed before fixes.

| ID | Defect | Reproduction / expected |
|---|---|---|
| N01 | Ceiling negates minimum exact integer and panics | `ceil((-2)^127)` must remain that integer |
| N02 | Inverse sine cancels tiny imaginary signals | `Im(asin(i*10^-20))` must be about `10^-20` |
| N03 | Inverse tangent cancels tiny imaginary signals | `Im(atan(i*10^-20))` must be about `10^-20` |
| N04 | Complex hyperbolic tangent overflows at saturation | `tanh(1000+i)` must approach 1 |
| N05 | Normal quantile overflows before cancellation | `qnorm(.99,-10^308,10^308)` must be about `1.32635*10^308` |
| N06 | Polynomial root iteration ignores root scale | Roots of coefficients `[10^-200,0,10^200]` must be `±i*10^-200` |
| N07 | Negative integer powers underflow through overflowing reciprocal | `10^-320` is representable and nonzero |
| N08 | Number formatter emits infinite mantissa for finite subnormal | Formatting `1e-320` must preserve finite nonzero value |
| N09 | Quadratic discriminant overflow loses finite roots | `x²+10^200*x+1=0` has two finite roots |
| N10 | Numeric error projection removes meaningful complex branch component | `acos(1+i*10^-20)` has real and imaginary components about `10^-10` |
