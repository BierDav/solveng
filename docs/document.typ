#import "/lib.typ": *
#set page(width: 13cm, height: auto, margin: 0.6cm)
#show: worksheet

= Battery runtime
The battery holds $Q_0 := 3450 mAh$ and the load draws $I := 30 mA$,
so it lasts $t := Q_0 / I = 1 h$.

At half charge and a supply of $U := 5 V$:
```solveng
t_half := Q_0 / (2 I) = [h]
P := U I =
```
