// The solver: isolating, polynomials, systems, inequalities, functions.
#import "/tests/util.typ": *

#def("Q0 = 3450 mAh", "I = 30 mA")
#solve("Q0/2 = I t", unit: "h") #expect("t", 207000, u: "h")
#solve("I", unit: "A") #passes()
#reset

#solve("x^2 + 2x - 3 = 0") #expect("x", 1)
#solve("y^2 + 2y - 3 = 0", root: 1) #expect("y", -3)
#solve("a + b = 3", "a - b = 1") #expect("a", 2) #expect("b", 1)
#solve("cos(w) = w") #expect("w", 0.7390851332151607)
#reset

#def("U = 5 V", "I = 30 mA")
#solve("R <= U/I") #expect("R", 5 / 0.03)
#reset

#def("f(x) = x^2 + 1", "g(a, b) = a b + b")
#solve("y = f(3)") #expect("y", 10)
#solve("z = g(2, 3)") #expect("z", 9)
#reset

#def("R = 7.2 kOhm", "I = 415 uA")
#def("U = R I") #expect("U", 2.988)
#solve("Q = @U [2 s]") #expect("Q", 5.976)
#reset

// problems are reported, not ignored
#solve("Q0/2 = I t") #fails("unknowns")
#def("z = 7 furlongs") #fails("neither a unit nor a defined variable")
#def("x = 1", digitz: 2) #context assert(_st.get().at("x", default: none) == none)

// code in a solve-block: tagged code is calculated, plain code only with integration "code" or "full"
#solve-block[
  `x = 3` ```solveng y = 2 m```
  ```
  z = 4
  ```
  ```solveng
  w = y 2 -> cm
  ```
  #context assert("x" not in _st.get() and "z" not in _st.get(), message: "plain code was calculated")
  #expect("y", 2) #expect("w", 4)
]
#solve-config(integration: "code")[#solve-block[
  `x = 3`
  ```
  z = x 2
  ```
  $q = 2$
  #expect("x", 3) #expect("z", 6)
  #context assert("q" not in _st.get(), message: "math was read with integration: code")
]]
#solve-block[
  #solve-config(integration: "full")[`a = 1` $b = 2$]
  #solve-config(integration: "none")[`c = 1` $d = 2$ ```solveng e = 3```]
  #expect("a", 1) #expect("b", 2) #expect("e", 3)
  #context assert("c" not in _st.get() and "d" not in _st.get(), message: "integration: none read untagged code or math")
]


// editors read what a solve-block saw: each element (by location) and what the block defined
#solve-block[$k = 2 "V"$ `m = 1`]
#context {
  let r = query(<solveng>).map(it => it.value)
  assert(r.any(x => x.kind == "math" and x.active and type(x.element) == location))
  assert(r.any(x => x.kind == "code" and not x.active))
  assert(r.any(x => x.kind == "block" and x.defined.any(d => d.name == "k" and d.value == "2 V")))
}
