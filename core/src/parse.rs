//! Reading equations: code lines (`t := Q0/I = [h]`) and math as the Typst side writes it out (`t ≔ Q_0/I = 1 h`,
//! unit names as `⦃mA⦄`, keywords as `⦅solve(x)⦆`).

use serde::{Deserialize, Serialize};

use crate::num::{Num, Rat};
use crate::units::{UnitExpr, Units};

/// Where a line comes from: math decides units by how they are written, code by brackets.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Math,
    Code,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Tok {
    Num(String),
    Name(String),
    /// "text" in math: a name of several letters, a unit, or `as`.
    Quoted(String),
    /// A unit name from solveng's unit words or unit("…"): always a unit.
    UnitWord(String),
    /// [5 V] in code.
    Bracket(String),
    /// A keyword of a clause, ⦅digits(3)⦆ in math.
    Kw(String),
    /// @U: the formula of U.
    At(String),
    Op(&'static str),
}

const OPS: &[(&str, &str)] = &[
    (":=", ":="), ("≔", ":="), ("->", "->"), ("→", "->"), ("<=", "≤"), (">=", "≥"), ("!=", "≠"), ("≤", "≤"), ("≥", "≥"),
    ("≠", "≠"), ("=", "="), ("<", "<"), (">", ">"), ("+", "+"), ("-", "-"), ("−", "-"), ("*", "*"), ("∗", "*"),
    ("·", "·"), ("⋅", "·"), ("×", "×"), ("/", "/"), ("^", "^"), ("(", "("), (")", ")"), ("[", "("), ("]", ")"),
    (",", ","), (";", ";"), (":", ":"), ("..", ".."), ("…", ".."), ("|", "|"), ("±", "±"),
];

pub use solveng_names::GREEK;

pub fn greek(name: &str) -> String { solveng_names::normalize(name) }

pub fn lex(s: &str, mode: Mode) -> Result<Vec<Tok>, String> {
    let cs: Vec<char> = s.chars().collect();
    let mut out: Vec<Tok> = vec![];
    let mut i = 0;
    while i < cs.len() {
        let c = cs[i];
        if c.is_whitespace() {
            i += 1;
            continue;
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
            // a decimal point, or a decimal comma between digits (0,95)
            if i + 1 < cs.len() && (cs[i] == '.' || cs[i] == ',') && cs[i + 1].is_ascii_digit() {
                i += 1;
                while i < cs.len() && cs[i].is_ascii_digit() {
                    i += 1;
                }
            } else if i < cs.len() && cs[i] == '.' && !cs.get(i + 1).is_some_and(|&d| d == '.') {
                i += 1;
            }
            // 1e-3, but not 2 e (Euler's number) or 3 eV
            if i < cs.len() && (cs[i] == 'e' || cs[i] == 'E') {
                let mut j = i + 1;
                if j < cs.len() && matches!(cs[j], '-' | '+' | '−') {
                    j += 1;
                }
                if j < cs.len() && cs[j].is_ascii_digit() && !cs.get(i + 1).is_some_and(|c| c.is_alphabetic()) {
                    i = j;
                    while i < cs.len() && cs[i].is_ascii_digit() {
                        i += 1;
                    }
                }
            }
            out.push(Tok::Num(cs[st..i].iter().collect::<String>().replace(',', ".").replace('−', "-")));
            continue;
        }
        if solveng_names::is_start(c) || c == '"' {
            if c == '"' && cs.get(i + 1) == Some(&'"') { i += 2; continue; }
            let quoted = c == '"';
            let name = solveng_names::scan(&cs, i)?.ok_or("expected a variable name")?;
            i = name.end;
            let n = name.value;
            if let Some(index) = n.strip_prefix("integral_").filter(|s| !s.is_empty()) {
                out.push(Tok::Name("integral_".into()));
                out.extend(lex(index, mode)?);
            } else if quoted || n == "as" { out.push(Tok::Quoted(n)); }
            else { out.push(Tok::Name(n)); }
            continue;
        }
        match c {
            '∞' => {
                out.push(Tok::Name("infinity".into()));
                i += 1;
                continue;
            }
            '⦃' => {
                let st = i + 1;
                while i < cs.len() && cs[i] != '⦄' {
                    i += 1;
                }
                out.push(Tok::UnitWord(cs[st..i.min(cs.len())].iter().collect()));
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
            '[' if mode == Mode::Code => {
                let st = i + 1;
                let mut d = 0;
                while i < cs.len() {
                    if cs[i] == '[' {
                        d += 1;
                    } else if cs[i] == ']' {
                        d -= 1;
                        if d == 0 {
                            break;
                        }
                    }
                    i += 1;
                }
                if i >= cs.len() {
                    return Err("missing ']'".into());
                }
                out.push(Tok::Bracket(cs[st..i].iter().collect::<String>().trim().to_string()));
                i += 1;
                continue;
            }
            '@' => {
                i += 1;
                let name = solveng_names::scan(&cs, i)?.ok_or("a variable name after @")?;
                i = name.end;
                out.push(Tok::At(name.value));
                continue;
            }
            '°' => {
                i += 1;
                if i < cs.len() && cs[i] == 'C' {
                    i += 1;
                    out.push(Tok::UnitWord("°C".into()));
                } else {
                    out.push(Tok::UnitWord("°".into()));
                }
                continue;
            }
            '%' | '‰' => {
                i += 1;
                out.push(Tok::UnitWord(c.to_string()));
                continue;
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
    Call(String, Vec<Ast>),
    Op(crate::env::Operator, Vec<Ast>),
    Abs(Box<Ast>),
    /// a = b < c: a chain of comparisons, 1 where all hold.
    Cmp(Box<Ast>, Vec<(Rel, Ast)>),
    /// e as u: e's SI number as a value in u.
    Cast(Box<Ast>, UnitExpr),
    /// @U: the formula U was computed by.
    Inline(String),
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
}

#[derive(Clone, Debug, PartialEq)]
pub enum Head {
    Var(String),
    Fn { name: String, params: Vec<Param>, rest: bool },
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

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Clause {
    pub unit: Option<UnitExpr>,
    pub solve: Option<Vec<String>>,
    pub simplify: bool,
    pub frac: bool,
    pub digits: Option<u32>,
    pub sci: bool,
    pub eng: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Line {
    pub stmts: Vec<Stmt>,
    pub trail: Option<(Trail, Clause)>,
    /// Unit names written as single letters after numbers, which math sets upright.
    pub unit_letters: Vec<String>,
}

const BUILTIN_FNS: &[&str] = &[
    "sqrt", "exp", "ln", "log", "sin", "cos", "tan", "asin", "acos", "atan", "sinh", "cosh", "tanh", "abs", "arcsin",
    "arccos", "arctan", "cot", "sec", "csc", "diff", "integrate", "vec", "at", "rect", "tri", "sgn", "cases",
];
pub fn is_builtin(n: &str) -> bool {
    BUILTIN_FNS.contains(&n)
}
const KEYWORDS: &[&str] = &["solve", "simplify", "digits", "sci", "eng", "frac"];

/// What the parser needs to know about the document so far.
pub struct Ctx<'a> {
    pub units: &'a Units,
    pub mode: Mode,
    pub operators: &'a [crate::env::Operator],
    /// Names of user functions (and the one being defined).
    pub is_fn: &'a dyn Fn(&str) -> bool,
    /// Names of defined variables.
    pub is_var: &'a dyn Fn(&str) -> bool,
    /// strict: unit names can't be variables, units in math must be quoted or unit words.
    pub strict: bool,
}

struct P<'a, 'b> {
    t: Vec<Tok>,
    i: usize,
    c: &'b Ctx<'a>,
    letters: Vec<String>,
    /// Functions being defined in this line (their own name in a recursive call is refused later).
    defining: Vec<String>,
    abs_depth: usize,
    /// Inside an exponent: numbers take no units, casts belong to the power.
    in_exp: usize,
}

fn s_op(t: Option<&Tok>) -> Option<&'static str> {
    match t {
        Some(Tok::Op(o)) => Some(o),
        _ => None,
    }
}

impl<'a, 'b> P<'a, 'b> {
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

    /// Whether the token at k can be read as a unit right after a number.
    fn unit_at(&self, k: usize) -> Option<String> {
        if self.unit_frac(k).is_some() {
            return Some("frac".into());
        }
        match self.t.get(k)? {
            Tok::UnitWord(u) => Some(u.clone()),
            Tok::Quoted(q) if q != "as" && self.c.units.known(q) => Some(q.clone()),
            // as after a number is attoseconds
            Tok::Quoted(q) if q == "as" => Some(q.clone()),
            Tok::Name(n) if self.c.units.known(n) && !(self.c.strict && self.c.mode == Mode::Math) => Some(n.clone()),
            Tok::Name(n) if n == "as" => Some(n.clone()),
            _ => None,
        }
    }

    /// A unit run after a number: m/s^2, Ω mm^2/m, kg m. Returns None if there's none at i.
    fn unit_run(&mut self) -> Result<Option<UnitExpr>, String> {
        let mut terms: Vec<(String, i32)> = vec![];
        let mut below = false;
        loop {
            let k = self.i;
            if !terms.is_empty() && self.op() == Some("/") && self.unit_at(k + 1).is_some() && !self.cast_at(k + 1) {
                below = true;
                self.i += 1;
                continue;
            }
            // as after a unit casts: 2 A as mA
            if !terms.is_empty() && self.cast_at(k) {
                break;
            }
            // math writes mm^2/m as a fraction: frac(⦃mm⦄^(2), m)
            if let Some((num, den, end)) = self.unit_frac(k) {
                terms.extend(num.into_iter().map(|(n, e)| (n, if below { -e } else { e })));
                terms.extend(den.into_iter().map(|(n, e)| (n, if below { e } else { -e })));
                self.i = end;
                continue;
            }
            let Some(u) = self.unit_at(k) else { break };
            if self.c.mode == Mode::Code {
                if let Tok::Name(n) = &self.t[k] {
                    if (self.c.is_var)(n) {
                        return Err(format!(
                            "'{n}' is both a defined variable and a unit – write [… {n}] for the unit, or use * or parentheses for the variable (3*{n}, 1/({n}))"
                        ));
                    }
                }
            }
            if let Tok::Name(n) = &self.t[k] {
                if n.chars().count() == 1 {
                    self.letters.push(n.clone());
                }
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
                Tok::UnitWord(_) => true,
                Tok::Name(n) | Tok::Quoted(n) => self.c.units.known(n),
                Tok::Num(_) => true,
                Tok::Op(o) => matches!(*o, "^" | "(" | ")" | "-" | "*" | "·"),
                _ => false,
            }) || !toks.iter().any(|t| !matches!(t, Tok::Num(_) | Tok::Op(_))) {
                return None;
            }
            let s: String = toks.iter().map(|t| match t {
                Tok::UnitWord(u) | Tok::Name(u) | Tok::Quoted(u) => format!(" {u}"),
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
    fn cast_at(&self, k: usize) -> bool {
        matches!(self.t.get(k), Some(Tok::Quoted(q)) if q == "as") || matches!(self.t.get(k), Some(Tok::Name(n)) if n == "as")
    }

    /// The unit of a cast or a parameter type: A, mA, (A*as)/V, m^2, [mA], "A" "as".
    fn unit_spec(&mut self) -> Result<UnitExpr, String> {
        let st = self.i;
        let mut s = String::new();
        let mut depth = 0;
        loop {
            match self.peek() {
                Some(Tok::Bracket(b)) if s.is_empty() => {
                    s = b.clone();
                    self.i += 1;
                    break;
                }
                Some(Tok::UnitWord(u)) => s += &format!(" {u}"),
                Some(Tok::Quoted(q)) => s += &format!(" {q}"),
                Some(Tok::Name(n)) if self.c.units.known(n) || n == "any" => s += &format!(" {n}"),
                Some(Tok::Num(n)) if s.is_empty() && n == "1" => s += "1",
                Some(Tok::Name(n)) if s.is_empty() && n != "as" => return Err(format!("unknown unit '{n}'")),
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
            return Err("a unit after 'as' to cast to, like U as A".into());
        }
        if s.trim() == "any" {
            return Ok(UnitExpr::one("any"));
        }
        let p = self.c.units.parse_simplified(s.trim()).map_err(|e| format!("the unit: {e}"))?;
        Ok(p.u)
    }

    fn cmp(&mut self) -> Result<Ast, String> {
        let a = self.sum()?;
        let mut rest = vec![];
        while let Some(r) = self.op().and_then(Rel::of) {
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
        loop {
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
            Some(Tok::Op(o)) => matches!(*o, "(" | "|"),
            Some(Tok::Quoted(q)) => q != "as",
            Some(Tok::Kw(_)) => false,
            Some(Tok::Name(n)) => n != "as" && n != "dif",
            _ => true,
        }
    }
    fn term(&mut self) -> Result<Ast, String> {
        let mut fs: Vec<(MulOp, Ast)> = vec![(MulOp::Implicit, self.unary()?)];
        loop {
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
                // | ends an absolute value here
                Some("|") if self.abs_depth > 0 => break,
                _ if self.starts_factor() => fs.push((MulOp::Implicit, self.unary()?)),
                _ => break,
            }
        }
        Ok(collapse(fs))
    }
    fn unary(&mut self) -> Result<Ast, String> {
        if self.eat("-") {
            return Ok(Ast::Neg(Box::new(self.unary()?)));
        }
        if self.eat("+") {
            return self.unary();
        }
        self.power()
    }
    fn power(&mut self) -> Result<Ast, String> {
        let b = self.atom()?;
        let mut e = if self.eat("^") {
            self.in_exp += 1;
            let x = if self.eat("-") { self.power().map(|x| Ast::Neg(Box::new(x))) } else { self.power() };
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
        // U^2 as V: the cast is on the term with its power
        while self.cast_at(self.i) {
            self.i += 1;
            let u = self.unit_spec()?;
            e = Ast::Cast(Box::new(e), u);
        }
        Ok(e)
    }
    fn args(&mut self, name: &str) -> Result<Vec<Ast>, String> {
        let mut args = vec![];
        if self.eat(")") {
            return Ok(args);
        }
        loop {
            args.push(self.cmp()?);
            if self.eat(",") {
                continue;
            }
            self.expect(")", &format!("after the arguments of {name}"))?;
            return Ok(args);
        }
    }
    fn atom(&mut self) -> Result<Ast, String> {
        let t = self.peek().cloned().ok_or("unexpected end of the expression")?;
        self.i += 1;
        match t {
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
                    return Err(format!("'{u}' is neither a unit nor a defined variable"));
                }
                self.i -= 1;
                let u = self.unit_run()?.unwrap();
                Ok(Ast::Unit(u))
            }
            Tok::Quoted(q) => {
                if self.op() == Some("(")
                    && let Some(op) = self.c.operators.iter().find(|op| greek(&op.name) == greek(&q)).cloned() {
                        self.i += 1;
                        return Ok(Ast::Op(op, self.args(&q)?));
                    }
                if q == "as" {
                    return Err("a value before 'as' to cast".into());
                }
                // a quoted unit on its own is one of it; else a name of several letters
                if self.c.units.known(&q) && !(self.c.is_var)(&q) {
                    self.i -= 1;
                    let u = self.unit_run()?.unwrap();
                    return Ok(Ast::Unit(u));
                }
                self.name(greek(&q))
            }
            Tok::Name(n) => self.name(n),
            Tok::Bracket(b) => self.bracket(&b),
            Tok::At(n) => Ok(Ast::Inline(n)),
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
                let e = self.sum()?;
                self.abs_depth -= 1;
                self.expect("|", "closing the absolute value")?;
                Ok(Ast::Abs(Box::new(e)))
            }
            Tok::Op(o) => Err(format!("unexpected '{o}'")),
            Tok::Kw(k) => Err(format!("unexpected keyword '{k}'")),
        }
    }
    fn name(&mut self, n: String) -> Result<Ast, String> {
        if n == "dif" && self.eat("/") {
            self.expect("(", "around the differential")?;
            if !matches!(self.peek(), Some(Tok::Name(n)) if n == "dif") { return Err("expected dif in the denominator".into()); }
            self.i += 1;
            let Some(Tok::Name(var)) = self.peek().cloned() else { return Err("a differentiation variable after dif".into()); };
            self.i += 1;
            self.expect(")", "after the differential")?;
            return Ok(Ast::Call("diff".into(), vec![self.sum()?, Ast::Name(var)]));
        }
        if n == "integral_" {
            let lo = self.atom()?;
            self.expect("^", "before the upper integral bound")?;
            let hi = self.atom()?;
            let body = self.sum()?;
            if !matches!(self.peek(), Some(Tok::Name(n)) if n == "dif") { return Err("an integral ends with dif and its variable".into()); }
            self.i += 1;
            let Some(Tok::Name(var)) = self.peek().cloned() else { return Err("an integration variable after dif".into()); };
            self.i += 1;
            return Ok(Ast::Call("integrate".into(), vec![body, Ast::Name(var), lo, hi]));
        }
        // Typst's frac(a, b), as math writes a fraction out: one factor
        if n == "frac" && self.op() == Some("(") {
            self.i += 1;
            let x = std::mem::take(&mut self.in_exp);
            let a = self.args("frac");
            self.in_exp = x;
            let mut a = a?;
            if a.len() != 2 {
                return Err("frac takes two arguments".into());
            }
            let d = a.pop().unwrap();
            return Ok(Ast::Div(Box::new(a.pop().unwrap()), Box::new(d)));
        }
        if self.op() == Some("(") {
            self.i += 1;
            let args = self.args(&n)?;
            return Ok(Ast::Call(n, args));
        }
        // sin x: a built-in function applies to what follows
        if is_builtin(&n) && self.starts_factor() {
            let a = self.power()?;
            return Ok(Ast::Call(n, vec![a]));
        }
        Ok(Ast::Name(n))
    }
    /// [5 V], [A], [3450/2 mAh] in code: the unit is meant even if there is a variable with its name.
    fn bracket(&mut self, b: &str) -> Result<Ast, String> {
        // the numeric part: digits, operators and parentheses before the first letter
        let idx = b
            .char_indices()
            .find(|&(i, c)| {
                (c.is_alphabetic() && !((c == 'e' || c == 'E') && i > 0 && b[..i].ends_with(|d: char| d.is_ascii_digit()) && b[i + 1..].starts_with(|d: char| d.is_ascii_digit() || d == '-')))
                    || "°%‰µΩ".contains(c)
            })
            .map(|(i, _)| i)
            .unwrap_or(b.len());
        let (np, up) = (b[..idx].trim(), b[idx..].trim());
        if up.is_empty() {
            return Err(format!("[{b}]: a unit in the brackets, like [5 V] or [A]"));
        }
        let u = self.c.units.parse(up, false)?;
        if np.is_empty() {
            return Ok(Ast::Unit(u.u));
        }
        let sub = Ctx { operators: &[], units: self.c.units, mode: Mode::Code, is_fn: &|_| false, is_var: &|_| false, strict: false };
        let toks = lex(np, Mode::Code)?;
        let mut p = P { t: toks, i: 0, c: &sub, letters: vec![], defining: vec![], abs_depth: 0, in_exp: 0 };
        let n = p.sum()?;
        if p.i < p.t.len() || !numeric(&n) {
            return Err(format!("cannot read the number '{np}' in [{b}]"));
        }
        Ok(Ast::Qty(Box::new(n), u.u))
    }
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
            Tok::Op("(") => depth += 1,
            Tok::Op(")") => depth -= 1,
            Tok::Op("->") if depth == 0 => arrow = Some(i),
            Tok::Op("=") if depth == 0 => eq = Some(i),
            _ => {}
        }
    }
    if let Some(i) = arrow {
        let cl = clause(&t[i + 1..], c)?.ok_or("-> takes keywords and a unit: -> solve(x), -> simplify, -> digits(3)")?;
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
    let mut cl = Clause::default();
    if t.is_empty() {
        return Ok(Some(cl));
    }
    let mut groups: Vec<Vec<Tok>> = vec![vec![]];
    let mut depth = 0;
    for k in t {
        match k {
            Tok::Op("(") => depth += 1,
            Tok::Op(")") => depth -= 1,
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
        // a keyword: ⦅solve(x)⦆ from math, solve(x) in code
        if let [Tok::Kw(k)] = g.as_slice() {
            kws.push(k.clone());
            continue;
        }
        if let Some(Tok::Name(n)) = g.first() {
            if KEYWORDS.contains(&n.as_str()) && (g.len() == 1 || s_op(g.get(1)) == Some("(")) {
                let mut s = n.clone();
                for k in &g[1..] {
                    s += &tok_text(k);
                }
                kws.push(s);
                continue;
            }
        }
        // a unit: names (units only), powers, /, brackets; no numbers but exponents. A single letter is a unit only
        // after a 1 (t = 1 h) or with a unit of more letters (km/h): alone it's a variable, t = h compares
        let mut s = String::new();
        let mut prev_pow = false;
        let mut named = matches!(g.first(), Some(Tok::Num(n)) if n == "1");
        for k in &g {
            match k {
                Tok::Num(n) if prev_pow || n == "1" => s += n,
                Tok::Num(_) => return Ok(None),
                Tok::Name(n) | Tok::Quoted(n) | Tok::UnitWord(n) => {
                    if !c.units.known(n) {
                        return Ok(None);
                    }
                    named |= !matches!(k, Tok::Name(_)) || n.chars().count() > 1;
                    s += &format!(" {n}");
                }
                Tok::Bracket(_) => named = true,
                Tok::Op(o @ ("*" | "·" | "/" | "^" | "(" | ")")) => s += o,
                Tok::Op("-") if prev_pow => s += "-",
                _ => return Ok(None),
            }
            if let Tok::Bracket(b) = k {
                s += &format!(" {b}");
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
            "solve" => {
                if args.is_empty() {
                    return Err("solve: what to solve for, solve(x) or solve(x, y)".into());
                }
                cl.solve = Some(args);
            }
            "digits" => {
                let d: u32 = args.first().and_then(|a| a.parse().ok()).filter(|d| (1..=15).contains(d)).ok_or_else(|| format!("digits: 1…15, got {}", args.join(", ")))?;
                cl.digits = Some(d);
            }
            "simplify" | "sci" | "eng" | "frac" => {
                if !args.is_empty() {
                    return Err(format!("{name} takes no arguments"));
                }
                match name.as_str() {
                    "simplify" => cl.simplify = true,
                    "sci" => cl.sci = true,
                    "eng" => cl.eng = true,
                    _ => cl.frac = true,
                }
            }
            _ => return Err(format!("unknown keyword '{name}' (keywords: {})", KEYWORDS.join(", "))),
        }
    }
    if units.len() > 1 {
        return Err(format!("one unit per result, got {} and {}", units[0], units[1]));
    }
    if let Some(u) = units.pop() {
        let u = u.trim_start_matches('[').trim_end_matches(']');
        cl.unit = Some(c.units.parse(u, false)?.u);
    }
    Ok(Some(cl))
}

fn tok_text(t: &Tok) -> String {
    match t {
        Tok::Num(s) | Tok::Name(s) => s.clone(),
        Tok::Quoted(s) => format!("\"{s}\""),
        Tok::UnitWord(s) => s.clone(),
        Tok::Bracket(s) => format!("[{s}]"),
        Tok::Kw(s) => s.clone(),
        Tok::At(s) => format!("@{s}"),
        Tok::Op(o) => o.to_string(),
    }
}

/// The parameters of a function being defined: I: mA, R, x: any, ..
fn params(t: &[Tok], c: &Ctx) -> Result<(Vec<Param>, bool), String> {
    let mut groups: Vec<Vec<Tok>> = vec![vec![]];
    let mut depth = 0;
    for k in t {
        match k {
            Tok::Op("(") => depth += 1,
            Tok::Op(")") => depth -= 1,
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
        let ty = if g.len() > 1 {
            if s_op(g.get(1)) != Some(":") {
                return Err(format!("the parameters are names, optionally with a unit (I: mA), got '{}'", g.iter().map(tok_text).collect::<Vec<_>>().join(" ")));
            }
            let sub: Vec<Tok> = g[2..].to_vec();
            let mut p = P { t: sub, i: 0, c, letters: vec![], defining: vec![], abs_depth: 0, in_exp: 0 };
            let u = p.unit_spec().map_err(|e| format!("the unit of {name}: {}", e.trim_start_matches("the unit: ")))?;
            if p.i < p.t.len() {
                return Err(format!("the unit of {name}: can't read '{}'", p.t[p.i..].iter().map(tok_text).collect::<Vec<_>>().join(" ")));
            }
            Some(if u.0.len() == 1 && u.0[0].0 == "any" { ParamType::Any } else { ParamType::Unit(u) })
        } else {
            None
        };
        ps.push(Param { name, ty });
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

/// Reads a line: statements separated by ; (or , in math), an optional trailing = or -> with its clause.
pub fn parse_line(src: &str, c: &Ctx) -> Result<Line, String> {
    let src = if c.mode == Mode::Code { src.split("//").next().unwrap_or("") } else { src };
    let t = lex(src, c.mode)?;
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
            Tok::Op("(") => depth += 1,
            Tok::Op(")") => depth -= 1,
            Tok::Op(";") | Tok::Op(",") if depth == 0 => {
                parts.push(&head[st..i]);
                st = i + 1;
            }
            _ => {}
        }
    }
    parts.push(&head[st..]);
    let mut stmts = vec![];
    let mut letters = vec![];
    for p in parts {
        if p.is_empty() {
            continue;
        }
        let (s, l) = stmt(p, c)?;
        stmts.push(s);
        letters.extend(l);
    }
    Ok(Line { stmts, trail, unit_letters: letters })
}

fn stmt(t: &[Tok], c: &Ctx) -> Result<(Stmt, Vec<String>), String> {
    let def = t.iter().position(|k| *k == Tok::Op(":="));
    let mut p = P { t: vec![], i: 0, c, letters: vec![], defining: vec![], abs_depth: 0, in_exp: 0 };
    if let Some(d) = def {
        let lhs = &t[..d];
        let rhs = &t[d + 1..];
        if rhs.is_empty() {
            return Err("nothing after :=".into());
        }
        let head = match lhs {
            [Tok::Name(n)] => Head::Var(n.clone()),
            [Tok::Quoted(n)] => Head::Var(greek(n)),
            [Tok::Name(n), Tok::Op("("), inner @ .., Tok::Op(")")] | [Tok::Quoted(n), Tok::Op("("), inner @ .., Tok::Op(")")] => {
                let n = greek(n);
                let (params, rest) = params(inner, c)?;
                p.defining.push(n.clone());
                Head::Fn { name: n, params, rest }
            }
            [Tok::Name(n), Tok::Op(":"), ..] => {
                let u = lhs[2..].iter().map(tok_text).collect::<Vec<_>>().join(" ");
                return Err(format!("'{n}: {u}' – a variable has the unit of its value, so it takes no type: write {n} := … = {u} to show it in {u} (types are for a function's parameters, f(x: {u}))"));
            }
            [Tok::Name(n), Tok::Op("("), ..]
                if lhs.iter().rposition(|k| *k == Tok::Op(":")).is_some_and(|i| i > 2 && lhs[i - 1] == Tok::Op(")")) =>
            {
                return Err(format!("{n}(…): … – a function returns what its body gives, so it takes no type: cast the body to give it that unit, {n}(…) := (…) as …"));
            }
            _ => return Err(format!("'{}' can't be defined – := defines a name (x := …) or a function (f(x) := …)", src_text(lhs))),
        };
        p.t = rhs.to_vec();
        let e = p.cmp()?;
        if p.i < p.t.len() {
            return Err(format!("unexpected '{}'", tok_text(&p.t[p.i])));
        }
        if c.strict {
            let names: Vec<&str> = match &head {
                Head::Var(n) => vec![n],
                Head::Fn { name, params, .. } => std::iter::once(name.as_str()).chain(params.iter().map(|x| x.name.as_str())).collect(),
            };
            if let Some(n) = names.into_iter().find(|n| c.units.known(n)) {
                return Err(format!("'{n}' is a unit and can't be a variable name (option strict) – pick another name like {n}_1, or remove the unit with worksheet-config(units: ({n}: none))"));
            }
        }
        return Ok((Stmt::Def { head, rhs: e, src: src_text(rhs) }, p.letters));
    }
    p.t = t.to_vec();
    let e = p.cmp()?;
    if p.i < p.t.len() {
        let k = &p.t[p.i];
        if *k == Tok::Op("..") {
            return Err("'..' stands only in the parameters of a function being defined, f(x, ..) := …".into());
        }
        return Err(format!("unexpected '{}'", tok_text(k)));
    }
    Ok((Stmt::Expr(e), p.letters))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(s: &str, mode: Mode) -> Line {
        let u = Units::default();
        let c = Ctx { operators: &[], units: &u, mode, is_fn: &|n| n == "f" || n == "q", is_var: &|n| n == "h", strict: false };
        parse_line(s, &c).unwrap()
    }
    fn err(s: &str, mode: Mode) -> String {
        let u = Units::default();
        let c = Ctx { operators: &[], units: &u, mode, is_fn: &|_| false, is_var: &|n| n == "h", strict: false };
        parse_line(s, &c).unwrap_err()
    }

    #[test]
    fn math() {
        let l = line("t ≔ Q_0/I = 1 h", Mode::Math);
        assert_eq!(l.trail.as_ref().unwrap().0, Trail::Eval);
        assert_eq!(l.trail.unwrap().1.unit, Some(UnitExpr::one("h")));
        // a letter alone is a variable: t = V compares; units of more letters are units alone
        let l = line("t = V", Mode::Math);
        assert!(l.trail.is_none());
        let l = line("v = ⦃km⦄/h", Mode::Math);
        assert!(l.trail.unwrap().1.unit.is_some());
        let l = line("t := Q0/I = [h]", Mode::Code);
        assert!(l.trail.unwrap().1.unit.is_some());
        let l = line("M ≔ 1 = 2 =", Mode::Math);
        assert!(matches!(&l.stmts[0], Stmt::Def { rhs: Ast::Cmp(..), .. }));
        let l = line("x ≔ 5 V + 3 ⦃mA⦄", Mode::Math);
        assert!(matches!(&l.stmts[0], Stmt::Def { rhs: Ast::Add(..), .. }));
        let l = line("0 = 1 + frac(3, x) + y^(2) → ⦅solve(x)⦆", Mode::Math);
        assert_eq!(l.trail.unwrap().1.solve, Some(vec!["x".to_string()]));
        let l = line("q(x: A, ..) ≔ 1 + frac(3, 3)x + y^(2) → ⦅simplify⦆", Mode::Math);
        match &l.stmts[0] {
            Stmt::Def { head: Head::Fn { name, params, rest }, .. } => {
                assert_eq!(name, "q");
                assert_eq!(params.len(), 1);
                assert!(rest);
                assert_eq!(params[0].ty, Some(ParamType::Unit(UnitExpr::one("A"))));
            }
            s => panic!("{s:?}"),
        }
        // a unit after a number, whatever is defined; 3 ⋅ h is a variable
        let l = line("t ≔ 3 h", Mode::Math);
        assert!(matches!(&l.stmts[0], Stmt::Def { rhs: Ast::Qty(..), .. }));
        let l = line("t ≔ 3 ⋅ h", Mode::Math);
        assert!(matches!(&l.stmts[0], Stmt::Def { rhs: Ast::Mul(..), .. }));
        let l = line("ρ ≔ 0.0178 Ω frac(⦃mm⦄^(2), m)", Mode::Math);
        match &l.stmts[0] {
            Stmt::Def { rhs: Ast::Qty(_, u), .. } => assert_eq!(u.text(), "Ω·mm^2/m"),
            s => panic!("{s:?}"),
        }
        let l = line("Q_0 ≔ frac(3450, 2) ⦃mAh⦄", Mode::Math);
        assert!(matches!(&l.stmts[0], Stmt::Def { rhs: Ast::Qty(..), .. }));
        let l = line("U ≔ 1,5 V", Mode::Math);
        match &l.stmts[0] {
            Stmt::Def { rhs: Ast::Qty(n, _), .. } => assert_eq!(**n, Ast::Num("1.5".into(), Num::R(Rat::new(3, 2).unwrap()))),
            s => panic!("{s:?}"),
        }
        let l = line("t = ⦃min⦄, ⦅digits(2)⦆", Mode::Math);
        assert_eq!(l.trail.unwrap().1.digits, Some(2));
        let l = line("x + y = 3, x - y = 1 → ⦅solve(x, y)⦆", Mode::Math);
        assert_eq!(l.stmts.len(), 2);
        let l = line("V ≔ 2 A = 2000 ⦃mA⦄", Mode::Math);
        assert!(l.trail.is_none());
        let l = line("Δr_R ≔ 2", Mode::Math);
        assert!(matches!(&l.stmts[0], Stmt::Def { head: Head::Var(n), .. } if n == "Δr_R"));
    }

    #[test]
    fn code() {
        let l = line("t := Q0/I = [h], digits(3)", Mode::Code);
        let cl = l.trail.unwrap().1;
        assert_eq!(cl.digits, Some(3));
        assert_eq!(cl.unit, Some(UnitExpr::one("h")));
        let l = line("P := [5 V] I =", Mode::Code);
        assert!(l.trail.is_some());
        let l = line("a + b = 3; a - b = 1 -> solve(a, b)", Mode::Code);
        assert_eq!(l.stmts.len(), 2);
        assert!(err("y := 3 h", Mode::Code).contains("both a defined variable and a unit"));
        let l = line("J := U as mA", Mode::Code);
        assert!(matches!(&l.stmts[0], Stmt::Def { rhs: Ast::Cast(..), .. }));
        let l = line("a := 3 as", Mode::Code);
        assert!(matches!(&l.stmts[0], Stmt::Def { rhs: Ast::Qty(..), .. }));
        let l = line("L := U^2 as V", Mode::Code);
        assert!(matches!(&l.stmts[0], Stmt::Def { rhs: Ast::Cast(..), .. }));
        let l = line("y := f(3) // comment", Mode::Code);
        assert!(matches!(&l.stmts[0], Stmt::Def { rhs: Ast::Call(..), .. }));
        let l = line("b := Delta\"\"r_R * 2", Mode::Code);
        assert!(matches!(&l.stmts[0], Stmt::Def { rhs: Ast::Mul(f), .. } if f[0].1 == Ast::Name("Δr_R".into())));
        let l = line("x := Delta_T + DeltaT + alpha2", Mode::Code);
        match &l.stmts[0] {
            Stmt::Def { rhs: Ast::Add(ts), .. } => {
                assert_eq!(ts[0].1, Ast::Name("Δ_T".into()));
                assert_eq!(ts[1].1, Ast::Name("DeltaT".into()));
                assert_eq!(ts[2].1, Ast::Name("α2".into()));
            }
            s => panic!("{s:?}"),
        }
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
    let t = lex(src, c.mode)?;
    let mut depth = 0i32;
    let mut at = None;
    for (i, k) in t.iter().enumerate() {
        match k {
            Tok::Op("(") => depth += 1,
            Tok::Op(")") => depth -= 1,
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
        let mut p = P { t: toks.to_vec(), i: 0, c, letters: vec![], defining: defining.into_iter().collect(), abs_depth: 0, in_exp: 0 };
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

/// Equation separators outside grouping; unfinished code lines continue on the next line.
pub fn statements(src: &str, mode: Mode) -> Vec<String> {
    let mut out = vec![];
    let mut buf = String::new();
    let (mut depth, mut quoted, mut comment) = (0i32, false, false);
    let mut chars = src.chars().peekable();
    while let Some(c) = chars.next() {
        if comment {
            if c != '\n' { continue; }
            comment = false;
        }
        if !quoted && c == '/' && chars.peek() == Some(&'/') {
            chars.next();
            comment = true;
            continue;
        }
        if c == '"' { quoted = !quoted; }
        if !quoted {
            if c == '(' || c == '[' { depth += 1; }
            if c == ')' || c == ']' { depth -= 1; }
            let newline = c == '\n' && mode == Mode::Code && depth == 0;
            let unfinished = buf.trim_end().ends_with(['+', '-', '*', '/', '^', ':', ',', '(']) || buf.trim_end().ends_with(":=");
            if c == '\\' || (newline && !unfinished) {
                if !buf.trim().is_empty() { out.push(buf.trim().to_string()); }
                buf.clear();
                continue;
            }
        }
        buf.push(c);
    }
    if !buf.trim().is_empty() { out.push(buf.trim().to_string()); }
    out
}
