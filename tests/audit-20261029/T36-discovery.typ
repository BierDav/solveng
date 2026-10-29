#import "/lib.typ": *
#import modifiers: solve
#import "/tests/util.typ": reported
#show: worksheet
$x^2 = 4 -> solve(x, typo: 1)$
#reported("typo")
