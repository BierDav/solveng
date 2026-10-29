# Extended Solveng bug audit

The user requested at least 100 confirmed bugs, with a fixing subagent assigned
for every ten findings. Counts refer to reproducible
findings in supported behavior, including documented validation contracts.
Variants of a finding are grouped, and absent features are not counted.

| Batch | Findings | Evidence |
|---|---:|---|
| Original audit | 14 | [Initial report](bug-audit-2026-10-29.md) |
| Further numerical operations | 10 | [N01–N10](audit-numeric-20261029.md), `core/tests/audit_numeric_20261029.rs` |
| Symbolic operations | 17 | [S01–S17](audit-symbolic-20261029.md), `core/tests/audit_symbolic_20261029.rs` |
| Frontend and interactive graphs | 23 | [TYP01–TYP23](audit-typst-20261029.md), `tests/audit-20261029/` |
| Graph engine and linear algebra | 10 | [P01–P10](audit-graph-linalg-20261029.md), `core/tests/audit_graph_linalg_20261029.rs` |
| Compiled API and unit parsing | 10 | [A01–A10](audit-api-20261029.md), `core/tests/audit_api_20261029.rs` |
| Sequences and matrix follow-up | 6 | [B01–B06](audit-sequences-20261029.md), `core/tests/audit_sequences_20261029.rs` |
| Editor and language server | 10 | [E01–E10](audit-editor-20261029.md), IDE/LSP regression tests |
| **Confirmed and repaired** | **100** | Regression coverage added |

Each subreport records expected and observed behavior. Regression tests assert
the intended result rather than accepting the bug. This includes 86 findings
beyond the original 14. Temporary discovery probes were removed.

## Validation and limits

- Final `cargo test --workspace --no-fail-fast`: 646 passed, 0 failed, 21 ignored.

- Rebuilt `src/solveng.wasm` with `plugin/build.sh` after the final engine changes.
- All 29 new Typst audit fixtures compile. Including existing fixtures, 48/54
  compile after updating the response-marker fixture and graph style-cycle/unit
  expectations to the corrected behavior.
- All documented examples compile (`python3 tests/examples.py`).
- Expanded core randomized tests pass with `FUZZ_N=3000 FUZZ_SEED=29102026`.
- JavaScript formatter, interpolation, and runtime replacement checks pass.
  HTML checks verify dash phase and gradient diagnostics.
- `git diff --check` passes. Changes are uncommitted.

Six pre-existing Typst fixtures still fail: `matrices`, `scopes`, `signals`,
`solved-functions`, `solving`, and `unit_visibility`. Their assertions rely on
older scope/solve behavior; they were already failing before this audit. They
have not been silently treated as successful checks. The Rust fixtures that
explicitly asserted solving creates global definitions were updated to match
the current read-only solve contract; solved definitions retain all branches.

Interactive HTML gradient strokes are now rejected with a visible documented
diagnostic instead of silently replaced by a flat color. Paged gradients remain
supported. Numeric algorithms retain documented bounded iteration and size limits.

The combined run exposed a repeated-root regression after adding complex roots.
It was corrected by exact synthetic deflation before complex iteration, without
using approximate samples to prove roots real. Related polynomial and symbolic
regressions pass.

