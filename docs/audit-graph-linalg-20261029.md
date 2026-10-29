# Graph and linear algebra audit

The ten regression tests in `core/tests/audit_graph_linalg_20261029.rs` all failed
before fixes; confirmation output was saved to `/tmp/audit-graph-linalg-confirmed.txt`.

| ID | Confirmed defect |
|---|---|
| P01 | Multiplying valid large unit exponents overflows dimension storage and panics instead of diagnosing a representational limit. |
| P02 | `atan2((-2)^127,1)` panics because exact rational absolute value cannot negate the minimum integer. |
| P03 | Transposing `vec()` panics through a zero-sized matrix chunk. |
| P04 | Rank discards incompatible entry dimensions: `rank(mat(1 m,1 s;1 s,1 m))` returns 1 instead of rejecting inconsistent dimensions. |
| P05 | Graph sampling over `[-10^308,10^308]` overflows its interval width and fails to plot `x/10^308`. |
| P06 | Graph auto-ranging treats a varying `10^-20*x` as flat on an absolute scale and hides it in a roughly `[-1,1]` y range. |
| P07 | Dimension errors from later graph curves are silently ignored; `[x,x+1 m]` plots invalid quantities while reversing curve order reports the error. |
| P08 | Infinite explicit bounds return a successful graph containing NaN bounds instead of rejecting the range. |
| P09 | Clipping the data line joining `(-10^308,-10^308)` and `(10^308,10^308)` to `[-10^307,10^307]` produces points outside the range. |
| P10 | Interpolating marks on that finite data line produces NaN y values or NaN root coordinates instead of the origin. |

All ten defects are fixed. A further representative-boundary test checks reverse
quadrants, valid dimension patterns, undefined rank pivots, both curve orders,
nonfinite marks, and mirrored tiny and wide plots. Existing matrix and complex
graph integration suites passed (16 tests). Dimensional exponent storage now has
room for intermediate products; evaluated dimensions retain the explicit exponent
limit, and float-valued dimensional powers use the checked rational path.
