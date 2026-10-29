# Sequence and matrix follow-up audit

Six additional failures are reproduced in `core/tests/audit_sequences_20261029.rs`.
All six tests failed before fixes.

| ID | Defect |
|---|---|
| B01 | Absolute index tolerance makes `range(0,1/10^10)` empty, although its first element 0 lies strictly below the end; near-integer endpoints lose elements too. |
| B02 | Finite bounds ±10^308 and step10^308 are rejected because the intermediate difference overflows before division. |
| B03 | Updating `A[0,0]` when A is a two-row, single-column matrix discards the second row. |
| B04 | Raising a singular matrix inverse to zero suppresses its domain error and returns a value. |
| B05 | Rank treats an undefined symbolic pivot as a valid nonzero value: `rank(mat(1/0,0;0,1))` returns2. |
| B06 | Indexed matrix assignment limits update count but not the dense storage it allocates. A single update at `[999999,999999]` requests 10^12 cells. The regression safely reproduces the missing one-million-cell bound with `[1000,1000]`. |
