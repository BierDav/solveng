#import "/lib.typ": *
#set page(width: 13cm, height: auto, margin: 0.6cm)
#def("R = 50 Ω", "P0 = 2 W")
#solve("P0 = U^2/R")
#graph("P = U^2/R", x: "U")
