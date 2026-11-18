// Typst's math symbols (sym), the ones engineering math uses, with their variants.

const GREEK: [string, string][] = [
  ["alpha", "α"], ["beta", "β"], ["gamma", "γ"], ["delta", "δ"], ["epsilon", "ε"], ["epsilon.alt", "ϵ"], ["zeta", "ζ"], ["eta", "η"],
  ["theta", "θ"], ["theta.alt", "ϑ"], ["iota", "ι"], ["kappa", "κ"], ["kappa.alt", "ϰ"], ["lambda", "λ"], ["mu", "μ"], ["nu", "ν"], ["xi", "ξ"],
  ["omicron", "ο"], ["pi", "π"], ["pi.alt", "ϖ"], ["rho", "ρ"], ["rho.alt", "ϱ"], ["sigma", "σ"], ["sigma.alt", "ς"], ["tau", "τ"],
  ["upsilon", "υ"], ["phi", "φ"], ["phi.alt", "ϕ"], ["chi", "χ"], ["psi", "ψ"], ["omega", "ω"],
  ["Alpha", "Α"], ["Beta", "Β"], ["Gamma", "Γ"], ["Delta", "Δ"], ["Epsilon", "Ε"], ["Zeta", "Ζ"], ["Eta", "Η"], ["Theta", "Θ"],
  ["Iota", "Ι"], ["Kappa", "Κ"], ["Lambda", "Λ"], ["Mu", "Μ"], ["Nu", "Ν"], ["Xi", "Ξ"], ["Omicron", "Ο"], ["Pi", "Π"], ["Rho", "Ρ"],
  ["Sigma", "Σ"], ["Tau", "Τ"], ["Upsilon", "Υ"], ["Phi", "Φ"], ["Chi", "Χ"], ["Psi", "Ψ"], ["Omega", "Ω"],
];

const OTHER: [string, string][] = [
  ["dot", "⋅"], ["dot.op", "⋅"], ["dot.c", "·"], ["dots", "…"], ["dots.h", "…"], ["dots.c", "⋯"], ["dots.v", "⋮"], ["dots.down", "⋱"],
  ["times", "×"], ["times.big", "⨉"], ["div", "÷"], ["plus", "+"], ["plus.minus", "±"], ["minus", "−"], ["minus.plus", "∓"],
  ["ast", "∗"], ["star", "⋆"], ["compose", "∘"], ["circle.small", "∘"], ["convolve", "∗"],
  ["eq", "="], ["eq.not", "≠"], ["eq.def", "≝"], ["eq.quest", "≟"], ["equiv", "≡"], ["approx", "≈"], ["tilde.op", "∼"], ["prop", "∝"],
  ["lt", "<"], ["gt", ">"], ["lt.eq", "≤"], ["gt.eq", "≥"], ["lt.double", "≪"], ["gt.double", "≫"], ["lt.eq.slant", "⩽"], ["gt.eq.slant", "⩾"],
  ["colon", ":"], ["colon.eq", "≔"], ["eq.colon", "≕"], ["comma", ","], ["semi", ";"], ["excl", "!"], ["quest", "?"],
  ["infinity", "∞"], ["oo", "∞"], ["partial", "∂"], ["nabla", "∇"], ["diff", "∂"],
  ["sum", "∑"], ["product", "∏"], ["product.co", "∐"], ["integral", "∫"], ["integral.double", "∬"], ["integral.triple", "∭"], ["integral.cont", "∮"],
  ["arrow.r", "→"], ["arrow.l", "←"], ["arrow.l.r", "↔"], ["arrow.r.double", "⇒"], ["arrow.l.double", "⇐"], ["arrow.l.r.double", "⇔"],
  ["arrow.r.long", "⟶"], ["arrow.r.bar", "↦"], ["arrow.t", "↑"], ["arrow.b", "↓"], ["arrow.r.squiggly", "⇝"],
  ["in", "∈"], ["in.not", "∉"], ["in.rev", "∋"], ["subset", "⊂"], ["subset.eq", "⊆"], ["supset", "⊃"], ["supset.eq", "⊇"],
  ["union", "∪"], ["sect", "∩"], ["without", "∖"], ["emptyset", "∅"], ["nothing", "∅"],
  ["and", "∧"], ["or", "∨"], ["not", "¬"], ["forall", "∀"], ["exists", "∃"], ["exists.not", "∄"], ["top", "⊤"], ["bot", "⊥"],
  ["RR", "ℝ"], ["NN", "ℕ"], ["ZZ", "ℤ"], ["QQ", "ℚ"], ["CC", "ℂ"], ["PP", "ℙ"],
  ["degree", "°"], ["prime", "′"], ["prime.double", "″"], ["percent", "%"], ["permille", "‰"], ["ell", "ℓ"], ["planck", "ℎ"], ["planck.reduce", "ℏ"], ["hbar", "ℏ"],
  ["angle", "∠"], ["perp", "⟂"], ["parallel", "∥"], ["bar.v", "|"], ["bar.v.double", "‖"], ["tilde", "∼"], ["hat", "^"],
  ["paren.l", "("], ["paren.r", ")"], ["bracket.l", "["], ["bracket.r", "]"], ["brace.l", "{"], ["brace.r", "}"],
  ["angle.l", "⟨"], ["angle.r", "⟩"], ["ceil.l", "⌈"], ["ceil.r", "⌉"], ["floor.l", "⌊"], ["floor.r", "⌋"],
  ["at", "@"], ["amp", "&"], ["ohm", "Ω"], ["micro", "µ"], ["euro", "€"], ["dollar", "$"], ["hash", "#"], ["slash", "/"], ["backslash", "\\"],
  ["checkmark", "✓"], ["triangle", "△"], ["triangle.stroked.t", "△"], ["square", "□"], ["circle", "○"], ["bullet", "•"],
  ["therefore", "∴"], ["because", "∵"], ["qed", "∎"], ["aleph", "ℵ"], ["Re", "ℜ"], ["Im", "ℑ"],
];

