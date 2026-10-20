// Helpers for the tests: read what def/solve stored and check it.
/// ## Testing
///
/// The tests use [tytanic](https://github.com/typst-community/tytanic) (`cargo install tytanic`); the scripts need Python 3 and `typst` on the `PATH`:
///
/// ```sh
/// tt run                                        # units, solving, lambdify and the README examples (reference images)
/// tt update readme                              # accept changed output of the README examples
/// python3 tests/examples.py                     # every example in README.md and docs/reference.md compiles on its own
/// python3 tests/reference.py                    # docs/reference.md from the doc comments in src/ (--check: is it up to date?)
/// python3 tests/fuzz.py                         # random inputs checked against Python, every category, 300 cases each
/// python3 tests/fuzz.py -n 2000 -c solve1 --seed 7 --save /tmp/fails
/// ```
///
/// Fuzzer categories: `parse` (quantities and math content), `solve1` (random nonlinear equations), `poly`, `system`, `units` (physics formulas with random prefixes), `edge` (hostile inputs must be refused or solved right), `edge-doc` (the same inputs must never stop the compilation), `format`, `ratio`, `display` (random options must never crash). `--typst …` (fuzzer) and `TYPST=…` (examples) pick another compiler, e.g. the oldest supported version.

#import "/lib.typ": *
#import "/src/lambdify.typ": _st, _math-src, _DEF, _U

// the value of x is v (in SI), optionally stored with unit u
#let expect(x, v, u: none) = context {
  let q = _st.get().at(x, default: none)
  assert(q != none, message: x + " is not defined")
  assert(calc.abs(q.v - v) <= 1e-9 * calc.abs(v) + 1e-300, message: x + " = " + repr(q.v) + ", want " + repr(v))
  if u != none { assert.eq(q.u, u, message: x + " has unit " + repr(q.u) + ", want " + repr(u)) }
}
// the last def/solve reported a problem containing msg
#let fails(msg) = context {
  let e = _st.get().at("..").find(r => r.k == "error")
  assert(e != none, message: "expected an error containing '" + msg + "'")
  assert(e.err.contains(msg), message: "error '" + e.err + "' doesn't contain '" + msg + "'")
}
// the last def/solve went through without problems
#let passes() = context {
  let e = _st.get().at("..").find(r => r.k == "error")
  assert(e == none, message: "unexpected error: " + if e != none { e.err })
}
