#import "/lib.typ": *
#import "/tests/util.typ": expect, passes
#import functions: *
#set page(width: 18cm, height: auto)
#show: worksheet.with(errors: "panic")
$"standard" := tri(1/4) + rect(0) + sgn(0) =$
#passes() #expect("standard", 1.75)
$Lambda(t) := tri(t)$
$g_2(t) := sum_(k=0)^infinity Lambda(t-k+1/2) ->$
#passes()
$"summed" := g_2(0) =$
#passes() #expect("summed", 1)
$"distant" := g_2(1000000) =$
#passes() #expect("distant", 1)
#graph("g_2(t)", xrange: (1000000, 1000003))
#graph("g_2(x)", xrange: (0, 1))
#graph($Lambda(t)$, xrange: (-2, 2))
#worksheet("named := Lambda(1/4) =")
#passes() #expect("named", 0.75)
$"direct" := Λ(3/4) =$
#passes() #expect("direct", 0.25)
$"calculus" := integrate(tri(t), t, -2, 2) =$
#passes() #expect("calculus", 1)
