//! Built-in operations: what they are called, which arguments they take, and how a name reaches one.
//!
//! An operation is only ever reached by an explicit reference: a binding of solveng's `operations`, `math.op` or
//! `operation`; a plain name is a name.
//!
//! Arguments bind as in Typst: positional arguments to the positional parameters in order, named ones
//! (`step: 10 ms`) to the named parameters, which have defaults.

use std::collections::HashMap;


/// One parameter of an operation.
#[derive(Clone, Copy, Debug)]
pub struct Param {
    pub name: &'static str,
    /// Positional parameters can be required; named ones are optional.
    pub required: bool,
}

/// What an operation takes.
#[derive(Clone, Copy, Debug)]
pub struct Sig {
    pub positional: &'static [Param],
    /// Takes any number of positional arguments (vec, cases).
    pub variadic: bool,
    pub named: &'static [&'static str],
}

/// A built-in operation, under its canonical name.
#[derive(Clone, Copy, Debug)]
pub struct Builtin {
    pub name: &'static str,
    pub sig: Sig,
}

const fn req(name: &'static str) -> Param {
    Param { name, required: true }
}
const fn opt(name: &'static str) -> Param {
    Param { name, required: false }
}

const X: &[Param] = &[req("x")];
const A: &[Param] = &[req("A")];
const fn one(name: &'static str) -> Builtin {
    Builtin { name, sig: Sig { positional: X, variadic: false, named: &[] } }
}
const fn matrix(name: &'static str) -> Builtin {
    Builtin { name, sig: Sig { positional: A, variadic: false, named: &[] } }
}
const fn sig(name: &'static str, positional: &'static [Param], named: &'static [&'static str]) -> Builtin {
    Builtin { name, sig: Sig { positional, variadic: false, named } }
}

/// Every built-in operation. Names of Typst's math syntax (mat, frac) are no operations: they are always read as
/// that syntax.
pub const BUILTINS: &[Builtin] = &[
    one("sqrt"), one("exp"), one("ln"), one("log"), one("sin"), one("cos"), one("tan"), one("arcsin"), one("arccos"),
    one("arctan"), one("sinh"), one("cosh"), one("tanh"), one("abs"), one("rect"), one("tri"), one("sgn"), one("floor"),
    one("ceil"), one("factorial"), one("Re"), one("Im"), one("conj"), one("arg"),
    sig("number", &[req("value"), req("unit")], &[]),
    matrix("det"), matrix("tr"), matrix("rank"), one("norm"), sig("identity", &[req("n")], &[]), matrix("rows"), matrix("cols"),
    sig("at", &[req("v"), req("i"), opt("j")], &[]),
    sig("diff", &[req("expression"), req("variable")], &[]),
    sig("integrate", &[req("expression"), req("variable"), opt("lower"), opt("upper")], &[]),
    sig("sum", &[req("expression"), req("index"), req("lower"), req("upper")], &[]),
    sig("laplace", &[req("expression"), opt("variable"), opt("result")], &[]),
    sig("invlaplace", &[req("expression"), opt("variable"), opt("result")], &[]),
    sig("convolve", &[req("f"), req("g"), opt("variable"), opt("at")], &["step"]),
    sig("range", &[req("start"), opt("end")], &["step"]),
    Builtin { name: "vec", sig: Sig { positional: &[], variadic: true, named: &[] } },
    Builtin { name: "cases", sig: Sig { positional: &[], variadic: true, named: &[] } },
];

/// Other spellings of operations.
const ALIASES: &[(&str, &str)] = &[("asin", "arcsin"), ("acos", "arccos"), ("atan", "arctan"), ("re", "Re"), ("im", "Im")];

/// The operation of a name, without overlays.
pub fn builtin(name: &str) -> Option<&'static Builtin> {
    let name = ALIASES.iter().find(|(a, _)| *a == name).map_or(name, |(_, n)| n);
    BUILTINS.iter().find(|b| b.name == name)
}

/// Names that are always Typst's math syntax and never worksheet functions.
pub fn is_syntax(name: &str) -> bool {
    matches!(name, "mat" | "frac")
}

