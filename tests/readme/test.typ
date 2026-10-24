// The examples of the README, compared with reference images.
#import "/lib.typ": *
#import modifiers: solve, substitute
#set page(width: 16cm, height: auto, margin: 1cm)

// Engineering calculations in Typst math
#[
  #import units: mA, mAh, h
  #worksheet[
    $Q_0 := 3450 mAh$, $I := 30 mA$

    $t := Q_0 / I = #h$

    $Q_0/2 = I t -> solve(t)$
  ]
]

// Imported values
#[
  #import units: m, s
  #import constants: pi, e, c
  #import functions: tri
  #worksheet[
    $P := 2 pi =$

    $"velocity" := #c =$

    $f(x) := #e^x$

    $tri(1/4) =$
  ]
]

// Base units and derived units
#[
  #let second = base-unit("time", display: () => math.upright("s"))
  #let minute = unit($60 second$, display: () => math.upright("min"))
  #worksheet[
    $t := 2 minute = second$
  ]
]

// Constants and operations of your own
#[
  #import units: m, s, kg
  #let g0 = constant("standard-gravity", $9.80665 #m / #s^2$)
  #let triangle = operation("tri", display: () => math.op("triangle"))
  #worksheet(constants: ("standard-gravity": $g_0$))[
    $F := 2 kg #g0 =$

    $triangle(1/2) =$
  ]
]

// Complex numbers
#[
  #import units: Ω, Hz, mH
  #import constants: pi, i
  #import functions: Re, Im
  #worksheet(constants: (imaginary: $j$))[
    $Z := 100 #Ω + #i 2 pi 50 Hz dot 300 mH =$

    $abs(Z) =$, $arg(Z) =$, $Im(Z) =$

    $#i -> substitute(#i)$
  ]
]

// Worksheets and results
#[
  #import units: V, mA, Ω, m, s
  #worksheet(digits: 3)[
    $U := 12 #V$, $R := 100 #Ω$

    #worksheet(scope: "shared")[$I := U/R = mA$]

    #worksheet($I/3 =$, form: "fraction")

    #worksheet(evaluate: false)[Equations like $E = m c^2$ stay as written.]

    The current is #value($I = mA$).

    #worksheet(preferred-units: ($#m / #s$,), scope: "inherit")[$v := 36 #m / (2 #s) =$]

    #clear("R")

    #defined(style: "inline")
  ]
]
