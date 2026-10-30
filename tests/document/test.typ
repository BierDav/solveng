// A worksheet without a single call: the whole document is calculated.
#import "/lib.typ": *
#import units: mA, mAh, h, V
#set page(width: 13cm, height: auto, margin: 0.6cm)
#show: math

= Battery runtime
The battery holds $Q_0 := 3450 mAh$ and the load draws $I := 30 mA$,
so it lasts $t := Q_0 / I => #h$.

At half charge and a supply of $U := 5 #V$:
$t_"half" := Q_0 / (2 I) => #h$

$P := U I =>$
