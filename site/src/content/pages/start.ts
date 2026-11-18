import type { Page } from "../types";

export const pages: Page[] = [
  {
    slug: "introduction",
    title: "Introduction",
    group: "start",
    icon: "✦",
    summary: "solveng calculates the math you write in Typst – with units, exact results and every step shown.",
    keywords: ["overview", "what is", "mathcad", "worksheet"],
    blocks: [
      { md: "solveng turns the equations of a Typst document into a **worksheet**: you write the math once, as you would in your notes, and the document shows the results next to it. Values carry their units, exact numbers stay exact, and equations can be solved where they stand.\n\nEverything on this site runs on solveng's real engine – the same WebAssembly plugin Typst loads – right here in your browser. Edit any example and watch it recalculate." },
      {
        example: {
          title: "A battery, in four lines",
          code: `#import "@preview/solveng:3.0.0": *
#import units: mA, mAh, h
#import modifiers: solve
#show: math

$Q_0 := {{capacity}} mAh$, $I := {{current}} mA$
$t := Q_0 / I => #h$
$Q_0/2 = I t -> solve(t)$`,
          controls: [
            { kind: "slider", name: "capacity", label: "Capacity", min: 500, max: 6000, step: 50, value: 3450, suffix: " mAh" },
            { kind: "slider", name: "current", label: "Current", min: 5, max: 500, step: 5, value: 30, suffix: " mA" },
          ],
          caption: "Drag the sliders: `:=` defines, `=>` shows the value in hours, `->` solves for half the charge.",
        },
      },
      { h2: "Three operators" },
      { md: "A worksheet needs only three new things, and `=` keeps its meaning from mathematics:" },
      { widget: "operator-cards" },
      { h2: "Why it reads like your notes" },
      { md: "- **Units are values.** `30 mA` times `2 h` is `60 mAh`; adding volts to amperes is an error, not a silent mistake. [Units →](/docs/units)\n- **Exact until you ask.** `1/3 + 1/6` is `1/2`, `sin(pi/4)` is `√2/2`; `=>` rounds, `->` keeps it symbolic. [Symbolic results →](/docs/symbolic)\n- **Solve where you stand.** `-> solve(x)` isolates a name symbolically, or numerically when everything else has a value. [Solving →](/docs/solving)\n- **Graphs from the same math.** `#graph($P(U: #V) = U^2/R$)` picks ranges, ticks and axis units itself. [Graphs →](/docs/graphs)" },
      {
        quiz: {
          question: "After `$x := 3$`, what does `$x^2 + 1 =>$` show?",
          options: ["`x^2 + 1 ⇒ 10`", "`x^2 + 1 → x^2 + 1`", "An error: `=>` needs a unit"],
          answer: 0,
          explain: "`=>` shows the value: x is 3, so the result is 10. A unit after `=>` is optional – without one solveng picks a fitting one.",
        },
      },
      { callout: "tip", title: "Where next", md: "New to solveng? Follow [Getting started](/docs/getting-started) to set up your first worksheet, then take the [tour](/docs/tour)." },
    ],
    related: ["getting-started", "operators", "tour"],
  },
  {
    slug: "getting-started",
    title: "Getting started",
    group: "start",
    icon: "▶",
    summary: "Import solveng, switch it on with `#show: math`, import the units you use – and your document calculates.",
    keywords: ["install", "import", "setup", "first worksheet", "show rule", "hash", "#"],
    blocks: [
      { h2: "Install" },
      { md: "There is nothing to install. solveng is a Typst package on Typst Universe: the first time a document imports it, Typst downloads it – in the web app and with the `typst` command line alike (Typst 0.14 or newer). One line brings it in:" },
      { code: `#import "@preview/solveng:3.0.0": *`, lang: "typ" },
      { md: "That line brings the functions you call in code – `math`, `mathscope`, `graph`, `defined`, `value`, `clear`, `compile`, `base-unit`, `unit`, `constant`, `operation` – the parameter type `any`, and four modules: `units`, `constants`, `operations` and `modifiers`. It brings no units and doesn't calculate anything yet." },
      { h2: "Switch it on" },
      { md: "`#show: math` calculates every equation that follows. Without it the math is just math, as Typst always shows it. Write it after your imports, or after a template's show rule." },
      {
        example: {
          title: "The smallest worksheet",
          code: `#import "@preview/solveng:3.0.0": *
#show: math

$a := 3$, $b := 4$
$c := sqrt(a^2 + b^2) =>$`,
          caption: "`:=` defines, `=>` shows the value. Everything else stays ordinary Typst math.",
        },
      },
      { callout: "note", title: "math shadows math", md: "solveng's `math` takes the place of Typst's `math` module in your document. Typst's own functions are still there as `std.math.upright`, `std.math.op` and so on. If you'd rather keep the name, import it under another one: `#import \"@preview/solveng:3.0.0\": math as engineering-math`, then `#show: engineering-math`." },
      { h2: "Import what you use" },
      { md: "Units, constants, operations and modifiers are ordinary Typst bindings. Import the ones a document uses, from the module they live in:\n\n- `#import units: V, mA, kΩ, h` – units, with every SI prefix (`mA`, `kΩ`, `µs`). [Units →](/docs/units)\n- `#import constants: pi, e, i, c` – π, Euler's e, the imaginary unit, the speed of light. [Constants →](/docs/constants)\n- `#import operations: tri, integrate` – operations beyond Typst's `sin`, `sqrt`, `ln`, … which need no import. [Operations →](/docs/operations/tri)\n- `#import modifiers: solve, digits` – what goes after `=>` and `->`. [Modifiers →](/docs/modifiers/solve)\n\nBecause they are bindings, solveng never guesses: `A` is a variable, `#A` is the ampere. A name you didn't import is an error, not a silent variable. Imports follow Typst's scoping – they hold in the file or block where they're written." },
      {
        example: {
          title: "All four modules",
          code: `#import "@preview/solveng:3.0.0": *
#import units: mA, mAh, h
#import constants: pi
#import operations: tri
#import modifiers: digits
#show: math

$t := 3450 mAh / (30 mA) => #h, digits(3)$
$tri(1/4) =>$, $2 pi =>$`,
        },
      },
      { h2: "The # before one letter" },
      { md: "In Typst math, a single letter is a variable: `s` is s, the letter. A binding of one letter – the second `#s`, the ampere `#A`, the volt `#V`, the constant `#e` – needs a `#` in front to mean the binding. Longer names don't: `mA`, `kΩ`, `pi` are found as they are." },
      {
        example: {
          title: "A the variable, #A the ampere",
          code: `#import "@preview/solveng:3.0.0": *
#import units: A, s
#show: math

$A := 12$
$x := A dot 2 #A =>$
$t := 90 #s =>$`,
          caption: "The same letter twice, with two meanings. Take the `#` away from `#s` and solveng asks you to define s.",
        },
      },
      {
        quiz: {
          question: "After `#import units: s`, what is `$t := 2 s$`?",
          options: ["Two seconds", "2 times a variable s, which isn't defined yet", "An error at the import"],
          answer: 1,
          explain: "One letter in math is a variable. `s` there is a name like any other; `$t := 2 #s$` is two seconds. `:=` accepts the undefined s – it stays a name until it's defined – so the problem only shows when you ask for t's value.",
        },
      },
      { h2: "Your first worksheet" },
      { md: "An LED needs a series resistor. Supply voltage, the LED's forward voltage and its current give the resistor and the power it burns. Drag the supply voltage:" },
      {
        example: {
          title: "LED series resistor",
          code: `#import "@preview/solveng:3.0.0": *
#import units: V, mA, Ω, mW
#show: math

$U_0 := {{u}} #V$, $U_F := 2 #V$, $I_F := 20 mA$
$R := (U_0 - U_F) / I_F => #Ω$
$P_R := (U_0 - U_F) I_F => mW$
#defined()`,
          controls: [{ kind: "slider", name: "u", label: "Supply", min: 3, max: 24, step: 0.5, value: 5, suffix: " V" }],
          caption: "Units carry through: volts over milliamperes is ohms, volts times milliamperes is milliwatts. `#defined()` lists what the worksheet knows.",
        },
      },
      { callout: "fun", title: "Try it", md: "Set the supply to 24 V and look at P_R. A common 0.25 W resistor wouldn't survive – add `$P_R <= 250 mW =>$` to let the worksheet check it (1 is true, 0 false)." },
      { callout: "tip", title: "Editors", md: "solveng comes with editor support – completions for units, hover help, quick fixes for missing imports. See [Editors](/docs/editors). Next, learn the [operators](/docs/operators) properly, or take the [tour](/docs/tour)." },
    ],
    related: ["introduction", "operators", "tour", "units", "editors"],
  },
  {
    slug: "operators",
    title: "The operators",
    group: "start",
    icon: "≔",
    summary: "`:=` defines, `=` compares or states, `=>` shows the value and `->` shows the exact, symbolic result.",
    keywords: [":=", "=>", "->", "=", "define", "value", "symbolic", "mathcad", "checked statement"],
    blocks: [
      { md: "A worksheet is ordinary Typst math with three new operators. `=` keeps its meaning from mathematics." },
      { widget: "operator-cards" },
      { table: { head: ["write", "shows", "does"], rows: [
        ["`a := …`", "≔", "defines a name or a function"],
        ["`a = b`", "=", "compares inside an expression; on its own it states something, checked when it can be"],
        ["`… =>`", "⇒ value", "the numerical value, in a unit you can pick, with modifiers"],
        ["`… ->`", "→ result", "the exact, symbolic result; solves with `solve(x)`"],
      ] } },
      { h2: ":= defines" },
      { md: "`:=` gives a name a value or a formula. It works symbolically: names defined by then are put in right away, names that aren't stay names until they are. So you can write a formula before its parts." },
      {
        example: {
          title: "Defining before the parts exist",
          code: `#import "@preview/solveng:3.0.0": *
#import units: cm, m
#show: math

$h := 5 z$
$z := 20 cm$
$h =>$`,
          caption: "When h is defined, z is just a name. By `h =>`, z has a value, and h is 1 m.",
        },
      },
      { md: "What's put in is the value *at that point*. Defining a name again changes what comes after, not what came before – like writing a new line on paper, not like a spreadsheet." },
      {
        quiz: {
          question: "What does the last line show?",
          code: `$z := 20 cm$
$h := 5 z$
$z := 1 #m$
$h =>$`,
          options: ["`h ⇒ 1 m`", "`h ⇒ 5 m`", "An error: z is defined twice"],
          answer: 0,
          explain: "When h was defined, z was 20 cm, so h is 100 cm – shown as 1 m. The later `z := 1 #m` changes everything after it, but h was already fixed. Redefining is allowed.",
        },
      },
      { md: "Functions are defined the same way, `$f(x) := x^2 + 1$`, with typed parameters and defaults – see [Functions](/docs/functions)." },
      { h2: "= compares, states or is prose" },
      { md: "`=` means what it means in maths, and solveng reads it by where it stands:\n\n- **Inside an expression it compares**: `$M := 2 #A = 2000 mA$` is 1 (true), `$1 = 2$` is 0. `<`, `!=`, `and`, `or` work the same way. [Comparisons →](/docs/comparisons)\n- **On its own, with every name known, it's checked**: `$U = R I$` after defining all three must hold, or you get an error. A worksheet that states something wrong tells you.\n- **On its own, with an unknown name, it's prose**: `$E = m c^2$` with nothing defined is left alone, just math." },
      {
        example: {
          title: "Three readings of =",
          code: `#import "@preview/solveng:3.0.0": *
#import units: V, mA, Ω
#show: math

$U := 5 #V$, $I := 5 mA$, $R := 1000 #Ω$
$"ok" := U = R I =>$
$U = R I$
$E = m c^2$`,
          caption: "A comparison in a definition (1: true), a checked statement (it holds, so nothing to report) and prose (E, m, c have no values).",
        },
      },
      {
        example: {
          title: "A statement that doesn't hold",
          code: `#import "@preview/solveng:3.0.0": *
#import units: V, mA, Ω
#show: math

$U := 5 #V$, $I := 5 mA$, $R := 1000 #Ω$
$U = 2 R I$`,
          expectError: true,
          caption: "`=` states what's true; if it isn't, the worksheet says so.",
        },
      },
      { callout: "warning", title: "Formulas meant as text", md: "Because a statement with known names is checked, a textbook formula that happens to use defined names (`$a^2 + b^2 = c^2$` after defining a, b and c) must hold. Put such formulas in `#math(evaluate: false)[…]`." },
      { h2: "=> shows the value" },
      { md: "A trailing `=>` shows the numerical value. Every name in front of it needs a value by then. After `=>` you can write a unit to show the result in, and modifiers separated by commas: `$t => #h, digits(2)$`. Without a unit, solveng picks one from your inputs, with a fitting prefix. A trailing `=` works the same way, as in Mathcad." },
      {
        example: {
          title: "Unit and modifiers after =>",
          code: `#import "@preview/solveng:3.0.0": *
#import units: V, mV, kΩ
#import modifiers: digits, eng
#show: math

$U := 2/3 #V$
$U =>$
$U => mV$
$U => mV, digits({{d}})$
$U / (47 kΩ) => eng$`,
          controls: [{ kind: "slider", name: "d", label: "digits", min: 1, max: 15, step: 1, value: 3 }],
          caption: "The unit is any expression of units: `km/#h`, `#m / #s^2`. `digits(n)` sets significant digits, `sci` and `eng` the notation.",
        },
      },
      { h2: "-> shows the exact result" },
      { md: "A trailing `->` shows the symbolic result: defined values put in, exact numbers kept exact (fractions, roots and π stay), undefined names kept. On an equation it shows where the equation holds; with `solve(x)` it solves for x. `expand`, `factor`, `substitute` and `steps` shape the result." },
      {
        example: {
          title: "Exact, not rounded",
          code: `#import "@preview/solveng:3.0.0": *
#import constants: pi
#import modifiers: solve, factor
#show: math

$1/3 + 1/6 ->$
$sin(pi/4) ->$, $sin(pi/4) =>$
$x^3 - x -> factor$
$x^2 - 5x + 6 = 0 -> solve(x)$
$a^2 p = a p ->$`,
          caption: "`->` keeps √2/2 where `=>` rounds to 0.7071. The last line has no solve: `->` shows where the equation holds.",
        },
      },
      { md: "Solving shows solutions; it doesn't define anything. To keep them, define the name with the solve: `$x := x^2 - 5x + 6 = 0 -> solve(x)$` is a vector of both solutions. [Solving →](/docs/solving)" },
      {
        quiz: {
          question: "After `$x^2 = 4 -> solve(x)$`, what does `$x =>$` show?",
          options: ["`x ⇒ 2`", "Both solutions, −2 and 2", "An error: x isn't defined"],
          answer: 2,
          explain: "`solve` shows solutions without changing definitions, so x is still undefined. Write `$x := x^2 = 4 -> solve(x)$` to keep them.",
        },
      },
      { h2: "Which one when?" },
      { md: "- **`=>`** for numbers you'll read off: a current, a time, a resistor to buy.\n- **`->`** for formulas and exact results: what an expression simplifies to, an equation solved for a name, a derivative.\n- **`:=`** for everything you'll use again, values and formulas alike.\n- **`=`** for comparisons and statements the worksheet should check." },
    ],
    related: ["introduction", "functions", "comparisons", "symbolic", "solving", "modifiers/solve", "modifiers/digits"],
  },
  {
    slug: "tour",
    title: "A tour: the RC low-pass",
    group: "start",
    icon: "◠",
    summary: "One RC low-pass filter, start to finish: components with units, a transfer function, a design solved for R, the step response and a summary.",
    keywords: ["rc", "low-pass", "filter", "worked example", "cutoff", "time constant", "step response"],
    blocks: [
      { md: "A resistor and a capacitor make the simplest filter there is: low frequencies pass, high ones are damped. Along the way you'll meet most of what solveng does. Every step is a live worksheet – drag, edit, break things." },
      { h2: "1 · Components and units" },
      { md: "Values carry units. Multiply kiloohms by nanofarads and you get a time; `=>` shows it in the unit you ask for." },
      {
        example: {
          title: "Time constant and cutoff frequency",
          code: `#import "@preview/solveng:3.0.0": *
#import units: kΩ, nF, kHz, µs
#import constants: pi
#show: math

$R := {{r}} kΩ$, $C := {{c}} nF$
$tau := R C => µs$
$f_c := 1/(2 pi tau) => kHz$`,
          controls: [
            { kind: "slider", name: "r", label: "R", min: 1, max: 100, step: 1, value: 10, suffix: " kΩ" },
            { kind: "slider", name: "c", label: "C", min: 1, max: 100, step: 1, value: 22, suffix: " nF" },
          ],
          caption: "`2 pi tau` is implicit multiplication, as on paper. `pi` comes from `constants`.",
        },
      },
      { h2: "2 · A function with a typed parameter" },
      { md: "The filter's gain at frequency f is `1/sqrt(1 + (f/f_c)^2)`. Writing the parameter as `f: kHz` says what f is: calls must pass a frequency, and graphs put f on an axis in kHz." },
      {
        example: {
          title: "Gain over frequency",
          code: `#import "@preview/solveng:3.0.0": *
#import units: kΩ, nF, kHz
#import constants: pi
#show: math

$R := {{r}} kΩ$, $C := 22 nF$
$f_c := 1/(2 pi R C) => kHz$
$H(f: kHz) := 1/sqrt(1 + (f/f_c)^2)$
$H(f_c) =>$, $H(10 f_c) =>$
#graph($H(f)$, x-range: (0, 5), mark: $f_c$)`,
          controls: [{ kind: "slider", name: "r", label: "R", min: 2, max: 50, step: 1, value: 10, suffix: " kΩ" }],
          caption: "At the cutoff the gain is 1/√2 ≈ 0.707; a decade above it, about 0.1. Drag R and watch the knee move.",
        },
      },
      { h2: "3 · Solving: design for 1 kHz" },
      { md: "Turn it around: which R gives a 1 kHz cutoff with this capacitor? Write the equation and solve it. Defining R with the solve keeps the answer for what follows." },
      {
        example: {
          title: "Solve for the resistor",
          code: `#import "@preview/solveng:3.0.0": *
#import units: nF, kHz, kΩ
#import constants: pi
#import modifiers: solve
#show: math

$C := 22 nF$
$1/(2 pi R C) = {{f}} kHz -> solve(R)$
$R := 1/(2 pi R C) = {{f}} kHz -> solve(R)$
$R => kΩ$`,
          controls: [{ kind: "slider", name: "f", label: "Cutoff", min: 0.1, max: 10, step: 0.1, value: 1, suffix: " kHz" }],
          caption: "The first line shows the solution, the second keeps it – exactly, with π in it; `=>` rounds it for the parts list.",
        },
      },
      { h2: "4 · The step response" },
      { md: "Switch 5 V onto the filter at t = 0 and the capacitor charges: `u(t) = U_0 (1 − e^(−t/τ))`. How long until it reaches 90 %?" },
      {
        example: {
          title: "Charging curve and t₉₀",
          code: `#import "@preview/solveng:3.0.0": *
#import units: kΩ, nF, V, ms
#import constants: e
#import modifiers: solve
#show: math

$R := 10 kΩ$, $C := {{c}} nF$, $U_0 := 5 #V$
$tau := R C$
$u(t: ms) := U_0 dot (1 - #e^(-t/tau))$
$t_90 := u(t) = 0.9 U_0 -> solve(t)$
$t_90 => ms$
#graph($u(t)$, x-range: (0, $5 tau$), mark: $t_90$)`,
          controls: [{ kind: "slider", name: "c", label: "C", min: 5, max: 100, step: 5, value: 22, suffix: " nF" }],
          caption: "t₉₀ = τ ln 10 ≈ 2.3 τ, whatever R and C are – the symbolic solution shows why.",
        },
      },
      { callout: "warning", title: "U_0 (…) is a call", md: "A name directly followed by parentheses is a function call, as in `f(x)`. Write `U_0 dot (1 - …)` or `(1 - …) U_0` for a product." },
      { h2: "5 · What the worksheet knows" },
      { md: "`#defined()` lists every definition so far, values and functions, aligned at `=`. It's a good last line for a calculation you hand in." },
      {
        example: {
          title: "The summary",
          code: `#import "@preview/solveng:3.0.0": *
#import units: kΩ, nF, kHz, V
#import constants: pi
#show: math

$R := 10 kΩ$, $C := 22 nF$, $U_0 := 5 #V$
$tau := R C$
$f_c := 1/(2 pi tau)$
$H(f: kHz) := 1/sqrt(1 + (f/f_c)^2)$
#defined()`,
        },
      },
      { callout: "fun", title: "Challenge", md: "Add a second RC stage with the same R and C: `$H_2(f: kHz) := H(f)^2$`, then `#graph($H(f) \\ H_2(f)$, x-range: (0, 5))`. Where does H₂ reach 0.707? Hint: `H_2(f) = 1/sqrt(2) -> solve(f)`." },
      { md: "That's the tour. From here, the guides go deeper: [Units](/docs/units), [Functions](/docs/functions), [Solving](/docs/solving), [Graphs](/docs/graphs)." },
    ],
    related: ["units", "functions", "solving", "graphs", "functions/defined"],
  },
];
