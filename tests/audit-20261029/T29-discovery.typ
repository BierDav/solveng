#import "/lib.typ": *
#import "/src/operation.typ": _references
#let bit = base-unit("information", display: "bits")
#let v = std.math.root(none, bit)
#assert.eq(_references(v), _references(bit), message: "radical must retain nested binding displays")
