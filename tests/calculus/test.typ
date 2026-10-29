#import "/lib.typ": *
#import "/tests/util.typ": expect, fails, passes, result, undefined
#import constants: e, i, pi
#import units: s, A
#set page(width: 18cm, height: auto)
#show: worksheet
#calculate($
t := 1 \
g := 2
$)
#expect("t", 1)
#expect("g", 2)
#calculate($t := 1 \ g := 23$)
#expect("g", 23)
// code passed to worksheet is text, not calculated
#worksheet(raw("k := 24"))
#undefined("k")
#clear("t")
$q := 23 + 3$
$q =>$
$f(x) := #e^x$
$dif / (dif t) f(t) ->$
$integral_(0)^(1) x^2 dif x ->$
$integral_(0)^(1) exp(-x^2) dif x =>$
#passes()
$f(x) := #e^x$
$ dif / (dif t) f(t) -> $
#passes()
#result("e^(t)")
$ a := integral_0^1 x^2 dif x => $
#passes()
#expect("a", 1 / 3)
$ b := integral_0^1 #e^(-x^2) dif x => $
#passes()
#expect("b", 0.746824132812427)
$ c := integral_(-infinity)^0 #e^x dif x => $
#passes()
#expect("c", 1)
$ sigma(tau) := #e^tau $
$ F(t) := integral_(-infinity)^t sigma(tau) dif tau $
$ dif / (dif t) F(t) -> $
#passes()
#result("e^(t)")
#graph($ t \ g := 2 $)

$tau := 1$
#graph($ integral_(-infinity)^t sigma(tau) dif tau $, x-range: (-1, 1), mark: ())
#clear("tau")
#graph($ integral_(-infinity)^t sigma(tau) dif tau $, x-range: (-1, 1), mark: ())

#clear("sigma", "f")
$f(t) := integral_(-infinity)^t sigma(tau) dif tau$
#passes()
$dif / (dif t) f(t) ->$
#passes()
#result("σ(t)")
$f(t) ->$
#passes()
$sigma(u) := #e^u$
$"late" := f(0) =>$
#passes()
#expect("late", 1)
#clear("q", "g")
$g(x) := dif / (dif x) q(x)$
$g(2) ->$
#passes()
$"lateDerivative" := g(2)$
$q(t) := t^2$
$"lateDerivative" =>$
#passes()
#expect("lateDerivative", 4)

// the differential ends the integrand, also inside a group or a numerator
$ z := integral_0^(pi/2)(#e^(#i t) dif t) + 12 -> $
#passes()
#result("\"13\" + i")
$ integral_0^1 (dif x)/(x + 1) -> $
#passes()
$ (partial^2 x^2 y^3)/(partial x partial y) -> $
#passes()
#result("\"6\" x dot y^(\"2\")")
$ "charge" := integral_0^(2 #s) 3 #A dif t => $
#passes()
#expect("charge", 6)

// indefinite integrals: the antiderivative; diff and integrate shown as mathematics writes them
#import operations: diff, integrate
$ integral #e^x dif x -> $
#passes()
$ integral x^2 sin(x) dif x -> $
#passes()
$ G(x) := integral x #e^x dif x $
$ "at1" := G(1) => $
#expect("at1", 0)
$ integrate(x^2, x) -> $
$ diff(x^2 + 1, x) -> $
$ integral sin(x^2) dif x -> $
#passes()
$ H(x) := integral sin(x^2) dif x $
#passes()
$ H(1) => $
#fails("no antiderivative")
// comments in math are no part of it
$ /* a comment */ k := 2 // another
$
#expect("k", 2)
