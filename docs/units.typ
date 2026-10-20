#import "/lib.typ": *
#set page(width: 13cm, height: auto, margin: 0.6cm)
#def("A = 12")
#solve("x = A [A]")
#solve("y = A A", unit: "A")
#def("T = 2 s")
#def("f = 1/T")
#solve("f = 1/(T)")
