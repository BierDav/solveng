#import "/tests/util.typ": *
#import units: ms, s, km
#import operations: tri, rect, dirac, convolve, range
#set page(width: 18cm, height: auto)
#show: worksheet

// range variables and elements, as Mathcad writes them
$N := 200$, $Delta""t := 10 ms$
$i := 0..N$
#passes()
$t[i] := -1 #s + i Delta""t$
#passes()
$x[i] := tri(t[i] / (1 #s))$
#passes()
$x_a := x[100] =>$
#expect("x_a", 1)
$x_b := x[150] =>$
#expect("x_b", 0.5)

// the discrete convolution, a named argument in math
$y := convolve(x, x, step: Delta""t)$
#passes()
$y_m := y[200] =>$
#expect("y_m", 0.6667, u: none)
$k := 0 .. 2 N$
$tau[k] := -2 #s + k Delta""t$
#passes()

// the continuous one, exact
$c(t) := convolve(tri(tau), tri(tau), tau, t) ->$
#passes()
$c_h := c(1/2) =>$
#expect("c_h", 23 / 48)
$r(t) := convolve(rect(t), rect(t), t) ->$
#passes()

// A Dirac impulse shifts the signal, including its exact boundary values.
$s_0(t) := rect(t) + tri(2 t)$
$s_a(t) := convolve(s_0(t), dirac(t - 3/2), t) ->$
#passes()
$s_p := s_a(3/2) =>$ #expect("s_p", 2)
$s_e := s_a(2) =>$ #expect("s_e", 0.5)
#graph($s_a(z)$, x-range: (-5, 5), errors: "panic")
$d_2(t) := convolve(dirac(t - 1), dirac(t - 2), t) ->$
#passes()
$d_v := dirac(0) =>$
#fails("distribution")

// the sampled result over the exact one (the data's unit: s, as the step's)
$c_(s) (t) := c(t / (1 #s)) dot 1 #s$
#graph($y(tau) \ c_s(tau)$, errors: "panic")

// Typst's range: the end left out; in code it stays Typst's
$q := range(1, 10, step: 3) =>$
#passes()
$q_1 := q[1] =>$
#expect("q_1", 4)
#assert.eq(range(3), (0, 1, 2))
#assert.eq(range(1, 7, step: 2), (1, 3, 5))

// Explicit operator bindings work whatever their name; a quoted name is a worksheet function.
#worksheet[
  #let triangle = math.op("tri")
  $a := triangle(1/2) =>$
  #expect("a", 0.5)
  $c := "tri"(0) =>$
  #fails("tri")
  #let conv = operation("convolve")
  $d := conv(vec(1, 1), vec(1, 1), step: 2) =>$
  #passes()
]

// Inline explicit references, and units in both spacings.
#worksheet[
  $x := 1/2$
  $a := op("tri")(x) =>$ #expect("a", 0.5)
  $b := 1#km =>$ #expect("b", 1000)
  $c := 1 km =>$ #expect("c", 1000)
  $d := op("rect")(0) =>$ #expect("d", 1)
]
