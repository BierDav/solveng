#import "/lib.typ": *
#import "/tests/util.typ": expect, passes
#let rect = math.op("rect")
#let tri = math.op("tri")
#let sgn = math.op("sgn")
#set page(width: 18cm, height: auto)
#show: worksheet
$f(t) := rect(t) ⋅ tri(t) * sgn(t)$
$"sample" := f(0.25) =$
#passes() #expect("sample", 0.75)
$"edge" := rect(0.5) =$
#passes() #expect("edge", 0.5)
$"origin" := sgn(0) =$
#passes() #expect("origin", 0)
$rect(t) + 2 rect(t) ->$
#passes()
#worksheet("a*rect(0) + b*tri(0) = 5; a*rect(1) + b*tri(0.5) = 1 -> solve(a,b)")
#passes() #expect("a", 3) #expect("b", 2)
$"area" := integral_(-2)^2 rect(t) dif t =$
#passes() #expect("area", 1)
#graph($rect(t) \ tri(t) \ sgn(t)$, xrange: (-2, 2), mark: ())
