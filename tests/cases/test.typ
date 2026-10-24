#import "/src/unit-words.typ" as units
#import "/src/constants.typ" as constants
#import "/src/builtins.typ": *
#import "/lib.typ": *
#import "/tests/util.typ": expect, passes
#import functions: *
#set page(width: 18cm, height: auto)
#show: worksheet.with(errors: "panic")
$
  sigma(t) := cases(
    0 & "für" t < 0,
    1 & "für" t >= 0
  )
$
$"negative" := σ(-1) =$
#passes() #expect("negative", 0)
$"boundary" := σ(0) =$
#passes() #expect("boundary", 1)
$"positive" := σ(1) =$
#passes() #expect("positive", 1)
#graph($σ(x)$)
$f(t) := cases(t^2 & "if" t < 0, t & "otherwise")$
$"fallback" := f(2) =$
#passes() #expect("fallback", 2)
$g(t) := f(t-1) -> simplify$
#passes()
#defined()