export const SYMBOLS: Map<string, string> = new Map([...GREEK, ...OTHER]);

/** Spacing in math, in em. */
export const SPACES: Record<string, number> = {
  thin: 1 / 6, med: 2 / 9, thick: 5 / 18, quad: 1, wide: 2, "space": 0.25, "space.thin": 1 / 6, "space.quad": 1, "space.nobreak": 0.25, "space.en": 0.5, "space.hair": 1 / 12,
};

/** Typst's text operators in math (math.op). */
export const OPERATORS = new Set([
  "arccos", "arcsin", "arctan", "arg", "cos", "cosh", "cot", "coth", "csc", "csch", "ctg", "deg", "det", "dim", "exp", "gcd", "lcm",
  "hom", "id", "im", "inf", "ker", "lg", "lim", "liminf", "limsup", "ln", "log", "max", "min", "mod", "Pr", "sec", "sech", "sin", "sinc",
  "sinh", "sup", "tan", "tanh", "tg", "tr",
]);

export const ACCENTS: Record<string, string> = {
  hat: "^", tilde: "~", bar: "¯", macron: "¯", dot: "˙", dot_double: "¨", ddot: "¨", dddot: "⃛", acute: "´", grave: "`", breve: "˘",
  caron: "ˇ", circle: "˚", vec: "→", arrow: "→", overline: "‾",
};

export const COLORS: Record<string, string> = {
  black: "#000000", gray: "#aaaaaa", silver: "#dddddd", white: "#ffffff", navy: "#001f3f", blue: "#0074d9", aqua: "#7fdbff",
  teal: "#39cccc", eastern: "#239dad", purple: "#b10dc9", fuchsia: "#f012be", maroon: "#85144b", red: "#ff4136", orange: "#ff851b",
  yellow: "#ffdc00", olive: "#3d9970", green: "#2ecc40", lime: "#01ff70",
};
