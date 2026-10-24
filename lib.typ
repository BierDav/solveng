// solveng – unit-aware calculations that read like your notes, like a Mathcad worksheet.
//
//   #import units: mA, mAh, h              // units, constants and operations are imported where you use them
//   #show: worksheet                       // the math in the document is calculated:
//   $I := 30 mA$, $Q_0 := 3450 mAh$        //   := defines
//   $t := Q_0 / I = #h$                    //   a trailing = shows the value (bold), in hours
//   $x^2 = 4 -> solve(x)$                  //   -> the symbolic result, or solves

#import "src/api.typ": worksheet
#import "src/core.typ": clear
#import "src/keywords.typ": solve, substitute, simplify, digits, sci, eng, fraction, any
#import "src/defined.typ": defined
#import "src/graph.typ": graph
#import "src/lambdify.typ": lambdify
#import "src/value.typ": value
#import "src/operation.typ": base-unit, unit, constant, operation
// bindings only where imported, never through `*`: #import units: km, mA, s, #import functions: tri, sgn and
// #import constants: pi, i
#import "src/unit-words.typ" as units
#import "src/builtins.typ" as functions

#import "src/constants.typ" as constants
