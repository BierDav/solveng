#import "/lib.typ": *
#import "/tests/util.typ": expect, passes, result
#set page(width: 18cm, height: auto)
#show: worksheet
#worksheet($
t := 1 \
g := 2
$)
#expect("t", 1)
#expect("g", 2)
#worksheet("t := 1 \\ g := 23")
#expect("g", 23)
#worksheet("t := 1 \ g := 24")
#expect("g", 24)
#worksheet("u := 23 + 3
v := u +
  2")
#expect("v", 28)
#clear("t")
```solveng
q := 23 + 3
q =
f(x) := e^x
dif / (dif t) f(t) ->
integral_(0)^(1) x^2 dif x ->
integral_(0)^(1) exp(-x^2) dif x =
```
#passes()
$f(x) := e^x$
$ dif / (dif t) f(t) -> $
#passes()
#result("e^(t)")
$ a := integral_0^1 x^2 dif x = $
#passes()
#expect("a", 1 / 3)
$ b := integral_0^1 e^(-x^2) dif x = $
#passes()
#expect("b", 0.746824132812427)
$ c := integral_(-infinity)^0 e^x dif x = $
#passes()
#expect("c", 1)
$ sigma(tau) := e^tau $
$ F(t) := integral_(-infinity)^t sigma(tau) dif tau $
$ dif / (dif t) F(t) -> $
#passes()
#result("e^(t)")
#graph($ t \ g := 2 $)

$tau := 1$
#graph($ integral_(-infinity)^t sigma(tau) dif tau $, xrange: (-1, 1), mark: ())
#clear("tau")
#graph($ integral_(-infinity)^t sigma(tau) dif tau $, xrange: (-1, 1), mark: ())

#clear("sigma", "f")
$f(t) := integral_(-infinity)^t sigma(tau) dif tau$
#passes()
$dif / (dif t) f(t) ->$
#passes()
#result("σ(t)")
$f(t) ->$
#passes()
$sigma(u) := e^u$
$"late" := f(0) =$
#passes()
#expect("late", 1)
#clear("q", "g")
$g(x) := dif / (dif x) q(x)$
$g(2) ->$
#passes()
$"lateDerivative" := g(2)$
$q(t) := t^2$
$"lateDerivative" =$
#passes()
#expect("lateDerivative", 4)
