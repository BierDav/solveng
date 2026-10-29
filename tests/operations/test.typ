// Operations: imported from `operations`, or referenced explicitly with math.op and operation.
#import "/lib.typ": *
#import "/tests/util.typ": expect, passes, fails
#import operations: tri, rect, sgn, integrate
#set page(width: 18cm, height: auto)
#show: worksheet.with(errors: "panic")
$"standard" := tri(1/4) + rect(0) + sgn(0) =>$
#passes() #expect("standard", 1.75)
$Lambda(t) := tri(t)$
$g_2(t) := sum_(k=0)^infinity Lambda(t-k+1/2) ->$
#passes()
$"summed" := g_2(0) =>$
#passes() #expect("summed", 1)
$"distant" := g_2(1000000) =>$
#passes() #expect("distant", 1)
#graph($g_2(t)$, x-range: (1000000, 1000003))
#graph($g_2(x)$, x-range: (0, 1))
#graph($Lambda(t)$, x-range: (-2, 2))
#calculate($"named" := Lambda(1/4) =>$)
#passes() #expect("named", 0.75)
$"direct" := Λ(3/4) =>$
#passes() #expect("direct", 0.25)
$"calculus" := integrate(tri(t), t, -2, 2) =>$
#passes() #expect("calculus", 1)

// explicit references, whatever their binding is called
#let triangle = math.op("tri")
#let step = operation("sgn", display: () => math.op("step"))
$"explicit" := triangle(1/2) + op("rect")(0) + step(-3) =>$
#passes() #expect("explicit", 0.5)
// without an import a name is no operation: quoted, it is a worksheet function
#worksheet(errors: "document")[$"quoted" := "tri"(0) =>$ #fails("tri isn't defined")]

#import operations: sinc, cosc, tanc, diff
$"cardinal_origin" := sinc(0) + cosc(0) + tanc(0) =>$
#passes() #expect("cardinal_origin", 2)
$"cardinal_sine" := sinc(1) =>$
#passes() #expect("cardinal_sine", calc.sin(1))
$"cardinal_cosine" := cosc(0.00000001) =>$
#passes() #expect("cardinal_cosine", 0.000000005)
$"cardinal_tangent" := tanc(1) =>$
#passes() #expect("cardinal_tangent", calc.tan(1))
$f_c(x) := diff(cosc(x), x)$
$"cardinal_derivative" := f_c(0) =>$
#passes() #expect("cardinal_derivative", 0.5)
#graph($sinc(x) \ cosc(x) \ tanc(x)$, x-range: (-1, 1), errors: "panic")

#import operations: atan2, min, max, mean, prod
$"quadrant" := atan2(1, -1) =>$
#passes() #expect("quadrant", 3 * calc.pi / 4)
$"smallest" := min(3, -2, 7) =>$
#passes() #expect("smallest", -2)
$"largest" := max(3, -2, 7) =>$
#passes() #expect("largest", 7)
$"average" := mean(vec(1, 2, 6)) =>$
#passes() #expect("average", 3)
$"product" := prod(k, k, 1, 5) =>$
#passes() #expect("product", 120)
$"binary_log" := log(8, 2) =>$
#passes() #expect("binary_log", 3)
