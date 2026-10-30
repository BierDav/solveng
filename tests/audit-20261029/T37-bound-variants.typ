#import "/lib.typ": *
#import "/src/engine.typ": _src
#import "/src/config.typ": _E
#show: math
#let bad = rect(width: 1pt, height: 1pt)
#assert(_E(_src($sum_(#bad)^2 k$.body)))
#assert(_E(_src($sum_0^(#bad) k$.body)))
#assert(_E(_src($integral_(#bad)^2 x dif x$.body)))
#assert(_E(_src($integral_0^(#bad) x dif x$.body)))
$sum_0^(#bad) k$
$integral_(#bad)^2 x dif x$
