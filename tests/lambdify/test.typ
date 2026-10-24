// lambdify: expressions as plain functions.
#import "/tests/util.typ": *
#import units: V, W, mV, mW, Ω

#worksheet($R := 50 #Ω$, hidden: true)
#context {
  let P = lambdify($U^2/R$, "U", unit: $#W$, units: (U: $#V$))
  assert.eq(P(10), 2.0)
  assert.eq(P($20 #V$), 8.0)
  assert.eq(P($20000 mV$), 8.0)
  let Pm = lambdify($U^2/R$, "U", unit: $mW$, units: (U: $#V$))
  assert(calc.abs(Pm(1) - 20) < 1e-9)
  // a defined variable as a parameter is replaced by the argument
  let P2 = lambdify($U^2/R$, "U", "R")
  assert.eq(P2(10, 25), 4.0)
  // without parameters: the undefined variables in order
  let f = lambdify($x^2 + 1$)
  assert.eq(f(3), 10.0)
  // undefined points give none
  let g = lambdify($sqrt(x)$)
  assert.eq(g(-1), none)
  assert.eq(lambdify($1/x$)(0), none)
}
