// Units and variables: what is a unit, what is a variable, and what is refused as unclear.
#import "/tests/util.typ": *

// code: a unit right after a number or in brackets, names in equations are variables
#worksheet[
  #def("A := 12", "x := A [A]") #expect("x", 12, u: "A")
  #def("z := [A] A") #expect("z", 12)
  // a value whose unit is also a variable is refused
  #def("T := 2 s")
  #def("f := 1/T") #passes() #expect("f", 0.5)
  #def("y := 3 T") #fails("both a defined variable and a unit")
  #def("y2 := 3*T") #expect("y2", 6)
  #def("B := [3 T]") #expect("B", 3, u: "T")
]

// Math uses quoted single-letter units; bare single letters stay variables.
#worksheet[
  $h := 2$ $t := 3 h$ #expect("t", 6)
  $t_h := 3 unit("h")$ #expect("t_h", 10800)
  $t_1 := 3 dot h$ #expect("t_1", 6)
  $U := 5 "V"$ #expect("U", 5, u: "V")
  $R := 4.7 kΩ$ #expect("R", 4700)
  $R_2 := 1 unit("kΩ")$ #expect("R_2", 1000)
  $s := 2 mΩ$ #expect("s", 0.002)
  $rho := 0.0178 "Ω" mm^2/"m"$ #expect("ρ", 1.78e-8)
  $Q_0 := frac(3450, 2) mAh$ #expect("Q_0", 6210)
  $p := 3 unit("bar")$ #fails("unknown unit 'bar'")
  $I := 2 "A"$ $P := I^2 dot 3 "Ω" = 1 "W"$ #expect("P", 12)
  $U_2 := 1,5 "V"$ #expect("U_2", 1.5)
  $m_O := 2 "mΩ"$ #expect("m_O", 0.002)
  $A_x := 12$ $x := A_x dot 2 "A" =$ #expect("x", 24)
]

// A·A is A², not "A A"; mol and cd
#worksheet[
  #def("I := 2 A", "P := I I") #expect("P", 4, u: "A^2")
  #def("n := 2 mmol", "c := 3 cd") #expect("n", 0.002, u: "mmol") #expect("c", 3)
]

// custom units, removing units, preferred result units
#worksheet(units: (bar: "100 kPa", rpm: "1/min"))[
  #def("p := 3 bar") #expect("p", 3e5)
  #def("F := p [2 cm^2]") #expect("F", 60, u: "N")
  #def("w := 3000 rpm") #expect("w", 50)
  #def("w2 := w = [Hz]") #expect("w2", 50, u: "Hz")
  #def("p2 := 2 mbar") #expect("p2", 200)
  #worksheet(inherit: true, preferred-units: ("bar",))[
    #def("p3 := 2 p") #expect("p3", 6e5, u: "bar")
  ]
  #def("p4 := 2 p") #expect("p4", 6e5, u: "bar") // the unit of the inputs
]
#worksheet(units: (T: none))[
  #def("T := 2 s", "y := 3 T") #expect("y", 6) // T is no unit any more: 3 times T
  #def("x := [2 T]") #passes() #expect("x", 2, u: "T")
]
#worksheet(units: (T: "1 kg/(s^2 A)"))[
  #def("B := 2 T") #expect("B", 2)
]

// strict: unit names can't be variables
#worksheet(strict: true)[
  #def("A := 3 m^2") #fails("is a unit and can't be a variable")
  #def("f(V) := 2 V") #fails("is a unit and can't be a variable")
  #def("Area := 3 m^2") #passes()
  #def("P_1 := Area [2 Pa]") #expect("P_1", 6)
  #def("5 = A_1 + 2 -> solve(A_1)") #expect("A_1", 3)
]

// auto: a base unit of its own, with prefixes and in products; numbers are plain factors
#worksheet(units: (ü: auto, sheep: auto, dozen: 12))[
  #def("x := 3 mü", "y := 2 ü", "n := 2 dozen", "a := 4 [sheep m]")
  #def("w := x * y", "v := x + y = [ü]")
  #expect("x", 0.003) #expect("y", 2) #expect("n", 24) #expect("a", 4) #expect("w", 0.006) #expect("v", 2.003, u: "ü")
  #def("z := x + [1 m] =") #fails("different units")
]

// options are checked
#worksheet(units: (furl: "100000 furlong"))[
  // a unit the engine can't read is left out (with a note)
  #def("p := 1 furl =") #fails("'furl' in 'p' isn't defined")
]

// Omega is Ω; a prefix goes with it glued, as in math, never written together
#worksheet[
  #def("R1 := 3 k\"\"Omega", "R2 := 3 kΩ", "R3 := 2 Omega")
  #expect("R1", 3000) #expect("R2", 3000) #expect("R3", 2)
  #def("R5 := 3 kOmega =") #fails("kOmega")
]
