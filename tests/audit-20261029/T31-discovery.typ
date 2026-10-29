#import "/lib.typ": *
#graph($x$, x-range: (0,1), colors: gradient.linear(red, blue))

#import "/src/config.typ": _target
#import "/tests/util.typ": reported
#context if _target() == "html" { reported("gradient") }
