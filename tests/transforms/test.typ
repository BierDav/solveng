#import "/src/unit-words.typ" as units
#import "/src/constants.typ" as constants
#import "/src/builtins.typ": *
#import "/lib.typ": *
#import "/tests/util.typ": expect, passes, fails
#import operations: *
#set page(width: 18cm, height: auto)
#show: worksheet.with(errors: "panic")
$F(p) := laplace(exp(-t), t, p)$
#passes()
$"forward" := F(2) =$
#passes() #expect("forward", 1/3)
$f(x) := invlaplace(1/(p+1), p, x)$
#passes()
$"inverse" := f(0) =$
#passes() #expect("inverse", 1)
$laplace(t^2) ->$
#passes()
$invlaplace(1/(s^2+4)) ->$
#passes()
$G(p) := laplace(t^2, t, p)$
$"called" := G(2) =$
#passes() #expect("called", 0.25)
$H(p) := laplace(t^2, t, p)$
$"code"_"value" := H(2) =$
#passes() #expect("code_value", 0.25)

$Lambda(t) := tri(t)$
$s(t) := rect(t)$
$g(t) := Λ(2t)$
$H(s) := laplace(s(t), t, s) / laplace(g(t), t, s) ->$
#passes()
$"signalratio" := H(2) =$
#passes() #expect("signalratio", calc.exp(1) - 1)

$s_2(t) := invlaplace(H(s)/s, s, t) ->$
#passes()
$"stepfirst" := s_2(0.25) =$
#passes() #expect("stepfirst", calc.exp(0.5))
$"stepsecond" := s_2(0.75) =$
#passes() #expect("stepsecond", calc.exp(1.5) - 1.5 * calc.exp(0.5))
$"stepcorner" := s_2(0.5) =$
#passes() #expect("stepcorner", calc.exp(1) - 1)


// Full input-output pair, shifted together before the unilateral transform.
$x(t) := rect(t-1/2)$
$y(t) := tri(2*(t-1/2))$
$H(s) := laplace(y(t), t, s)/laplace(x(t), t, s) ->$
$g_2(t) := invlaplace(H(s)/s, s, t) ->$
#passes()
$"trianglefirst" := g_2(0.25) =$
#passes() #expect("trianglefirst", 0.5)
$"trianglepeak" := g_2(0.5) =$
#passes() #expect("trianglepeak", 1)
$"trianglezero" := g_2(1) =$
#passes() #expect("trianglezero", 0)
$"trianglelater" := g_2(2.75) =$
#passes() #expect("trianglelater", 0.5)
#graph($g_2(t)$, xrange: (-0.5, 3))
#passes()

// General recurrence with multiple delays and complex continuous poles.
$r(t) := invlaplace((2*s+1+exp(-s))/(s^2+2*s+5+(s+1)*exp(-s)+exp(-2*s)), s, t) ->$
#passes()
$"rzero" := r(0) =$
#passes() #expect("rzero", 2)
#graph($r(t)$, xrange: (0, 3))
#passes()

// a transform is an operation, never a keyword after ->
#worksheet(errors: "document")[
  $x := t^2 -> laplace$ #fails("-> takes keywords and a unit")
]
