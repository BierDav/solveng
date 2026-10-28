//! Reading equations as the Typst side writes math out: operators, numbers and names; what Typst bindings stand
//! for as explicit references – units and constants as `⟦…⟧`, operations as `⦇tri⦈`, keywords as `⦅solve(x)⦆`.

use serde::{Deserialize, Serialize};

use crate::num::{Num, Rat};
use crate::units::{UnitExpr, Units};

#[derive(Clone, Debug, PartialEq)]
pub enum Tok {
    /// A constant's binding.
    Bound(Box<crate::bindings::Constant>),
    Num(String),
    Name(String),
    /// "text" in math: a name, of several letters.
    Quoted(String),
    /// A unit's binding, by its id.
    UnitWord(String),
    /// A keyword of a clause, ⦅digits(3)⦆ in math.
    Kw(String),
    /// An explicit reference to an operation, ⦇tri⦈: what Typst's math.op writes.
    Builtin(String),
    Op(&'static str),
}

const OPS: &[(&str, &str)] = &[
    ("∈", "∈"),
    (":=", ":="), ("≔", ":="), ("->", "->"), ("→", "->"), ("<=", "≤"), (">=", "≥"), ("!=", "≠"), ("≤", "≤"), ("≥", "≥"),
    ("≠", "≠"), ("=", "="), ("<", "<"), (">", ">"), ("+", "+"), ("-", "-"), ("−", "-"), ("*", "*"), ("∗", "*"),
    ("·", "·"), ("⋅", "·"), ("×", "×"), ("/", "/"), ("^", "^"), ("(", "("), (")", ")"), ("[", "("), ("]", ")"),
    (",", ","), (";", ";"), (":", ":"), ("..", ".."), ("…", ".."), ("|", "|"), ("‖", "‖"), ("±", "±"),
    // ⌊x⌋ and ⌈x⌉, as Typst writes floor(x) and ceil(x)
    ("⌊", "⌊"), ("⌋", "⌋"), ("⌈", "⌈"), ("⌉", "⌉"),
    // Typst's math writes `and` and `or` as these symbols
    ("∧", "∧"), ("∨", "∨"),
];

pub use solveng_names::GREEK;

/// What an index can follow: x[i], f(x)[i], A[i][j].
fn indexable(t: &Tok) -> bool {
    matches!(t, Tok::Name(_) | Tok::Quoted(_) | Tok::Op(")") | Tok::Op("]"))
}

pub fn greek(name: &str) -> String { solveng_names::normalize(name) }

pub fn lex(s: &str) -> Result<Vec<Tok>, String> {
    let cs: Vec<char> = s.chars().collect();
    let mut out: Vec<Tok> = vec![];
    // open square brackets: whether each indexes
    let mut brackets: Vec<bool> = vec![];
    let mut i = 0;
    while i < cs.len() {
        let c = cs[i];
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        // units are bindings, symbols are no units of their own
        if matches!(c, '°' | '%' | '‰') {
            let name = match (c, cs.get(i + 1)) { ('°', Some('C')) => "degC", ('°', _) => "deg", ('%', _) => "percent", _ => "permille" };
            return Err(format!("'{c}' isn't a unit by itself – write #{name} with #import units: {name}"));
        }
        if solveng_names::numeric_start(&cs, i) {
            let name = solveng_names::scan(&cs, i)?.ok_or("expected a joined variable name")?;
            i = name.end;
            out.push(Tok::Name(name.value));
            continue;
        }
        if c.is_ascii_digit() || (c == '.' && cs.get(i + 1).is_some_and(|d| d.is_ascii_digit())) {
            let st = i;
            while i < cs.len() && cs[i].is_ascii_digit() {
                i += 1;
            }
            // a decimal point; a comma always separates (0,1 is 0 and 1), the separator option only changes the output
            if i + 1 < cs.len() && cs[i] == '.' && cs[i + 1].is_ascii_digit() {
                i += 1;
                while i < cs.len() && cs[i].is_ascii_digit() {
                    i += 1;
                }
            } else if i < cs.len() && cs[i] == '.' && !cs.get(i + 1).is_some_and(|&d| d == '.') {
                i += 1;
            }
            // no e-notation: `1.12e4` is the coefficient 1.12 and the name e4
            out.push(Tok::Num(cs[st..i].iter().collect::<String>().replace('−', "-")));
            continue;
        }
        if solveng_names::is_start(c) || c == '"' {
            if c == '"' && cs.get(i + 1) == Some(&'"') { i += 2; continue; }
            let quoted = c == '"';
            let name = solveng_names::scan(&cs, i)?.ok_or("expected a variable name")?;
            i = name.end;
            let n = name.value;
            if let Some(index) = n.strip_prefix("integral_").or_else(|| n.strip_prefix("sum_")).filter(|s| !s.is_empty()) {
                out.push(Tok::Name(if n.starts_with("sum_") { "sum_" } else { "integral_" }.into()));
                out.extend(lex(index)?);
            } else if quoted { out.push(Tok::Quoted(n)); }
            else { out.push(Tok::Name(n)); }
            continue;
        }
        if c == '⟦' {
            let start = i + 1;
            i = start;
            while i < cs.len() && cs[i] != '⟧' { i += 1; }
            if i == cs.len() { return Err("unclosed lexical value".into()); }
            let binding = crate::bindings::Binding::decode(&cs[start..i].iter().collect::<String>())?;
            out.push(match (binding.unit, binding.identity, binding.backing) {
                (false, Some(identity), Some(backing)) => Tok::Bound(Box::new(crate::bindings::Constant { identity, expr: binding.expr, backing })),
                (true, ..) => Tok::UnitWord(binding.id),
                _ => return Err("invalid constant value".into()),
            });
            i += 1;
            continue;
        }
        match c {
            // ∂ as in (∂ f) / (∂ x): a partial derivative
            '∂' => {
                out.push(Tok::Name("∂".into()));
                i += 1;
                continue;
            }
            // what Typst writes for ∫ and ∑ without both bounds, and products
            // ∫ without bounds: an indefinite integral (with both, Typst's attachment reads as integral_a^b)
            '∫' => {
                if matches!(cs.get(i + 1), Some('_' | '^')) {
                    return Err("a definite integral has both bounds, integral_a^b f dif x; an indefinite one none, integral f dif x".into());
                }
                out.push(Tok::Name("∫".into()));
                i += 1;
                continue;
            }
            '∬' | '∭' | '∮' => return Err("multiple and contour integrals aren't supported – nest integrals: integral_a^b integral_c^d f dif y dif x".into()),
            '∑' => return Err("a sum needs its index and both bounds: sum_(k=1)^n f".into()),
            '∏' => return Err("products (∏) aren't supported – write the factors out".into()),
            '∞' => {
                out.push(Tok::Name("infinity".into()));
                i += 1;
                continue;
            }
            // A^⊤: the transpose
            '⊤' | '⊺' => {
                out.push(Tok::Name("⊤".into()));
                i += 1;
                continue;
            }
            '⦇' => {
                let st = i + 1;
                while i < cs.len() && cs[i] != '⦈' {
                    i += 1;
                }
                out.push(Tok::Builtin(cs[st..i.min(cs.len())].iter().collect()));
                i += 1;
                continue;
            }
            // x[i]: an index right after what it indexes, without a space; [ … ] elsewhere groups
            '[' if i > 0 && !cs[i - 1].is_whitespace() && out.last().is_some_and(indexable) => {
                brackets.push(true);
                out.push(Tok::Op("["));
                i += 1;
                continue;
            }
            '[' => {
                brackets.push(false);
                out.push(Tok::Op("("));
                i += 1;
                continue;
            }
            ']' if !brackets.is_empty() => {
                out.push(Tok::Op(if brackets.pop().unwrap() { "]" } else { ")" }));
                i += 1;
                continue;
            }
            '⦅' => {
                let st = i + 1;
                while i < cs.len() && cs[i] != '⦆' {
                    i += 1;
                }
                out.push(Tok::Kw(cs[st..i.min(cs.len())].iter().collect()));
                i += 1;
                continue;
            }
            // Typst math writes the symbol at as @: at(v, 0) without the import of solveng's at
            '@' if cs[i + 1..].iter().find(|c| !c.is_whitespace()) == Some(&'(') => {
                return Err("'@(' – at(vector, index) needs solveng's at: #import operations: at (in math, at alone is the symbol @)".into());
            }
            '@' => {
                let name = solveng_names::scan(&cs, i + 1).ok().flatten().map_or_else(|| "x".to_string(), |n| n.value);
                return Err(format!("'@' isn't solveng syntax – to put {name}'s formula into a result, end the line with -> substitute({name})"));
            }
            '²' | '³' => {
                i += 1;
                out.push(Tok::Op("^"));
                out.push(Tok::Num(if c == '²' { "2" } else { "3" }.into()));
                continue;
            }
            _ => {}
        }
        let rest: String = cs[i..(i + 2).min(cs.len())].iter().collect();
        if let Some((k, v)) = OPS.iter().find(|(k, _)| rest.starts_with(k)) {
            i += k.chars().count();
            // . . as math writes it: ..
            if *v == ".." || (*k == "." && false) {
                out.push(Tok::Op(".."));
            } else {
                out.push(Tok::Op(v));
            }
            continue;
        }
        if c == '.' {
            // math writes .. as two dots, maybe apart
            let mut j = i + 1;
            while j < cs.len() && cs[j].is_whitespace() {
                j += 1;
            }
            if j < cs.len() && cs[j] == '.' {
                i = j + 1;
                out.push(Tok::Op(".."));
                continue;
            }
        }
        return Err(format!("unexpected '{c}'"));
    }
    Ok(out)
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Rel {
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
}

impl Rel {
    fn of(s: &str) -> Option<Rel> {
        Some(match s {
            "=" => Rel::Eq,
            "≠" => Rel::Ne,
            "<" => Rel::Lt,
            ">" => Rel::Gt,
            "≤" => Rel::Le,
            "≥" => Rel::Ge,
            _ => return None,
        })
    }
    pub fn flip(self) -> Rel {
        match self {
            Rel::Lt => Rel::Gt,
            Rel::Gt => Rel::Lt,
            Rel::Le => Rel::Ge,
            Rel::Ge => Rel::Le,
            r => r,
        }
    }
    pub fn text(self) -> &'static str {
        match self {
            Rel::Eq => "=",
            Rel::Ne => "≠",
            Rel::Lt => "<",
            Rel::Gt => ">",
            Rel::Le => "≤",
            Rel::Ge => "≥",
        }
    }
    pub fn holds(self, o: std::cmp::Ordering, same: bool) -> bool {
        use std::cmp::Ordering::*;
        match self {
            Rel::Eq => same,
            Rel::Ne => !same,
            Rel::Lt => !same && o == Less,
            Rel::Gt => !same && o == Greater,
            Rel::Le => same || o == Less,
            Rel::Ge => same || o == Greater,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum MulOp {
    Implicit,
    Dot,
    Times,
}

/// An expression as written.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Ast {
    /// A constant, from its binding.
    Bound(Box<crate::bindings::Constant>),
    /// A number as written, with its value.
    Num(String, Num),
    /// A number (or a numeric expression) with a unit: 30 mA, frac(3450, 2) mAh.
    Qty(Box<Ast>, UnitExpr),
    /// A unit on its own: one of it (A in [A], mAh after frac(…)).
    Unit(UnitExpr),
    Name(String),
    Neg(Box<Ast>),
    /// Terms, with whether each is subtracted.
    Add(Vec<(bool, Ast)>),
    /// Factors, with how each is joined to the one before.
    Mul(Vec<(MulOp, Ast)>),
    Div(Box<Ast>, Box<Ast>),
    Pow(Box<Ast>, Box<Ast>),
    /// A call of a worksheet function, or of an undefined one (kept symbolic until it is defined).
    Call(String, Vec<Ast>),
    /// A call of a built-in operation, under its canonical name.
    Builtin(String, Vec<Ast>),
    /// A named argument of a call: step: 10 ms.
    Arg(String, Box<Ast>),
    /// x[i], A[i, j]: elements, counted from 0; a vector of indices selects several.
    Index(Box<Ast>, Vec<Ast>),
    /// `:` as an index, all of a dimension: A[i, :] is row i, A[:, j] column j.
    All,
    /// a .. b: the values from a to b in steps of 1, both included (a range variable).
    Range(Box<Ast>, Box<Ast>),
    /// mat(a, b; c, d): the rows of a matrix.
    Matrix(Vec<Vec<Ast>>),
    Abs(Box<Ast>),
    /// a = b < c: a chain of comparisons, 1 where all hold.
    Cmp(Box<Ast>, Vec<(Rel, Ast)>),
    /// a and b (∧): 1 where all hold; binds tighter than or.
    And(Vec<Ast>),
    /// a or b (∨): 1 where any holds.
    Or(Vec<Ast>),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ParamType {
    Unit(UnitExpr),
    Any,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Param {
    pub name: String,
    pub ty: Option<ParamType>,
    /// A named parameter's default value, as written: f(x, n: 2).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<Ast>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Head {
    Var(String),
    Fn { name: String, params: Vec<Param>, rest: bool },
    /// x[i] := …: elements of a vector or matrix, for every value of a range variable among the indices.
    Index { name: String, indices: Vec<Ast> },
}

#[derive(Clone, Debug, PartialEq)]
pub enum Stmt {
    Def { head: Head, rhs: Ast, src: String },
    Expr(Ast),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Trail {
    /// = at the end: the value.
    Eval,
    /// -> at the end: the symbolic result, or a solution.
    Sym,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sign { Negative, Nonpositive, Zero, Nonnegative, Positive }
impl Sign {
    pub fn mask(self) -> u8 { match self { Self::Negative=>1, Self::Nonpositive=>3, Self::Zero=>2, Self::Nonnegative=>6, Self::Positive=>4 } }
    pub fn allows(self, value:f64) -> bool { self.mask() & if value < 0.0 { 1 } else if value == 0.0 { 2 } else { 4 } != 0 }
    pub fn intersect(self, other:Self) -> Option<Self> {
        Some(match self.mask() & other.mask() { 1=>Self::Negative,2=>Self::Zero,3=>Self::Nonpositive,4=>Self::Positive,6=>Self::Nonnegative,_=>return None })
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Clause {
    /// Real scalar assumptions, local to a symbolic result.
    pub real: Vec<String>,
    /// Allowed signs (-, 0, +), local to this result.
    pub signs: std::collections::BTreeMap<String, Sign>,
    pub unit: Option<UnitExpr>,
    pub solve: Option<Vec<String>>,
    /// -> substitute(x, …): the variables whose formulas the result shows put in.
    pub substitute: Vec<String>,
    pub simplify: bool,
    /// -> steps: the forms on the way to the result, one per line.
    pub steps: bool,
    /// -> expand: multiplied out; -> factor: in factors.
    pub expand: bool,
    pub factor: bool,
    pub frac: bool,
    pub digits: Option<u32>,
    pub sci: bool,
    pub eng: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Line {
    pub stmts: Vec<Stmt>,
    pub trail: Option<(Trail, Clause)>,
}

/// Names that can't be defined: Typst's math syntax.
pub fn is_builtin(n: &str) -> bool {
    crate::builtins::is_syntax(n)
}
/// Operations that apply to what follows without parentheses: sin x, det A.
fn applies_bare(op: &str) -> bool {
    matches!(op, "sqrt" | "exp" | "ln" | "log" | "sin" | "cos" | "tan" | "arcsin" | "arccos" | "arctan" | "sinh" | "cosh" | "tanh" | "abs" | "rect" | "tri" | "sgn" | "floor" | "ceil" | "factorial" | "det" | "tr")
}
/// The keywords of a clause, as their bindings name them (frac is Typst's fraction).
const KEYWORDS: &[&str] = &["solve", "substitute", "assume", "simplify", "expand", "factor", "steps", "digits", "sci", "eng", "frac"];

/// What the parser needs to know about the document so far.
pub struct Ctx<'a> {
    pub units: &'a Units,
    /// Names of user functions (and the one being defined).
    pub is_fn: &'a dyn Fn(&str) -> bool,
    /// Names of defined variables.
    pub is_var: &'a dyn Fn(&str) -> bool,
}

struct P<'a, 'b> {
    t: Vec<Tok>,
    i: usize,
    c: &'b Ctx<'a>,
    /// Functions being defined in this line (their own name in a recursive call is refused later).
    defining: Vec<String>,
    abs_depth: usize,
    recursion_depth: usize,
    /// Inside an exponent: numbers take no units.
    in_exp: usize,
    integrand: Integrand,
}

/// The integrand being read. Its differential (dif x) is a factor of 1 that ends it, wherever it stands: after
/// the integrand, at the end of a group around it, in the numerator of a fraction (dif x / (1 + x^2)).
#[derive(Clone, Default)]
struct Integrand {
    /// A differential may stand here: not in arguments, exponents, indices, denominators.
    open: bool,
    /// The differential's variable, once read.
    var: Option<String>,
}

fn visible_tokens(source: &str) -> Result<Vec<Tok>, String> {
    let tokens = lex(source)?;
    let mut depth = 0usize;
    for token in &tokens {
        match token {
            Tok::Op("(" | "[" | "⌊" | "⌈") => {
                depth += 1;
                if depth > 64 { return Err("input nesting limit exceeded (64 levels)".into()); }
            }
            Tok::Op(")" | "]" | "⌋" | "⌉") => depth = depth.saturating_sub(1),
            _ => (),
        }
    }
    tokens.into_iter().map(|token| match token {
        // math.op of a name that is no operation (lim): a name like any other
        Tok::Builtin(name) if crate::builtins::explicit(&name).is_none() => Ok(Tok::Name(name)),
        token => Ok(token),
    }).collect()
}

fn s_op(t: Option<&Tok>) -> Option<&'static str> {
    match t {
        Some(Tok::Op(o)) => Some(o),
        _ => None,
    }
}

impl<'a, 'b> P<'a, 'b> {
    /// The operation a name calls: only an explicit reference (⦇tri⦈) reaches one; a plain name is a name.
    fn operation(&self, n: &str, explicit: bool) -> Option<&'static crate::builtins::Builtin> {
        if explicit { crate::builtins::explicit(n) } else { None }
    }
    fn peek(&self) -> Option<&Tok> {
        self.t.get(self.i)
    }
    fn op(&self) -> Option<&'static str> {
        s_op(self.peek())
    }
    fn eat(&mut self, o: &str) -> bool {
        if self.op() == Some(o) {
            self.i += 1;
            true
        } else {
            false
        }
    }
    fn expect(&mut self, o: &str, what: &str) -> Result<(), String> {
        if self.eat(o) { Ok(()) } else { Err(format!("missing '{o}' {what}")) }
    }
    /// Reads where no differential may stand (arguments, exponents, denominators); the integrand's state stays.
    fn closed<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        let outer = std::mem::take(&mut self.integrand);
        let r = f(self);
        self.integrand = outer;
        r
    }
    /// Whether the integrand ended with its differential: nothing after it belongs to it.
    fn ended(&self) -> bool {
        self.integrand.var.is_some()
    }
    /// The variable of a differential, after dif.
    fn differential_var(&mut self, what: &str) -> Result<String, String> {
        let var = match self.peek() {
            Some(Tok::Name(n)) if n != "dif" => n.clone(),
            Some(Tok::Quoted(q)) => greek(q),
            _ => return Err(format!("{what} after dif")),
        };
        self.i += 1;
        Ok(var)
    }
    /// An optional order ^n of a differential: dif^2, dif x^2.
    fn order(&mut self) -> Result<i32, String> {
        if self.op() != Some("^") { return Ok(1); }
        match self.small_int(self.i + 1) {
            Some((n, end)) if n >= 1 => { self.i = end; Ok(n) }
            _ => Err("the order of a derivative is a positive whole number: dif^2 / (dif x^2)".into()),
        }
    }

    /// Whether the token at k can be read as a unit right after a number.
    fn unit_at(&self, k: usize) -> Option<String> {
        if self.unit_frac(k).is_some() {
            return Some("frac".into());
        }
        match self.t.get(k)? {
            Tok::UnitWord(u) => Some(u.clone()),
            _ => None,
        }
    }

    /// A unit run after a number: m/s^2, Ω mm^2/m, kg m. Returns None if there's none at i.
    fn unit_run(&mut self) -> Result<Option<UnitExpr>, String> {
        let mut terms: Vec<(String, i32)> = vec![];
        let mut below = false;
        loop {
            let k = self.i;
            if !terms.is_empty() && self.op() == Some("/") && self.unit_at(k + 1).is_some() {
                below = true;
                self.i += 1;
                continue;
            }
            // math writes mm^2/m as a fraction: frac(mm^(2), m)
            if let Some((num, den, end)) = self.unit_frac(k) {
                terms.extend(num.into_iter().map(|(n, e)| (n, if below { -e } else { e })));
                terms.extend(den.into_iter().map(|(n, e)| (n, if below { e } else { -e })));
                self.i = end;
                continue;
            }
            let Some(u) = self.unit_at(k) else { break };
            if matches!(&self.t[k], Tok::UnitWord(_)) && !self.c.units.known(&u) {
                return Err(self.c.units.unknown(&u));
            }
            self.i += 1;
            let mut p = 1;
            if self.op() == Some("^") {
                if let Some(e) = self.small_int(self.i + 1) {
                    p = e.0;
                    self.i = e.1;
                }
            }
            terms.push((u, if below { -p } else { p }));
        }
        Ok(if terms.is_empty() { None } else { Some(UnitExpr(terms)) })
    }
    /// frac(units, units) at k -> (above, below, index after it), if both sides are units.
    fn unit_frac(&self, k: usize) -> Option<(Vec<(String, i32)>, Vec<(String, i32)>, usize)> {
        if !matches!(self.t.get(k), Some(Tok::Name(n)) if n == "frac") || s_op(self.t.get(k + 1)) != Some("(") {
            return None;
        }
        let (mut d, mut j, mut comma) = (0, k + 1, None);
        while j < self.t.len() {
            match &self.t[j] {
                Tok::Op("(") => d += 1,
                Tok::Op(")") => {
                    d -= 1;
                    if d == 0 {
                        break;
                    }
                }
                Tok::Op(",") if d == 1 => comma = Some(j),
                _ => {}
            }
            j += 1;
        }
        let c = comma?;
        let side = |a: usize, b: usize| -> Option<Vec<(String, i32)>> {
            let toks = &self.t[a..b];
            if toks.is_empty() || !toks.iter().all(|t| match t {
                Tok::UnitWord(_) | Tok::Num(_) => true,
                Tok::Op(o) => matches!(*o, "^" | "(" | ")" | "-" | "*" | "·"),
                _ => false,
            }) || !toks.iter().any(|t| !matches!(t, Tok::Num(_) | Tok::Op(_))) {
                return None;
            }
            let s: String = toks.iter().map(|t| match t {
                Tok::UnitWord(u) => format!(" {u}"),
                t => tok_text(t),
            }).collect();
            self.c.units.parse(s.trim(), false).ok().map(|p| p.u.0)
        };
        Some((side(k + 2, c)?, side(c + 1, j)?, j + 1))
    }
    /// An integer exponent at k: 2, -1, (2), (-1) -> (value, index after it).
    fn small_int(&self, k: usize) -> Option<(i32, usize)> {
        let par = s_op(self.t.get(k)) == Some("(");
        let mut j = k + par as usize;
        let neg = s_op(self.t.get(j)) == Some("-");
        j += neg as usize;
        let Some(Tok::Num(n)) = self.t.get(j) else { return None };
        let v: i32 = n.parse().ok()?;
        j += 1;
        if par {
            if s_op(self.t.get(j)) != Some(")") {
                return None;
            }
            j += 1;
        }
        Some((if neg { -v } else { v }, j))
    }
    /// The unit of a parameter type: A, mA, A·s/V, m^2; any; 1.
    fn unit_spec(&mut self) -> Result<UnitExpr, String> {
        let st = self.i;
        let mut s = String::new();
        let mut depth = 0;
        loop {
            match self.peek() {
                Some(Tok::UnitWord(u)) => s += &format!(" {u}"),
                Some(Tok::Kw(k)) if s.is_empty() && k == "any" => s += " any",
                Some(Tok::Num(n)) if s.is_empty() && n == "1" => s += "1",
                Some(Tok::Num(n)) if s.ends_with('^') || s.ends_with("^-") || s.ends_with("^(") || s.ends_with("^(-") => s += n,
                Some(Tok::Op(o @ ("*" | "·" | "/" | "^"))) if !s.is_empty() => s += o,
                Some(Tok::Op("-")) if s.ends_with('^') || s.ends_with("^(") => s += "-",
                Some(Tok::Op("(")) => {
                    depth += 1;
                    s += "(";
                }
                Some(Tok::Op(")")) if depth > 0 => {
                    depth -= 1;
                    s += ")";
                }
                _ => break,
            }
            self.i += 1;
        }
        if s.trim().is_empty() {
            self.i = st;
            return Err("a unit after the colon, like f(I: mA)".into());
        }
        if s.trim() == "any" {
            return Ok(UnitExpr::one("any"));
        }
        let p = self.c.units.parse_simplified(s.trim()).map_err(|e| format!("the unit: {e}"))?;
        Ok(p.u)
    }

    /// Conditions joined with or (∨) and and (∧), and binding tighter; comparisons bind tighter than both.
    fn cmp(&mut self) -> Result<Ast, String> {
        let mut any = vec![self.all()?];
        while !self.ended() && self.eat("∨") {
            any.push(self.all()?);
        }
        Ok(if any.len() == 1 { any.pop().unwrap() } else { Ast::Or(any) })
    }
    fn all(&mut self) -> Result<Ast, String> {
        let mut all = vec![self.chain()?];
        while !self.ended() && self.eat("∧") {
            all.push(self.chain()?);
        }
        Ok(if all.len() == 1 { all.pop().unwrap() } else { Ast::And(all) })
    }
    fn chain(&mut self) -> Result<Ast, String> {
        let a = self.sum()?;
        let mut rest = vec![];
        while let Some(r) = self.op().and_then(Rel::of).filter(|_| !self.ended()) {
            self.i += 1;
            rest.push((r, self.sum()?));
        }
        Ok(if rest.is_empty() { a } else { Ast::Cmp(Box::new(a), rest) })
    }
    fn sum(&mut self) -> Result<Ast, String> {
        let mut terms = vec![];
        let neg = if self.eat("-") {
            true
        } else {
            self.eat("+");
            false
        };
        terms.push((neg, self.term()?));
        while !self.ended() {
            let n = match self.op() {
                Some("+") => false,
                Some("-") => true,
                _ => break,
            };
            self.i += 1;
            terms.push((n, self.term()?));
        }
        Ok(if terms.len() == 1 && !terms[0].0 { terms.pop().unwrap().1 } else if terms.len() == 1 { Ast::Neg(Box::new(terms.pop().unwrap().1)) } else { Ast::Add(terms) })
    }
    fn starts_factor(&self) -> bool {
        match self.peek() {
            None => false,
            Some(Tok::Op(o)) => matches!(*o, "(" | "|" | "‖" | "⌊" | "⌈"),
            Some(Tok::Quoted(_)) => true,
            Some(Tok::Kw(_)) => false,
            Some(Tok::Builtin(_) | Tok::Bound(_)) => true,
            _ => true,
        }
    }
    fn term(&mut self) -> Result<Ast, String> {
        let mut fs: Vec<(MulOp, Ast)> = vec![(MulOp::Implicit, self.unary()?)];
        while !self.ended() {
            match self.op() {
                Some(o @ ("*" | "·" | "×")) => {
                    self.i += 1;
                    fs.push((if o == "×" { MulOp::Times } else { MulOp::Dot }, self.unary()?));
                }
                Some("/") => {
                    self.i += 1;
                    let b = self.unary()?;
                    let a = collapse(std::mem::take(&mut fs));
                    fs.push((MulOp::Implicit, Ast::Div(Box::new(a), Box::new(b))));
                }
                // | ends an absolute value here, ‖ a norm
                Some("|" | "‖") if self.abs_depth > 0 => break,
                _ if self.starts_factor() => fs.push((MulOp::Implicit, self.unary()?)),
                _ => break,
            }
        }
        Ok(collapse(fs))
    }
    fn unary(&mut self) -> Result<Ast, String> {
        if self.recursion_depth >= 64 { return Err("expression nesting limit exceeded (64 levels)".into()); }
        self.recursion_depth += 1;
        let result = self.unary_inner();
        self.recursion_depth -= 1;
        result
    }
    fn unary_inner(&mut self) -> Result<Ast, String> {
        if self.eat("-") {
            return Ok(Ast::Neg(Box::new(self.unary()?)));
        }
        if self.eat("+") {
            return self.unary();
        }
        self.power()
    }
    fn power(&mut self) -> Result<Ast, String> {
        if self.recursion_depth >= 64 { return Err("expression nesting limit exceeded (64 levels)".into()); }
        self.recursion_depth += 1;
        let result = self.power_inner();
        self.recursion_depth -= 1;
        result
    }
    fn power_inner(&mut self) -> Result<Ast, String> {
        let mut b = self.atom()?;
        if self.ended() {
            return Ok(b);
        }
        // x[i], A[i, j], f(x)[0]
        while self.eat("[") {
            let outer = std::mem::take(&mut self.integrand);
            let x = std::mem::take(&mut self.in_exp);
            let mut idx = vec![];
            loop {
                if s_op(self.peek()) == Some(":") && matches!(s_op(self.t.get(self.i + 1)), Some("," | "]")) {
                    self.i += 1;
                    idx.push(Ast::All);
                    if self.eat(",") { continue; }
                    break;
                }
                let i = self.cmp();
                if i.is_err() { self.in_exp = x; }
                idx.push(i?);
                if self.eat(",") { continue; }
                break;
            }
            self.in_exp = x;
            self.integrand = outer;
            self.expect("]", "closing the index")?;
            b = Ast::Index(Box::new(b), idx);
        }
        let mut e = if self.eat("^") {
            self.in_exp += 1;
            let x = self.closed(|p| if p.eat("-") { p.power().map(|x| Ast::Neg(Box::new(x))) } else { p.power() });
            self.in_exp -= 1;
            Ast::Pow(Box::new(b), Box::new(x?))
        } else {
            b
        };
        if self.in_exp > 0 {
            return Ok(e);
        }
        // a numeric factor with a unit after it: frac(3450, 2) mAh, 2^3 m^2
        if numeric(&e) && !matches!(e, Ast::Qty(..)) {
            if let Some(u) = self.unit_run()? {
                e = Ast::Qty(Box::new(e), u);
            }
        }
        Ok(e)
    }
    fn args(&mut self, name: &str) -> Result<Vec<Ast>, String> {
        self.closed(|p| p.args_in(name))
    }
    fn args_in(&mut self, name: &str) -> Result<Vec<Ast>, String> {
        let mut args = vec![];
        if self.eat(")") {
            return Ok(args);
        }
        loop {
            // a named argument: step: 10 ms
            if let (Some(Tok::Name(n) | Tok::Quoted(n)), Some(Tok::Op(":"))) = (self.peek().cloned(), self.t.get(self.i + 1).cloned()) {
                self.i += 2;
                let n = greek(&n);
                if args.iter().any(|a| matches!(a, Ast::Arg(m, _) if *m == n)) {
                    return Err(format!("{name}: duplicate argument: {n}"));
                }
                args.push(Ast::Arg(n, Box::new(self.cmp()?)));
            } else {
                if args.iter().any(|a| matches!(a, Ast::Arg(..))) {
                    return Err(format!("{name}: positional arguments come before named ones"));
                }
                args.push(self.cmp()?);
            }
            if self.eat(",") {
                continue;
            }
            self.expect(")", &format!("after the arguments of {name}"))?;
            return Ok(args);
        }
    }
    /// The rows of mat(a, b; c, d), after its opening parenthesis.
    fn rows(&mut self) -> Result<Vec<Vec<Ast>>, String> {
        self.closed(|p| p.rows_in())
    }
    fn rows_in(&mut self) -> Result<Vec<Vec<Ast>>, String> {
        let mut rows: Vec<Vec<Ast>> = vec![vec![]];
        loop {
            rows.last_mut().unwrap().push(self.cmp()?);
            if self.eat(",") {
                continue;
            }
            if self.eat(";") {
                if self.eat(")") {
                    break;
                }
                rows.push(vec![]);
                continue;
            }
            self.expect(")", "after the elements of mat – a comma between elements, a semicolon between rows")?;
            break;
        }
        // rows of blocks may differ in their number of elements; the sizes are checked once the blocks are known
        Ok(rows)
    }
    fn atom(&mut self) -> Result<Ast, String> {
        let t = self.peek().cloned().ok_or("unexpected end of the expression")?;
        self.i += 1;
        match t {
            Tok::Bound(e) => Ok(Ast::Bound(e)),
            Tok::Num(raw) => {
                let v = Rat::parse_decimal(&raw).map(Num::R).or_else(|| raw.parse::<f64>().ok().map(Num::F)).ok_or_else(|| format!("cannot read the number '{raw}'"))?;
                let n = Ast::Num(raw, v);
                // 2^3 m^2: the power comes first
                if self.op() == Some("^") || self.in_exp > 0 {
                    return Ok(n);
                }
                Ok(match self.unit_run()? {
                    Some(u) => Ast::Qty(Box::new(n), u),
                    None => n,
                })
            }
            Tok::UnitWord(u) => {
                if !self.c.units.known(&u) {
                    return Err(self.c.units.unknown(&u));
                }
                self.i -= 1;
                let u = self.unit_run()?.unwrap();
                Ok(Ast::Unit(u))
            }
            Tok::Quoted(q) => {
                // a name of several letters
                let n = greek(&q);
                if self.op() == Some("(") {
                    self.i += 1;
                    let x = std::mem::take(&mut self.in_exp);
                    let args = self.args(&n);
                    self.in_exp = x;
                    return Ok(Ast::Call(n, args?));
                }
                Ok(Ast::Name(n))
            }
            Tok::Name(n) => self.name(n),
            Tok::Builtin(n) => self.call_or_name(n, true),
            Tok::Op("(") => {
                let x = std::mem::take(&mut self.in_exp);
                let e = self.cmp();
                self.in_exp = x;
                let e = e?;
                self.expect(")", "")?;
                Ok(e)
            }
            Tok::Op("|") => {
                self.abs_depth += 1;
                let e = self.closed(|p| p.sum());
                self.abs_depth -= 1;
                let e = e?;
                self.expect("|", "closing the absolute value")?;
                Ok(Ast::Abs(Box::new(e)))
            }
            // ‖v‖, as Typst writes norm(v)
            Tok::Op("‖") => {
                self.abs_depth += 1;
                let e = self.closed(|p| p.sum());
                self.abs_depth -= 1;
                let e = e?;
                self.expect("‖", "closing the norm")?;
                Ok(Ast::Builtin("norm".into(), vec![e]))
            }
            // ⌊x⌋ and ⌈x⌉, as Typst writes floor(x) and ceil(x)
            Tok::Op(o @ ("⌊" | "⌈")) => {
                let x = std::mem::take(&mut self.in_exp);
                let e = self.closed(|p| p.sum());
                self.in_exp = x;
                let e = e?;
                let (close, n) = if o == "⌊" { ("⌋", "floor") } else { ("⌉", "ceiling") };
                self.expect(close, &format!("closing the {n}"))?;
                Ok(Ast::Builtin(if o == "⌊" { "floor" } else { "ceil" }.into(), vec![e]))
            }
            Tok::Op(o) => Err(format!("unexpected '{o}'")),
            Tok::Kw(k) => Err(format!("unexpected keyword '{k}'")),
        }
    }
    fn name(&mut self, n: String) -> Result<Ast, String> {
        if n == "dif" {
            return self.differential();
        }
        if n == "∂" {
            if self.integrand.open {
                return Err("an integral's differential is dif x (dif, not diff: diff is ∂)".into());
            }
            return Err("∂ makes a partial derivative: (∂ f) / (∂ x), (∂^2 f) / (∂ x ∂ y)".into());
        }
        if n == "sum_" {
            self.expect("(", "around the sum index and lower bound")?;
            let Some(Tok::Name(var) | Tok::Quoted(var)) = self.peek().cloned() else { return Err("a sum index must be a name".into()); };
            self.i += 1;
            self.expect("=", "after the sum index")?;
            let lo = self.sum()?;
            self.expect(")", "after the lower sum bound")?;
            self.expect("^", "before the upper sum bound")?;
            // A bare upper-bound name ends before a parenthesized summand.
            let hi = match self.peek().cloned() {
                Some(Tok::Name(n) | Tok::Quoted(n)) => { self.i += 1; Ast::Name(n) }
                _ => self.atom()?,
            };
            let body = self.term()?;
            return Ok(Ast::Builtin("sum".into(), vec![body, Ast::Name(var), lo, hi]));
        }
        if n == "integral_" {
            return self.integral();
        }
        if n == "∫" {
            let (body, var) = self.integrand()?;
            return Ok(Ast::Builtin("integrate".into(), vec![body, Ast::Name(var)]));
        }
        // Typst's frac(a, b), as math writes a fraction out: one factor
        // Typst's frac(a, b), as math writes a fraction out: one factor
        if n == "frac" && self.op() == Some("(") {
            return self.frac();
        }
        if n == "mat" && self.op() == Some("(") {
            self.i += 1;
            let x = std::mem::take(&mut self.in_exp);
            let rows = self.rows();
            self.in_exp = x;
            return Ok(Ast::Matrix(rows?));
        }
        self.call_or_name(n, false)
    }
    /// dif / (dif x) f: a derivative (as Typst writes frac(dif, dif x)); dif x in an integrand: its differential.
    fn differential(&mut self) -> Result<Ast, String> {
        if self.eat("/") {
            self.expect("(", "around the differential")?;
            if !matches!(self.peek(), Some(Tok::Name(n)) if n == "dif") { return Err("expected dif in the denominator".into()); }
            self.i += 1;
            let var = self.differential_var("a differentiation variable")?;
            let order = self.order()?;
            self.expect(")", "after the differential")?;
            let body = self.term()?;
            return Ok(derivative(body, &vec![var; order as usize]));
        }
        if !self.integrand.open {
            return Err("dif stands for a differential: it ends an integral (integral_a^b f(x) dif x) or makes a derivative (dif / (dif x) f, (dif f) / (dif x))".into());
        }
        if self.ended() {
            return Err("an integral has one differential – nest integrals for several variables".into());
        }
        let var = self.differential_var("an integration variable")?;
        self.integrand.var = Some(var);
        Ok(Ast::Num("1".into(), Num::R(Rat::ONE)))
    }
    /// integral_a^b f dif x: the integrand ends with its differential.
    fn integral(&mut self) -> Result<Ast, String> {
        let lo = self.closed(|p| p.atom())?;
        self.expect("^", "before the upper integral bound")?;
        let hi = self.closed(|p| p.atom())?;
        let (body, var) = self.integrand()?;
        Ok(Ast::Builtin("integrate".into(), vec![body, Ast::Name(var), lo, hi]))
    }
    /// An integral's integrand and the variable of the differential ending it.
    fn integrand(&mut self) -> Result<(Ast, String), String> {
        let start = self.i;
        let outer = std::mem::replace(&mut self.integrand, Integrand { open: true, var: None });
        let body = self.sum();
        let inner = std::mem::replace(&mut self.integrand, outer);
        let body = body?;
        let Some(var) = inner.var else { return Err("an integral ends with dif and its variable".into()) };
        // ∫ dif x f(x): the differential ended an empty integrand, and f(x) would multiply the integral
        if matches!(self.t.get(start), Some(Tok::Name(n)) if n == "dif") && self.starts_factor() {
            return Err(format!("the differential ends the integrand: write integral … f dif {var}, not dif {var} f"));
        }
        Ok((body, var))
    }
    /// frac(a, b); in Leibniz's notation frac(dif f, dif x), frac(dif^2 f, dif x^2), frac(dif, dif x) f: a derivative.
    fn frac(&mut self) -> Result<Ast, String> {
        self.i += 1;
        let x = std::mem::take(&mut self.in_exp);
        let r = if self.leibniz() { self.closed(|p| p.leibniz_frac()) } else { self.plain_frac().map(Frac::Value) };
        self.in_exp = x;
        match r? {
            Frac::Value(v) => Ok(v),
            // frac(dif, dif x) f: what it differentiates follows
            Frac::Operator(vars) => {
                let body = self.term()?;
                Ok(derivative(body, &vars))
            }
        }
    }
    /// Whether the fraction after its parenthesis is a derivative: both its parts start with dif.
    fn leibniz(&self) -> bool {
        let is_dif = |k: usize| matches!(self.t.get(k), Some(Tok::Name(n)) if n == "dif" || n == "∂");
        if !is_dif(self.i) { return false; }
        let mut d = 0;
        for k in self.i..self.t.len() {
            match &self.t[k] {
                Tok::Op("(" | "[") => d += 1,
                Tok::Op(")" | "]") if d == 0 => return false,
                Tok::Op(")" | "]") => d -= 1,
                Tok::Op(",") if d == 0 => return is_dif(k + 1),
                _ => {}
            }
        }
        false
    }
    fn plain_frac(&mut self) -> Result<Ast, String> {
        let a = self.cmp()?;
        if !self.eat(",") { return Err("frac takes two arguments".into()); }
        let b = self.closed(|p| p.cmp())?;
        if self.op() == Some(",") { return Err("frac takes two arguments".into()); }
        self.expect(")", "after the arguments of frac")?;
        Ok(Ast::Div(Box::new(a), Box::new(b)))
    }
    /// frac(dif^n f, dif x^n), frac(∂^2 f, ∂ x ∂ y) after the parenthesis.
    fn leibniz_frac(&mut self) -> Result<Frac, String> {
        let Some(Tok::Name(d)) = self.peek().cloned() else { unreachable!() };
        self.i += 1;
        let order = self.order()?;
        let body = if self.op() == Some(",") { None } else { Some(self.cmp()?) };
        self.expect(",", "between the differentials")?;
        // below: one differential per variable, each with its order
        let mut vars = vec![];
        while self.op() != Some(")") {
            if !matches!(self.peek(), Some(Tok::Name(n)) if *n == d) {
                return Err(format!("below a derivative stand differentials: {d} x, {d} x^2, {d} x {d} y"));
            }
            self.i += 1;
            let var = self.differential_var("a differentiation variable")?;
            let n = self.order()?;
            vars.extend(std::iter::repeat_n(var, n as usize));
        }
        self.i += 1;
        if vars.len() != order as usize {
            return Err(format!("a derivative of order {order} has {order} differentials below ({d} x^{order}), not {}", vars.len()));
        }
        Ok(match body {
            Some(b) => Frac::Value(derivative(b, &vars)),
            None => Frac::Operator(vars),
        })
    }
    /// f(x), tri(t), sin x, or a name alone.
    fn call_or_name(&mut self, n: String, explicit: bool) -> Result<Ast, String> {
        let op = self.operation(&n, explicit);
        if self.op() == Some("(") {
            self.i += 1;
            let x = std::mem::take(&mut self.in_exp);
            let args = self.args(&n);
            self.in_exp = x;
            let args = args?;
            return Ok(match op {
                Some(b) => Ast::Builtin(b.name.to_string(), args),
                None => Ast::Call(n, args),
            });
        }
        // sin x: a built-in function applies to what follows
        if let Some(b) = op.filter(|b| applies_bare(b.name)) {
            if self.starts_factor() {
                let a = self.power()?;
                return Ok(Ast::Builtin(b.name.to_string(), vec![a]));
            }
        }
        Ok(Ast::Name(n))
    }
    /// An expression, or a range a .. b.
    fn ranged(&mut self) -> Result<Ast, String> {
        let a = self.cmp()?;
        if self.eat("..") {
            let b = self.cmp()?;
            return Ok(Ast::Range(Box::new(a), Box::new(b)));
        }
        Ok(a)
    }
}

/// A fraction read: a value, or d^n/dx^n applying to what follows it.
enum Frac {
    Value(Ast),
    Operator(Vec<String>),
}

/// The derivative by each variable in turn, the last one first.
fn derivative(body: Ast, vars: &[String]) -> Ast {
    vars.iter().rev().fold(body, |b, v| Ast::Builtin("diff".into(), vec![b, Ast::Name(v.clone())]))
}

/// An expression of numbers only (a quantity's numeric part).
pub fn numeric(a: &Ast) -> bool {
    match a {
        Ast::Num(..) => true,
        Ast::Neg(x) => numeric(x),
        Ast::Add(ts) => ts.iter().all(|t| numeric(&t.1)),
        Ast::Mul(fs) => fs.iter().all(|f| numeric(&f.1)),
        Ast::Div(a, b) | Ast::Pow(a, b) => numeric(a) && numeric(b),
        _ => false,
    }
}

fn collapse(mut fs: Vec<(MulOp, Ast)>) -> Ast {
    if fs.len() == 1 { fs.pop().unwrap().1 } else { Ast::Mul(fs) }
}

/// Splits a line at its trailing = or -> and reads the clause after it: (head tokens, trail).
fn split_trail(t: &[Tok], c: &Ctx) -> Result<(usize, Option<(Trail, Clause)>), String> {
    let mut depth = 0i32;
    let (mut arrow, mut eq) = (None, None);
    for (i, k) in t.iter().enumerate() {
        match k {
            Tok::Op("(" | "[") => depth += 1,
            Tok::Op(")" | "]") => depth -= 1,
            Tok::Op("->") if depth == 0 => arrow = Some(i),
            Tok::Op("=") if depth == 0 => eq = Some(i),
            _ => {}
        }
    }
    if let Some(i) = arrow {
        let Some(cl) = clause(&t[i + 1..], c)? else {
            // what can't be a clause: a name alone is no unit (units are bindings), anything else no modifier either
            let tail = &t[i + 1..];
            let names: Vec<&String> = tail.iter().filter_map(|k| match k { Tok::Name(n) | Tok::Quoted(n) => Some(n), _ => None }).collect();
            return Err(match names.as_slice() {
                [n] if tail.iter().all(|k| matches!(k, Tok::Name(_) | Tok::Quoted(_) | Tok::Kw(_) | Tok::Op(","))) => format!("'{n}' after -> is a name, not a unit – a unit is a binding: #{n} with #import units: {n}, or one of your own"),
                _ => format!("'{}' after -> is neither a modifier nor a unit – -> takes modifiers and a unit: -> simplify, -> solve(x), -> digits(3), -> #km", src_text(tail).split_whitespace().collect::<Vec<_>>().join(" ")),
            });
        };
        return Ok((i, Some((Trail::Sym, cl))));
    }
    if let Some(i) = eq {
        // = followed by more than a unit and keywords is a comparison
        if let Some(cl) = clause(&t[i + 1..], c)? {
            return Ok((i, Some((Trail::Eval, cl))));
        }
    }
    Ok((t.len(), None))
}

/// Keywords and a unit, separated by commas; None if it isn't one (2000 mA is a value, not a clause).
fn clause(t: &[Tok], c: &Ctx) -> Result<Option<Clause>, String> {
    // names written with math.op are read as names here; math writes m/s as frac(m, s)
    let t: Vec<Tok> = t.iter().map(|k| match k { Tok::Builtin(n) => Tok::Name(n.clone()), k => k.clone() }).collect();
    let t = unfrac(&t);
    let t = t.as_slice();
    let mut cl = Clause::default();
    if t.is_empty() {
        return Ok(Some(cl));
    }
    let mut groups: Vec<Vec<Tok>> = vec![vec![]];
    let mut depth = 0;
    for k in t {
        match k {
            Tok::Op("(" | "[") => depth += 1,
            Tok::Op(")" | "]") => depth -= 1,
            _ => {}
        }
        if *k == Tok::Op(",") && depth == 0 {
            groups.push(vec![]);
        } else {
            groups.last_mut().unwrap().push(k.clone());
        }
    }
    let mut kws: Vec<String> = vec![];
    let mut units: Vec<String> = vec![];
    for g in groups {
        if g.is_empty() {
            return Ok(None);
        }
        // a keyword: ⦅solve(x)⦆
        if let [Tok::Kw(k)] = g.as_slice() {
            kws.push(k.clone());
            continue;
        }
        // a unit: unit bindings, powers, products and quotients; no numbers but exponents and a leading 1 (1 #h)
        let mut s = String::new();
        let mut prev_pow = false;
        let mut named = false;
        for k in &g {
            match k {
                Tok::Num(n) if prev_pow || n == "1" => s += n,
                Tok::UnitWord(n) => {
                    if !c.units.known(n) {
                        return Ok(None);
                    }
                    named = true;
                    s += &format!(" {n}");
                }
                Tok::Op(o @ ("*" | "·" | "/" | "^" | "(" | ")")) => s += o,
                Tok::Op("-") if prev_pow => s += "-",
                _ => return Ok(None),
            }
            prev_pow = matches!(k, Tok::Op("^")) || (prev_pow && matches!(k, Tok::Op("(") | Tok::Op("-")));
        }
        if !named {
            return Ok(None);
        }
        units.push(s.trim().to_string());
    }
    for k in kws {
        let (name, args) = match k.find('(') {
            Some(i) => (k[..i].trim().to_string(), Some(k[i + 1..].trim_end().trim_end_matches(')').to_string())),
            None => (k.trim().to_string(), None),
        };
        let args: Vec<String> = args.map(|a| a.split(',').map(|x| greek(x.trim())).filter(|x| !x.is_empty()).collect()).unwrap_or_default();
        match name.as_str() {
            "assume" => {
                if args.is_empty() { return Err("assume: write assume(a in RR, b in RR)".into()); }
                for arg in args {
                    let tokens = lex(&arg)?;
                    let (n, sign) = match tokens.as_slice() {
                        [Tok::Name(n) | Tok::Quoted(n), Tok::Op("∈"), Tok::Name(domain)] if domain == "RR" || domain == "ℝ" => (n,None),
                        [Tok::Name(n) | Tok::Quoted(n), Tok::Name(op), Tok::Name(domain)] if op == "in" && (domain == "RR" || domain == "ℝ") => (n,None),
                        [Tok::Name(n) | Tok::Quoted(n), Tok::Op(op), Tok::Num(zero)] if Rat::parse_decimal(zero).is_some_and(|v| v.is_zero()) => {
                            (n,Some(match *op { ">"=>Sign::Positive,"≥"=>Sign::Nonnegative,"<"=>Sign::Negative,"≤"=>Sign::Nonpositive,"="=>Sign::Zero,_=>return Err(format!("assume: unsupported sign constraint '{arg}'")) }))
                        }
                        [Tok::Num(zero), Tok::Op(op), Tok::Name(n) | Tok::Quoted(n)] if Rat::parse_decimal(zero).is_some_and(|v| v.is_zero()) => {
                            (n,Some(match *op { "<"=>Sign::Positive,"≤"=>Sign::Nonnegative,">"=>Sign::Negative,"≥"=>Sign::Nonpositive,"="=>Sign::Zero,_=>return Err(format!("assume: unsupported sign constraint '{arg}'")) }))
                        }
                        _ => return Err(format!("assume: use a in RR or a sign constraint such as a > 0; got '{arg}'")),
                    };
                    if let Some(sign) = sign {
                        let sign = match cl.signs.get(n) { Some(previous)=>previous.intersect(sign).ok_or_else(||format!("assume: contradictory sign constraints for {n}"))?,None=>sign };
                        cl.signs.insert(n.clone(),sign);
                    }
                    if !cl.real.contains(n) { cl.real.push(n.clone()); }
                }
            }
            "solve" => {
                if args.is_empty() {
                    return Err("solve: what to solve for, solve(x) or solve(x, y)".into());
                }
                cl.solve = Some(args);
            }
            "substitute" => {
                // variables by name, constants by their bindings
                let name = |n: &String| n.starts_with('⟦') || solveng_names::scan(&n.chars().collect::<Vec<_>>(), 0).ok().flatten().is_some_and(|v| v.end == n.chars().count());
                if args.is_empty() || !args.iter().all(name) {
                    return Err("substitute: the variables and constants whose expressions to put in, substitute(x) or substitute(#c, y)".into());
                }
                for n in args {
                    if !cl.substitute.contains(&n) { cl.substitute.push(n); }
                }
            }
            "digits" => {
                let d: u32 = args.first().and_then(|a| a.parse().ok()).filter(|d| (1..=15).contains(d)).ok_or_else(|| format!("digits: 1…15, got {}", args.join(", ")))?;
                cl.digits = Some(d);
            }
            "simplify" | "expand" | "factor" | "steps" | "sci" | "eng" | "frac" => {
                if !args.is_empty() {
                    return Err(format!("{name} takes no arguments"));
                }
                match name.as_str() {
                    "simplify" => cl.simplify = true,
                    "steps" => cl.steps = true,
                    "expand" => cl.expand = true,
                    "factor" => cl.factor = true,
                    "sci" => cl.sci = true,
                    "eng" => cl.eng = true,
                    _ => cl.frac = true,
                }
            }
            _ => return Err(format!("unknown keyword '{name}' (keywords: {})", KEYWORDS.join(", "))),
        }
    }
    if cl.expand && cl.factor {
        return Err("expand multiplies out, factor splits into factors – one of them".into());
    }
    if units.len() > 1 {
        return Err(format!("one unit per result, got {} and {}", units[0], units[1]));
    }
    if let Some(u) = units.pop() {
        cl.unit = Some(c.units.parse(&u, false)?.u);
    }
    Ok(Some(cl))
}

/// frac(a, b) as (a)/(b), at every depth.
fn unfrac(t: &[Tok]) -> Vec<Tok> {
    let mut out = vec![];
    let mut i = 0;
    while i < t.len() {
        if matches!(&t[i], Tok::Name(n) if n == "frac") && t.get(i + 1) == Some(&Tok::Op("(")) {
            let (mut depth, mut comma, mut j) = (0, None, i + 1);
            while j < t.len() {
                match &t[j] {
                    Tok::Op("(") => depth += 1,
                    Tok::Op(")") => {
                        depth -= 1;
                        if depth == 0 { break; }
                    }
                    Tok::Op(",") if depth == 1 && comma.is_none() => comma = Some(j),
                    _ => {}
                }
                j += 1;
            }
            if let Some(c) = comma.filter(|_| j < t.len()) {
                out.push(Tok::Op("("));
                out.extend(unfrac(&t[i + 2..c]));
                out.extend([Tok::Op(")"), Tok::Op("/"), Tok::Op("(")]);
                out.extend(unfrac(&t[c + 1..j]));
                out.push(Tok::Op(")"));
                i = j + 1;
                continue;
            }
        }
        out.push(t[i].clone());
        i += 1;
    }
    out
}

fn tok_text(t: &Tok) -> String {
    match t {
        Tok::Bound(c) => c.identity.clone(),
        Tok::Num(s) | Tok::Name(s) => s.clone(),
        Tok::Quoted(s) => format!("\"{s}\""),
        Tok::UnitWord(s) => s.clone(),
        Tok::Kw(s) | Tok::Builtin(s) => s.clone(),
        Tok::Op(o) => o.to_string(),
    }
}

/// The parameters of a function being defined: I: mA, R, x: any, ..
fn params(t: &[Tok], c: &Ctx) -> Result<(Vec<Param>, bool), String> {
    let mut groups: Vec<Vec<Tok>> = vec![vec![]];
    let mut depth = 0;
    for k in t {
        match k {
            Tok::Op("(" | "[") => depth += 1,
            Tok::Op(")" | "]") => depth -= 1,
            _ => {}
        }
        if *k == Tok::Op(",") && depth == 0 {
            groups.push(vec![]);
        } else {
            groups.last_mut().unwrap().push(k.clone());
        }
    }
    let n = groups.len();
    let mut ps = vec![];
    let mut rest = false;
    for (i, g) in groups.into_iter().enumerate() {
        if g.is_empty() && n == 1 {
            break;
        }
        if g == [Tok::Op("..")] {
            if i != n - 1 {
                return Err("'..' comes last in the parameters: it stands for the rest".into());
            }
            rest = true;
            continue;
        }
        let name = match g.first() {
            Some(Tok::Name(x) | Tok::Quoted(x)) => x.clone(),
            _ => return Err(format!("the parameters are names, optionally with a unit (I: mA), and .. for the rest, got '{}'", g.iter().map(tok_text).collect::<String>())),
        };
        let mut default = None;
        let ty = if g.len() > 1 {
            if s_op(g.get(1)) != Some(":") {
                return Err(format!("the parameters are names, optionally with a unit (I: mA) or a default value (n: 2), got '{}'", g.iter().map(tok_text).collect::<Vec<_>>().join(" ")));
            }
            let sub: Vec<Tok> = g[2..].to_vec();
            // an expression of unit bindings only (and any, 1) is a type (I: mA); anything else a default value
            // (n: 2, t: 1 s), as in Typst a named parameter
            let value = {
                let mut p = P { t: sub.clone(), i: 0, c, defining: vec![], abs_depth: 0, recursion_depth: 0, in_exp: 0, integrand: Integrand::default() };
                !(p.unit_spec().is_ok() && p.i == p.t.len())
            };
            if value {
                let mut q = P { t: sub, i: 0, c, defining: vec![], abs_depth: 0, recursion_depth: 0, in_exp: 0, integrand: Integrand::default() };
                let d = q.cmp().map_err(|e| format!("the default value of {name}: {e}"))?;
                if q.i < q.t.len() {
                    return Err(format!("the default value of {name}: can't read '{}'", q.t[q.i..].iter().map(tok_text).collect::<Vec<_>>().join(" ")));
                }
                default = Some(d);
                None
            } else {
                let mut p = P { t: sub, i: 0, c, defining: vec![], abs_depth: 0, recursion_depth: 0, in_exp: 0, integrand: Integrand::default() };
                let u = p.unit_spec().map_err(|e| format!("the unit of {name}: {}", e.trim_start_matches("the unit: ")))?;
                if p.i < p.t.len() {
                    return Err(format!("the unit of {name}: can't read '{}'", p.t[p.i..].iter().map(tok_text).collect::<Vec<_>>().join(" ")));
                }
                Some(if u.0.len() == 1 && u.0[0].0 == "any" { ParamType::Any } else { ParamType::Unit(u) })
            }
        } else {
            None
        };
        if default.is_none() && ps.iter().any(|q: &Param| q.default.is_some()) {
            return Err(format!("{name}: parameters with a default value (named parameters) come after the others"));
        }
        ps.push(Param { name, ty, default });
    }
    Ok((ps, rest))
}

fn src_text(t: &[Tok]) -> String {
    let mut s = String::new();
    for (i, k) in t.iter().enumerate() {
        let x = tok_text(k);
        let tight = matches!(k, Tok::Op("(" | ")" | "^" | ",")) || (i > 0 && matches!(t[i - 1], Tok::Op("(" | "^")));
        if i > 0 && !tight {
            s.push(' ');
        }
        s += &x;
        if matches!(k, Tok::Op(",")) {
            s.push(' ');
        }
    }
    s
}

/// Whether a line is meant for solveng though it can't be read: it ends with a bare = or ->, or what comes before
/// its arrow is a calculation (`x^2 + 1 -> 2`) or uses a unit or a defined name (`1 "min" -> s`), or a keyword
/// follows it – unlike prose such as `f: X -> Y` or `n -> infinity`.
pub fn meant(src: &str, c: &Ctx) -> bool {
    // a bare trailing = or -> asks for a result, also of what can't be lexed
    let end = src.trim_end();
    if end.ends_with("->") || end.ends_with('→') || (end.ends_with('=') && !end.ends_with("<=") && !end.ends_with(">=") && !end.ends_with("!=")) { return true; }
    let Ok(t) = visible_tokens(src) else { return false };
    let mut depth = 0i32;
    let mut arrow = None;
    for (i, k) in t.iter().enumerate() {
        match k {
            Tok::Op("(" | "[") => depth += 1,
            Tok::Op(")" | "]") => depth -= 1,
            Tok::Op("->") if depth == 0 => arrow = Some(i),
            _ => {}
        }
    }
    let Some(arrow) = arrow else { return false };
    // solveng's keywords after the arrow
    if t[arrow + 1..].iter().any(|k| matches!(k, Tok::Kw(_))) { return true; }
    // a calculation before it, more than a name or number (x² + 1 -> …), unlike x -> 0 or f: X -> Y
    let head = &t[..arrow];
    if head.len() > 1 && stmt(head, c).is_ok() { return true; }
    t[..arrow].iter().any(|k| match k {
        Tok::UnitWord(_) => true,
        Tok::Quoted(n) | Tok::Name(n) => (c.is_var)(n) || (c.is_fn)(n),
        _ => false,
    })
}

/// Reads a line: statements separated by ; (or , in math), an optional trailing = or -> with its clause.
pub fn parse_line(src: &str, c: &Ctx) -> Result<Line, String> {
    let t = visible_tokens(src)?;
    if t.is_empty() {
        return Err("nothing to calculate".into());
    }
    let (end, trail) = split_trail(&t, c)?;
    let head = &t[..end];
    if head.is_empty() {
        return Err(format!("nothing before the {}", if matches!(trail, Some((Trail::Sym, _))) { "->" } else { "=" }));
    }
    // statements: ; always, , at the top level (math systems)
    let mut parts: Vec<&[Tok]> = vec![];
    let (mut depth, mut st) = (0i32, 0);
    for (i, k) in head.iter().enumerate() {
        match k {
            Tok::Op("(" | "[") => depth += 1,
            Tok::Op(")" | "]") => depth -= 1,
            Tok::Op(";") | Tok::Op(",") if depth == 0 => {
                parts.push(&head[st..i]);
                st = i + 1;
            }
            _ => {}
        }
    }
    parts.push(&head[st..]);
    let mut stmts = vec![];
    for p in parts {
        if p.is_empty() {
            continue;
        }
        stmts.push(stmt(p, c)?);
    }
    Ok(Line { stmts, trail })
}

fn stmt(t: &[Tok], c: &Ctx) -> Result<Stmt, String> {
    let def = t.iter().position(|k| *k == Tok::Op(":="));
    let mut p = P { t: vec![], i: 0, c, defining: vec![], abs_depth: 0, recursion_depth: 0, in_exp: 0, integrand: Integrand::default() };
    if let Some(d) = def {
        let lhs = &t[..d];
        let rhs = &t[d + 1..];
        if rhs.is_empty() {
            return Err("nothing after :=".into());
        }
        let head = match lhs {
            [Tok::Name(n)] => Head::Var(n.clone()),
            [Tok::Quoted(n)] => Head::Var(greek(n)),
            [Tok::Name(n) | Tok::Quoted(n), Tok::Op("["), inner @ .., Tok::Op("]")] => {
                let mut q = P { t: inner.to_vec(), i: 0, c, defining: vec![], abs_depth: 0, recursion_depth: 0, in_exp: 0, integrand: Integrand::default() };
                let mut indices = vec![];
                loop {
                    indices.push(q.cmp()?);
                    if q.eat(",") { continue; }
                    break;
                }
                if q.i < q.t.len() || indices.len() > 2 {
                    return Err(format!("'{}' can't be defined – an element is x[i] or A[i, j]", src_text(lhs)));
                }
                Head::Index { name: greek(n), indices }
            }
            [Tok::Name(n), Tok::Op("("), inner @ .., Tok::Op(")")] | [Tok::Quoted(n), Tok::Op("("), inner @ .., Tok::Op(")")] => {
                let n = greek(n);
                let (params, rest) = params(inner, c)?;
                p.defining.push(n.clone());
                Head::Fn { name: n, params, rest }
            }
            [Tok::Builtin(n), ..] => {
                return Err(format!("{n} is a built-in operation here – to define a function of that name, write it as a name: \"{n}\"(…) := … in math"));
            }
            [Tok::Name(n), Tok::Op(":"), ..] => {
                let u = lhs[2..].iter().map(tok_text).collect::<Vec<_>>().join(" ");
                return Err(format!("'{n}: {u}' – a variable has the unit of its value, so it takes no type: write {n} := … = {u} to show it in {u} (types are for a function's parameters, f(x: {u}))"));
            }
            [Tok::Name(n), Tok::Op("("), ..]
                if lhs.iter().rposition(|k| *k == Tok::Op(":")).is_some_and(|i| i > 2 && lhs[i - 1] == Tok::Op(")")) =>
            {
                return Err(format!("{n}(…): … – a function returns what its body gives, so it takes no type: give the body that unit, by dividing or multiplying by units"));
            }
            _ => return Err(format!("'{}' can't be defined – := defines a name (x := …) or a function (f(x) := …)", src_text(lhs))),
        };
        p.t = rhs.to_vec();
        let e = p.ranged()?;
        if p.i < p.t.len() {
            return Err(format!("unexpected '{}'", tok_text(&p.t[p.i])));
        }
        return Ok(Stmt::Def { head, rhs: e, src: src_text(rhs) });
    }
    p.t = t.to_vec();
    let e = p.ranged()?;
    if p.i < p.t.len() {
        let k = &p.t[p.i];
        if *k == Tok::Op("..") {
            return Err("'..' makes a range of the values between two bounds, i := 0 .. 10; in a function's parameters, f(x, ..), it stands for the rest".into());
        }
        return Err(format!("unexpected '{}'", tok_text(k)));
    }
    Ok(Stmt::Expr(e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support as t;

    /// A readable line read as Typst sends it, with the units it uses.
    fn read(s: &str) -> Result<(Line, Units), String> {
        let src = t::src(s, &crate::env::Env::default());
        let u = Units::lexical(&crate::format::Cfg::default(), &crate::env::Env::default(), &src)?;
        let c = Ctx { units: &u, is_fn: &|n| n == "f" || n == "q", is_var: &|n| n == "h" };
        Ok((parse_line(&src, &c)?, u))
    }
    fn line(s: &str) -> Line {
        read(s).unwrap().0
    }
    /// The unit of the trailing clause, by name.
    fn clause_unit(s: &str) -> Option<String> {
        let (l, u) = read(s).unwrap();
        l.trail.unwrap().1.unit.map(|x| u.text(&x))
    }

    #[test]
    fn math() {
        assert_eq!(line("t ≔ Q_0/I = 1 #h").trail.as_ref().unwrap().0, Trail::Eval);
        assert_eq!(clause_unit("t ≔ Q_0/I = 1 #h").as_deref(), Some("h"));
        assert_eq!(clause_unit("v = km/#h").as_deref(), Some("km/h"));
        // a name alone is a variable: t = V compares
        assert!(line("t = V").trail.is_none());
        assert!(matches!(&line("M ≔ 1 = 2 =").stmts[0], Stmt::Def { rhs: Ast::Cmp(..), .. }));
        assert!(matches!(&line("x ≔ 5 V + 3 mA").stmts[0], Stmt::Def { rhs: Ast::Add(..), .. }));
        assert_eq!(line("0 = 1 + frac(3, x) + y^(2) → solve(x)").trail.unwrap().1.solve, Some(vec!["x".to_string()]));
        let (l, u) = read("q(x: A, ..) ≔ 1 + frac(3, 3)x + y^(2) → simplify").unwrap();
        match &l.stmts[0] {
            Stmt::Def { head: Head::Fn { name, params, rest }, .. } => {
                assert_eq!(name, "q");
                assert_eq!(params.len(), 1);
                assert!(rest);
                assert!(matches!(&params[0].ty, Some(ParamType::Unit(x)) if u.text(x) == "A"));
            }
            s => panic!("{s:?}"),
        }
        // a type is made of unit bindings only, anything else is a default value
        match &line("q(x, n: 2) ≔ x^n").stmts[0] {
            Stmt::Def { head: Head::Fn { params, .. }, .. } => assert!(params[1].default.is_some() && params[1].ty.is_none()),
            s => panic!("{s:?}"),
        }
        // unit bindings are units; plain letters are variables
        assert!(matches!(&line("t ≔ 3 #h").stmts[0], Stmt::Def { rhs: Ast::Qty(..), .. }));
        assert!(matches!(&line("t ≔ 3 ⋅ h").stmts[0], Stmt::Def { rhs: Ast::Mul(..), .. }));
        let (l, u) = read("ρ ≔ 0.0178 Ω frac(#mm^(2), #m)").unwrap();
        match &l.stmts[0] {
            Stmt::Def { rhs: Ast::Qty(_, x), .. } => assert_eq!(u.text(x), "Ω·mm^2/m"),
            s => panic!("{s:?}"),
        }
        assert!(matches!(&line("Q_0 ≔ frac(3450, 2) #mAh").stmts[0], Stmt::Def { rhs: Ast::Qty(..), .. }));
        match &line("U ≔ 1.5 V").stmts[0] {
            Stmt::Def { rhs: Ast::Qty(n, _), .. } => assert_eq!(**n, Ast::Num("1.5".into(), Num::R(Rat::new(3, 2).unwrap()))),
            s => panic!("{s:?}"),
        }
        // a comma always separates: 0,1 is two numbers, not the decimal 0.1
        match &line("M ≔ mat(2,4; 0,1)").stmts[0] {
            Stmt::Def { rhs: Ast::Matrix(rs), .. } => assert_eq!((rs.len(), rs[1].len()), (2, 2)),
            s => panic!("{s:?}"),
        }
        assert_eq!(line("t = min, digits(2)").trail.unwrap().1.digits, Some(2));
        assert_eq!(line("x + y = 3, x - y = 1 → solve(x, y)").stmts.len(), 2);
        assert!(line("V ≔ 2 A = 2000 mA").trail.is_none());
        assert!(matches!(&line("Δr_R ≔ 2").stmts[0], Stmt::Def { head: Head::Var(n), .. } if n == "Δr_R"));
        assert!(matches!(&line("b ≔ Delta\"\"r_R · 2").stmts[0], Stmt::Def { rhs: Ast::Mul(f), .. } if f[0].1 == Ast::Name("Δr_R".into())));
        match &line("x ≔ Delta_T + DeltaT + alpha2").stmts[0] {
            Stmt::Def { rhs: Ast::Add(ts), .. } => {
                assert_eq!(ts[0].1, Ast::Name("Δ_T".into()));
                assert_eq!(ts[1].1, Ast::Name("DeltaT".into()));
                assert_eq!(ts[2].1, Ast::Name("α2".into()));
            }
            s => panic!("{s:?}"),
        }
    }

    #[test]
    fn floor_brackets() {
        // ⌊x⌋, as Typst writes floor(x): a factor like (x), also inside |…|
        let l = format!("{:?}", line("y := 2⌊x / 2⌋ + |⌊x⌋ - 1|"));
        assert_eq!(l.matches("Builtin(\"floor\"").count(), 2, "{l}");
        assert!(read("y := ⌊x").unwrap_err().contains("closing the floor"));
        let l = format!("{:?}", line("y := ⌈x / 2⌉ - ⌊x⌋"));
        assert!(l.contains("Builtin(\"ceil\"") && l.contains("Builtin(\"floor\""), "{l}");
        assert!(read("y := ⌈x⌋").unwrap_err().contains("closing the ceiling"));
    }

    #[test]
    fn names_are_never_units_or_operations() {
        // after ->, a name is no unit: units are bindings
        assert!(read("x -> m").unwrap_err().contains("a unit is a binding"));
        // symbols are no units of their own
        let plain = |s: &str| parse_line(s, &Ctx { units: &Units::default(), is_fn: &|_| false, is_var: &|_| false }).unwrap_err();
        assert!(plain("x ≔ 5 %").contains("#percent"));
        assert!(plain("x ≔ 90 °").contains("#deg"));
        // an operation's name is a name unless it's referenced: tri(t) calls a worksheet function
        assert!(matches!(&parse_line("y ≔ tri(t)", &Ctx { units: &Units::default(), is_fn: &|_| false, is_var: &|_| false }).unwrap().stmts[0], Stmt::Def { rhs: Ast::Call(..), .. }));
        assert!(matches!(&line("y ≔ tri(t)").stmts[0], Stmt::Def { rhs: Ast::Builtin(..), .. }));
    }

    #[test]
    fn no_scientific_notation() {
        // `1.12e4` is the coefficient 1.12 times the (undefined) variable e4, not a number
        match &line("x ≔ 1.12e4").stmts[0] {
            Stmt::Def { rhs: Ast::Mul(ts), .. } => {
                assert_eq!(ts[0].1, Ast::Num("1.12".into(), Num::R(Rat::new(28, 25).unwrap())));
                assert_eq!(ts[1].1, Ast::Name("e4".into()));
            }
            s => panic!("{s:?}"),
        }
        // other names keep their digits
        assert!(matches!(&line("x ≔ 2 t2").stmts[0], Stmt::Def { rhs: Ast::Mul(ts), .. } if ts[1].1 == Ast::Name("t2".into())));
        assert!(matches!(&line("x ≔ 2 e^2").stmts[0], Stmt::Def { rhs: Ast::Mul(..), .. }));
    }
}

/// A curve of a graph: an expression (`cos(w)`), or a function of what it's plotted against
/// (`P(U: V) = U^2/R`, `P(..) = U0^2/R_L`).
#[derive(Clone, Debug, PartialEq)]
pub struct Curve {
    /// The function's name and parameters, for a named curve.
    pub head: Option<(String, Vec<Param>, bool)>,
    pub body: Ast,
    /// Written with `:=`: the curve also defines what it names, as the same line in a worksheet does.
    pub defines: bool,
}

pub fn parse_curve(src: &str, c: &Ctx) -> Result<Curve, String> {
    let t = visible_tokens(src)?;
    let mut depth = 0i32;
    let mut at = None;
    for (i, k) in t.iter().enumerate() {
        match k {
            Tok::Op("(" | "[") => depth += 1,
            Tok::Op(")" | "]") => depth -= 1,
            Tok::Op("=") | Tok::Op(":=") if depth == 0 => {
                if at.is_some() {
                    return Err(format!("more than one '=' in '{src}'"));
                }
                at = Some(i);
            }
            _ => {}
        }
    }
    let body = |toks: &[Tok], defining: Option<String>| -> Result<Ast, String> {
        let mut p = P { t: toks.to_vec(), i: 0, c, defining: defining.into_iter().collect(), abs_depth: 0, recursion_depth: 0, in_exp: 0, integrand: Integrand::default() };
        let e = p.sum()?;
        if p.i < p.t.len() {
            return Err(format!("unexpected '{}' in '{src}'", tok_text(&p.t[p.i])));
        }
        Ok(e)
    };
    let Some(i) = at else { return Ok(Curve { head: None, body: body(&t, None)?, defines: false }) };
    let (lhs, rhs) = (&t[..i], &t[i + 1..]);
    // := (re)defines, so a function of that name already is no call: y(y) := X(y) after y(x) := …
    let defines = t[i] == Tok::Op(":=");
    match lhs {
        [Tok::Name(n), Tok::Op("("), inner @ .., Tok::Op(")")] if !is_builtin(n) && (defines || !(c.is_fn)(n)) => {
            let (ps, rest) = params(inner, c)?;
            Ok(Curve { head: Some((n.clone(), ps, rest)), body: body(rhs, None)?, defines })
        }
        // y := X(x): y is X(x) with x left free, plotted over x
        [Tok::Name(n)] if defines && !is_builtin(n) => Ok(Curve { head: Some((n.clone(), vec![], true)), body: body(rhs, None)?, defines }),
        [Tok::Name(n)] => Err(format!("NAMED:{n}")),
        _ => Err(format!("EQUATION:{}|{}", src_text(lhs), src_text(rhs))),
    }
}

/// The equations of a line, separated by `\` (a line break in math) outside quotes.
pub fn statements(src: &str) -> Vec<String> {
    let mut out = vec![];
    let mut buf = String::new();
    let mut quoted = false;
    for c in src.chars() {
        if c == '"' { quoted = !quoted; }
        if c == '\\' && !quoted {
            if !buf.trim().is_empty() { out.push(buf.trim().to_string()); }
            buf.clear();
            continue;
        }
        buf.push(c);
    }
    if !buf.trim().is_empty() { out.push(buf.trim().to_string()); }
    out
}
