#import "/lib.typ": *
#set page(width: 13cm, height: auto, margin: 0.6cm)
#def("Q0 = 3450 mAh", "I = 30 mA")
#definitions()
#solve("Q0/2 = I t", unit: "h")
#def("U = 5 V")
#solve("R <= U/I")
#solve("x^2 + 2x - 3 = 0")
