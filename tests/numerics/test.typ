// Numerical engineering APIs: exports, units, scope, error flow and solved functions.
#import "/lib.typ": *
#import "/tests/util.typ": expect, passes, fails, result
#import operations: lsolve, eigenvals, singularvals, svds, cond, median, variance, stdev, normalcdf, normalquantile, pnorm, qnorm, linterp, linfit, dft, idft, gamma, erf, besselj, minimize, odesolve, celsius, fahrenheit, polyroots, at, rows, number, Im
#import modifiers: solve, assume, simplify
#import units: m, s, V, K, degC
#set page(width: 20cm, height: auto)
#show: worksheet.with(errors: "panic")
$"least" := at(lsolve(mat(1, 0; 1, 1; 1, 2), vec(1, 2, 2)), 0) =$
#passes() #expect("least", 7/6)
$"eigsum" := at(eigenvals(mat(2, 1; 1, 2)), 0) + at(eigenvals(mat(2, 1; 1, 2)), 1) =$
#passes() #expect("eigsum", 4)
$"singular" := at(singularvals(mat(3 #m, 0; 0, 4 #m ; 0, 0)), 0) =$
#passes() #expect("singular", 4)
$"alias_singular" := at(svds(mat(3, 0; 0, 4; 0, 0)), 1) =$
#passes() #expect("alias_singular", 3)
$"condition" := cond(mat(1, 0; 0, 1/100000000)) =$
#passes() #expect("condition", 100000000)
$"middle" := median(vec(9 #m, 1 #m, 3 #m, 7 #m)) =$
#passes() #expect("middle", 5)
$"variance" := variance(vec(1 #m, 2 #m, 3 #m)) =$
#passes() #expect("variance", 1)
$"deviation" := stdev(vec(2, 4, 6)) =$
#passes() #expect("deviation", 2)
$"probability" := normalcdf(10 #m, 10 #m, 2 #m) =$
#passes() #expect("probability", 0.5)
$"quantile" := normalquantile(1/2, 10 #m, 2 #m) =$
#passes() #expect("quantile", 10)
$"alias_probability" := pnorm(0, 0, 1) =$
#passes() #expect("alias_probability", 0.5)
$"alias_quantile" := qnorm(1/2, 0, 1) =$
#passes() #expect("alias_quantile", 0)
$"interpolation" := linterp(vec(0 #s, 2 #s, 5 #s), vec(1 #V, 5 #V, 11 #V), 3 #s) =$
#passes() #expect("interpolation", 7)
$"slope" := at(linfit(vec(0 #s, 1 #s, 2 #s), vec(3 #V, 5 #V, 7 #V)), 1) =$
#passes() #expect("slope", 2)
$"phase" := Im(at(dft(vec(0, 1, 0, 0)), 1)) =$
#passes() #expect("phase", -1)
$"roundtrip" := at(idft(dft(vec(2, -1, 3, 0))), 2) =$
#passes() #expect("roundtrip", 3)
$"gammaresult" := gamma(1/2) =$
#passes() #expect("gammaresult", calc.sqrt(calc.pi))
$"erf" := erf(1) =$
#passes() #expect("erf", 0.8427007929497149)
$"bessel" := besselj(0, 1) =$
#passes() #expect("bessel", 0.7651976865579666)
$"minimum" := minimize((x-[3 #m])^2, x, 0 #m, 2 #m) =$
#passes() #expect("minimum", 2)
$"ode" := odesolve(-y/[2 #s], y, t, 3 #m, 0 #s, 2 #s) =$
#passes() #expect("ode", 3*calc.exp(-1))
$"kelvin" := number(celsius(20), #K) =$
#passes() #expect("kelvin", 293.15)
$"freezing" := number(fahrenheit(32), #K) =$
#passes() #expect("freezing", 273.15)
$"absolute_zero" := number(fahrenheit(-459.67), #K) =$
#passes() #expect("absolute_zero", 0)
$"difference" := number(20 degC, #K) =$
#passes() #expect("difference", 20)
$"roots"(p, q) := x^3+p x+q = 0 -> solve(x)$
$"root_count" := rows("roots"(1, 1/4)) =$
#passes() #expect("root_count", 3)
$"polynomial_count" := rows(polyroots(vec(1, 0, 1))) =$
#passes() #expect("polynomial_count", 2)
$"zetaresult" := sum_(k=1)^infinity 1/k^2 =$
#passes() #expect("zetaresult", calc.pi*calc.pi/6)
$sqrt(x^2) -> assume(x > 0)$
#passes() #result("x")
$x^2 < 1 -> solve(x)$
#passes() #result("\"−1\" < x < \"1\"")
#graph($erf(x)$, xrange: (-2, 2), errors: "panic")
#worksheet(errors: "document")[$celsius(-274) =$ #fails("absolute temperature")]
#worksheet(errors: "document")[$linterp(vec(0, 1), vec(1, 2), 2) =$ #fails("outside")]
