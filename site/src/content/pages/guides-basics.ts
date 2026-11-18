import type { Page } from "../types";

const IMPORT = `#import "@preview/solveng:3.0.0": *`;

export const pages: Page[] = [
  {
    slug: "values",
    title: "Writing values",
    group: "guides",
    icon: "✎",
    summary: "Numbers, names, Greek letters, subscripts and operators – how to write the values a worksheet calculates with.",
    keywords: ["numbers", "names", "greek", "subscript", "implicit multiplication", "e-notation", "decimal point", "decimal separator", "quotes"],
    blocks: [
      { md: "A worksheet is Typst math, so values are written the way Typst writes math. A few rules decide what is a number, what is a name and what multiplies what." },
      { h2: "Numbers and units" },
      { md: "A value is a number, often times a unit. The number can be arithmetic: fractions, products, powers. A space or nothing between number and unit is a product." },
      {
        example: {
          title: "Values as you'd write them",
          code: `${IMPORT}
#import units: kΩ, A, V, m, mAh
#show: math

$R := 4.7 kΩ$
$I := frac(2, 4) #A$, $Q_0 := 3450/2 mAh$
$U := (1 + 2) dot 1.5 #V$, $A := 2^3 #m^2$
$U / R =>$`,
          caption: "`A` is the variable, `#A` the ampere – one-letter units need a `#`.",
        },
      },
      { h3: "Decimal point, always" },
      { md: "Numbers are written with a point, `7.2`, in every language: in Typst math a comma separates arguments, so `mat(2,4,6)` is three numbers. Results follow the document's language – a German document shows 2,4. The option `decimal-separator` picks another." },
      {
        example: {
          title: "A German worksheet",
          code: `#set text(lang: "de")
${IMPORT}
#import units: V
#show: math

$U := 7.2 #V$
$U / 3 =>$`,
        },
      },
      { h3: "No e-notation" },
      { md: "Write large and small numbers out, or as a power of ten: `11200`, `1.12 dot 10^4`, `2.5 dot 10^(-7)`. `1.12e4` isn't a number in math – it's 1.12 times something called `e4`. Scientific notation is only how results are *shown* (option `notation-thresholds`, modifier `sci`)." },
      {
        example: {
          code: `${IMPORT}
#import units: F
#import modifiers: sci
#show: math

$x := 1.12 dot 10^4 =>$
$C := 2.2 dot 10^(-6) #F$
$C => sci$`,
        },
      },
      {
        quiz: {
          question: "What is `$x := 1.12e4$` in a worksheet?",
          options: ["The number 11200", "1.12 times a name `e4`", "1.12 times Euler's number to the 4th"],
          answer: 1,
          explain: "Math has no e-notation: digits and letters split into the number 1.12 and the name `e4`. Typst even stops at it, since `e4` isn't a known name. Write `1.12 dot 10^4`.",
        },
      },
      { h2: "Names" },
      { md: "A name is one letter, a Greek letter, or either with a subscript:\n\n- **Letters**: `x`, `R`, `U` – one letter each. In Typst math, `ab` would be one unknown identifier, so several letters need quotes (below).\n- **Greek**: `alpha`, `omega`, `tau`, `Delta` – Typst's symbol names, shown as α, ω, τ, Δ.\n- **Subscripts**: `R_1`, `omega_0`, `U_\"in\"` (quoted text upright), `R_(a b)` for a grouped subscript, R_ab.\n- **Several letters**: in quotes, `\"rate\"`, `\"t2\"`. Quoted text is always a name, even when it spells a unit: `$\"mA\" := 3$` is a variable.\n- **Joined Greek**: `Delta t` is two names, Δ times t. `Delta\"\"t` joins them into one name, Δt." },
      {
        example: {
          title: "Names of every kind",
          code: `${IMPORT}
#import units: V, Ω, s
#show: math

$alpha := 30$, $omega_0 := 2$
$U_"in" := 5 #V$, $R_(a b) := 2 #Ω$
$"rate" := 3 / #s$
$Delta""t := 4 #s$
$U_"in" / R_(a b) =>$, $"rate" Delta""t =>$`,
        },
      },
      { h2: "Operators and functions" },
      { md: "- **Arithmetic**: `+ - / ^`, and for products `dot` (·), `times` (×), `*` or just a space.\n- **Typst's math functions**, no import needed: `sqrt`, `root`, `exp`, `ln`, `log`, `sin`, `cos`, `tan`, `arcsin`, `arccos`, `arctan`, `sinh`, `cosh`, `tanh`, `abs`, `arg`.\n- **solveng's operations** from `operations`: `tri`, `integrate`, `rank`, … [Operations →](/docs/operations/tri)\n- **Your own functions**, `f(x) := …`. [Functions →](/docs/functions)" },
      {
        example: {
          title: "Functions without imports",
          code: `${IMPORT}
#show: math

$a := 6$
$sqrt(a^2 + 64) =>$, $abs(-a) =>$
$ln(a) =>$, $exp(1) =>$, $log(1000) =>$
$a/2 times 3 =>$`,
        },
      },
      { h3: "Implicit multiplication" },
      { md: "Things written next to each other multiply: `2 pi f C`, `2x`, `R I`. One exception: a name directly followed by parentheses is a **call**, as in `f(x)`. So `U_0 (1 + x)` calls U₀ – write `U_0 dot (1 + x)` or `(1 + x) U_0` for the product." },
      {
        example: {
          title: "Same product, four spellings",
          code: `${IMPORT}
#import units: Hz, nF
#import constants: pi
#show: math

$f := 50 Hz$, $C := 100 nF$
$X_1 := 1/(2 pi f C) =>$
$X_2 := 1/(2 dot pi dot f dot C) =>$
$X_3 := 1/(2 times pi times f times C) =>$
$X_4 := 1/(2 * pi * f * C) =>$`,
        },
      },
      {
        example: {
          title: "A name before parentheses is a call",
          code: `${IMPORT}
#show: math

$a := 3$
$a (2 + 1) =>$`,
          expectError: true,
          caption: "a is a value, so it can't be called. `a dot (2 + 1)` is 9.",
        },
      },
    ],
    related: ["units", "constants", "functions", "operators", "reference/typst-math"],
  },
  {
    slug: "units",
    title: "Units",
    group: "guides",
    icon: "㎐",
    summary: "Values carry units: import the unit bindings you use, convert after `=>`, and let mismatched dimensions show up as errors instead of wrong numbers.",
    keywords: ["unit", "SI", "prefix", "conversion", "dimension", "kΩ", "mA", "degC", "celsius", "preferred-units", "number"],
    blocks: [
      { md: "Every value can carry a unit, and every calculation checks them. `30 mA` times `2 h` is a charge; adding volts to amperes is an error, not a silent mistake." },
      { h2: "Unit bindings" },
      { md: "Units are Typst bindings of the `units` module: import the ones you use. It has the SI base units (`m kg s A K mol cd`), the derived ones (`V Ω W J C F H S Hz N Pa T Wb`), `min h Ah Wh As Ws eV l L`, angles `rad` and `deg`, `degC`, `percent` and `permille` – each with every SI prefix: `mA`, `kΩ`, `µs`, `GHz`, `nF`.\n\n- One letter needs a `#` in math: `$2 #s$`, `$100 #Ω$`. Longer names don't: `$30 mA$`.\n- Powers, quotients and products as in any formula: `#m^2`, `#m / #s^2`, `kg #m`.\n\nEvery binding is listed in the [unit table](/docs/reference/units)." },
      {
        example: {
          title: "Units carry through",
          code: `${IMPORT}
#import units: m, s, kg, kN, mA, mAh, h
#show: math

$v := 30 #m / #s$, $t := 2 #s$
$d := v t =>$
$F := 1200 kg dot 3 #m / #s^2 =>$
$Q := 30 mA dot 2 #h =>$`,
          caption: "The result's unit comes from the calculation: m/s times s is m, kg·m/s² is shown as kN.",
        },
      },
      {
        example: {
          title: "Dimensions must match",
          code: `${IMPORT}
#import units: V, A
#show: math

$3 #V + 2 #A =>$`,
          expectError: true,
        },
      },
      { h2: "Converting after =>" },
      { md: "After `=>` (or `->`) write the unit to show the result in – any expression of units with the same dimension. Without one, solveng picks a unit of your inputs or the SI unit, with a fitting prefix." },
      {
        example: {
          title: "Same value, other units",
          code: `${IMPORT}
#import units: m, s, km, h, mm, µm, deg, rad, kW, kWh, MJ
#show: math

$v := 30 #m / #s$
$v => km/#h$
$d := 0.25 mm => µm$
$phi := 90 deg => rad$
$E := 2 kW dot 3 #h => kWh$
$E => MJ$`,
        },
      },
      { h3: "Prefixes and preferred units" },
      { md: "Computed results get an SI prefix that keeps the number readable. The option `prefixes` picks which ones: `\"all\"` (default), `none`, or a list like `(\"m\", \"k\")`. `preferred-units` names units that win for their dimension – km/h for every speed, say." },
      {
        example: {
          title: "Pick the prefixes",
          code: `${IMPORT}
#import units: V, mA, kΩ
#show: math.with(prefixes: {{p}})

$U := 3.3 #V$, $R := 4.7 kΩ$
$I := U / R =>$
$P := U I =>$`,
          controls: [{ kind: "select", name: "p", label: "prefixes", options: [`"all"`, `none`, `("k",)`], value: `"all"` }],
        },
      },
      {
        example: {
          title: "Speeds in km/h",
          code: `${IMPORT}
#import units: m, s, km, h
#show: math.with(preferred-units: ($km / #h$,))

$d := 100 #m$, $t := 9.58 #s$
$v := d / t =>$`,
        },
      },
      { h2: "Typeable spellings" },
      { md: "Not every keyboard has Ω and µ. `Omega` is Ω, the prefix `u` is µ (`uA`, `uF`; `µA` and `μA` work too), `deg` is °, `degC` is °C. Results always show the proper symbols." },
      {
        example: {
          code: `${IMPORT}
#import units: Omega, uA, uF
#show: math

$R := 220 #Omega$, $I := 15 uA$, $C := 4.7 uF$
$R I =>$, $R C =>$`,
        },
      },
      { h2: "Numbers from values" },
      { md: "Empirical formulas want a number in a given unit. Dividing by a unit gives exactly that: `U / #V` is U's number in volts, prefix-aware, so `U / mV` is the number in millivolts. `number(U, mV)` from `operations` does the same, but checks the dimension – `number(U, #A)` of a voltage is an error." },
      {
        example: {
          code: `${IMPORT}
#import units: V, mV
#import operations: number
#show: math

$U := 3.3 #V$
$U / #V =>$, $U / mV =>$, $number(U, mV) =>$`,
        },
      },
      { md: "An empirical formula is then written as it stands in the handbook: `$F(d: mm) := 3.2 (d / mm)^0.6 #N$`. Dividing by a reference value works just as well, and names the constants:" },
      {
        example: {
          title: "An empirical formula",
          code: `${IMPORT}
#import units: m, mm, N
#show: math

$F_0 := 3.2 #N$, $d_0 := 1 mm$
$F(d: mm) := F_0 dot (d / d_0)^0.6$
$F({{d}} mm) =>$
$F(0.1 #m) =>$`,
          controls: [{ kind: "slider", name: "d", label: "d", min: 1, max: 100, step: 1, value: 10, suffix: " mm" }],
          caption: "`d / d_0` is a plain number whatever unit d arrives in: 0.1 m becomes 100. Note the `dot`: `F_0 (…)` would call F₀.",
        },
      },
      { h2: "°C is a difference" },
      { md: "`degC` is a temperature *difference*: 20 °C means 20 K of difference, not a reading on the thermometer. For absolute temperatures – in `p V = n R T`, or a ratio of temperatures – use `celsius(20)` and `fahrenheit(68)` from `operations`, which give kelvin." },
      {
        example: {
          title: "Readings and differences",
          code: `${IMPORT}
#import units: degC
#import operations: celsius
#show: math

$T_1 := celsius({{t}})$, $T_2 := celsius(80)$
$T_1 =>$
$Delta""T := T_2 - T_1 => degC$
$T_2 / T_1 =>$`,
          controls: [{ kind: "slider", name: "t", label: "T₁", min: -40, max: 60, step: 5, value: 20, suffix: " °C" }],
          caption: "The ratio of two temperatures only makes sense in kelvin – which is what `celsius` gives.",
        },
      },
      {
        quiz: {
          question: "What does `$T := 20 degC$` then `$T => #K$` show?",
          code: `#import units: K, degC`,
          options: ["`293.15 K`", "`20 K`", "An error: °C can't be converted"],
          answer: 1,
          explain: "`degC` is a difference: 20 °C of difference is 20 K of difference. For the reading 20 °C as an absolute temperature, write `celsius(20)`, which is 293.15 K.",
        },
      },
      { callout: "tip", title: "Your own units", md: "bar, rpm, inches, bits, euros: [Custom units](/docs/custom-units) shows how to define them with `unit` and `base-unit`." },
    ],
    related: ["custom-units", "values", "reference/units", "operations/number", "operations/celsius", "reference/options"],
  },
  {
    slug: "custom-units",
    title: "Custom units",
    group: "guides",
    icon: "⚖",
    summary: "Define units with `unit` (from units you have) and `base-unit` (a new dimension), and say how they're shown with `display`.",
    keywords: ["base-unit", "unit", "display", "identity", "bar", "rpm", "inch", "bit", "byte", "currency"],
    blocks: [
      { md: "The `units` module covers SI. Everything else – bar, rpm, inches, bytes, euros – you define in one line, with two functions from the package." },
      { h2: "unit: from units you have" },
      { md: "`unit(expression)` makes a unit out of an expression of other units: `unit($100 kPa$)` is a bar. `display` says how it's shown – a string (set upright), content, or a function returning content." },
      {
        example: {
          title: "bar, rpm and inches",
          code: `${IMPORT}
#import units: kPa, min, cm, s
#let bar = unit($100 kPa$, display: "bar")
#let rpm = unit($1/min$, display: $"rpm"$)
#let inches = unit($2.54 #cm$, display: "in")
#show: math

$p := 3 bar$, $p => kPa$
$n := 3000 rpm$, $n => 1/#s$
$l := 12 inches => cm$`,
        },
      },
      { md: "Without `display`, a unit shows its expression: `unit($2.54 #cm$)` is shown as 2.54 cm, joined to a number by a dot – `2 #inch` is 2 · 2.54 cm. Binding names keep their spelling: a unit named `tau` isn't shown as τ. Names with `_` or `-` need a `#` in math (`$1 #fluid_ounce$`), since Typst would read a subscript or a minus." },
      {
        example: {
          title: "A unit without display",
          code: `${IMPORT}
#import units: cm
#let inch = unit($2.54 #cm$)
#show: math

$a := 3 #inch = inch$
$a => cm$`,
          caption: "`= inch` after a definition shows the value in that unit. A `#` before a longer binding name is optional – `#inch` and `inch` are the same.",
        },
      },
      { h2: "base-unit: a new dimension" },
      { md: "Some things aren't made of SI units at all: information, money, apples. `base-unit(identity)` creates a new, independent dimension. The **identity** names it; two base units with the same identity are the same dimension, even when declared in different files – so templates and documents agree." },
      {
        example: {
          title: "Bits and bytes",
          code: `${IMPORT}
#import units: s, ms
#let bit = base-unit("information", display: "bit")
#let byte = unit($8 bit$, display: "B")
#let kB = unit($1000 byte$, display: "kB")
#show: math

$r := {{r}} kB / #s$
$t := 2 kB / r => ms$
$r => bit / #s$`,
          controls: [{ kind: "slider", name: "r", label: "rate", min: 1, max: 1000, step: 1, value: 64, suffix: " kB/s" }],
        },
      },
      {
        example: {
          title: "What does the heater cost?",
          code: `${IMPORT}
#import units: kW, kWh, h
#let euro = base-unit("currency-eur", display: "€")
#let cent = unit($euro / 100$, display: "ct")
#show: math

$p := 32 cent / kWh$
$P := 2 kW$, $t := {{t}} #h$
$K := p P t => euro$`,
          controls: [{ kind: "slider", name: "t", label: "hours", min: 1, max: 24, step: 1, value: 3, suffix: " h" }],
          caption: "Money is a dimension like any other: cents per kilowatt-hour times kilowatts times hours is euros.",
        },
      },
      {
        example: {
          title: "Apples and kilograms don't add",
          code: `${IMPORT}
#import units: kg
#let apple = base-unit("apple")
#show: math

$n := 12 apple$
$n / 3 =>$
$n + 2 kg =>$`,
          expectError: true,
        },
      },
      { h2: "Identity and display" },
      { md: "Identity is what a unit *means*; display is only how it *looks*. Two base units with the same identity are interchangeable, whatever they're called or shown as:" },
      {
        example: {
          code: `${IMPORT}
#let bit = base-unit("information", display: "bit")
#let b = base-unit("information", display: "b")
#show: math

$x := 3 bit = #b$`,
        },
      },
      { callout: "note", title: "Where units hold", md: "Like every Typst binding, a unit holds where it's visible: in the file or block where it's defined, and wherever it's imported. Put your units in a template file and `#import \"units.typ\": bar, rpm`." },
    ],
    related: ["units", "functions/unit", "functions/base-unit", "reference/units"],
  },
  {
    slug: "constants",
    title: "Constants",
    group: "guides",
    icon: "π",
    summary: "`pi`, `e`, `i` and `c` come from `constants`; `constant()` defines your own, and `constant-displays` says how they're shown.",
    keywords: ["pi", "euler", "imaginary", "speed of light", "constant", "constant-displays", "substitute", "j"],
    blocks: [
      { md: "A constant is a value with an identity: π is π whatever you call the binding. solveng keeps constants exact – `sin(pi/4)` is √2/2 – and shows them by their identity." },
      { h2: "The built-in four" },
      { md: "`#import constants: pi, e, i, c` brings π, Euler's number, the imaginary unit and the speed of light. `e`, `i` and `c` are one letter, so they need a `#` in math: `#e^x`, `3 + 4 #i`, `m #c^2`. `pi` doesn't." },
      {
        example: {
          code: `${IMPORT}
#import constants: pi, e, i, c
#import units: kg
#show: math

$2 pi =>$, $#e^1 =>$
$#e^(#i pi) ->$
$E := 1 kg dot #c^2 =>$`,
          caption: "Euler's identity, exactly: e^(iπ) is −1.",
        },
      },
      {
        example: {
          title: "Without the import, π is just a letter",
          code: `${IMPORT}
#show: math

$2 pi =>$`,
          expectError: true,
          caption: "π here is a name like x – it has no value until you import `pi` from `constants`.",
        },
      },
      { h2: "Exact until you ask" },
      { md: "`->` keeps π, e and i exactly; `=>` gives numbers. Other constants (like c) are kept by name in exact results and calculated with their value: `$E := m #c^2 ->$` is m c²." },
      {
        example: {
          code: `${IMPORT}
#import constants: pi, c
#import units: kg
#import modifiers: substitute
#show: math

$sin(pi/4) ->$, $sin(pi/4) =>$
$E := m #c^2 ->$
$E -> substitute(#c)$`,
          caption: "`substitute(#c)` puts c's expression in: 299792458 m/s.",
        },
      },
      { h2: "Your own constants" },
      { md: "`constant(identity, expression)` names a value. The expression can carry units. The identity says what it is – shown by the option `constant-displays`; without a display, a constant is shown as its expression." },
      {
        example: {
          title: "Standard gravity",
          code: `${IMPORT}
#import units: m, s, kg
#let g0 = constant("standard-gravity", $9.80665 #m / #s^2$)
#show: math.with(constant-displays: ("standard-gravity": $g_0$))

$F := {{m}} kg #g0 =>$
$F ->$`,
          controls: [{ kind: "slider", name: "m", label: "mass", min: 1, max: 100, step: 1, value: 2, suffix: " kg" }],
        },
      },
      { h2: "Showing i as j" },
      { md: "Electrical engineers write j for the imaginary unit. `constant-displays` changes how a constant looks everywhere – in your input and in results, also where solveng produces i itself." },
      {
        example: {
          code: `${IMPORT}
#import constants: i
#show: math.with(constant-displays: (imaginary: $j$))

$z := 3 + 4 #i$
$z^2 ->$
$abs(z) =>$
$sqrt(-4) ->$`,
          caption: "`sqrt(-4)` is 2j, though no i was written. [Complex numbers →](/docs/complex)",
        },
      },
      {
        quiz: {
          question: "Which identities does solveng calculate with *exactly* (keeping them symbolic in `->` results and simplifying identities like e^(iπ) = −1)?",
          options: ["Every constant", "pi, euler and imaginary", "Only pi"],
          answer: 1,
          explain: "The engine knows π, e and i. Other constants, built-in c included, stay as their name in exact results and are calculated with their expression for `=>`.",
        },
      },
    ],
    related: ["values", "complex", "functions/constant", "modifiers/substitute", "reference/options"],
  },
  {
    slug: "functions",
    title: "Functions",
    group: "guides",
    icon: "ƒ",
    summary: "Define functions with `:=`, give parameters units (`f(I: mA)`), defaults (`n: 2`) or let `..` collect them – units carry through every call.",
    keywords: ["function", "parameter", "typed", "any", "default", "named", "..", "deferred", "unit propagation"],
    blocks: [
      { md: "A function is defined like a value, with its parameters in parentheses: `$f(x) := x^2 + 1$`. Its body is symbolic, so it can use names that aren't defined yet." },
      { h2: "Definitions and deferred names" },
      { md: "Names in the body that aren't parameters and aren't defined stay names – they're looked up when the function is called. Define them later and the function follows." },
      {
        example: {
          title: "A function before its coefficients",
          code: `${IMPORT}
#show: math

$f(x) := x^2 + 1$
$f(3) =>$
$g(x) := a x + b$
$g(2) ->$
$a := 3$, $b := 1$
$g(2) =>$`,
          caption: "Before a and b exist, `g(2)` is 2a + b; afterwards, 7.",
        },
      },
      { h2: "Typed parameters" },
      { md: "`f(I: mA)` says I is a current. A call must pass something with that dimension – `P(20 mA)` and `P(0.1 #A)` both work, `P(2)` is an error. A graph of the function puts I on an axis in mA. `any` accepts every unit." },
      {
        example: {
          title: "Power in a resistor",
          code: `${IMPORT}
#import units: Ω, mA, A
#show: math

$R := 220 #Ω$
$P(I: mA) := R I^2$
$P({{i}} mA) =>$, $P(0.1 #A) =>$
#graph($P(I)$, x-range: (0, 100))`,
          controls: [{ kind: "slider", name: "i", label: "I", min: 1, max: 100, step: 1, value: 20, suffix: " mA" }],
        },
      },
      {
        example: {
          title: "A missing unit",
          code: `${IMPORT}
#import units: Ω, mA
#show: math

$R := 220 #Ω$
$P(I: mA) := R I^2$
$P(2) =>$`,
          expectError: true,
        },
      },
      {
        example: {
          title: "any takes every unit",
          code: `${IMPORT}
#import units: A, m
#show: math

$h(x: any) := 2 x$
$h(3 #A) =>$, $h(3 #m) =>$, $h(5) =>$`,
        },
      },
      { h2: "Units propagate" },
      { md: "A parameter passed on to one with a unit takes that unit: with `q(x: #A)`, `f(y) := q(y)` makes y a current too." },
      {
        example: {
          code: `${IMPORT}
#import units: A
#show: math

$q(x: #A) := 2 x$
$f(y) := q(y)$
$f(3 #A) =>$`,
        },
      },
      { h2: ".. collects the rest" },
      { md: "`..` makes every other undefined name of the body a parameter, in the order they appear: `$P(..) := U^2/R$` is P(U, R)." },
      {
        example: {
          code: `${IMPORT}
#import units: V, Ω
#show: math

$P(..) := U^2 / R$
$P(10 #V, 5 #Ω) =>$`,
        },
      },
      { h2: "Named parameters with defaults" },
      { md: "A parameter with a default is passed by name, as in Typst: `f(x, n: 2)`. After the colon, units alone are a type (`I: mA`), anything else a default (`n: 2`, `R: 2 #Ω`, `a: a_0`). Defaults are taken where the function is defined and come after the other parameters." },
      {
        example: {
          title: "Distance under constant acceleration",
          code: `${IMPORT}
#import units: m, s
#show: math

$v := 2 #m / #s$, $a_0 := 0 #m / #s^2$
$s(t: #s, v_0: v, a: a_0) := v_0 t + a t^2 / 2$
$s(3 #s) =>$
$s(3 #s, a: {{a}} #m / #s^2) =>$`,
          controls: [{ kind: "slider", name: "a", label: "a", min: -2, max: 10, step: 0.5, value: 1, suffix: " m/s²" }],
          caption: "The default for a carries its unit: `a: 0` alone would make `a t^2/2` a plain number times s², which can't be added to metres.",
        },
      },
      {
        quiz: {
          question: "After `$f(x, n: 2) := x^n$`, what is `$f(3, k: 3) =>$`?",
          options: ["`27`", "`9` – k is ignored", "An error: f has no parameter k"],
          answer: 2,
          explain: "A named argument that isn't a parameter is an error, as in Typst – a typo shouldn't silently fall back to the default. `f(3, n: 3)` is 27.",
        },
      },
      { callout: "note", title: "Limits", md: "Functions can't call themselves. A value and a function share their name: defining one replaces the other." },
    ],
    related: ["values", "units", "piecewise", "graphs", "functions/any", "operators"],
  },
  {
    slug: "comparisons",
    title: "Comparisons",
    group: "guides",
    icon: "⩵",
    summary: "`=`, `<`, `!=` and friends compare inside expressions (1 for true, 0 for false); an `=` on its own is a checked statement or prose.",
    keywords: ["compare", "equal", "less than", "and", "or", "boolean", "check", "statement", "prose"],
    blocks: [
      { md: "As in Mathcad, a comparison is a value: 1 when it holds, 0 when it doesn't. That makes checks part of the calculation." },
      { h2: "Comparing in expressions" },
      { md: "`=` inside an expression compares. So do `<`, `<=`, `>`, `>=` and `!=` (shown as ≤, ≥, ≠). Units are compared properly: 2 A equals 2000 mA. Values of different dimensions are never equal – that's 0, not an error." },
      {
        example: {
          code: `${IMPORT}
#import units: A, V, mA, m, cm
#show: math

$M := 2 #A = 2000 mA =>$
$k := 2 #m = 200 cm =>$
$1 #A = 2 #V =>$, $1 #A != 2 #V =>$`,
        },
      },
      { h3: "Chains, and, or" },
      { md: "A chain like `0 < x <= 1` holds when every step does. `and` and `or` (shown ∧ and ∨) join comparisons, `and` binding tighter." },
      {
        example: {
          code: `${IMPORT}
#show: math

$x := {{x}}$
$0 < x <= 1 =>$
$x < 0 or x > 2 =>$
$x > 0 and x != 1 =>$`,
          controls: [{ kind: "slider", name: "x", label: "x", min: -2, max: 4, step: 0.25, value: 0.5 }],
        },
      },
      {
        example: {
          title: "Is the resistor big enough?",
          code: `${IMPORT}
#import units: V, Ω, mW
#show: math

$U := {{u}} #V$, $R := 470 #Ω$
$P := U^2 / R => mW$
$"ok" := P <= 250 mW =>$`,
          controls: [{ kind: "slider", name: "u", label: "U", min: 1, max: 20, step: 0.5, value: 9, suffix: " V" }],
          caption: "Drag U past about 10.8 V and ok drops to 0.",
        },
      },
      { h2: "Checked statements and prose" },
      { md: "An `=` that isn't inside a definition or a value states something:\n\n- **Every name known**: it's checked. If it doesn't hold, the worksheet reports it. A calculation that says `U = R I` and means it.\n- **A name without a value**: it's left alone, as prose – `$E = m c^2$` in a paragraph stays a formula." },
      {
        example: {
          code: `${IMPORT}
#import units: V, mA, Ω
#show: math

$U := 5 #V$, $I := 5 mA$, $R := 1000 #Ω$
$U = R I$
$P = U I$`,
          caption: "The first statement holds, so it's shown as written. The second is prose: P has no value.",
        },
      },
      {
        example: {
          title: "A statement that's wrong",
          code: `${IMPORT}
#import units: V, mA, Ω
#show: math

$U := 5 #V$, $I := 5 mA$, $R := 1000 #Ω$
$U = 2 R I$`,
          expectError: true,
        },
      },
      {
        quiz: {
          question: "After defining `a`, `b` and `c` as the sides 3, 4 and 6, you write the textbook formula `$a^2 + b^2 = c^2$` in the text. What happens?",
          options: ["Nothing, it's prose", "solveng reports that it doesn't hold", "c is redefined as 5"],
          answer: 1,
          explain: "Every name has a value, so it's a checked statement – and 9 + 16 ≠ 36. Put formulas meant as text in `#math(evaluate: false)[…]`, or use names that aren't defined.",
        },
      },
      { callout: "tip", title: "Conditions", md: "Comparisons are the conditions of [piecewise functions](/docs/piecewise): `cases(0 & x < 0, x & \"otherwise\")`. With `-> solve(x)`, an inequality like `x^2 < 1` is solved to −1 < x < 1 – see [Solving](/docs/solving)." },
    ],
    related: ["operators", "piecewise", "solving", "errors"],
  },
  {
    slug: "piecewise",
    title: "Piecewise functions",
    group: "guides",
    icon: "⌐",
    summary: "`cases` picks the first branch whose condition holds; text like \"for\" or \"otherwise\" is a label, and graphs break where the function jumps.",
    keywords: ["cases", "piecewise", "condition", "otherwise", "sonst", "branch", "step", "jump"],
    blocks: [
      { md: "Typst's `cases` writes a function with branches; solveng calculates it. Each row is a value, `&`, and a condition. The first branch whose condition holds is taken – only that one is evaluated." },
      {
        example: {
          title: "A ramp that saturates",
          code: `${IMPORT}
#show: math

$f(x) := cases(0 & "for" x < 0, x & "for" 0 <= x < 1, 1 & "otherwise")$
$f(-2) =>$, $f(0.5) =>$, $f(3) =>$
#graph($f(x)$)`,
        },
      },
      { h2: "Labels" },
      { md: "Quoted text in a condition is a **label**, not a name: `\"for\"`, `\"if\"`, `\"für\"`. A row with only a label – `\"otherwise\"`, `\"else\"`, `\"sonst\"` – is the default branch. Without a default and with no matching branch, the value is undefined. Write the labels your document's language uses." },
      {
        example: {
          title: "Auf Deutsch",
          code: `#set text(lang: "de")
${IMPORT}
#show: math

$f(x) := cases(0 & "für" x < 0, x^2 & "sonst")$
$f(-1) =>$, $f(1.5) =>$`,
        },
      },
      { h2: "Units and engineering" },
      { md: "Branches can carry units, and conditions compare values with units. A plain `0` fits any branch. A diode-like clipper:" },
      {
        example: {
          title: "Ideal rectifier with threshold",
          code: `${IMPORT}
#import units: V
#show: math

$U_"th" := {{u}} #V$
$y(x: #V) := cases(0 & "for" x < U_"th", x - U_"th" & "otherwise")$
$y(3 #V) =>$
#graph($y(x)$, x-range: ($-1 #V$, $5 #V$))`,
          controls: [{ kind: "slider", name: "u", label: "U_th", min: 0, max: 3, step: 0.1, value: 0.7, suffix: " V" }],
        },
      },
      { h2: "Graphs of piecewise functions" },
      { md: "Where a branch changes with a jump, the graph breaks its line. A value that holds at a single x – `2 & \"for\" x = 1` – is drawn as a dot. `joined: true` joins the pieces instead." },
      {
        example: {
          title: "Jumps and a lone point",
          code: `${IMPORT}
#show: math

$f(x) := cases(1 & "for" x < 0, 2 & "for" x = 1, x & "otherwise")$
#graph($f(x)$, x-range: (-2, 3), joined: {{j}})`,
          controls: [{ kind: "toggle", name: "j", label: "joined", value: false, on: "true", off: "false" }],
        },
      },
      {
        quiz: {
          question: "What is `f(1)` for `$f(x) := cases(1 & x > 0, 2 & x > 0.5)$`?",
          options: ["`1`", "`2`", "Undefined: two branches match"],
          answer: 0,
          explain: "The first branch whose condition holds is taken. x = 1 is > 0, so f(1) is 1; the second branch is never reached for x > 0.",
        },
      },
      { callout: "tip", title: "Signals", md: "For steps and pulses, `rect`, `tri` and `sgn` from `operations` are often shorter than `cases`, and integrate exactly. See [Signals](/docs/signals)." },
    ],
    related: ["comparisons", "functions", "graphs", "signals"],
  },
  {
    slug: "vectors",
    title: "Vectors and ranges",
    group: "guides",
    icon: "⃗",
    summary: "`vec` writes a vector, `0..N` a range variable, `x[i]` an element – and `x[i] := …` defines a whole vector at once, as in Mathcad.",
    keywords: ["vector", "vec", "range", "range variable", "index", "element", "at", "slice", "subscript", "data"],
    blocks: [
      { md: "A vector is a column of values, `vec(3, 4)`. Its elements can carry units. Arithmetic works element by element where that makes sense, and `norm`, `dot` and `times` are the length, dot and cross product." },
      {
        example: {
          code: `${IMPORT}
#import units: V
#show: math

$v := vec(3, 4)$
$norm(v) =>$, $2 v =>$, $v + vec(1, 1) =>$
$U := vec(1, 2, 3) #V =>$
$vec(1, 0, 0) times vec(0, 1, 0) =>$`,
        },
      },
      { h2: "Elements" },
      { md: "`x[i]` is an element, counted from 0, written right after the name without a space. `at(x, i)` from `operations` is the same as a function. A vector of indices selects several elements." },
      {
        example: {
          code: `${IMPORT}
#import units: V
#import operations: at
#show: math

$U := vec(1.5, 3.3, 5, 12) #V$
$U[{{i}}] =>$, $at(U, 0) =>$
$U[vec(0, 2)] =>$`,
          controls: [{ kind: "slider", name: "i", label: "index", min: 0, max: 3, step: 1, value: 1 }],
        },
      },
      { callout: "warning", title: "a[b + c] is an element", md: "In math an index follows its name directly, so `a[b + c]` is an element of a. For a product, write `a dot [b + c]` or `a (b + c)`." },
      { h2: "Range variables" },
      { md: "`$i := 0..N$` is a range variable: the values from 0 to N, both included, in steps of 1. Used as an index, it defines a whole vector at once: `$x[i] := …$` calculates an element for each value of i. Elements not defined yet are 0; defining one past the end extends the vector." },
      {
        example: {
          title: "Sampling a triangle pulse",
          code: `${IMPORT}
#import operations: tri
#show: math

$N := {{n}}$
$i := 0..N$
$t[i] := 4 i / N - 2$
$x[i] := tri(t[i])$
#graph($x(t)$)`,
          controls: [{ kind: "slider", name: "n", label: "N", min: 4, max: 40, step: 2, value: 8 }],
          caption: "`#graph($x(t)$)` plots the vector x over the vector t, a dot for each sample while there are at most 30. More samples, finer pulse.",
        },
      },
      {
        example: {
          title: "Squares, plotted against their index",
          code: `${IMPORT}
#show: math

$i := 0..5$
$y[i] := i^2$
$y =>$
#graph($y$)`,
        },
      },
      { md: "For values with units, or steps other than 1, use `range(start, end, step: …)` from `operations` – like Typst's range, it leaves the end out." },
      {
        example: {
          code: `${IMPORT}
#import units: s, ms
#import operations: range
#show: math

$t := range(0 #s, 1 #s, step: {{s}} ms) =>$`,
          controls: [{ kind: "slider", name: "s", label: "step", min: 100, max: 500, step: 50, value: 250, suffix: " ms" }],
        },
      },
      { h2: "Rows and columns" },
      { md: "With two indices, `A[i, j]` is a matrix element; `:` takes a whole row or column. `A[i, :]` is row i, `A[:, j]` column j – both plottable as data." },
      {
        example: {
          title: "A multiplication table",
          code: `${IMPORT}
#show: math

$i := 0..4$, $j := 0..4$
$A[i, j] := (i + 1)(j + 1)$
$A[2, 3] =>$
$A[:, 2] =>$
#graph($A[:, {{c}}]$)`,
          controls: [{ kind: "slider", name: "c", label: "column", min: 0, max: 4, step: 1, value: 2 }],
        },
      },
      {
        quiz: {
          question: "After `$i := 0..3$`, how many values does i have?",
          options: ["3", "4", "Depends on the step"],
          answer: 1,
          explain: "A range variable includes both ends: 0, 1, 2, 3. `range(0, 3)` from `operations` leaves the end out, as Typst's range does: 0, 1, 2.",
        },
      },
    ],
    related: ["matrices", "sums", "graphs", "operations/range", "operations/at"],
  },
  {
    slug: "matrices",
    title: "Matrices",
    group: "guides",
    icon: "▦",
    summary: "Matrices multiply, invert and transpose as written; `det`, `rank` and `identity` help, and systems of equations solve for a vector of unknowns.",
    keywords: ["matrix", "mat", "determinant", "inverse", "transpose", "rank", "identity", "linear system", "stiffness"],
    blocks: [
      { md: "Typst's `mat` writes a matrix, rows separated by `;`. solveng calculates with it: `A B` multiplies, `A^(-1)` inverts, `A^T` transposes; `det(A)`, `tr(A)` need no import." },
      {
        example: {
          title: "Writing a matrix",
          code: `${IMPORT}
#show: math

$A := mat(2, 1; 1, 3)$, $b := vec(5, 10)$
$det(A) =>$, $tr(A) =>$
$x := A^(-1) b =>$`,
          caption: "Rows are separated by `;`, elements by `,`. `A^(-1) b` solves the system A x = b.",
        },
      },
      { md: "Matrices can also be defined element by element, as in Mathcad: `A[i, j] := …` with range variables, or one element at a time. That's handy when the elements follow a rule." },
      {
        example: {
          title: "Element by element",
          code: `${IMPORT}
#show: math

$A[0, 0] := 2$, $A[0, 1] := 1$
$A[1, 0] := 1$, $A[1, 1] := 3$
$det(A) =>$, $tr(A) =>$
$A^(-1) vec(5, 10) =>$`,
        },
      },
      { h2: "A spring chain" },
      { md: "Three springs of stiffness k in a row, the last mass pulled by a force F. The stiffness matrix is 2k on the diagonal and −k beside it – a rule, so `K[i, j] := cases(…)` writes it once. `K^(-1) F` gives the displacements." },
      {
        example: {
          title: "Stiffness matrix and displacements",
          code: `${IMPORT}
#import units: kN, mm
#import operations: rank, rows
#show: math

$k := {{k}} kN / mm$
$i := 0..2$, $j := 0..2$
$K[i, j] := cases(2 k & "for" i = j, -k & "for" abs(i - j) = 1, 0 & "otherwise")$
$K[1, 1] => kN / mm$, $rows(K) =>$, $rank(K) =>$
$F := vec(0, 0, 1) kN$
$u := K^(-1) F => mm$`,
          controls: [{ kind: "slider", name: "k", label: "k", min: 0.5, max: 10, step: 0.5, value: 2, suffix: " kN/mm" }],
          caption: "Without `=> kN / mm`, solveng would show K[1, 1] as 4 MJ/m² – the same dimension, but not what you meant. Name the unit when it matters.",
        },
      },
      { h2: "Products, rank and identity" },
      { md: "Products follow linear algebra: a matrix times a vector, `v^T v` a number, an index on a product picks its element. `identity(n)` and `rank(A)` come from `operations`." },
      {
        example: {
          code: `${IMPORT}
#import operations: identity, rank
#show: math

$i := 0..2$
$v[i] := i + 1$
$v^T v =>$
$identity(3) v =>$
$rank(identity(3)) =>$
$j := 0..2$
$M[i, j] := (i + 1)(j + 1)$
$rank(M) =>$`,
          caption: "M is a multiplication table: every row is a multiple of the first, so its rank is 1.",
        },
      },
      { h2: "Linear systems" },
      { md: "A system of equations is a `vec` of equations; solve it for its unknowns: `-> solve(x, y)`. A matrix equation `A vec(x, y) = b` works the same way. Unknowns are scalars – write them as names, not as one vector name." },
      {
        example: {
          title: "Mesh currents",
          code: `${IMPORT}
#import units: V, Ω
#import modifiers: solve
#show: math

$R_1 := 100 #Ω$, $R_2 := 200 #Ω$, $R_3 := {{r}} #Ω$, $U_0 := 10 #V$
$vec((R_1 + R_3) I_1 - R_3 I_2 = U_0, -R_3 I_1 + (R_2 + R_3) I_2 = 0) -> solve(I_1, I_2)$`,
          controls: [{ kind: "slider", name: "r", label: "R₃", min: 10, max: 1000, step: 10, value: 300, suffix: " Ω" }],
        },
      },
      {
        example: {
          title: "A matrix equation",
          code: `${IMPORT}
#import modifiers: solve
#show: math

$A := mat(2, 1; 1, 3)$
$A vec(x, y) = vec(5, 10) -> solve(x, y)$
$s := vec(x + y = 10, x - y = 2) -> solve(x, y)$
$s =>$`,
          caption: "A definition keeps the solutions as a vector, in the order the unknowns are named.",
        },
      },
      {
        quiz: {
          question: "Before `A` and `B` have values, what does `A B - B A ->` give?",
          options: ["`0` – names without a value are scalars, and scalars commute", "`A B - B A`", "An error"],
          answer: 0,
          explain: "A name without a value is a scalar, so A B = B A. Define matrices before multiplying them symbolically – or pass them to a function.",
        },
      },
      { callout: "tip", title: "More linear algebra", md: "[linear-solve](/docs/operations/linear-solve) for least squares, [eigenvals](/docs/operations/eigenvals), [singularvals](/docs/operations/singularvals) and [cond](/docs/operations/cond) for spectra and conditioning." },
    ],
    related: ["vectors", "solving", "operations/rank", "operations/identity", "operations/linear-solve", "operations/eigenvals"],
  },
  {
    slug: "sums",
    title: "Sums and products",
    group: "guides",
    icon: "Σ",
    summary: "`sum_(k=0)^N` sums over integer bounds, up to infinity; `prod` multiplies – exact when small, numerical when infinite.",
    keywords: ["sum", "series", "infinite", "prod", "product", "fourier", "sigma"],
    blocks: [
      { md: "Sums are written as in Typst: `sum_(k=1)^10 k^2`. The index is local and counts over whole numbers, both bounds included. Summands can carry units." },
      {
        example: {
          code: `${IMPORT}
#import units: m
#show: math

$sum_(k=1)^10 k^2 =>$
$sum_(k=1)^4 k #m =>$
$sum_(k=1)^n k ->$`,
          caption: "With a symbolic bound, `->` keeps the sum as it is.",
        },
      },
      { h2: "Parenthesize sums in the summand" },
      { md: "The sum takes what follows it up to the next `+` or `-`, as in written maths. A summand with an addition needs parentheses." },
      {
        example: {
          code: `${IMPORT}
#show: math

$sum_(k=1)^3 (k + 1) =>$
$sum_(k=1)^3 k + 1 =>$`,
        },
      },
      {
        quiz: {
          question: "What is `$sum_(k=1)^3 k + 1 =>$`?",
          options: ["9", "7", "An error"],
          answer: 1,
          explain: "It's (1 + 2 + 3) + 1 = 7: the `+ 1` is outside the sum. `sum_(k=1)^3 (k + 1)` is 2 + 3 + 4 = 9.",
        },
      },
      { h2: "Infinite sums" },
      { md: "With `infinity` as the upper bound, `->` keeps the series symbolic and `=>` evaluates it numerically – with a check that the tail shrinks and acceleration for slow series. A series that doesn't converge is an error." },
      {
        example: {
          title: "Basel problem",
          code: `${IMPORT}
#import constants: pi
#show: math

$P := sum_(k=1)^infinity 1/k^2 =>$
$P - pi^2/6 =>$
$sum_(k=0)^infinity 1/2^k =>$`,
          caption: "Euler showed the first sum is π²/6; the difference is rounding at the 15th digit.",
        },
      },
      { h2: "Sums in functions and graphs" },
      { md: "A sum inside a function is evaluated at every point a graph samples. A square wave, from the first N + 1 terms of its Fourier series:" },
      {
        example: {
          title: "Fourier series of a square wave",
          code: `${IMPORT}
#import constants: pi
#show: math

$N := {{n}}$
$s(t) := 4/pi sum_(k=0)^N sin((2k + 1) t)/(2k + 1)$
#graph($s(t)$, x-range: (0, $2 pi$))`,
          controls: [{ kind: "slider", name: "n", label: "N", min: 0, max: 30, step: 1, value: 3 }],
          caption: "More terms, sharper edges – but the overshoot at the jumps stays near 9 %: Gibbs' phenomenon.",
        },
      },
      { md: "Sums over a vector's elements use the index: `sum_(k=0)^3 x[k]`." },
      {
        example: {
          code: `${IMPORT}
#import units: V
#show: math

$U := vec(1.2, 3.4, 2.2, 0.9) #V$
$sum_(k=0)^3 U[k] =>$`,
        },
      },
      { h2: "Products" },
      { md: "`prod(expression, index, lower, upper)` from `operations` multiplies over inclusive integer bounds; an empty product is 1. `factorial(n)` is the common case." },
      {
        example: {
          code: `${IMPORT}
#import operations: prod, factorial
#show: math

$prod(k, k, 1, 5) =>$, $factorial(5) =>$
$prod(1 - 1/(4k^2), k, 1, {{n}}) =>$`,
          controls: [{ kind: "slider", name: "n", label: "terms", min: 1, max: 1000, step: 1, value: 10 }],
          caption: "Wallis' product approaches 2/π ≈ 0.6366 as the terms grow.",
        },
      },
    ],
    related: ["vectors", "signals", "calculus", "operations/prod", "operations/factorial"],
  },
];
