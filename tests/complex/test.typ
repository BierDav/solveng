// Complex numbers and constant-displays: an expression with i is calculated in the complex numbers, one without stays real;
// constants carry an identity, shown as the worksheet's option constants says.
#import "/tests/util.typ": *
#import "/src/engine.typ": _constant-display, _m
#import units: m, s, kg, min, Ω, mH, Hz, V
#import constants: pi, e, i, c
#import operations: Re, Im, conj
#import modifiers: solve, substitute, assume, steps
#set page(width: 18cm, height: auto)
#show: math

// i arithmetic
#mathscope[
  $sqrt(-4) + #i =>$ #passes() #result("\"3\" i")
  $#i^2 =>$ #result("\"−1\"")
  $#i^2 ->$ #result("\"−1\"")
  $1 / #i =>$ #result("-i")
  $(1 + #i)^2 =>$ #result("\"2\" i")
  $(2 + #i)(2 - #i) ->$ #result("\"5\"")
  $#e^(#i pi) + 1 =>$ #result("\"0\"")
  $#e^(#i pi) + 1 ->$ #result("\"0\"")
  $z := 3 + 4 #i$
  $z =>$ #result("\"3\" + \"4\" i")
  $z / (1 - #i) =>$ #result("\"−0.5\" + \"3.5\" i")
  // the parts, the magnitude, the angle and the conjugate
  $Re(z) =>$ #result("\"3\"")
  $Im(z) =>$ #result("\"4\"")
  $abs(z) =>$ #result("\"5\"")
  $arg(z) =>$ #result("\"0.9273\"")
  $conj(z) =>$ #result("\"3\" - \"4\" i")
  $z conj(z) =>$ #result("\"25\"")
  $arg(#i) ->$ #result("frac(pi, \"2\")")
  // real parts are values of their own
  $x := Re(z) =>$ #expect("x", 3) $y := Im(z) =>$ #expect("y", 4) $r := abs(z) =>$ #expect("r", 5)
  $phi := arg(z) =>$ #expect("φ", calc.atan2(3, 4).rad())
  // a constant stays itself; substitute puts in its expression, as written when it's alone
  $#i ->$ #result("i")
  $#i -> substitute(#i)$ #result("sqrt(\"−1\")")
]

// Real projection is linear even while the signals are still undefined.
#mathscope[
  $a := 2$, $b := -3$
  $x(t) := a s_a(t) + b s_b(t)$
  $y_1(t) := Re(x(t))$
  $y_"a1"(t) := Re(s_a(t))$
  $y_"a2"(t) := Re(s_b(t))$
  $y_2(t) := a y_"a1"(t) + b y_"a2"(t)$
  $y_1(t) = y_2(t) ->$ #passes() #result("\"true\"")
  $Re(z w) = Re(z) Re(w) - Im(z) Im(w) ->$ #result("\"true\"")
]

// Sufficient real-domain conditions and assumptions local to one result.
#mathscope[
  $x(t) := a s_a(t) + b s_b(t)$
  $y_1(t) := Re(x(t))$
  $y_2(t) := a Re(s_a(t)) + b Re(s_b(t))$
  $y_1(t) = y_2(t) ->$ #passes() #result("\"true\" \"for\" a in RR and b in RR")
  $y_1(t) = y_2(t) -> assume(a in RR, b in RR)$ #passes() #result("\"true\"")
  $Re(a z) = a Re(z) -> assume(a in RR)$ #result("\"true\"")
  $Re(a z) = a Re(z) -> assume(a in RR), steps$ #passes()
  $Re(a z) = a Re(z) ->$ #result("\"true\" \"for\" a in RR")
]

// with units: an impedance
#mathscope[
  $Z := 100 #Ω + #i 2 pi 50 Hz 10 mH =>$ #passes() #result("lr((\"100\" + \"3.142\" i)) thin Ω")
  $R := Re(Z) =>$ #expect("R", 100, u: "Ω")
  $X := Im(Z) =>$ #expect("X", calc.pi, u: "Ω")
  $Z_m := abs(Z) =>$ #expect("Z_m", calc.sqrt(100 * 100 + calc.pi * calc.pi), u: "Ω")
  $phi := arg(Z) =>$ #expect("φ", calc.atan2(100, calc.pi).rad())
  $U := 10 #V$
  $I := U / Z =>$ #passes() #result("lr((\"99.9\" - \"3.138\" i)) thin mA")
  $I_m := abs(I) =>$ #expect("I_m", 10 / calc.sqrt(100 * 100 + calc.pi * calc.pi), u: "mA")
]

