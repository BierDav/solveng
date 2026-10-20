// Helpers for the tests: read what def/solve stored and check it.
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
