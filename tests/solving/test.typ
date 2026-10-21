// The solver: isolating, polynomials, systems, inequalities, functions.
#import "/tests/util.typ": *

#def("Q0 = 3450 mAh", "I = 30 mA")
#solves("Q0/2 = I t", unit: "h") #expect("t", 207000, u: "h")
#solves("I", unit: "A") #passes()
#reset

#solves("x^2 + 2x - 3 = 0") #expect("x", 1)
#solves("y^2 + 2y - 3 = 0", root: 1) #expect("y", -3)
#solves("a + b = 3", "a - b = 1") #expect("a", 2) #expect("b", 1)
#solves("cos(w) = w") #expect("w", 0.7390851332151607)
#reset

#def("U = 5 V", "I = 30 mA")
#solves("R <= U/I") #expect("R", 5 / 0.03)
#reset

#def("f(x) = x^2 + 1", "g(a, b) = a b + b")
#solves("y = f(3)") #expect("y", 10)
#solves("z = g(2, 3)") #expect("z", 9)
#reset

#def("R = 7.2 kΩ", "I = 415 uA")
#def("U = R I") #expect("U", 2.988)
#solves("Q = @U [2 s]") #expect("Q", 5.976)
#reset

// problems are reported, not ignored
#solves("Q0/2 = I t") #fails("unknowns")
#def("z = 7 furlongs") #fails("neither a unit nor a defined variable")
#def("x = 1", digitz: 2) #context assert(_st.get().at("x", default: none) == none)

