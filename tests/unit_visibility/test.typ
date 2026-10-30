#import "/tests/util.typ": *
#import units: m, s, km
#let seconds = base-unit("time", display: () => std.math.upright("seconds"))
#let minute = unit($60 seconds$, display: () => std.math.upright("min"))
#let tau = unit($2 #m$, display: () => std.math.upright("tau"))
#show: math.with(errors: "panic")
$t := 2 minute => #s$ #expect("t", 120, u: "s")
#mathscope(scope: "isolated")[$x := 3 tau =>$ #expect("x", 6, u: "tau")]
#mathscope(scope: "inherit")[$y := t / 2 = seconds$ #expect("y", 60)]
#mathscope(scope: "shared")[$z := 4 tau => #m$]
#expect("z", 8)
#let kmAlias = km
$d := 2 kmAlias => #m$ #expect("d", 2000)
#let greek = base-unit("alpha", display: () => $alpha$)
#let literal = base-unit("α", display: () => std.math.upright("other"))
$a := 1 greek =>$
#math(errors: "document", $a + literal =>$)
#fails("different units")
