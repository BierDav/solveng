#import "/lib.typ": *
#import "/src/engine.typ": _out, _line-response, _calculation-out, _run, _req-cfg
#import "/src/api.typ": _st, _cf
#import "/tests/util.typ": expect, passes, result
#set page(width: 18cm, height: auto)

// Malformed responses become complete, contextual calculation errors; state is retained.
#let previous = bytes((1, 2, 3))
#let source = "Re(a z) = a Re(z) ->"
#let check-bad(response, fragment) = {
  let checked = _line-response(response, previous, source)
  assert.eq(checked.env, previous)
  assert.eq(checked.out.prose, false)
  assert.eq(checked.out.mark, none)
  assert.eq(checked.out.result, none)
  assert.eq(checked.out.defined, ())
  assert.eq(checked.out.errors.len(), 1)
  let message = checked.out.errors.first()
  assert(message.contains(source), message: message)
  assert(message.contains(fragment), message: message)
}
#check-bad(none, "response dictionary")
#check-bad((env: previous), "missing response field")
#check-bad((env: 1, out: _out()), "'env' must be bytes")
#check-bad((env: previous, out: none), "'out' must be a dictionary")
#check-bad((env: previous, out: (errors: (), result: none)), "missing calculation fields: prose")
#check-bad((env: previous, out: _out() + (prose: "false")), "'prose' must be a boolean")
#check-bad((env: previous, out: _out() + (errors: none)), "'errors' must be an array")
#check-bad((env: previous, out: _out() + (defined: (1,))), "'defined' must contain strings")
#check-bad((env: previous, out: _out() + (mark: "?")), "'mark'")
#check-bad((env: previous, out: _out() + (result: 2)), "'result'")
#let valid = (env: previous, out: _out(result: "\"2\"", mark: "=>"))
#assert.eq(_line-response(valid, bytes(()), source), valid)

// A shared last-result belonging to a different operation/equation is diagnosed.
#for operation in ("graph", "clear") {
  let out = _calculation-out((last: _out(operation: operation)), source)
  assert(out.errors.first().contains(operation))
  assert(out.errors.first().contains(source))
}
#let wrong = _calculation-out((last: _out(operation: "line", source: "other =")), source)
#assert(wrong.errors.first().contains("other ="))
#assert(_calculation-out((:), source).errors.first().contains("dictionary"))
#assert(_calculation-out((last: (errors: (), result: none)), source).errors.first().contains("prose"))

#show: math.with(errors: "panic")
$x := 3$, $x + 2 =>$ #result("\"5\"")
#clear("x")
#context {
  assert.eq(_st.get().last.operation, "clear")
  assert.eq(_st.get().last.prose, false)
  assert.eq(_st.get().last.result, none)
  assert.eq(measure(math(hidden: true, errors: "panic")[$z := 1 =>$]).height, 0pt)
}
$x := 4$, $x^2 =>$ #result("\"16\"")
#graph($x t$, x-range: (0, 1), samples: 30)
#context {
  assert.eq(_st.get().last.operation, "graph")
  assert.eq(_st.get().last.prose, false)
  assert.eq(_st.get().last.result, none)
  assert.eq(measure(math(hidden: true, errors: "panic")[$z := 1 =>$]).height, 0pt)
}
$x + 1 =>$ #passes() #result("\"5\"")
#mathscope(scope: "inherit")[#math(clear-at: heading)[
  = First
  $y := 2 x =>$ #expect("y", 8)
  = Next
  $y := 3 x =>$ #expect("y", 12)
]]
$x =>$ #result("\"4\"")
#context {
  // Measurement evaluates equation contexts but discards their state updates.
  // It must not read the graph/clear result that preceded the measured content.
  let hidden = measure(math(hidden: true, errors: "panic")[$z := 1 =>$ $w := 2$])
  assert.eq(hidden.height, 0pt)
}
