// The first example of the README: a document without def or solve.
#import "/lib.typ": *
#set page(width: 13cm, height: auto, margin: 0.6cm)
#show: solve-block

= Battery runtime
The battery holds $Q_0 = 3450 "mAh"$ and the load draws $I = 30 "mA"$,
so it lasts $t = Q_0 / I$ – that is `= t -> h`.

At half charge and a supply of $U = 5 "V"$:
```
t_half = Q_0 / (2 I) -> h
P = U I
```
