#import "/src/engine.typ": _restyle
#import "/src/config.typ": _DEF
#let v = std.math.vec([1], [2], delim: "[", gap: 2em)
#let r = _restyle(v, _DEF)
#assert.eq(r.fields(), v.fields(), message: "vector styling fields must survive worksheet restyling")
