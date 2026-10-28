#import "/lib.typ": *
#import constants: e, i, pi
#import operations: rect, tri
#show: worksheet.with(errors: "panic")

$p(t_0) := rect(t - t_0) + tri(2 * (t - t_0))$
$s(t) := p(1.5) + p(3.5) + p(-1.5) + p(-3.5)$
$S(f) := integral_(-infinity)^infinity
  s(t) * #e^(-#i * 2 #pi * f * t) dif t$

$S(0.2) =$
#graph($S(f)$, xrange: (-5, 5))

$T(f) := integral_(-infinity)^infinity
  rect(t - 1.5) * #e^(-#i * 2 #pi * f * t) dif t$
#graph($Re(T(f)) \\ Im(T(f)) \\ abs(T(f)) \\ arg(T(f))$, xrange: (0.1, 0.3))
