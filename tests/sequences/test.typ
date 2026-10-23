#import "/tests/util.typ": *
#set page(width: 18cm, height: auto)
#show: worksheet

// range variables and elements, as Mathcad writes them
$N := 200$, $Delta""t := 10 ms$
$i := 0..N$
#passes()
$t[i] := -1 "s" + i Delta""t$
#passes()
$x[i] := tri(t[i] / (1 "s"))$
#passes()
$x_a := x[100] =$
#expect("x_a", 1)
$x_b := x[150] =$
#expect("x_b", 0.5)

// the discrete convolution, a named argument in math
$y := convolve(x, x, step: Delta""t)$
#passes()
$y_m := y[200] =$
#expect("y_m", 0.6667, u: none)
$k := 0 .. 2 N$
$tau[k] := -2 "s" + k Delta""t$
#passes()

// the continuous one, exact
$c(t) := convolve(tri(tau), tri(tau), tau, t) ->$
#passes()
$c_h := c(1/2) =$
#expect("c_h", 23 / 48)
$r(t) := convolve(rect(t), rect(t), t) ->$
#passes()

// the sampled result over the exact one (the data's unit: s, as the step's)
$c_(s) (t) := c(t / (1 "s")) dot 1 "s"$
#graph("y(tau) \\ c_s(tau)", errors: "panic")

// Typst's range: the end left out; in code it stays Typst's
$q := range(1, 10, step: 3) =$
#passes()
$q_1 := q[1] =$
#expect("q_1", 4)
#assert.eq(range(3), (0, 1, 2))
#assert.eq(range(1, 7, step: 2), (1, 3, 5))

// function overlays
#worksheet(functions: (conv: "convolve", Λ: "tri"))[
  #let conv = operation("conv")
  $a := conv(vec(1, 1), vec(1, 1), step: 2) =$
  #passes()
  $b := "Λ"(1/2) =$
  #expect("b", 0.5)
  #worksheet("d := conv(vec(1), vec(3))")
  #passes()
]
#worksheet(functions: (tri: none))[
  // the exported binding is math.op: an explicit reference, still the operation
  $e := tri(1/2) =$
  #expect("e", 0.5)
  // a plain name is hidden
  #worksheet("f(x) := tri(x)")
  #worksheet("g := f(0) =")
  #fails("tri")
]
#worksheet(builtin-functions: false, functions: (pulse: "rect"))[
  #worksheet("h := pulse(0) =")
  #expect("h", 1)
  $m := sin(0) =$
  #expect("m", 0)
]
#worksheet(functions: (a: "b", b: "a"))[]
