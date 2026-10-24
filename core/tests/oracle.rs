//! Results from SymPy and mpmath (tests/oracle/generate.py), checked through the worksheet as they would be
//! written: every case is a line of math in a fresh worksheet, with its operations and constants as references. Values are compared as numbers (within the
//! case's tolerance), symbolic results at sample points, solutions as sets.
//!
//! What the engine can't do yet, or gets wrong, is listed in oracle/known.txt: the test fails for a case that
//! fails without being listed, and for a listed case that passes now, so the list only ever shrinks.
//! ORACLE_BLESS=1 rewrites it; ORACLE_ONLY=<part of an id> runs some cases; ORACLE_VERBOSE=1 prints all.

use std::collections::BTreeMap;
use std::fmt::Write as _;

mod support;

use solveng_core::env::Env;
use solveng_core::expr::{Expr, simp};
use solveng_core::linalg::Array;
use solveng_core::poly::{coeffs, together};
use solveng_core::sheet::Out;

const DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/oracle");

/// From best to worst.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Status {
    Pass,
    /// Right, but not as simple as it can be (fractions not cancelled).
    Weak,
    /// An error where there is a result.
    Unsupported,
    /// A result that isn't the right one.
    Wrong,
}

impl Status {
    fn name(self) -> &'static str {
        match self {
            Status::Pass => "pass",
            Status::Weak => "weak",
            Status::Unsupported => "unsupported",
            Status::Wrong => "wrong",
        }
    }
}

struct Outcome {
    status: Status,
    detail: String,
}

impl Outcome {
    fn pass() -> Outcome {
        Outcome { status: Status::Pass, detail: String::new() }
    }
    fn new(status: Status, detail: impl Into<String>) -> Outcome {
        Outcome { status, detail: detail.into() }
    }
    /// The worse of two.
    fn and(self, o: Outcome) -> Outcome {
        if o.status > self.status { o } else { self }
    }
}

// ---------- numbers ----------

fn number(s: &str) -> f64 {
    match s.split_once('/') {
        Some((n, d)) => n.parse::<f64>().unwrap() / d.parse::<f64>().unwrap(),
        None => s.parse().unwrap_or_else(|_| panic!("not a number: {s}")),
    }
}

/// A value: a number, or rows×cols numbers.
#[derive(Clone, Debug)]
struct Value {
    shape: Option<(usize, usize)>,
    v: Vec<f64>,
}

fn parse_value(s: &str) -> Value {
    match s.split_once(':') {
        Some((shape, v)) => {
            let (r, c) = shape.split_once('x').unwrap();
            Value { shape: Some((r.parse().unwrap(), c.parse().unwrap())), v: v.split(',').map(number).collect() }
        }
        None => Value { shape: None, v: vec![number(s)] },
    }
}

/// Within `tol` relative; an exact 0 within rounding.
fn close(a: f64, b: f64, tol: f64) -> bool {
    a == b || (a - b).abs() <= tol * a.abs().max(b.abs()).max(if b == 0.0 { 1e-3 } else { 1e-12 })
}

fn same(got: &Value, want: &Value, tol: f64) -> bool {
    got.shape == want.shape && got.v.len() == want.v.len() && got.v.iter().zip(&want.v).all(|(a, b)| close(*a, *b, tol))
}

// ---------- the worksheet ----------

fn run(env: &mut Env, src: &str) -> Out {
    support::line(env, src)
}

/// The value of a defined name, with what's defined now put in.
fn got(env: &Env, name: &str) -> Result<Value, String> {
    let e = simp(&env.bind_with_units(&env.vars[name].expr, &Default::default(), &support::units(env))?);
    // a complex value is NaN: only real ones are compared
    let num = |e: &Expr| solveng_core::eval::complex_value(e).map(|z| if z.is_real() { z.re.v.f() } else { f64::NAN }).map_err(|e| e.message());
    Ok(match Array::of(&e) {
        Some(a) => Value { shape: Some((a.rows, a.cols)), v: a.a.iter().map(num).collect::<Result<_, _>>()? },
        None => Value { shape: None, v: vec![num(&e)?] },
    })
}

fn errors(out: &Out) -> Option<String> {
    (!out.errors.is_empty()).then(|| out.errors.join("; "))
}

/// name=value pairs.
fn assignments(s: &str) -> Vec<(String, String)> {
    if s == "-" || s.is_empty() {
        return vec![];
    }
    s.split(',').map(|kv| kv.split_once('=').map(|(k, v)| (k.to_string(), format!("({v})"))).unwrap()).collect()
}

// ---------- kinds ----------

