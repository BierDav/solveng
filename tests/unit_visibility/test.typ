#import "/tests/util.typ": *
#import units: m, s, km
#let seconds = base-unit("time", display: () => math.upright("seconds"))
#let minute = unit($60 seconds$, display: () => math.upright("min"))
#let tau = unit($2 #m$, display: () => math.upright("tau"))
#show: worksheet.with(errors: "panic")
$t := 2 minute => #s$ #expect("t", 120, u: "s")
#worksheet(scope: "isolated")[$x := 3 tau =>$ #expect("x", 6, u: "tau")]
#worksheet(scope: "inherit")[$y := t / 2 = seconds$ #expect("y", 60)]
#worksheet(scope: "shared")[$z := 4 tau => #m$]
#expect("z", 8)
#let kmAlias = km
$d := 2 kmAlias => #m$ #expect("d", 2000)
#let greek = base-unit("alpha", display: () => $alpha$)
#let literal = base-unit("α", display: () => math.upright("other"))
$a := 1 greek =>$
#worksheet(errors: "document", $a + literal =>$)
#fails("different units")
