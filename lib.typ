// solveng – unit-aware "given → solve" calculations with minimal boilerplate.
//
//   #def("R = 7.2 kΩ", "I = 415 µA")  // stores the variables
//   #solve("U = R I")                  // finds the unknown, rearranges, substitutes, evaluates
//   #solve("Q0/2 = I t", unit: "h")    // optional target unit
//   #solve("Q = @I t")                 // @I inlines the stored expression of I
//   #val("U")  #reset  #solve-config(digits: 3, si: true)
//   #solve-block[ $I = "30 mA"$ … ]     // math equations and code blocks inside are definitions


#import "src/graph.typ": def, solve, solve-block, solve-config, definitions, val, reset, graph, curve
