#import "/src/engine.typ": _restyle
#import "/src/config.typ": _DEF
#let v = math.lr(size: 200%, $[1 + 2]$)
#let r = _restyle(v, _DEF)
#assert.eq(r.fields(), v.fields(), message: "delimiter size must survive worksheet restyling")