fn value_case(f: &[&str]) -> Outcome {
    let (src, want, tol) = (f[0], parse_value(f[1]), if f.get(2).is_none_or(|t| *t == "-") { 1e-9 } else { number(f[2]) });
    let mut env = Env::default();
    if let Some(e) = errors(&run(&mut env, &format!("v := {src} ="))) {
        return Outcome::new(Status::Unsupported, e);
    }
    match got(&env, "v") {
        Ok(g) if same(&g, &want, tol) => Outcome::pass(),
        Ok(g) => Outcome::new(Status::Wrong, format!("got {:?} {:?}, want {:?} {:?}", g.shape, g.v, want.shape, want.v)),
        Err(e) => Outcome::new(Status::Unsupported, e),
    }
}

fn identity_case(f: &[&str]) -> Outcome {
    let (src, points, wants, degrees) = (f[0], f[2], f[3], f[4]);
    let mut env = Env::default();
    if let Some(e) = errors(&run(&mut env, &format!("s := {src} -> simplify"))) {
        return Outcome::new(Status::Unsupported, e);
    }
    let mut out = Outcome::pass();
    for (point, want) in points.split('|').zip(wants.split('|')) {
        let mut at = env.clone();
        for (k, v) in assignments(point) {
            run(&mut at, &format!("{k} := {v}"));
        }
        match got(&at, "s") {
            Ok(g) if same(&g, &parse_value(want), 1e-9) => {}
            Ok(g) => return Outcome::new(Status::Wrong, format!("at {point}: got {:?}, want {want}", g.v)),
            Err(e) => return Outcome::new(Status::Unsupported, format!("at {point}: {e}")),
        }
    }
    if degrees == "const" {
        let s = simp(&env.vars["s"].expr);
        if !s.syms().is_empty() {
            out = out.and(Outcome::new(Status::Weak, "not simplified to a constant"));
        }
    } else if degrees != "-" {
        // as far cancelled as SymPy's: no higher degree above or below in any name
        let (n, d) = together(&simp(&env.vars["s"].expr));
        for part in degrees.split(',') {
            let (name, nd) = part.split_once(':').unwrap();
            let (wn, wd) = nd.split_once('/').unwrap();
            let degree = |e: &Expr| coeffs(e, name).map(|c| c.iter().rposition(|c| !c.is_zero()).unwrap_or(0));
            match (degree(&n), degree(&d)) {
                (Some(gn), Some(gd)) if gn > wn.parse().unwrap() || gd > wd.parse().unwrap() => {
                    out = out.and(Outcome::new(Status::Weak, format!("degrees in {name}: {gn}/{gd}, cancelled {wn}/{wd}")));
                }
                (Some(_), Some(_)) => {}
                _ => out = out.and(Outcome::new(Status::Weak, format!("not a fraction of polynomials in {name}"))),
            }
        }
    }
    out
}

/// Solutions as sets of tuples: each wanted one matched by a different one found.
/// The real solutions are compared (SymPy's are real; the engine's complex ones are NaN in `solutions`).
fn same_solutions(got: &[Vec<f64>], want: &[Vec<f64>]) -> bool {
    let got: Vec<Vec<f64>> = got.iter().filter(|s| s.iter().all(|x| !x.is_nan())).cloned().collect();
    if got.len() != want.len() {
        return false;
    }
    let mut used = vec![false; got.len()];
    want.iter().all(|w| {
        let hit = (0..got.len()).find(|&i| !used[i] && got[i].len() == w.len() && got[i].iter().zip(w).all(|(a, b)| close(*a, *b, 1e-9)));
        hit.inspect(|&i| used[i] = true).is_some()
    })
}

