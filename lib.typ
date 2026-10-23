// solveng – unit-aware calculations that read like your notes, like a Mathcad worksheet.
//
//   #show: worksheet                    // the math in the document is calculated:
//   #import units: mA, mAh               //   unit words, imported where you want them
//   $I := 30 mA$, $Q_0 := 3450 mAh$      //   := defines
//   $t := Q_0 / I = "h"$                 //   a trailing = shows the value (bold), in hours
//   $x^2 = 4 -> solve(x)$                 //   -> the symbolic result, or solves
//   ```solveng
//   P := [5 V] I =                       // solveng code blocks show every step
//   ```

#import "src/api.typ": worksheet
#import "src/core.typ": clear
#import "src/keywords.typ": solve, substitute, simplify, digits, sci, eng
#import "src/defined.typ": defined
#import "src/graph.typ": graph
#import "src/lambdify.typ": lambdify
#import "src/value.typ": value
#import "src/operation.typ": unit, operation
// shorthands only where imported, never through `*`: #import units: km, mA, s and #import functions: tri, sgn
#import "src/unit-words.typ" as units
#import "src/builtins.typ" as functions
