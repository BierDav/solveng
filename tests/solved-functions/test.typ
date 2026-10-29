#import "/lib.typ": *
#import "/tests/util.typ": expect, passes, fails, result
#import operations: at
#import modifiers: solve
#set page(width: 18cm, height: auto)
#show: worksheet
$b := 99$
$"pytagoras"(a, b, c) := a^2 + b^2 = c^2$
$"circle"(x, r) := "pytagoras"(x, b, r) -> solve(b)$
#passes() #expect("b", 99)
$"circle"(3, 5) =>$
#passes() #result("vec(\"−4\", \"4\")")
$"lower"(x, r) := at("circle"(x, r), 0)$
$"upper"(x, r) := at("circle"(x, r), 1)$
$L := "lower"(3, 5) =>$
#expect("L", -4)
$U := "upper"(3, 5) =>$
#expect("U", 4)
// equations passed to worksheet share the definitions
#calculate($"circle"(x, r) := "pytagoras"(x, b, r) -> solve(b)$)
#passes()
#calculate($"circle"(3, 5) =>$)
#passes() #result("vec(\"−4\", \"4\")")
$"pair"(a) := vec(u + v = a, u - v = 2) -> solve(u, v)$
#passes()
$"pair"(6) =>$
#passes() #result("vec(\"4\", \"2\")")
$"pytagoras"(3, b, 5) -> solve(b)$
#passes()
#clear("b")
#graph($"circle"(x, 5)$, x-range: (-5, 5), mark: ())
// a fraction fills the grid cell, labels included
#grid(
  columns: (1fr, 1fr),
  gutter: 4pt,
  [#graph($"circle"(x, 5)$, x-range: (-5, 5), mark: (), width: 1fr)],
  [#graph($"circle"(x, 4)$, x-range: (-5, 5), mark: (), width: 1fr)],
)
$"pytagoras"(a, b, c) := a^2 + b^2 = c^2$
$"circle"(x, r) := "pytagoras"(x, b, r) -> solve(b)$
$"circle"(3, 5) =>$
#passes() #result("vec(\"−4\", \"4\")")
$"roots" := y^2 = 9 -> solve(y)$
$"roots" =>$
#passes() #result("vec(\"−3\", \"3\")")
$"bad"(x, r) := "pytagoras"(x, b, r) = b -> solve(b)$
#fails("omit the extra = b")
// a graph after solve(f) is over f, as before it: f's value doesn't stop it from varying
#import units: Hz, kHz
$f_0 := 1 #kHz$
$S_1(f: #Hz) := 1/sqrt(1 + (f/f_0)^2)$
$S_1(f) = 1/2 -> solve(f)$
#passes() #expect("f", calc.sqrt(3) * 1000)
#graph($S_1(f)$, errors: "panic")
#graph($S_1(f) \ 1/2$, mark: (), errors: "panic")
