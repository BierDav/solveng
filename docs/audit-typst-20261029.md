# Typst and HTML graph audit (2026-10-29)

TYP01–TYP20 are fixed, with passing regression fixtures and JS tests. TYP19 is resolved by explicitly documenting and diagnosing the interactive HTML gradient limitation; paged gradients remain supported. Reproductions use
`typst compile --root . tests/audit-20261029/Txx.typ /tmp/Txx.pdf` from the project root.
All listed failures were reproduced against the rebuilt plugin. Invalid inputs
are included where the documented API promises a document error rather than a
compiler/plugin crash. Related input variants are grouped into one defect.

| ID | Fixture | Expected | Actual / cause |
|---|---|---|---|
| TYP01 | T18 (T19 function variant) | `clear("x")` in `scope:"shared"` keeps the deletion after exit | `x` reappears. `finish` merges inner definitions onto original outer env and cannot represent deletions. |
| TYP02 | T06 (T03–T05 related styles) | Per-curve `auto` inherits worksheet style | `joined:auto` reaches `not` and crashes. Raw per-curve options are not normalized against inherited options. |
| TYP03 | T02 | Reject per-curve color arrays with graph diagnostic, or resolve them consistently | `_graph-style` accepts `(red,blue)`, then raw array passed as stroke paint crashes lilaq. Scalar curve style fields use the global style-array validator. |
| TYP04 | T07 | `clear(none)` gives normal invalid-name diagnostic | `find` returns `none`, indistinguishable from no bad argument; later accesses `none.body` and crashes. |
| TYP05 | T08 | `chips:(type:(fill:1))` gives normal config diagnostic | `_chip-colours` checks keys but not values; `box(fill:1)` crashes. Other palette values have the same root. |
| TYP06 | T09 | `words:(any:1)` gives normal config diagnostic | Type is unchecked; CBOR request fails deserializing integer into string. |
| TYP07 | T10 | Non-math content in a graph range gives document error | `XV` accepts all content, then `src` panics for `[bad]`; the graph's own `errors:"document"` policy is bypassed. Marks and includes share this root. |
| TYP08 | T16 | `defined` accepts the documented worksheet options | `constant-displays` is missing from the explicit parameter list and rejected by Typst. |
| TYP09 | T17 (T01 arbitrary unknown option) | Graph supports current constant display option and validates unknown options | `constant-displays` falls into `args.named`, then is passed to lilaq, whose raw elembic assertion aborts compilation. |
| TYP10 | plot-probe.js | Exact sample `1e-15` formats as nonzero | `fmt` uses absolute `1e-12` zero threshold and prints `0`, independent of graph unit/magnitude. |
| TYP11 | plot-probe.js | No function value outside a segment domain | Interval membership multiplies tiny differences, underflows to zero, accepts outside x; x=0 on segment [1e-200,2e-200] returns y=0. |
| TYP12 | plot-probe.js | Finite interpolation when result is representable | Direct `(yb-ya)*(x-xa)/(xb-xa)` overflows; midpoint of y=[-1e308,1e308] returns Infinity instead of 0. Wide x intervals return wrong finite values; same unscaled interpolation root. |
| TYP13 | plot-lifecycle-probe.js | Replacing a script before DOM ready leaves one active plot runtime | `stop` does not unregister pending `DOMContentLoaded`; both old and new `start` callbacks run, installing two styles and two mutation observers. |

Run the JS probes with `node tests/audit-20261029/plot-probe.js` and
`node tests/audit-20261029/plot-lifecycle-probe.js`; they extract/evaluate the actual
local implementation. Lifecycle probe uses a minimal DOM (no browser work).

Not counted: T11 invalid prefix whitelist and T12 reversed notation thresholds
need further contract clarification. T13 zero relative width is accepted by
Typst but no confirmed incorrect result. T14/T15 invalid predicate return types
lack explicit handling but are user callback contract violations. T20 clear-all
restores initial inherited definitions as documented, so it is not a separate
bug. T21–T24 did not establish defects.

First-batch validation: all 14 isolated Typst regression fixtures pass; `node tests/audit-20261029/plot.test.js` passes.

## Second batch (fixed)

The original TYP11–TYP13 are the first three items below.

| ID | Fixture / evidence | Expected | Actual / cause |
|---|---|---|---|
| TYP14 | T25-discovery.typ | Alternating auto/black override repeats every 2 curves over an underlying 3-color cycle | `_merge` emits only max(3,2)=3 entries, then repeats that wrong period; the fourth curve becomes red instead of black. |
| TYP15 | T26-discovery.typ | Unknown defined argument reports document error as documented | Raw `unexpected argument: banana` stops compilation; no catch-all named argument validation. |
| TYP16 | T27-discovery.typ and T28-discovery.typ | Preserve user vector delimiter/gap and `lr(size:200%)` | `_restyle` reconstructs these nodes with defaults, dropping explicit styling fields. |
| TYP17 | T29-discovery.typ | Retain all nested binding presentation references | `_references(math.root(none, bit))` is empty, while direct `_references(bit)` contains the presentation; traversal omits radicand/rows fields. |
| TYP18 | T30-discovery.typ, HTML export | Respect custom dash phase in interactive graph | Graph style accepts `(array:(4pt,2pt),phase:2pt)`, but JSON exports only `"dash":"5.33 2.67"`; no dashoffset reaches SVG. |
| TYP19 | T31-discovery.typ, HTML export | Gradient curve stays gradient in interactive HTML | `hex(c)` samples every gradient at 50%, exporting one color; browser redraw replaces the correct static gradient with a flat stroke. |
| TYP20 | T32-discovery.typ | Runtime clear failures emit error metadata read by editor | `clear("missing")` draws warning but `reported("missing")` finds no error metadata; `clear` calls `_errc` without `_err-notes` / metadata. |

TYP11/12 helper probes run actual extracted `valueAt`, TYP13 uses actual runtime
script in a minimal loading DOM. TYP18/19 verified by compiling to HTML and
inspecting the actual `.solveng-plot-spec` JSON. Source fixtures are included for
rebuilding those artifacts; no browser automation was needed.


Second-batch validation: T25–T29 and T32 fixtures compile; Node formatting and
interpolation assertions pass; lifecycle probe reports exactly one listener,
style and mutation observer after replacement. HTML T30 exports the correct
2.6666666666666665px dash offset. HTML T31 compiles and reports the gradient
limitation as document metadata rather than silently flattening its paint.

## Third-batch follow-up (fixed)

| ID | Fixture | Expected | Actual / cause |
|---|---|---|---|
| TYP21 | T33-discovery.typ | Alignment point is presentation only; `$x &:= 1$` defines x | `_src` has no align-point case and returns unsupported formula, silently skipping calculation. |
| TYP22 | T36-discovery.typ | Reject unexpected named modifier arguments | `solve(x,typo:1)` silently ignores typo; solve/substitute/assume wrappers take `.pos()` and discard `.named()`. |
| TYP23 | T37-discovery.typ | Unsupported content in a sum/integral bound propagates a normal unsupported-formula result | `_src` directly concatenates bound result without checking `_E`, causing `cannot add string and dictionary`. |

T34 negative dash lengths and T35 negative relative width did not establish a
compiler crash and have not been counted. Unsupported callbacks and speculative
formatting changes are not counted.

TYP21–TYP23 follow-up validation: original T33/T36/T37 fixtures and three
additional variant fixtures compile. Alignment is ignored as presentation outside
cases separators; modifier argument packs validate named arguments before
serialization; sum/integral source conversion checks both bound results before
concatenation. Variants cover all three modifier wrappers and both bounds of
both operators.