// solving: in the complex numbers
#mathscope[
  $w^2 = -#i -> solve(w)$ #passes() #result("w_(1) = \"−0.7071\" + \"0.7071\" i, quad w_(2) = \"0.7071\" - \"0.7071\" i")
  $p^2 + 1 = 0 -> solve(p)$ #result("p_(1) = -i, quad p_(2) = i")
]

// values are complex, also without i written; complex numbers have no order
#mathscope[#math(errors: "document")[
  $sqrt(-4) =>$ #passes() #result("\"2\" i")
  $sqrt(-4) ->$ #result("\"2\" i")
  $ln(-1) ->$ #result("i dot pi")
  $(-8)^(1/3) =>$ #result("\"−2\"")
  $#i < 1 =>$ #fails("complex numbers have no order")
]]

// the option constants shows an identity differently, in that worksheet only
#mathscope[
  #mathscope[#math(constant-displays: (imaginary: $j$))[
    $a := 2 #i =>$ #passes() #result("\"2\" i")
    #context {
      assert.eq(_constant-display("imaginary", none), $j$)
      let shown = repr(_m(_st.get().at("last").result, _cf.get()))
      assert(shown.contains("[j]") and not shown.contains("[i]"), message: shown)
    }
  ]]
  $b := 2 #i =>$ #result("\"2\" i")
  #context {
    assert.eq(_constant-display("imaginary", none), $i$)
    let shown = repr(_m(_st.get().at("last").result, _cf.get()))
    assert(shown.contains("[i]") and not shown.contains("[j]"), message: shown)
  }
]

// built-in constants and constants of your own
#mathscope[
  $"circle" := 2 pi =>$ #expect("circle", 2 * calc.pi)
  $"lightspeed" := #c =>$ #expect("lightspeed", 299792458, u: "Mm/s")
  // constants stay symbolic until substitute puts their expressions in
  $#c / 2 ->$ #context assert(_st.get().last.result.starts-with("frac(#svconst(\"speed-of-light\""), message: _st.get().last.result)
  $#c -> substitute(#c)$ #result("\"299792458\" thin m\\/s")
  // a variable's value with the constant's expression put in
  $E_0 := 2 kg #c^2$
  $E_0 ->$ #context assert(_st.get().last.result.contains("svconst(\"speed-of-light\""), message: _st.get().last.result)
  $E_0 -> substitute(#c)$ #context assert(not _st.get().last.result.contains("svconst"), message: _st.get().last.result)
  $t := 2 min =>$ #expect("t", 120, u: "min") // the unit min, not Typst's min
  // an identity the engine doesn't know: kept by it, calculated with its expression
  #let g0 = constant("standard-gravity", $9.80665 #m / #s^2$)
  $F := 2 kg dot g0 =>$ #expect("F", 2 * 9.80665)
  $g0 -> substitute(g0)$ #result("\"9.80665\" thin m\\/s^(2)")
  $2 kg dot g0 -> substitute(g0)$ #result("\"19.61\" thin N")
  #mathscope[#math(constant-displays: ("standard-gravity": $g_0$))[
    #context assert.eq(_constant-display("standard-gravity", none), $g_0$)
  ]]
  #context assert.eq(_constant-display("standard-gravity", none), none)
  // a constant of a known identity, under another name
  #let j = constant("imaginary", $sqrt(-1)$)
  $q := 2 #j + #i =>$ #result("\"3\" i")
  #let tau = constant("tau", $2 pi$)
  $T := tau =>$ #expect("T", 2 * calc.pi)
]
