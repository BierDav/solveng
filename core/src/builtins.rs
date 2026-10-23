//! Built-in operations: what they are called, which arguments they take, and how a name reaches one.
//!
//! Lookup has layers, as units do: the underlying built-in operations, the built-in overlay, then user overlays
//! (`functions: (…)`) in order. The built-in overlay makes the standard names (`tri`, `convolve`, …) available
//! to plain lookup: names in code lines and quoted names in math. `builtin-functions: false` disables that
//! overlay, never the operations themselves. A user overlay maps names to operations, `(conv: "convolve")`, or
//! hides a name from plain lookup, `(tri: none)`. A worksheet's own function of a name takes precedence over
//! plain lookup.
//!
//! An explicit reference – what Typst's `math.op("tri")` writes, read as `⦇tri⦈` – always means an operation:
//! it resolves through the user overlays at the place of use and falls back to the underlying operations, even
//! below a `none` or with the built-in overlay disabled.
//!
//! Arguments bind as in Typst: positional arguments to the positional parameters in order, named ones
//! (`step: 10 ms`) to the named parameters, which have defaults.

use std::collections::HashMap;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

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
    one("factorial"),
    matrix("det"), matrix("tr"), matrix("rank"), one("norm"), sig("identity", &[req("n")], &[]), matrix("rows"), matrix("cols"),
    sig("at", &[req("v"), req("i"), opt("j")], &[]),
    sig("diff", &[req("expression"), req("variable")], &[]),
    sig("integrate", &[req("expression"), req("variable"), req("lower"), req("upper")], &[]),
    sig("sum", &[req("expression"), req("index"), req("lower"), req("upper")], &[]),
    sig("laplace", &[req("expression"), opt("variable"), opt("result")], &[]),
    sig("invlaplace", &[req("expression"), opt("variable"), opt("result")], &[]),
    sig("convolve", &[req("f"), req("g"), opt("variable"), opt("at")], &["step"]),
    sig("range", &[req("start"), opt("end")], &["step"]),
    Builtin { name: "vec", sig: Sig { positional: &[], variadic: true, named: &[] } },
    Builtin { name: "cases", sig: Sig { positional: &[], variadic: true, named: &[] } },
];

/// Other spellings of operations.
const ALIASES: &[(&str, &str)] = &[("asin", "arcsin"), ("acos", "arccos"), ("atan", "arctan")];

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

/// How an entry of the option `functions` reads: `(conv: "convolve", tri: none)`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FnOpt {
    /// "none" comes as this string: hidden from plain lookup.
    Key(Hidden),
    /// The operation (or another name of this or a lower overlay) the name stands for.
    Name(String),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Hidden {
    None,
}

/// The function overlays active at a place: how names reach operations.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Functions {
    /// Plain lookup through the built-in overlay.
    builtin: bool,
    /// What each user overlay maps, resolved: name → operation, or None for hidden.
    layers: Vec<IndexMap<String, Option<&'static str>>>,
}

impl Functions {
    /// The standard names, without user overlays.
    pub fn standard() -> Functions {
        Functions { builtin: true, layers: vec![] }
    }

    /// The overlays of a configuration; an entry that names no operation, or a cycle, is an error.
    pub fn configured(cfg: &crate::format::Cfg) -> Result<Functions, String> {
        let layers = if cfg.function_layers.is_empty() { vec![cfg.functions.clone()] } else { cfg.function_layers.clone() };
        Functions::layered(&layers, cfg.builtin_functions)
    }

    fn layered(layers: &[IndexMap<String, FnOpt>], builtin: bool) -> Result<Functions, String> {
        let layers: Vec<IndexMap<String, FnOpt>> =
            layers.iter().map(|l| l.iter().map(|(k, v)| (solveng_names::normalize(k), v.clone())).collect()).collect();
        let mut out = Functions { builtin, layers: vec![] };
        for (level, layer) in layers.iter().enumerate() {
            let mut resolved = IndexMap::new();
            for (name, value) in layer {
                if solveng_names::scan(&name.chars().collect::<Vec<_>>(), 0).ok().flatten().is_none_or(|n| n.end != name.chars().count()) {
                    return Err(format!("functions: '{name}' can't be a function name"));
                }
                if is_syntax(name) {
                    return Err(format!("functions: '{name}' is Typst's math syntax and can't be mapped"));
                }
                let op = match value {
                    FnOpt::Key(Hidden::None) => None,
                    FnOpt::Name(_) => Some(resolve(&layers, name, level + 1, &mut vec![])?),
                };
                resolved.insert(name.clone(), op);
            }
            out.layers.push(resolved);
        }
        Ok(out)
    }

