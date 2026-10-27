#import "/lib.typ": *
#import "/src/display.typ": _own
#import "/tests/util.typ": passes, expect
#import operations: rect, tri
#import units: ms, V
#set page(width: 18cm, height: auto)
#show: worksheet.with(errors: "panic")

$f(t) := rect(t) + tri(t)$
$q(t: ms) := t / (1 ms) * 1 #V$

// Labels must identify themselves as generated equations even when measured away
// from graph's state updates, as happens inside HTML layout adapters.
#let labels-only(body) = {
  show math.equation: it => {
    assert(_own(it), message: "graph label can be evaluated as worksheet math: " + repr(it))
    it
  }
  body
}
#labels-only[
  #graph($f(t)$)
  #graph($q(t) \ -q(t)$, width: 12cm, height: 7cm, xrange: (-2, 2), mark: (0.5, "crossings"))
]

// Measuring graphs must leave subsequent calculations and definitions intact.
$"sample" := f(0) =$
#passes() #expect("sample", 2)
$"voltage" := q(2 ms) =$
#passes() #expect("voltage", 2)