/// An argument as written: positional, or named (`step: 10 ms`).
pub enum Arg<T> {
    Pos(T),
    Named(String, T),
}

/// Arguments bound to an operation's parameters, by parameter name; missing optional ones are absent.
pub struct Bound<T> {
    pub positional: Vec<T>,
    pub named: HashMap<&'static str, T>,
}

impl<T> Bound<T> {
    /// The positional argument of a parameter, by index.
    pub fn get(&self, i: usize) -> Option<&T> {
        self.positional.get(i)
    }
}

/// Binds arguments to an operation's parameters, with Typst's rules and messages: positional arguments fill
/// the positional parameters in order, named ones go to the named parameters, each at most once.
pub fn bind<T>(b: &Builtin, args: Vec<Arg<T>>) -> Result<Bound<T>, String> {
    let mut positional = vec![];
    let mut named: HashMap<&'static str, T> = HashMap::new();
    for a in args {
        match a {
            Arg::Pos(x) => {
                if !b.sig.variadic && positional.len() == b.sig.positional.len() {
                    return Err(format!("{}: unexpected argument{}", b.name, usage(b)));
                }
                positional.push(x);
            }
            Arg::Named(n, x) => {
                let Some(key) = b.sig.named.iter().find(|k| **k == n) else {
                    let positional_name = b.sig.positional.iter().any(|p| p.name == n);
                    return Err(format!(
                        "{}: unexpected argument: {n}{}",
                        b.name,
                        if positional_name { format!(" – {n} is positional, pass it without its name") } else { usage(b) }
                    ));
                };
                if named.insert(key, x).is_some() {
                    return Err(format!("{}: duplicate argument: {n}", b.name));
                }
            }
        }
    }
    if let Some(p) = b.sig.positional.iter().skip(positional.len()).find(|p| p.required) {
        return Err(format!("{}: missing argument: {}{}", b.name, p.name, usage(b)));
    }
    Ok(Bound { positional, named })
}

/// How an operation is called, for messages: " – convolve(f, g, variable, at, step: …)".
pub fn usage(b: &Builtin) -> String {
    let mut ps: Vec<String> = b.sig.positional.iter().map(|p| p.name.to_string()).collect();
    if b.sig.variadic {
        ps.push("…".into());
    }
    ps.extend(b.sig.named.iter().map(|n| format!("{n}: …")));
    format!(" – {}({})", b.name, ps.join(", "))
}

/// The operation an explicit reference names (math.op("tri"), what Typst writes as ⦇tri⦈), under any spelling.
pub fn explicit(name: &str) -> Option<&'static Builtin> {
    builtin(&solveng_names::normalize(name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binding() {
        let c = builtin("convolve").unwrap();
        let b = bind(c, vec![Arg::Pos(1), Arg::Pos(2), Arg::Named("step".into(), 3)]).unwrap();
        assert_eq!(b.positional, vec![1, 2]);
        assert_eq!(b.named["step"], 3);
        assert!(bind(c, vec![Arg::Pos(1)]).err().unwrap().contains("missing argument: g"));
        assert!(bind(c, vec![Arg::Pos(1), Arg::Pos(2), Arg::Named("width".into(), 3)]).err().unwrap().contains("unexpected argument: width"));
        assert!(bind(c, vec![Arg::Pos(1), Arg::Pos(2), Arg::Named("step".into(), 3), Arg::Named("step".into(), 3)]).err().unwrap().contains("duplicate"));
        assert!(bind(c, (0..5).map(Arg::Pos).collect()).err().unwrap().contains("unexpected argument"));
        assert!(bind(c, vec![Arg::Named("f".into(), 1)]).err().unwrap().contains("positional"));
    }
}

#[cfg(test)]
mod names {
    /// Editors know the operations by the names crate's list.
    #[test]
    fn editors_know_every_operation() {
        for b in super::BUILTINS {
            assert!(solveng_names::OPERATIONS.contains(&b.name), "{} is missing from solveng_names::OPERATIONS", b.name);
        }
    }
}