fn solve_case(f: &[&str]) -> Outcome {
    let (eqs, unknowns, params, sols, how) = (f[0], f[1], assignments(f[2]), f[3], f[4]);
    let want: Vec<Vec<f64>> = if sols == "none" { vec![] } else { sols.split('|').map(|s| s.split(',').map(number).collect()).collect() };
    // with the parameters' values: the solutions as numbers
    let mut env = Env::default();
    for (k, v) in &params {
        run(&mut env, &format!("{k} := {v}"));
    }
    let out = run(&mut env, &format!("{eqs} -> solve({unknowns})"));
    let mut outcome = match errors(&out) {
        Some(e) => Outcome::new(Status::Unsupported, format!("numeric: {e}")),
        None if same_solutions(&out.solutions, &want) => Outcome::pass(),
        None => Outcome::new(Status::Wrong, format!("numeric: got {:?}, want {want:?}", out.solutions)),
    };
    // a formula of the parameters, evaluated at their values
    if how == "exact" && !params.is_empty() {
        let mut env = Env::default();
        let names: Vec<&str> = params.iter().map(|(k, _)| k.as_str()).collect();
        let body = if eqs.contains("; ") { format!("vec({})", eqs.replace("; ", ", ")) } else { eqs.to_string() };
        let symbolic = match errors(&run(&mut env, &format!("F({}) := {body} -> solve({unknowns})", names.join(", ")))) {
            Some(e) => Outcome::new(Status::Unsupported, format!("symbolic: {e}")),
            None => {
                let args: Vec<&str> = params.iter().map(|(_, v)| v.as_str()).collect();
                match errors(&run(&mut env, &format!("v := F({}) =", args.join(", ")))).map(Err).unwrap_or_else(|| got(&env, "v")) {
                    Err(e) => Outcome::new(Status::Unsupported, format!("symbolic: {e}")),
                    Ok(g) => {
                        // one solution of one unknown is a number, several a vector; for several unknowns a
                        // vector per solution, nested when there are several
                        let k = unknowns.split(',').count();
                        let found: Vec<Vec<f64>> = if k == 1 { g.v.iter().map(|v| vec![*v]).collect() } else { g.v.chunks(k).map(<[f64]>::to_vec).collect() };
                        if same_solutions(&found, &want) { Outcome::pass() } else { Outcome::new(Status::Wrong, format!("symbolic: got {found:?}, want {want:?}")) }
                    }
                }
            }
        };
        outcome = outcome.and(symbolic);
    }
    outcome
}

// ---------- the run ----------

fn known() -> BTreeMap<String, (Status, String)> {
    let text = std::fs::read_to_string(format!("{DIR}/known.txt")).unwrap_or_default();
    text.lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .map(|l| {
            let mut f = l.splitn(3, '\t');
            let id = f.next().unwrap().to_string();
            let status = match f.next().unwrap() {
                "weak" => Status::Weak,
                "unsupported" => Status::Unsupported,
                "wrong" => Status::Wrong,
                s => panic!("known.txt: unknown status {s}"),
            };
            (id, (status, f.next().unwrap_or("").to_string()))
        })
        .collect()
}

#[test]
fn oracle() {
    let only = std::env::var("ORACLE_ONLY").ok();
    let verbose = std::env::var("ORACLE_VERBOSE").is_ok();
    let known = known();
    let mut results: Vec<(String, Outcome)> = vec![];
    let mut summary = String::new();
    for file in ["values", "calculus", "identities", "solving"] {
        let text = std::fs::read_to_string(format!("{DIR}/{file}.txt")).unwrap();
        let mut counts = BTreeMap::new();
        for l in text.lines().filter(|l| !l.starts_with('#')) {
            let f: Vec<&str> = l.split('\t').collect();
            let id = f[1].to_string();
            if only.as_ref().is_some_and(|o| !id.contains(o.as_str())) {
                continue;
            }
            let outcome = match f[0] {
                "value" => value_case(&f[2..]),
                "identity" => identity_case(&f[2..]),
                "solve" => solve_case(&f[2..]),
                k => panic!("unknown kind {k}"),
            };
            if verbose {
                println!("{id}: {} {}", outcome.status.name(), outcome.detail);
            }
            *counts.entry(outcome.status).or_insert(0) += 1;
            results.push((id, outcome));
        }
        let _ = writeln!(summary, "{file}: {}", counts.iter().map(|(s, n)| format!("{n} {}", s.name())).collect::<Vec<_>>().join(", "));
    }
    println!("{summary}");
    if std::env::var("ORACLE_BLESS").is_ok() {
        let mut out = String::from("# cases of core/tests/oracle that the engine can't do yet (unsupported), gets wrong, or doesn't simplify\n# fully (weak); written by ORACLE_BLESS=1 cargo test --test oracle\n");
        for (id, o) in results.iter().filter(|(_, o)| o.status != Status::Pass) {
            let _ = writeln!(out, "{id}\t{}\t{}", o.status.name(), o.detail.replace(['\t', '\n'], " "));
        }
        std::fs::write(format!("{DIR}/known.txt"), out).unwrap();
        return;
    }
    let mut problems = vec![];
    for (id, o) in &results {
        match known.get(id) {
            None if o.status != Status::Pass => problems.push(format!("{id}: {} – {}", o.status.name(), o.detail)),
            Some((s, _)) if o.status > *s => problems.push(format!("{id}: {} (listed as {}) – {}", o.status.name(), s.name(), o.detail)),
            Some((s, _)) if o.status < *s => problems.push(format!("{id}: {} now (listed as {}) – shrink the list with ORACLE_BLESS=1", o.status.name(), s.name())),
            _ => {}
        }
    }
    assert!(problems.is_empty(), "{summary}\n{}", problems.join("\n"));
}
