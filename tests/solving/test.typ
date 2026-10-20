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
