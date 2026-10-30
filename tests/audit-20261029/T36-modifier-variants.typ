#import "/lib.typ": *
#import modifiers: solve, substitute, assume
#import "/tests/util.typ": reported
#show: math
$x^2 = 4 -> solve(x, name: 1)$
#reported("name")
$x + 1 -> substitute(x, typo: 1)$
#reported("typo")
$x + 1 -> assume(x > 0, typo: 1)$
#reported("typo")
