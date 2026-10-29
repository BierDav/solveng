# Editor and language-server audit

Seven IDE tests in `editor/ide/tests/audit_20261029.rs` and three LSP tests in
`editor/lsp/tests/audit_20261029/unit.rs` all failed before fixes.

| ID | Defect |
|---|---|
| E01 | Comma-separated numeric arguments (`0,10`) are combined into one number token. |
| E02 | A block comment before the sum index prevents local binder recognition and links summand references to an outer variable. |
| E03 | Calculus call binders (diff, integrate, taylor, jacobian) do not shadow outer worksheet definitions for navigation/renaming. |
| E04 | Solve targets are treated as new global definitions although runtime solving is read-only. |
| E05 | An equals sign inside a block comment becomes the source hover result (`x :=1 /* =2 */` hovers as `x=2 */`). |
| E06 | Comments before a declaration head or between its name and `:=` hide the declaration from navigation. |
| E07 | Unicode escapes in literal clear arguments are not decoded, so `clear("\\u{78}")` leaves x visible to the editor. |
| E08 | `file://localhost/tmp/a.typ` is decoded as a relative `localhost/tmp/a.typ` path. |
| E09 | Equivalent percent-encoded file URIs do not resolve to the same unsaved open buffer. |
| E10 | LSP semantic tokens truncate multiline comments at the first newline and never highlight subsequent comment lines. |

All ten defects are fixed. The new regression set passes (nine IDE cases including
nested comments and vector binders, four LSP cases including URI aliases). Existing
IDE/LSP suites were run with `--no-fail-fast`; all passed except sum-description
assertions, which passed after restoring the specific sum-index description.
The sums suite (eight tests) and stdio LSP smoke suite (six tests) were rerun and
passed. Three older tests were updated to the existing runtime contract that
solving does not define worksheet variables. No unrelated expectations changed.
