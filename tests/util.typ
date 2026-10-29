// Helpers for the tests: read what the worksheet defined and check it.
/// ## Testing
///
/// The engine is tested in Rust, the package with [tytanic](https://github.com/typst-community/tytanic) (`cargo install tytanic`); the scripts need Python 3 and `typst` on the `PATH`:
///
/// ```sh
/// cargo test                                    # the engine (units, parsing, algebra, solving, worksheets, graphs, random inputs) and the editor (IDE, language server)
/// plugin/build.sh                               # src/solveng.wasm from the engine (rustup target add wasm32-unknown-unknown; wasm-opt if installed)
/// tt run                                        # units, complex numbers, solving, names, lambdify, … and the README examples (reference images)
/// tt update readme document                     # accept changed output of the README and document examples
/// python3 tests/examples.py                     # every example in README.md and docs/reference.md compiles on its own
/// python3 tests/reference.py                    # docs/reference.md from the doc comments in src/ (--check: is it up to date?)
/// python3 scripts/unit-words.py                # src/unit-words.typ (the units module) from the unit table (--check: is it up to date?)
/// uv run --no-project --with sympy --with mpmath python tests/oracle/generate.py   # the reference results
/// ```
///
/// `cargo test` includes a fuzzer (`core/tests/fuzz.rs`): random nonlinear equations, polynomials, systems, physics formulas with random prefixes and random numbers to format, checked against direct calculation, and functions whose shown return unit must match what their calls produce; and one for the editor (`editor/ide/tests/fuzz.rs`): random and mutated documents through every editor feature at random positions, which must neither panic nor return ranges outside the text (`FUZZ_N=5000 cargo test --release fuzz` for more cases). `core/tests/oracle.rs` checks families of matrices, integrals, series, simplifications and equations against SymPy and mpmath (the results are in `core/tests/oracle/`, so `cargo test` needs neither); what the engine can't do yet is listed in `core/tests/oracle/known.txt`, which only shrinks: a listed case that passes fails the test until `ORACLE_BLESS=1 cargo test --test oracle` updates the list. `TYPST=…` picks another compiler for the examples, e.g. the oldest supported version.

#import "/lib.typ": *
#import "/src/api.typ": _rows, _st, _cf

// definitions as equations, silently
#let def(..eqs) = calculate(..eqs.pos(), hidden: true)
// the value of x is v (in SI), optionally shown in unit u
#let expect(x, v, u: none) = context {
  let r = _rows(_st.get(), _cf.get()).find(r => r.name == x)
  assert(r != none and r.si != none, message: x + " is not defined")
  assert(calc.abs(r.si - v) <= 1e-9 * calc.abs(v) + 1e-300, message: x + " = " + repr(r.si) + ", want " + repr(v))
  if u != none { assert.eq(r.unit, u, message: x + " has unit " + repr(r.unit) + ", want " + repr(u)) }
}
// x isn't defined
#let undefined(x) = context assert(_rows(_st.get(), _cf.get()).all(r => r.name != x), message: x + " is defined")
// the last line reported a problem containing msg
#let fails(msg) = context {
  let e = _st.get().at("last", default: (errors: ())).errors
  assert(e.len() > 0, message: "expected an error containing '" + msg + "'")
  assert(e.any(m => m.contains(msg)), message: "errors " + repr(e) + " don't contain '" + msg + "'")
}
// the last line went through without problems
#let passes() = context {
  let e = _st.get().at("last", default: (errors: ())).errors
  assert(e.len() == 0, message: "unexpected errors: " + repr(e))
}
// what the last line showed as its result (markup)
#let result(want) = context {
  let r = _st.get().at("last").result
  // constants by their usual names, units (#svref("id", "Ω"), upright("mA")) by their spelling
  let plain = r.replace("#svconst(\"euler\")", "e").replace("#svconst(\"pi\")", "pi").replace("#svconst(\"imaginary\")", "i")
  let plain = plain.replace(regex("#svref\\(\"[^\"]*\", \"([^\"]*)\"\\)"), m => m.captures.at(0))
  let plain = plain.replace(regex("upright\\(\"([^\"]*)\"\\)"), m => m.captures.at(0))
  assert.eq(plain, want, message: "result " + repr(r) + ", want " + repr(want))
}
// some problem reported in the document contains msg (problems with arguments have no line, so the whole document)
#let reported(msg) = context {
  let e = query(<solveng>).map(it => it.value).filter(x => x.kind == "error").map(x => x.message)
  assert(e.any(m => m.contains(msg)), message: "no problem containing '" + msg + "' in " + repr(e))
}