    /// The operation an explicit reference (math.op) stands for: through the user overlays, past hidden names,
    /// to the underlying operations.
    pub fn explicit(&self, name: &str) -> Option<&'static Builtin> {
        let name = solveng_names::normalize(name);
        for layer in self.layers.iter().rev() {
            if let Some(Some(op)) = layer.get(&name) {
                return builtin(op);
            }
        }
        builtin(&name)
    }

    /// The operation a plain name (code, quoted math) stands for: the user overlays, then the built-in overlay
    /// when enabled; a hidden name stops the lookup.
    pub fn plain(&self, name: &str) -> Option<&'static Builtin> {
        let name = solveng_names::normalize(name);
        for layer in self.layers.iter().rev() {
            match layer.get(&name) {
                Some(Some(op)) => return builtin(op),
                Some(None) => return None,
                None => {}
            }
        }
        if self.builtin { builtin(&name) } else { None }
    }

    /// Names plain lookup reaches, with the operation each stands for (for editors and messages).
    pub fn names(&self) -> Vec<(String, &'static str)> {
        let mut out: IndexMap<String, &'static str> = IndexMap::new();
        if self.builtin {
            for b in BUILTINS {
                out.insert(b.name.to_string(), b.name);
            }
            for (a, n) in ALIASES {
                out.insert(a.to_string(), n);
            }
        }
        for layer in &self.layers {
            for (k, v) in layer {
                match v {
                    Some(op) => { out.insert(k.clone(), op); }
                    None => { out.shift_remove(k); }
                }
            }
        }
        out.into_iter().collect()
    }
}

/// What a name in an overlay's definition stands for, from `level` down: entries of the same overlay regardless
/// of their order, a self reference below, the underlying operations last.
fn resolve(layers: &[IndexMap<String, FnOpt>], name: &str, level: usize, stack: &mut Vec<(String, usize)>) -> Result<&'static str, String> {
    if stack.iter().any(|(n, l)| n == name && *l == level) {
        let path: Vec<&str> = stack.iter().map(|(n, _)| n.as_str()).chain([name]).collect();
        return Err(format!("functions: circular dependency {}", path.join(" → ")));
    }
    for l in (1..=level).rev() {
        match layers[l - 1].get(name) {
            Some(FnOpt::Name(target)) => {
                let target = solveng_names::normalize(target);
                // a self reference is the name below this overlay
                let next = if target == name { l - 1 } else { l };
                stack.push((name.to_string(), level));
                let r = resolve(layers, &target, next, stack);
                stack.pop();
                return r;
            }
            // definitions see past hidden names, as explicit references do
            Some(FnOpt::Key(Hidden::None)) | None => {}
        }
    }
    builtin(name).map(|b| b.name).ok_or_else(|| format!("functions: '{name}' is no built-in operation (operations: {})", BUILTINS.iter().map(|b| b.name).collect::<Vec<_>>().join(", ")))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layers(ls: &[&[(&str, Option<&str>)]], builtin: bool) -> Result<Functions, String> {
        let ls: Vec<IndexMap<String, FnOpt>> = ls
            .iter()
            .map(|l| l.iter().map(|(k, v)| (k.to_string(), v.map_or(FnOpt::Key(Hidden::None), |s| FnOpt::Name(s.into())))).collect())
            .collect();
        Functions::layered(&ls, builtin)
    }

    #[test]
    fn lookup() {
        let f = Functions::standard();
        assert_eq!(f.plain("tri").unwrap().name, "tri");
        assert_eq!(f.plain("asin").unwrap().name, "arcsin");
        assert!(f.plain("conv").is_none());
        let f = layers(&[&[("conv", Some("convolve")), ("tri", None)]], true).unwrap();
        assert_eq!(f.plain("conv").unwrap().name, "convolve");
        assert!(f.plain("tri").is_none());
        assert_eq!(f.explicit("tri").unwrap().name, "tri");
        assert_eq!(f.explicit("conv").unwrap().name, "convolve");
        // no built-in overlay: only what the user overlays map
        let f = layers(&[&[("tri", Some("tri"))]], false).unwrap();
        assert_eq!(f.plain("tri").unwrap().name, "tri");
        assert!(f.plain("rect").is_none());
        assert_eq!(f.explicit("rect").unwrap().name, "rect");
        // order doesn't matter within an overlay; a self reference goes below
        let f = layers(&[&[("a", Some("b")), ("b", Some("rect"))], &[("rect", Some("tri")), ("c", Some("rect"))]], true).unwrap();
        assert_eq!(f.plain("a").unwrap().name, "rect");
        assert_eq!(f.plain("c").unwrap().name, "tri");
        assert_eq!(f.explicit("rect").unwrap().name, "tri");
        let f = layers(&[&[("tri", Some("rect"))], &[("tri", Some("tri"))]], true).unwrap();
        assert_eq!(f.plain("tri").unwrap().name, "rect");
        assert!(layers(&[&[("a", Some("b")), ("b", Some("a"))]], true).unwrap_err().contains("circular"));
        assert!(layers(&[&[("a", Some("nothing"))]], true).unwrap_err().contains("no built-in operation"));
    }

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
