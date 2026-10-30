#import "/lib.typ": *
#import units: Ω, W, V
#set page(width: 13cm, height: auto, margin: 0.6cm)
#math($R := 50 #Ω$, $P_0 := 2 #W$, hidden: true)
#graph($P(U: #V) = U^2/R$, ymark: $P_0$)
