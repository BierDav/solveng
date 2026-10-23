// solveng – unit-aware calculations that read like your notes, like a Mathcad worksheet.
//
//   #show: worksheet                    // the math in the document is calculated:
//   $I := 30 mA$, $Q_0 := 3450 mAh$      //   := defines
//   $t := Q_0 / I = h$                    //   a trailing = shows the value (bold), in hours
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
// units as names, unquoted in math: $4 mA$, and unit("…") for explicit references and shortcuts
#import "src/unit-words.typ": *

// Non-native math operation names are available without local declarations.
#import "src/builtins.typ": *
