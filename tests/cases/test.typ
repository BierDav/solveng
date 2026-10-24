#import "/src/unit-words.typ" as units
#import "/src/constants.typ" as constants
#import "/src/builtins.typ": *
#import "/lib.typ": *
#import "/tests/util.typ": expect, passes
#import operations: *
#import modifiers: simplify
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
// and (∧), or (∨) in conditions, and binding tighter; a branch for one x alone is a dot in the graph
$p(x) := cases(
  2 & "for" x = 1,
  x & "for" x < 0 or x > 2)
$
$p_1 := p(1) =$ #expect("p_1", 2)
$p_2 := p(-1) + p(3) =$ #expect("p_2", 2)
$b_1 := 1 < 2 and 3 < 2 or 1 = 1 =$ #expect("b_1", 1)
$b_2 := 1 < 2 and (3 < 2 or 1 = 2) =$ #expect("b_2", 0)
$b_3 := 0 < 1 and 1 < 2 and 2 < 3 =$ #expect("b_3", 1)
#graph($p(x)$, xrange: (-2, 4))
// a branch's quoted text is its label, whatever it says: "für" and "sonst." are labels, not names
$
  g_1(t) := cases(
    2t & "für" 0 <= t <= 1/2,
    2-2t & "für" 1/2 < t <= 1,
    2t-2 & "für" 1 < t <= 3/2,
    4-2t & "für" 3/2 < t <= 2,
    0 & "sonst."
  )
$
$"corner" := g_1(1/2) =$ #expect("corner", 1)
$"fallback" := g_1(2) =$ #expect("fallback", 0)
$h(x) := cases(1 & "wenn nicht" x > 0, 0 & "sonst gilt nichts")$
$"wenn" := h(1) =$ #expect("wenn", 1)
$"sonst" := h(-1) =$ #expect("sonst", 0)
#defined()