// code in a worksheet: solveng code blocks are calculated, inline and untagged code never
#worksheet[
  `x := 3` ```solveng y := 2 m```
  ```
  z := 4
  ```
  ```solveng
  y := 2 m
  w := y 2 = [cm]
  ```
  #context assert("x" not in _st.get() and "z" not in _st.get(), message: "plain code was calculated")
  #expect("y", 2) #expect("w", 4, u: "cm")
]
#worksheet[
  #worksheet-config(integration: "none")[$d := 2$ ```solveng
  e := 3
  ```]
  #expect("e", 3)
  #context assert("d" not in _st.get(), message: "integration: none read math")
  $g := 5$ #expect("g", 5)
]

// code lines: a trailing = shows, -> solves, = alone states
#worksheet[
  ```solveng
  Q0 := 3450 mAh; I := 30 mA
  t := Q0/I = [h], digits(3)
  Q0/I =
  x^2 = 9 -> solve(x)
  a + b = 3; a - b = 1 -> solve(a, b)
  ```
  #expect("t", 414000, u: "h") #expect("x", 3) #expect("a", 2) #expect("b", 1)
]

// editors read what a worksheet saw: each element (by location) and what the block defined
#worksheet[$k := 2 V$ `m := 1`]
#context {
  let r = query(<solveng>).map(it => it.value)
  assert(r.any(x => x.kind == "math" and x.active and type(x.element) == location))
  assert(r.any(x => x.kind == "block" and x.defined.any(d => d.name == "k" and d.value == "2 V")))
}
// only the innermost worksheet reads an element
#worksheet[
  #worksheet[$h := 3$ #expect("h", 3)]
  $j := 4$ #expect("j", 4)
]
#context {
  let r = query(<solveng>).map(it => it.value).filter(x => x.kind == "math" and x.active)
  assert.eq(r.len(), r.map(x => x.element).dedup().len(), message: "an element was read twice")
}
// worksheet-config calculates nothing on its own
#worksheet-config(digits: 3)[$o := 1$]
#context assert("o" not in _st.get(), message: "worksheet-config calculated")

// calls: equations where they stand, hidden ones silently, options for them only
#worksheet($Delta_R := 5 cm$, "g_2 := 2 m")
#expect("Δ_R", 0.05) #expect("g_2", 2)
#worksheet($n_h := 7$, hidden: true) #expect("n_h", 7)
#worksheet[$n_i := 1$ #worksheet($n_j := n_i + 1$) #expect("n_j", 2)]
#context {
  let r = query(<solveng>).map(it => it.value).filter(x => x.kind == "math" and x.active)
  assert.eq(r.len(), r.map(x => x.element).dedup().len(), message: "a called equation was read again")
}

// math: := defines, = states (checked or prose), a trailing = shows, -> solves
#worksheet[
  $U := 5 V$ $R := 2 Ω$
  $E = m c^2$ #passes()
  $I_1 := U/R = mA$ #expect("I_1", 2.5)
  $Q_0/2 = I_1 t -> solve(t)$ #fails("unknown variable 'Q_0'")
  $Q_0 := 10 mAh$ $Q_0/2 = I_1 t -> solve(t)$ #expect("t", 7.2)
  $x^2 - 4 = 0 -> solve(x)$ #expect("x", 2)
  $P_1 <= U I_1 -> solve(P_1)$ #expect("P_1", 12.5)
  $c := 1/3 -> frac$ #expect("c", 1/3)
  $V := 2 A = 2000 mA$ #expect("V", 2)
]

// parameters with a unit: plain numbers get it, values must have its dimension, graphs take x's unit from it
#worksheet[
  #def("q(x: A) = x * [2 Ω]", "f(I: mA, R) = I R")
  #def("y = q(3)", "z = q(2 [mA])", "u = f(3, [2 Ω])")
  #expect("y", 6) #expect("z", 0.004) #expect("u", 0.006)
  #def("w = q(2 [V])")
  #fails("q takes x in A, got V")
  #worksheet-config(integration: "math")[
    $ p(I: mA) := 2 I $
    #def("k = p(5)")
    #expect("k", 0.01)
  ]
]

// function parameters: listed (only those, with defined variables), .. or () for the rest
#worksheet[
  #def("k = 2", "f(x) = x^2 + k", "g(x, ..) = x * m", "c() = [5 V]")
  #def("y1 = f(3)", "y2 = g(2, 4)", "y4 = c()")
  #expect("y1", 11) #expect("y2", 8) #expect("y4", 5)
  // f() takes nothing, unless implicit-parameters says otherwise
  #def("h() = a + b")
  #fails("implicit-parameters: \"empty\"")
  #worksheet-config(implicit-parameters: "empty")[
    #def("h2() = a + b", "y3 = h2(1, 2)")
    #expect("y3", 3)
    #def("h3(x) = x + n")
    #fails("implicit-parameters: \"allow\"")
  ]
  #worksheet-config(implicit-parameters: "allow")[
    #def("h4(x) = x + n", "h5() = p", "y5 = h4(1, 2) + h5(3)")
    #expect("y5", 6)
  ]
  #def("p(x) = x + n")
  #fails("'n' in p isn't defined")
  // a parameter's unit is known in the body: q(x) checks it
  #def("q(x: A) = x", "r(y: A) = q(y) * [2 Ω]", "z = r(2)")
  #expect("z", 4)
  #def("s(y: V) = q(y)")
  #fails("q takes x in A, got V")
]

// variables take no type, their unit is their value's; a cast gives a value a unit
#worksheet[
  #def("I: mA = 0.004 A")
  #fails("a variable has the unit of its value, so it takes no type")
  #def("R = 5 Ω")
  #def("U = R [4 mA]", unit: "mV")
  #expect("U", 0.02, u: "mV")
]

// casts bypass the units: e as u is e's number in SI units as a value in u, on the term before it with its power
#worksheet[
  #def("U = 5 V", "I = U as A", "J = U as mA", "n = 2 * U as 1", "K = (U * 2) as [mA]", "L = U^2 as V")
  #expect("I", 5) #expect("J", 0.005) #expect("n", 10) #expect("K", 0.01) #expect("L", 25)
  #context assert.eq(_st.get().at("J").u, "mA")
  #def("q(x: A) = x * [2 Ω]", "y = q(U as A)")
  #expect("y", 10)
  #def("z = q(U)")
  #fails("q takes x in A, got V")
  #def("w = U as furlong")
  #fails("the unit to cast to")
  #worksheet-config(integration: "math")[
    $ M := U "as" mA $
    #expect("M", 0.005)
  ]
]

// after a number as is the unit, attosecond; after anything else it casts
#worksheet[
  #def("U = 5 V", "a = 3 as", "b = U as as", "c = 3 as A")
  #expect("a", 3e-18) #expect("b", 5e-18) #expect("c", 3e-18)
  #worksheet-config(integration: "math")[
    $ e := 3 "as" $
    $ f := U "as" mA $
    $ q(x: mA) := 2 x $
    #def("g = q(3)")
    #expect("e", 3e-18) #expect("f", 0.005) #expect("g", 0.006)
  ]
  // hidden casts and types still calculate
  #worksheet-config(integration: "math", show-casts: "no", show-types: "no")[
    $ h := U "as" mA $
    $ w(x: A) := x $
    #def("W = w(2)")
    #expect("h", 0.005) #expect("W", 2)
  ]
  // units of casts and types: glued, grouped, simplified
  #worksheet-config(integration: "math", show-casts: "plain", show-types: "plain")[
    $ i := U "as" unit("A as") $
    $ j := U "as" (A * "as") / V $
    $ s_x(z: mm mm/m) := z $
    $ p(z: (A s)/V) := z $
    #def("k = U as (A as)/[V]", "l = p(2)")
    #expect("i", 5e-18) #expect("j", 5e-18) #expect("k", 5e-18) #expect("l", 2)
    #context assert.eq(_st.get().at("#fn").s_x.units.z.u, "µm")
  ]
]

// function types: any, 1, inferred from what parameters are passed to, the result's type; no written return types
#worksheet[
  #worksheet-config(implicit-parameters: "empty")[
    #def("q_t(x: A, y: any) = x y", "f_t(y_2: A, ..) = q_t(x_2, y_2)", "o_t(x, ..) = f_t(x, d)", "W_t(x: V) = f_t(x as A, x as A)", "Z_t(x: 1) = 3 x [A]", "g_t(l: m) = 2 l")
    #def("a_t = g_t([3 cm])", "z_t = Z_t(2)")
    #expect("a_t", 0.06) #expect("z_t", 6)
    #context {
      let fs = _st.get().at("#fn")
      assert.eq(fs.f_t.units.keys(), ("y_2", "x_2"))
      assert.eq(fs.o_t.units.keys(), ("x", "d"))
      assert.eq(fs.q_t.anys, ("y",))
      assert.eq((fs.q_t.ret, fs.f_t.ret, fs.W_t.ret, fs.Z_t.ret), (none, "A^2", "A^2", "A"))
    }
    #def("U_t(x: V) = f_t(x as A, x)")
    #fails("f_t takes x_2 in A, got V")
    #def("Y_t(x: 1): V = 3 x [A]")
    #fails("a function returns what its body gives")
  ]
]

// := assigns, the last assignment wins, functions too; = states what's true: in math it's checked, in strings it
// determines what isn't known too, and with a known function on the left it's an equation (U(o) = 20 finds o)
#worksheet[
  #worksheet-config(integration: "math")[
    $ I_r := 4 A $
    $ I_r = 4000 mA $ #passes()
    $ I_r := 2 A $
    $ J_r := 2 A "as" mA $
    $ t_r := 3 "as" $
    #def("f_r(x) = x", "f_r(x) := 2 x", "y_r = f_r(3)", "g_r(x) = x + 1", "g_r(x) = 9")
    #expect("I_r", 2) #expect("J_r", 0.002) #expect("t_r", 3e-18) #expect("y_r", 6) #expect("x", 8)
    #def("I_r = 3 A")
    #fails("I_r is 2 A already")
  ]
  #solves("K_r := 3")
  #context assert("K_r" not in _st.get(), message: "solve assigned")
]

// a function may compare: 1 where it holds, else 0; both sides in one unit
#worksheet[
  #worksheet-config(integration: "math")[
    $ U_c(x, y) := x = y $
    $ L_c(x: A, y: A) := x < y $
    #def("E_c(x) := x >= 2", "a_c = U_c(2, 2)", "b_c = U_c(2, 3)", "c_c = L_c(1, 2)", "d_c = E_c(1)", "g_c = L_c(2, [1 A])")
    #expect("a_c", 1) #expect("b_c", 0) #expect("c_c", 1) #expect("d_c", 0) #expect("g_c", 0)
    #def("k_c = L_c(1, [1 V])")
    #fails("L_c takes y in A, got V")
  ]
]

// a chain: its steps are equal, its last one gives the value, shown in the unit that fits
#worksheet[
  #def("R_h = 7.2 kΩ = 7200 Ω", "S_h = 2 A = 2000 mA")
  #expect("R_h", 7200, u: "kΩ") #expect("S_h", 2, u: "A")
  #def("X_h = 7.2 kΩ = 7300 Ω")
  #fails("has another value than")
  #def("P_h = [5 V] [2 A]", unit: "mW")
  #expect("P_h", 10, u: "mW")
]

// := with steps checks them; a comparison assigned is whether it holds, 1 or 0; values of different units are never
// equal (no error)
#worksheet[
  #worksheet-config(units: (über: auto))
  $ W_s := 2 A = 2000 mA $
  $ Z_s := 3 A < 4 A $
  $ U_s(x, y) := x = y $
  #def("a_s = U_s([2 A], [2 V])", "c_s := 2 A != 2 V")
  #expect("W_s", 2) #expect("Z_s", 1) #expect("a_s", 0) #expect("c_s", 1)
  #def("V_s := 2 über = 2 A")
  #fails("has another unit")
]
