#import "/lib.typ": *
#import "/tests/util.typ": expect, passes
#import operations: rect, tri, sgn, diff
#import modifiers: solve
#set page(width: 18cm, height: auto)
#show: worksheet
$f(t) := rect(t) ⋅ tri(t) * sgn(t)$
$"sample" := f(0.25) =>$
#passes() #expect("sample", 0.75)
$"edge" := rect(0.5) =>$
#passes() #expect("edge", 0.5)
$"origin" := sgn(0) =>$
#passes() #expect("origin", 0)
$rect(t) + 2 rect(t) ->$
#passes()
#calculate($a*rect(0) + b*tri(0) = 5; a*rect(1) + b*tri(0.5) = 1 -> solve(a,b)$)
#passes() #expect("a", 3) #expect("b", 2)
$"area" := integral_(-2)^2 rect(t) dif t =>$
#passes() #expect("area", 1)
#graph($rect(t) \ tri(t) \ sgn(t)$, x-range: (-2, 2), mark: ())

// Both derivative spellings are valid, including automatic ranges.
#enum(..($dif / (dif t) tri(t)$, $diff(tri(t), t)$).map(item => [
  #item
  #graph(item, errors: "panic")
]))
// Engine markup must render both differentials as d.
#import "/src/engine.typ": _m, _src, _restyle
#import "/src/config.typ": _DEF
#assert.eq(_m("frac(dif, dif t)", _DEF), $dif / (dif t)$)
#assert.eq(_src($dif / (dif t) tri(t)$.body), "dif / (dif t) ⦇tri⦈(t)")
// Worksheet restyling must preserve the operator inside a denominator sequence.
#let derivative = _restyle($dif / (dif t)$.body, _DEF)
#assert.eq(derivative.num, math.dif)
#assert.eq(derivative.denom.children.slice(0, math.dif.children.len()).join(), math.dif)
#assert.eq(_restyle($dif$.body, _DEF), math.dif)
