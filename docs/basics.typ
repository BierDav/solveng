#import "/lib.typ": *
#import units: mA, mAh, h, V
#import modifiers: solve
#set page(width: 13cm, height: auto, margin: 0.6cm)
#show: worksheet
$Q_0 := 3450 mAh$, $I := 30 mA$, $U := 5 #V$

$Q_0/2 = I t -> solve(t)$ and $t => #h$

$R <= U/I -> solve(R)$

$x^2 + 2x - 3 = 0 -> solve(x)$

Runtime: $Q_0/I => #h$
