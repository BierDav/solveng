#import "/src/unit-words.typ" as units
#import "/src/constants.typ" as constants
#import "/src/builtins.typ": *
#import "/lib.typ": *
#import "/tests/util.typ": expect, passes
#import functions: *
#set page(width: 18cm, height: auto)
#show: worksheet.with(errors: "panic")
$s_2(t) := sum_(k=0)^infinity sigma(t-k+3/4) ->$
#passes()
$sigma(t) := cases(0 & "if" t < 0, 1 & "otherwise")$
$"negative" := s_2(-2) =$
#passes() #expect("negative", 0)
$"origin" := s_2(0) =$
#passes() #expect("origin", 1)
$"positive" := s_2(2) =$
#passes() #expect("positive", 3)
#graph($s_2(t)$)
#graph($s_2(t)$, xrange: (-2, 3))
$"finite" := sum_(k=0)^3 k^2 =$
#passes() #expect("finite", 14)
$"geometric" := sum_(k=0)^infinity (1/2)^k =$
#passes() #expect("geometric", 2)
#worksheet($a := sum_(k=0)^3 k^2 =$)
#passes() #expect("a", 14)

// Two explicit conditions with the user's English label, then a named sum.
$sigma(t) := cases(
  0 & "for" t < 0,
  1 & "for" t >= 0
)$
$g_2(t) := sum_(k=0)^infinity sigma(t-k+3/4)$
#passes()
$"beforestep" := g_2(-0.750001) =$
#passes() #expect("beforestep", 0)
$"firststep" := g_2(-3/4) =$
#passes() #expect("firststep", 1)
$"secondstep" := g_2(1/4) =$
#passes() #expect("secondstep", 2)
#graph($g_2(t)$)

// A rename to a longer index uses a quoted Typst name.
$"renamed" := sum_("counter"=0)^3 "counter"^2 =$
#passes() #expect("renamed", 14)

// A flat triangle train must not create an unrepresentably small tick step.
$g_2(t) := sum_(k=0)^infinity tri(t-k+1/2) ->$
#passes()
#graph($g_2(x)$, xrange: (0, 1))
