import type { Page } from "../types";

export const pages: Page[] = [
  {
    slug: "scopes",
    title: "Settings and scopes",
    group: "guides",
    icon: "⧉",
    summary: "`math` changes how calculations look and whether they run; `mathscope` decides which definitions a block sees and keeps.",
    keywords: ["show rule", "math.with", "mathscope", "isolated", "inherit", "shared", "clear-at", "clear", "evaluate", "hidden", "variable scope", "settings"],
    blocks: [
      { md: "Two different things are easy to mix up, so solveng keeps them in two functions:\n\n- **Settings** – digits, units, notation, whether math is calculated at all – come from `math`. Settings never create or end a variable scope.\n- **Variable scopes** – which definitions a block sees, and whether its own definitions survive it – come from `mathscope`.\n\nThe document already has one variable scope, the global one. Most worksheets never need another." },
      { h2: "Turning calculations on" },
      { md: "Importing solveng changes nothing on its own. `#show: math` calculates every equation after it; `#show: math.with(digits: 3)` does the same with settings. Applying it again later only changes the settings – the definitions made so far stay, also when a template already turned calculations on." },
      {
        example: {
          title: "One show rule, then another",
          code: `#import "@preview/solveng:3.0.0": *
#import units: kΩ, V, mA
#show: math

$R := 4.7 kΩ$, $U := 5 #V$
$I := U / R => mA$

#show: math.with(digits: {{digits}})
$I => mA$`,
          controls: [{ kind: "slider", name: "digits", label: "digits of the second rule", min: 1, max: 10, step: 1, value: 7 }],
          caption: "The second show rule changes the digits; `I` is still defined, because no new scope started.",
        },
      },
      { h2: "Settings for a block: #math(…)[…]" },
      { md: "`#math(…)[…]` changes settings for its content only, and they revert when the block ends. Definitions made inside **stay in the current scope** – a `math` block is about looks, not about names.\n\nWhen settings nest, **the closest one wins, option by option**: an inner `digits` replaces the outer `digits`, but leaves the outer `notation` alone." },
      {
        example: {
          title: "Closest settings win, option by option",
          code: `#import "@preview/solveng:3.0.0": *
#show: math.with(digits: 4, notation: "{{notation}}")

$x := 1/7000 =>$
#math(digits: 8)[$y := x/7 =>$]
$y =>$`,
          controls: [{ kind: "select", name: "notation", label: "outer notation", options: ["auto", "plain", "scientific", "engineering"], value: "scientific" }],
          caption: "Inside the block `y` has 8 digits and the outer notation. After it, `y` is still defined, back at 4 digits.",
        },
      },
      {
        quiz: {
          question: "Is `y` defined after the block?",
          code: `#show: math
#math(digits: 8)[$y := 12/7 =>$]
$y =>$`,
          options: ["Yes – `#math` only changes settings", "No – every `#math` block is its own scope", "Only with `scope: \"shared\"`"],
          answer: 0,
          explain: "`math` never creates a variable scope; `y` lands in the global scope. Only `mathscope` makes a new one – and `scope:` is its argument, not `math`'s.",
        },
      },
      { h3: "evaluate: false – leave math alone" },
      { md: "A worksheet checks every `=` whose names all have values. That is great for `$U = R I$`, but a textbook formula that happens to use defined names must then hold too. `evaluate: false` leaves math exactly as written, even a single equation. Blocks inside inherit it; `evaluate: true` turns calculations back on." },
      {
        example: {
          title: "A formula as prose",
          code: `#import "@preview/solveng:3.0.0": *
#show: math

$a := 3$, $b := 4$, $c := 6$
In general, #math(evaluate: false)[$a^2 + b^2 = c^2$] only holds for right triangles.
$c =>$`,
          caption: "Without the `#math(evaluate: false)` around it, solveng would check `a² + b² = c²` with 3, 4 and 6 – and report that it doesn't hold.",
        },
      },
      { h3: "hidden: true – calculate quietly" },
      { md: "`hidden: true` calculates but doesn't show the math. Prose stays, and so do problems – a hidden mistake still gets its red note. It is the tidy way to set up values for a graph or a result in text." },
      {
        example: {
          title: "Hidden setup, visible result",
          code: `#import "@preview/solveng:3.0.0": *
#import units: Ω, V, mW
#show: math

#math(hidden: true)[$R := 220 #Ω$, $U := 3.3 #V$]
The LED resistor dissipates #value($U^2 / R => mW$).`,
        },
      },
      { h2: "Variable scopes: mathscope" },
      { md: "`#mathscope(scope: …)[…]` opens a new variable scope for its content. It inherits the settings around it; configure it with `math` inside or around it.\n\n- `\"isolated\"` (the default) starts **empty**: nothing from outside is visible, nothing leaks out.\n- `\"inherit\"` **reads** the outer definitions, but keeps its changes to itself.\n- `\"shared\"` reads them and **writes back**: its definitions remain after the block.\n\nA scope behaves the same around one equation as around a page of them." },
      {
        example: {
          title: "Three kinds of scope",
          code: `#import "@preview/solveng:3.0.0": *
#show: math

$x := 12$
#mathscope(scope: "{{scope}}")[
  $x := 99$
  $y := 2 x =>$
]
$x =>$`,
          controls: [{ kind: "select", name: "scope", label: "scope", options: ["isolated", "inherit", "shared"], value: "inherit" }],
          caption: "Only `\"shared\"` changes the outer `x`. Add `$x + 1 =>$` as the block's first line: `\"isolated\"` reports that `x` isn't defined, the others read 12.",
        },
      },
      { callout: "tip", title: "When to use which", md: "Use `\"isolated\"` for a self-contained exercise or a variant that must not see the main calculation, `\"inherit\"` for a what-if that borrows the main values, and `\"shared\"` when a template wraps content in a scope but the definitions should carry on." },
      { h2: "Starting fresh at headings: clear-at" },
      { md: "`clear-at` resets the innermost variable scope to its starting definitions at every matching heading or labelled element – handy for a document of independent exercises. Heading levels count the `=`: `==` is level 2. An inner `clear-at` replaces the outer one, and `clear-at: none` turns resets off locally." },
      {
        example: {
          title: "Each pump on its own",
          code: `#import "@preview/solveng:3.0.0": *
#import units: kW, m, s
#show: math.with(clear-at: heading.where(level: 1))

= Pump A
$P := 4 kW$, $Q := 0.02 #m^3 / #s$
$P / Q => kW #s / #m^3$

= Pump B
$P := 7.5 kW$
$P =>$`,
          caption: "At “Pump B” the scope resets: `Q` is gone, and writing `$Q =>$` there would be an error instead of silently reusing pump A's flow.",
        },
      },
      { h2: "Forgetting on purpose: clear" },
      { md: "`#clear(\"I\", $f$)` forgets the names given, `#clear(all: true)` everything defined so far in the scope. See [clear](/docs/functions/clear)." },
      {
        example: {
          code: `#import "@preview/solveng:3.0.0": *
#import units: mA, V
#show: math

$I := 20 mA$, $U := 5 #V$
#clear("I")
$U =>$
#defined()`,
          caption: "`defined` lists what's left: only `U`.",
        },
      },
      { h2: "math shadows Typst's math" },
      { md: "Importing `math` hides Typst's built-in `math` module. Native functions stay reachable as `std.math.upright`, `std.math.equation` and so on – or import solveng's under another name:" },
      { code: `#import "@preview/solveng:3.0.0": math as engineering-math, mathscope
#show: engineering-math.with(digits: 3)`, lang: "typ" },
    ],
    related: ["functions/math", "functions/mathscope", "functions/clear", "formatting", "reference/options"],
  },

  {
    slug: "formatting",
    title: "Formatting numbers",
    group: "guides",
    icon: "1.5",
    summary: "Digits, decimal separators, notation, fractions and prefixes: how results are written, set once for the document or per block.",
    keywords: ["digits", "trailing zeros", "decimal separator", "comma", "grouping", "thousands", "scientific", "engineering", "notation", "times", "fraction", "prefix", "rounding", "significant"],
    blocks: [
      { md: "Formatting options change how results are **written**, never what they are: solveng calculates exactly (or with about 15 digits) and rounds only for the page. Set them for the document with `#show: math.with(…)`, for a block with `#math(…)[…]`, or for one result with a modifier like `digits(2)` – see [Settings and scopes](/docs/scopes) for how they nest." },
      {
        example: {
          title: "The formatting playground",
          code: `#import "@preview/solveng:3.0.0": *
#import units: kΩ, nF, V
#import constants: pi
#set text(lang: "{{lang}}")
#show: math.with(digits: {{digits}}, notation: "{{notation}}", multiplication: "{{mult}}", trailing-zeros: {{zeros}}, grouping: {{grouping}})

$R := 4.7 kΩ$, $C := 22 nF$
$f := 1/(2 pi R C) =>$
$I := 5 #V / R =>$
$N := 2^24 =>$
$p := 1/7000 =>$
$q := 3/2 =>$`,
          controls: [
            { kind: "slider", name: "digits", label: "digits", min: 1, max: 15, step: 1, value: 4 },
            { kind: "select", name: "notation", label: "notation", options: ["auto", "plain", "scientific", "engineering"], value: "auto" },
            { kind: "select", name: "mult", label: "multiplication", options: ["dot", "times"], value: "dot" },
            { kind: "select", name: "lang", label: "document language", options: ["en", "de", "fr"], value: "en" },
            { kind: "toggle", name: "zeros", label: "trailing-zeros", value: false, on: "true", off: "false" },
            { kind: "toggle", name: "grouping", label: "thin-space grouping", value: false, on: "sym.space.thin", off: "none" },
          ],
          caption: "Play: with `plain`, grouping splits 16 780 000; with `de` or `fr` the decimal separator becomes a comma; `trailing-zeros` writes 1.500.",
        },
      },
      { h2: "Digits" },
      { md: "`digits` (default 4, 1…15) is the number of **significant** digits of computed values: `57.5 h`, `123500`, `0.001064`. Values you typed are shown as typed. Trailing zeros are dropped unless `trailing-zeros: true` keeps them up to `digits` (`2.500 V`) – useful in tables where columns should line up.\n\nFor a single result, the [digits](/docs/modifiers/digits) modifier goes after the unit: `$t => #h, digits(2)$`." },
      { h2: "Decimal separator and grouping" },
      { md: "You always **type** a decimal point: `$7.2 #V$`, also in a German document – a comma in math separates arguments, so `at(A, 0,1)` passes three. Results use `decimal-separator`, which by default follows the document language: `,` for de, fr, it, es, nl, ru, pl and many more, `.` otherwise. Set it to any string to override.\n\n`grouping` splits integer parts of five or more digits: `sym.space.thin` gives 12 345, `\"'\"` gives 12'345. The default `none` doesn't group." },
      {
        example: {
          title: "German results from the same input",
          code: `#import "@preview/solveng:3.0.0": *
#import units: V, mA, Ω
#set text(lang: "de")
#show: math.with(grouping: sym.space.thin)

$U := 7.2 #V$, $I := 3.3 mA$
$R := U / I => #Ω$
$n := 2^16 =>$`,
          caption: "Input with points, results with commas: 2181,8 Ω becomes 2,182 kΩ with prefixes, and 65 536 gets its thin space.",
        },
      },
      { h2: "Notation" },
      { md: "`notation: \"auto\"` writes plain numbers while the exponent stays in `notation-thresholds` – by default `(-3, 6)`, so from 0.001 up to 999 999 – and switches to scientific beyond. `\"plain\"` never switches, `\"scientific\"` always does, `\"engineering\"` uses exponents in steps of 3. `notation-thresholds: none` never switches either.\n\n`multiplication: \"times\"` writes 2 × 10⁶ instead of the default 2 · 10⁶. The modifiers [sci](/docs/modifiers/sci) and [eng](/docs/modifiers/eng) do the same for one result." },
      {
        example: {
          title: "Move the thresholds",
          code: `#import "@preview/solveng:3.0.0": *
#show: math.with(notation-thresholds: ({{low}}, {{high}}))

$a := 1234.5 dot 2 =>$
$b := 1/700 =>$
$c := 2^30 =>$`,
          controls: [
            { kind: "slider", name: "low", label: "low", min: -8, max: 0, step: 1, value: -3 },
            { kind: "slider", name: "high", label: "high", min: 1, max: 12, step: 1, value: 6 },
          ],
          caption: "A result is written plain while its exponent `e` satisfies `low ≤ e < high`.",
        },
      },
      { callout: "warning", title: "No e-notation in input", md: "Thresholds only change how results are **written**. In math, `1.12e4` is 1.12 times a variable `e4` – write `11200` or `1.12 dot 10^4`." },
      { h2: "Fractions" },
      { md: "`number-form` decides how values are written:\n\n- `\"auto\"` (default): typed values as typed, results rounded to `digits`.\n- `\"decimal\"`: everything rounded to `digits`.\n- `\"fraction\"`: results as exact fractions (`⅙ A`) where the denominator stays within `max-denominator` (default 1000), decimals beyond.\n- `\"input\"`: given values exactly as typed.\n\n`fraction-style` writes fractions `\"stacked\"` (default) or `\"slash\"` (1/2). The [fraction](/docs/modifiers/fraction) modifier does it for one result." },
      {
        example: {
          title: "How big may a denominator be?",
          code: `#import "@preview/solveng:3.0.0": *
#import constants: pi
#show: math.with(number-form: "fraction", max-denominator: {{den}}, fraction-style: "{{style}}")

$a := 1/3 + 1/6 =>$
$b := 0.4 =>$
$c := 7/360 =>$
$p := pi =>$`,
          controls: [
            { kind: "slider", name: "den", label: "max-denominator", min: 1, max: 1000, step: 1, value: 100 },
            { kind: "select", name: "style", label: "fraction-style", options: ["stacked", "slash"], value: "slash" },
          ],
          caption: "Below 360, `c` falls back to a decimal; π never becomes a fraction, because it isn't one.",
        },
      },
      { h2: "Prefixes" },
      { md: "Without a unit after `=>`, results get one: the unit you asked for, else one from `preferred-units`, else a unit of the inputs, else SI with a fitting prefix. `prefixes` limits which prefixes may be picked: `\"all\"` (a f p n µ m k M G T; times from a minute on in min or h), `none` for base units only, or an array like `(\"m\", \"k\")` (`\"u\"` is µ)." },
      {
        example: {
          title: "Allowed prefixes",
          code: `#import "@preview/solveng:3.0.0": *
#import units: kΩ, nF, m, J
#show: math.with(prefixes: {{prefixes}})

$tau := 4.7 kΩ dot 22 nF =>$
$E := 3.6 dot 10^9 #J =>$`,
          controls: [{ kind: "select", name: "prefixes", label: "prefixes", options: ["\"all\"", "none", "(\"m\", \"k\")"], value: "\"all\"" }],
          caption: "With only milli and kilo, 103.4 µs becomes 0.1034 ms; with `none`, seconds and joules in scientific notation.",
        },
      },
      { h2: "The approximation sign" },
      { md: "`show-approximation` (default `true`) is documented to write `≈` when a shown result is rounded. In solveng 2.0.0 the result marker stays ⇒ either way; the option is accepted and checked, so documents using it keep compiling." },
      {
        quiz: {
          question: "With `#show: math.with(digits: 2)`, what happens to `$x := 2/3$` and then `$3 x =>$`?",
          options: ["2 – rounding only changes how numbers are written", "2.1 – x was stored as 0.67", "1.98"],
          answer: 0,
          explain: "Options never change values: `x` stays exactly ⅔, so `3x` is exactly 2. Rounding happens only when a result is written.",
        },
      },
    ],
    related: ["reference/options", "modifiers/digits", "modifiers/sci", "modifiers/eng", "modifiers/fraction", "units", "scopes"],
  },

  {
    slug: "errors",
    title: "When something's wrong",
    group: "guides",
    icon: "⚠",
    summary: "Problems show up as red notes where they happen, with a hint how to fix them; `errors: \"panic\"` stops the compilation instead.",
    keywords: ["error", "problem", "red note", "panic", "CI", "undefined", "unit mismatch", "debug", "troubleshooting"],
    blocks: [
      { md: "A missing value, units that don't match, a real calculation that would need complex numbers, a typo: solveng shows each as a red note right where it happens – and the rest of the document still compiles. You keep writing; the note tells you what to do." },
      { h2: "Typical messages" },
      {
        example: {
          title: "Not defined yet",
          code: `#import "@preview/solveng:3.0.0": *
#import units: V, mA, W
#show: math

$I := 20 mA$
$P := U I => #W$`,
          expectError: true,
          caption: "`=>` needs every name to have a value. Define `U` first (`$U := 5 #V$` before this line), or solve for it: `$U := P = U I -> solve(U)$`.",
        },
      },
      {
        example: {
          title: "Apples and oranges",
          code: `#import "@preview/solveng:3.0.0": *
#import units: V, A
#show: math

$x := 2 #V + 3 #A =>$`,
          expectError: true,
          caption: "Values of different dimensions can't be added. Usually one of the inputs has the wrong unit.",
        },
      },
      {
        example: {
          title: "A statement that doesn't hold",
          code: `#import "@preview/solveng:3.0.0": *
#import units: V, Ω, mA
#show: math

$R := 100 #Ω$, $I := 20 mA$, $U := 3 #V$
$U = R I$`,
          expectError: true,
          caption: "`=` with only known names is checked. 100 Ω · 20 mA is 2 V, not 3 V. If you meant to define `U`, write `:=`; if it's a textbook formula, wrap it in `#math(evaluate: false)[…]`.",
        },
      },
      {
        example: {
          title: "A unit a parameter doesn't take",
          code: `#import "@preview/solveng:3.0.0": *
#import units: mA, mW
#show: math

$P(I: mA) := 5 I$
$P(3) =>$`,
          expectError: true,
          caption: "The parameter `I` takes milliamperes, so pass one: `$P(3 mA) =>$`. `any` takes every unit.",
        },
      },
      {
        example: {
          title: "Units where only numbers go",
          code: `#import "@preview/solveng:3.0.0": *
#import units: m
#show: math

$d := 3 #m$
$ln(d) =>$`,
          expectError: true,
          caption: "`ln`, `exp` and `sin` need plain numbers. Divide by a unit to get one: `$ln(d / #m) =>$`.",
        },
      },
      {
        example: {
          title: "A unit in the wrong place",
          code: `#import "@preview/solveng:3.0.0": *
#import units: mA
#show: math

$y := 5 => mA$`,
          expectError: true,
          caption: "The unit after `=>` must match the result's dimension. 5 has no unit, so ask for none – or write `5 mA`.",
        },
      },
      { md: "More messages you'll meet:\n\n- *complex numbers have no order* – `<` and `>` need real values; compare `Re(…)`, `Im(…)` or `abs(…)`.\n- *f is used in its own definition* – functions can't call themselves.\n- *after => use a unit and result modifiers, not a value* – often a one-letter unit without `#`: `=> W` names a variable W, write `=> #W`.\n- *unknown variable: e4* – that's Typst, not solveng: `1.12e4` reads as 1.12 times `e4`. Write `1.12 dot 10^4`.\n- *unknown argument 'digit' for math* – a typo in an option name; the message lists the right ones." },
      {
        quiz: {
          question: "What does `$l := 3 m$` define, after `#import units: m`?",
          options: ["3 metres", "3 times a variable `m`", "An error: `m` needs a `#`"],
          answer: 1,
          explain: "In Typst math a single letter is just a letter. Only `#m` refers to the imported binding – so this is `3 m` with `m` a variable, and no error appears until you need its value. Write `$l := 3 #m$`.",
        },
      },
      { h2: "Stopping the compilation: errors: \"panic\"" },
      { md: "Red notes are friendly while you write, but in CI a broken calculation should fail the build. `errors: \"panic\"` stops the compilation at the first problem, with the same message:" },
      { code: `#show: math.with(errors: "panic")`, lang: "typ" },
      { md: "Set it in a block with `#math(errors: \"panic\")[…]` to be strict only there. Unknown named arguments of `mathscope`, `defined` and `graph` are reported like other problems; those of `compile` always stop the compilation." },
      { callout: "tip", title: "Fix the first note first", md: "One undefined name causes a note in every line that uses it. Scroll up to the first red note, fix it, and most of the others disappear." },
    ],
    related: ["reference/options", "scopes", "units", "functions/math", "limits"],
  },

  {
    slug: "html",
    title: "HTML export",
    group: "guides",
    icon: "</>",
    summary: "With Typst's experimental HTML export, worksheets become MathML, graphs live SVGs and errors styled notes that follow light and dark mode.",
    keywords: ["html", "mathml", "web", "export", "stylesheet", "css", "dark mode", "svg", "browser"],
    blocks: [
      { md: "Typst can export HTML (experimental): `typst compile --features html --format html doc.typ`. solveng comes along without changes to your document:" },
      { code: "typst compile --features html --format html report.typ", lang: "sh" },
      { md: "- **Equations** become MathML, which every modern browser renders.\n- **`defined`** lists become columns that reflow with the page width, or a table with `style: \"column\"`.\n- **Graphs** become inline SVGs (via `html.frame`) that shrink with the page, and are drawn live in the page's colours.\n- **Errors** become styled notes." },
      { h2: "The stylesheet" },
      { md: "solveng adds one small stylesheet: the math font (STIX Two Math, Cambria Math or Latin Modern Math), spacing and table stripes. Style or override these classes:" },
      {
        table: {
          head: ["class", "what"],
          rows: [
            ["`solveng`", "every calculated equation"],
            ["`solveng-defs`", "a `defined` list in columns"],
            ["`solveng-list`", "a `defined` list as a list"],
            ["`solveng-plot`", "a graph"],
            ["`solveng-error`", "a problem's note"],
            ["`solveng-chip`, `solveng-type`", "a type chip, like the `mA` of `f(I: mA)`"],
          ],
        },
      },
      { h2: "Light and dark" },
      { md: "The page follows the reader's colour scheme: type chips and errors take their dark colours, and graphs are drawn with the page's `Canvas` and `currentColor`, so axes and labels stay readable in both. Give chips your own dark colours with the option `chips: (type: (fill: …, mark: …, ink: …, dark: (fill: …, mark: …, ink: …)))`. Without scripts, the static picture of a graph shows instead." },
      {
        example: {
          title: "This site is HTML too",
          code: `#import "@preview/solveng:3.0.0": *
#import units: V, Ω, W
#show: math

#math(hidden: true)[$R := {{R}} #Ω$, $P_0 := 2 #W$]
#graph($P(U: #V) = U^2/R$, ymark: $P_0$)`,
          controls: [{ kind: "slider", name: "R", label: "R", min: 10, max: 200, step: 5, value: 50, suffix: " Ω" }],
          caption: "Everything on this site – MathML equations, live graphs – is what a solveng worksheet looks like on the web. Switch your system to dark mode and the graph follows.",
        },
      },
      { callout: "note", title: "Experimental in Typst", md: "HTML export is still experimental in Typst and may change. In column layout browsers drop the space after a leading `=`." },
    ],
    related: ["graphs", "functions/graph", "functions/defined", "reference/options"],
  },

  {
    slug: "editors",
    title: "Editor support",
    group: "guides",
    icon: "✎",
    summary: "solveng's language server adds completions, highlighting, hovers, navigation and import quick fixes for calculations, next to tinymist.",
    keywords: ["lsp", "language server", "vscode", "neovim", "tinymist", "completion", "highlighting", "quick fix", "hover", "rename"],
    blocks: [
      { md: "Typst's language server, tinymist, knows Typst – but not that `:=` defines a variable or that `#mA` is a unit. **solveng-lsp** runs next to it and answers only where solveng reads the document: inside calculated math and the calls of solveng's functions." },
      { h2: "What it does" },
      { md: "- **Completions** for worksheet names, functions, constants and units in math, the modifiers after `=>` and `->`, and the options of solveng's functions.\n- **Highlighting**: definitions, units, constants, parameters and the result markers, so a theme can show a `=>` in bold and units like enum members.\n- **Hovers** with what a name depends on, and the parameter help of operations.\n- **Navigation**: go to definition, find usages, rename – following `mathscope` scopes, `clear-at`, `clear` and redefinitions as the compile does.\n- **Quick fixes**: *Import mA from solveng's units* for a unit that isn't imported yet, *Use #s to reference the Typst binding* for a one-letter unit written without `#`, and shorthands for long spellings.\n- **Formatting** of the whitespace in calculations, leaving the surrounding Typst alone." },
      { h2: "It follows your templates" },
      { md: "Completions and import quick fixes follow bindings that local templates re-export, including renamed imports. Highlighting also recognizes template functions that forward their content into `math` or `mathscope` blocks or apply a `math` show rule – so a worksheet inside `#report[…]` is highlighted like one at the top level." },
      { code: `// template.typ
#import "@preview/solveng:3.0.0": *
#import units: mA, V
#let report(body) = { show: math.with(digits: 3); body }

// main.typ – completions offer mA and V, and highlight the math
#import "template.typ": *
#show: report
$I := 20 mA$`, lang: "typ" },
      { md: "Layouts that are conditional or computed at compile time can't be read from the source alone. There the editor uses what the compiler reported: a client that compiles sends the `<solveng>` metadata (`query(<solveng>)`) with the notification `solveng/report`, and the errors solveng showed in the document become diagnostics, and computed values appear in completions." },
      { h2: "Installing" },
      { md: "The language server lives in the repository's `editor/lsp` crate. Build it with Cargo and register `solveng-lsp` in your editor as an additional server for Typst files, next to tinymist:" },
      { code: "cargo install --path editor/lsp\nsolveng-lsp --version", lang: "sh" },
      { callout: "tip", title: "No setup needed to calculate", md: "The editor support only helps you write. Compiling needs nothing but the package import – the calculations run in Typst itself." },
    ],
    related: ["getting-started", "reference/typst-math", "units", "reference/cheatsheet"],
  },

  {
    slug: "how-it-solves",
    title: "How it solves",
    group: "guides",
    icon: "⚙",
    summary: "Symbolically first – isolate, then formulas for polynomials, then elimination – and numerically once every other name has a value.",
    keywords: ["solver", "algorithm", "symbolic", "numeric", "newton", "bisection", "polynomial", "system", "elimination", "rust", "wasm"],
    blocks: [
      { md: "solveng calculates in a WebAssembly plugin written in Rust – the same one running on this page. When you write `-> solve(x)`, it tries strategies from exact to numerical, and stops at the first that works. Other names are parameters: they're kept symbolic until they have values." },
      { h2: "1. Isolate" },
      { md: "An unknown that occurs **once** is isolated by undoing what's around it, step by step. Units of the unknown are inferred from the equation, and mismatched units stop with an error." },
      {
        example: {
          code: `#import "@preview/solveng:3.0.0": *
#import units: V, W
#import modifiers: solve
#show: math

$P = U^2 / R -> solve(R)$
$U := {{U}} #V$, $P := 3 #W$
$P = U I -> solve(I)$
$P = U^2 / R -> solve(R)$`,
          controls: [{ kind: "slider", name: "U", label: "U", min: 1, max: 48, step: 1, value: 12, suffix: " V" }],
          caption: "First as a formula, then with values: the current comes out in A or mA, the resistance in Ω – inferred from the units of P and U.",
        },
      },
      { h2: "2. Polynomials" },
      { md: "When the unknown occurs several times, solveng brings both sides over one denominator. A **polynomial** in it is solved by formula up to degree 2, and by numerical polynomial roots above. Polynomial equations are solved in the complex numbers, so you get every root. A higher-degree solution with symbolic coefficients waits until they have values." },
      {
        example: {
          code: `#import "@preview/solveng:3.0.0": *
#import modifiers: solve
#show: math

$a x^2 + b x + c = 0 -> solve(x)$
$x^3 - 2 x - 5 = 0 -> solve(x)$`,
          caption: "The quadratic formula, symbolically – and Wallis's cubic numerically, with its complex pair.",
        },
      },
      { h2: "3. Systems" },
      { md: "Several equations, separated by commas, with several unknowns: solved **one equation at a time** where an equation has a single unknown, and by **elimination** where the system is linear. Everything else goes to the numerical solver." },
      {
        example: {
          code: `#import "@preview/solveng:3.0.0": *
#import modifiers: solve
#show: math

$a + b = 3, a - b = 1 -> solve(a, b)$
$x y = 2, x + y = 3 -> solve(x, y)$`,
          caption: "The second system has two solutions, and you get both.",
        },
      },
      { h2: "4. Numerically" },
      { md: "What can't be done symbolically is solved numerically, once every other name has a value. For one equation, solveng looks for **sign changes** over ±10⁻¹⁵…10¹⁵ (times the target unit) and bisects them, and starts **Newton's method** from every local minimum of the absolute value – which catches roots that touch zero without crossing it. Systems use damped Newton from 41 starts. The numerical search finds real roots." },
      {
        example: {
          code: `#import "@preview/solveng:3.0.0": *
#import constants: e
#import modifiers: solve
#show: math

$cos(x) = x -> solve(x)$
$#e^x = {{k}} x -> solve(x)$`,
          controls: [{ kind: "slider", name: "k", label: "k", min: 3, max: 10, step: 0.5, value: 3 }],
          caption: "eˣ = kx has two real solutions for every k > e; drag k and watch them move apart.",
        },
      },
      { h2: "Simplifying" },
      { md: "Results are simplified on the way: like terms combined, fractions cancelled, exact numbers kept exact. Equations also try a bounded **exact proof** in a common exponential form – `sin(x)^2 + cos(x)^2 = 1` is true – but numerical samples never count as proof." },
      {
        example: {
          code: `#import "@preview/solveng:3.0.0": *
#show: math

$(x^2 - 1)/(x - 1) ->$
$sin(x)^2 + cos(x)^2 = 1 ->$`,
        },
      },
      { h2: "Inequalities" },
      { md: "Real polynomial and rational inequalities are solved with a sign chart at every real root and pole, keeping open and closed ends and separate pieces: `x^2 < 1` gives −1 < x < 1, `x^2 > 1` gives x < −1 or x > 1." },
      {
        example: {
          code: `#import "@preview/solveng:3.0.0": *
#import modifiers: solve
#show: math

$x^2 {{op}} 1 -> solve(x)$`,
          controls: [{ kind: "select", name: "op", label: "relation", options: ["<", "<=", ">", ">="], value: "<" }],
        },
      },
      { h2: "Keeping the solutions" },
      { md: "`-> solve(x)` shows solutions and defines nothing. A **definition** keeps them: one solution gives a value, several a vector, and `at(…, 0)` picks the first. The solve variable is local, so a function can return the solutions for its parameters. Solved definitions need a symbolic solution." },
      {
        example: {
          code: `#import "@preview/solveng:3.0.0": *
#import operations: at
#import modifiers: solve
#show: math

$y(x, r) := x^2 + b^2 = r^2 -> solve(b)$
$y(3, 5) =>$
$at(y(3, 5), 1) =>$`,
          caption: "Both branches of the circle; `at` counts from 0.",
        },
      },
      {
        quiz: {
          question: "After `$x^2 = 4 -> solve(x)$`, what is `$x + 1 =>$`?",
          options: ["An error: x isn't defined", "3", "A vector (3, −1)"],
          answer: 0,
          explain: "`-> solve(x)` only shows solutions. To keep them, define: `$x := x^2 = 4 -> solve(x)$`, then `x` is the vector of both.",
        },
      },
      { md: "What the strategies can't do is collected in [Limits](/docs/limits)." },
    ],
    related: ["solving", "modifiers/solve", "limits", "symbolic", "numerics"],
  },

  {
    slug: "limits",
    title: "Limits",
    group: "guides",
    icon: "∞",
    summary: "What solveng can't do (yet): number ranges, roots it may miss, units it doesn't take, and math that gets checked when you didn't mean it.",
    keywords: ["limitations", "precision", "float", "overflow", "missed root", "known issues", "caveats"],
    blocks: [
      { md: "Every calculator has edges. These are solveng's – knowing them saves you a surprised look at a result." },
      { h2: "Numbers" },
      { md: "Numbers are **exact fractions** as long as numerator and denominator fit in 128 bits, then 64-bit floats: about 15 significant digits, magnitudes up to 10³⁰⁰. Values are complex numbers with the real value where there is one: `sqrt(-1)` is i, `(-8)^(1/3)` is −2." },
      {
        example: {
          code: `#import "@preview/solveng:3.0.0": *
#show: math

$1/3 + 1/6 ->$
$(-8)^(1/3) =>$
$sqrt(-4) =>$`,
        },
      },
      { h2: "Solving" },
      { md: "- **Isolating** gives the principal branch: `sqrt(x) = 2` gives only 4, `tan(x) = 1` only π/4 – not every multiple of π. Polynomials like `x^2 = 4` give all their roots.\n- **Symbolic formulas** for polynomials go up to degree 2; higher degrees with parameters stay `polyroots(…)` until their coefficients have values.\n- **The numerical search** can miss roots squeezed between nearby poles, roots of very fast oscillations, and roots outside ±10¹⁵ (scaled by the target unit). The result then says there's no solution.\n- **Systems** are solved numerically from a fixed set of starts; solutions far from them can be missed, and underdetermined systems show one arbitrary solution.\n- **Polynomial roots** near repeated or clustered roots can lose accuracy." },
      {
        example: {
          title: "Periodic, but one answer",
          code: `#import "@preview/solveng:3.0.0": *
#import modifiers: solve
#show: math

$tan(x) = 1 -> solve(x)$
$sqrt(x) = 2 -> solve(x)$
$x^4 + a x = 1 -> solve(x)$`,
          caption: "π/4 is one of infinitely many solutions. The quartic stays numeric roots, waiting for `a`.",
        },
      },
      { h2: "Real only" },
      { md: "Graphs, `compile`, comparisons with `<` and `>`, inequalities and the numerical search need real values. Complex numbers have no order, so `<` on them is an error." },
      { h2: "Units" },
      { md: "- **°C** (`degC`) is a temperature *difference*. For absolute readings use `celsius(20)` or `fahrenheit(68)`, which give kelvin – that's what `p V = n R T` needs.\n- **Unit expressions** take whole-number powers (`#m^2`, `#s^(-1)`); `#m^(1/2)` isn't a unit." },
      {
        example: {
          code: `#import "@preview/solveng:3.0.0": *
#import units: degC, K
#import operations: celsius
#show: math

$T := celsius(20) =>$
$Delta""T := 5 degC => #K$`,
          caption: "20 °C as a reading is 293.15 K; a difference of 5 °C is 5 K.",
        },
      },
      { h2: "Functions, ranges and matrices" },
      { md: "- **Functions** can't call themselves.\n- **Ranges** have at most 100 000 values, and their bounds and steps need values when they're defined.\n- **Indices** follow their name directly: `a[b + c]` is an element of `a`. Write `a dot [b + c]` or `a (b + c)` for a product.\n- **Names without a value are scalars**, so they commute in products. Define matrices before multiplying them, and write a vector of unknowns as `vec(x, y)`.\n- **Symbolic elimination** treats an entry as zero only when it simplifies to 0; floats count as zero within 10⁻¹³ of the largest element (times the size).\n- **Convolution** of functions is exact for piecewise polynomials with numeric breakpoints; exponentials, trigonometric functions and symbolic shifts stay an integral, evaluated numerically.\n- **Numerical operations** allow matrices up to 256 × 256, DFTs of 2048 samples and 100 000 ODE steps; they're estimates, not proofs." },
      { h2: "Math that gets checked" },
      { md: "In a worksheet, an equation with `=` and only known names is **checked**. A formula meant as plain text that happens to use defined names must then hold – or you get a red note. Put such formulas outside the worksheet, or in `#math(evaluate: false)[…]`." },
      {
        quiz: {
          question: "After `$a := 3$`, `$b := 4$`, `$c := 6$`, what does the sentence *“Pythagoras: $a^2 + b^2 = c^2$”* produce?",
          options: ["A red note: 25 isn't 36", "Just the formula, as prose", "It redefines c as 5"],
          answer: 0,
          explain: "Every name has a value, so the equation is checked – and fails. Wrap it in `#math(evaluate: false)[…]`, or use names that aren't defined.",
        },
      },
    ],
    related: ["how-it-solves", "errors", "complex", "units", "scopes"],
  },
];
