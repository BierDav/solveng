//! Values: numbers with units out of expressions, exactly where the arithmetic allows.

use crate::complex::{self, CVal, Z};
use crate::dim::Dim;
use crate::env::rat_of;
use crate::expr::{Expr, Func, Val, simp};
use crate::num::{Num, close};
use crate::parse::Rel;

/// Why an expression has no value.
#[derive(Clone, Debug, PartialEq)]
pub enum NoValue {
    /// Names that aren't defined.
    Unknown(Vec<String>),
    Error(String),
}

impl NoValue {
    pub fn message(&self) -> String {
        match self {
            NoValue::Unknown(ns) => {
                let one = ns.len() == 1;
                format!(
                    "{} {} defined",
                    ns.iter().map(|n| format!("'{n}'")).collect::<Vec<_>>().join(", "),
                    if one { "isn't" } else { "aren't" }
                )
            }
            NoValue::Error(m) => m.clone(),
        }
    }
}

/// The value of a bound expression (defined names put in): exact if it can be. A complex value is an error here:
/// see [`complex_value`].
pub fn value(e: &Expr) -> Result<Val, NoValue> {
    let z = complex_value(e)?;
    if z.is_real() {
        return Ok(z.re);
    }
    Err(NoValue::Error("the value is complex, and only a real value works here – Re(…), Im(…), abs(…) or arg(…) give real ones".into()))
}

/// The value of a bound expression, complex if it has `i` in it: exact if it can be.
pub fn complex_value(e: &Expr) -> Result<CVal, NoValue> {
    let s = simp(e);
    if let Some(z) = complex::parts(&s) {
        return Ok(z);
    }
    let mut pending = vec![&s];
    while let Some(e) = pending.pop() {
        if let Expr::Call(n, _) = e {
            return Err(NoValue::Error(if n == "sng" {
                "sng isn't defined – did you mean sgn(…)?".into()
            } else { format!("{n} isn't defined – define it first with :=") }));
        }
        pending.extend(e.children());
    }
    let free = s.syms();
    if !free.is_empty() {
        return Err(NoValue::Unknown(free));
    }
    if let Some(m) = crate::linalg::problem(&s) {
        return Err(NoValue::Error(m));
    }
    let (z, d) = eval_z(&s, Domain::Complex).map_err(NoValue::Error)?;
    let z = z.cleaned();
    Ok(CVal { re: Val::new(Num::F(z.re), d.clone()), im: Val::new(Num::F(z.im), d) })
}

fn units_text(d: &Dim) -> String {
    if d.is_one() { "1".into() } else { crate::units::Units::default().auto(d).text() }
}

/// Where an expression is calculated: in the complex numbers (values), or the real ones (graphs, numeric solving).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Domain {
    Real,
    Complex,
}

/// Numeric value with dimension, in the real numbers; problems as messages.
pub fn eval_d(e: &Expr) -> Result<(f64, Dim), String> {
    eval_z(e, Domain::Real).map(|(z, d)| (z.re, d))
}

/// Numeric value with dimension, in the complex numbers.
pub fn eval_c(e: &Expr) -> Result<(Z, Dim), String> {
    eval_z(e, Domain::Complex)
}

/// A real power: odd roots of negative numbers are real, (−8)^(1/3) = −2.
fn real_pow(bv: f64, xv: f64) -> Result<f64, String> {
    if bv == 0.0 && xv < 0.0 {
        return Err("division by zero".into());
    }
    if bv < 0.0 && xv.fract() != 0.0 {
        let r = rat_of(xv).filter(|r| r.denom() % 2 == 1);
        let Some(r) = r else {
            return Err(if (xv - 0.5).abs() < 1e-12 { "square root of a negative number".into() } else { "a negative number to a fractional power is not real".into() });
        };
        let m = (-bv).powf(xv);
        return Ok(if r.numer() % 2 == 0 { m } else { -m });
    }
    Ok(if xv.fract() == 0.0 && xv.abs() < 1e9 { bv.powi(xv as i32) } else { bv.powf(xv) })
}

/// The dimension of a power: a unit takes rational exponents only.
fn pow_dim(bd: &Dim, xv: Z) -> Result<Dim, String> {
    if bd.is_one() {
        return Ok(Dim::ONE);
    }
    let r = Some(xv).filter(|x| x.is_real()).and_then(|x| rat_of(x.re)).ok_or("a unit to a power that isn't a simple fraction")?;
    bd.pow(r).ok_or_else(|| "a unit to a power that isn't a simple fraction".into())
}

/// A function of a complex argument; None for the ones only defined for real arguments.
fn complex_f(f: Func, z: Z) -> Option<Z> {
    Some(match f {
        Func::Exp => z.exp(),
        Func::Ln => z.ln(),
        Func::Log => z.ln().scale(1.0 / std::f64::consts::LN_10),
        Func::Sin => z.sin(),
        Func::Cos => z.cos(),
        Func::Tan => z.sin().div(z.cos()),
        Func::Sinh => z.sinh(),
        Func::Cosh => z.cosh(),
        Func::Tanh => z.sinh().div(z.cosh()),
        Func::Asin => z.asin(),
        Func::Acos => z.acos(),
        Func::Atan => z.atan(),
        _ => return None,
    })
}

fn eval_z(e: &Expr, dom: Domain) -> Result<(Z, Dim), String> {
    let real = |e: &Expr, what: &str| -> Result<(f64, Dim), String> {
        let (z, d) = eval_z(e, dom)?;
        if !z.is_real() { return Err(format!("{what} must be real, not complex")); }
        Ok((z.re, d))
    };
    let r = match e {
        Expr::Cases(v) => {
            for (value, condition) in v {
                let (test, dim) = real(condition, "a cases condition")?;
                if !dim.is_one() { return Err("a cases condition must be unitless".into()); }
                if test != 0.0 { return eval_z(value, dom); }
            }
            return Err("no cases condition matches here".into());
        }
        Expr::Vector(_) => return Err("a vector has several components – select one with at(vector, index), starting at 0".into()),
        Expr::Matrix(_) => return Err("a matrix has several elements – select one with at(matrix, row, column), starting at 0".into()),
        Expr::Op(..) => return Err(crate::linalg::problem(e).unwrap_or_else(|| "this matrix operation has no value here".into())),
        Expr::C(v) => (Z::real(v.v.f()), v.d.clone()),
        Expr::Pi => (Z::real(std::f64::consts::PI), Dim::ONE),
        Expr::E => (Z::real(std::f64::consts::E), Dim::ONE),
        Expr::I if dom == Domain::Complex => (Z::I, Dim::ONE),
        Expr::Const(_, backing) => return eval_z(backing, dom),
        Expr::I => return Err("i is imaginary, and only real values work here".into()),
        Expr::Call(n, _) => return Err(format!("{n} isn't defined – define it first with :=")),
        Expr::Sym(n) => return Err(format!("'{n}' isn't defined")),
        Expr::Add(ts) => {
            let mut acc: Option<(Z, Dim)> = None;
            for t in ts {
                let (v, d) = eval_z(t, dom)?;
                acc = Some(match acc {
                    None => (v, d),
                    Some((a, ad)) => {
                        if ad != d {
                            return Err(format!("adding different units: {} and {}", units_text(&ad), units_text(&d)));
                        }
                        (a.add(v), d)
                    }
                });
            }
            acc.unwrap_or((Z::real(0.0), Dim::ONE))
        }
        Expr::Mul(fs) => {
            let mut acc = (Z::real(1.0), Dim::ONE);
            for f in fs {
                let (v, d) = eval_z(f, dom)?;
                acc = (acc.0.mul(v), acc.1.mul(&d));
            }
            acc
        }
        Expr::Pow(b, x) => {
            let (bv, bd) = eval_z(b, dom)?;
            let (xv, xd) = eval_z(x, dom)?;
            if !xd.is_one() {
                return Err("an exponent must be unitless".into());
            }
            let d = pow_dim(&bd, xv)?;
            if bv.is_real() && xv.is_real() {
                match real_pow(bv.re, xv.re) {
                    Ok(v) => (Z::real(v), d),
                    // a real power without a real value has its principal complex one: √−4 = 2i
                    Err(_) if dom == Domain::Complex && bv.re < 0.0 => (bv.pow(xv), d),
                    Err(m) => return Err(m),
                }
            } else {
                if bv == Z::real(0.0) && xv.re < 0.0 {
                    return Err("division by zero".into());
                }
                (bv.pow(xv), d)
            }
        }
        Expr::F(f, a) => {
            let (v, d) = eval_z(a, dom)?;
            match f {
                Func::Abs => return Ok((Z::real(v.abs()), d)),
                Func::Re => return Ok((Z::real(v.re), d)),
                Func::Im => return Ok((Z::real(v.im), d)),
                Func::Conj => return Ok((v.conj(), d)),
                Func::Arg if v == Z::real(0.0) => return Err("arg of 0 isn't defined".into()),
                Func::Arg => return Ok((Z::real(v.arg()), Dim::ONE)),
                _ => {}
            }
            if !d.is_one() {
                return Err(format!("the argument of {} must be unitless, it is in {}", f.name(), units_text(&d)));
            }
            let real_value = if v.is_real() { f.eval(v.re) } else { None };
            match (real_value, dom) {
                (Some(r), _) => (Z::real(r), Dim::ONE),
                (None, Domain::Complex) if complex_f(*f, v).is_some_and(|z| z.is_finite()) => (complex_f(*f, v).unwrap(), Dim::ONE),
                (None, Domain::Complex) if !v.is_real() => return Err(format!("{} takes a real argument, not a complex one", f.name())),
                (None, _) => return Err(match f {
                    Func::Ln | Func::Log => format!("{} of a number ≤ 0", f.name()),
                    Func::Asin | Func::Acos => format!("{} of a number outside −1…1", f.name()),
                    _ => format!("{} is out of range here", f.name()),
                }),
            }
        }
        Expr::Cmp(v, rs) => {
            let vals: Vec<(Z, Dim)> = v.iter().map(|x| eval_z(x, dom)).collect::<Result<_, _>>()?;
            let mut ok = true;
            for (i, r) in rs.iter().enumerate() {
                let (a, b) = (&vals[i], &vals[i + 1]);
                if a.1 != b.1 {
                    ok &= *r == Rel::Ne;
                    continue;
                }
                let same = close(a.0.re, b.0.re) && close(a.0.im, b.0.im);
                if matches!(r, Rel::Eq | Rel::Ne) {
                    ok &= same == (*r == Rel::Eq);
                } else if !a.0.is_real() || !b.0.is_real() {
                    return Err("complex numbers have no order: compare their parts, Re(…), Im(…), or abs(…)".into());
                } else {
                    ok &= r.holds(a.0.re.total_cmp(&b.0.re), same);
                }
            }
            (Z::real(if ok { 1.0 } else { 0.0 }), Dim::ONE)
        }
        Expr::Subs(..) | Expr::Derivative(..) => return Err("this derivative needs piecewise or distributional calculus; it is kept symbolic".into()),
        Expr::InverseLaplace(body, var, by) => {
            let (t, dimension) = real(by, "the inverse Laplace evaluation time")?;
            if !dimension.is_one() { return Err("inverse Laplace evaluation time must be unitless".into()); }
            dim_of(e, &|_| None)?;
            (Z::real(crate::inverse_laplace::evaluate(body, var, t, &|_| None)?), Dim::ONE)
        }
        Expr::Sum(body, var, lo, hi) => {
            let bound = |e: &Expr| -> Result<f64, String> {
                let (v, d) = if let Expr::C(v) = e { (v.v.f(), v.d.clone()) } else { real(e, "a sum bound")? };
                if !d.is_one() { return Err("sum bounds must be unitless".into()); }
                Ok(v)
            };
            let d = dim_of(body, &|n| if n == var { Some(Dim::ONE) } else { None })?.ok_or("unknown summand dimension")?;
            let points = crate::calculus::signal_points(body, var, &|_| None);
            let (lo, hi) = (bound(lo)?, bound(hi)?);
            // a complex summand: its real and imaginary parts summed on their own
            let part = |im: bool| crate::calculus::series(&|k| eval_z(&body.subst(var, &crate::expr::c(Num::F(k))), dom).ok().filter(|(_, sd)| *sd == d).map(|(v, _)| if im { v.im } else { v.re }), lo, hi, &points, crate::calculus::piecewise_affine(body, var));
            let im = if dom == Domain::Complex { part(true)? } else { 0.0 };
            (Z::new(part(false)?, im), d)
        }
        Expr::Integral(body, var, lo, hi) => {
            let bound = |e: &Expr| if let Expr::C(v) = e { Ok((v.v.f(), v.d.clone())) } else { real(e, "an integral bound") };
            let (l, ld) = bound(lo)?;
            let (h, hd) = bound(hi)?;
            if ld != hd { return Err("integral bounds have different units".into()); }
            let sample = |x| body.subst(var, &Expr::C(Val::new(Num::F(x), ld.clone())));
            let d = dim_of(body, &|n| if n == var { Some(ld.clone()) } else { None })?.ok_or("unknown integrand dimension")?;
            let points = crate::calculus::signal_points(body, var, &|_| None);
            let part = |im: bool| crate::calculus::piecewise_quadrature(&|x| eval_z(&sample(x), dom).ok().filter(|(_, sd)| *sd == d).map(|(v, _)| if im { v.im } else { v.re }), l, h, &points);
            let im = if dom == Domain::Complex { part(true)? } else { 0.0 };
            (Z::new(part(false)?, im), d.mul(&ld))
        }
        Expr::Number(a, k, d, _) => {
            let (v, da) = eval_z(a, dom)?;
            if da != *d {
                return Err(format!("number: the value is in {}, not in {}", units_text(&da), units_text(d)));
            }
            (v.scale(1.0 / k.f()), Dim::ONE)
        }
    };
    if !r.0.is_finite() {
        return Err(if r.0.is_nan() { "not a number here".into() } else { "too large (overflow)".into() });
    }
    Ok(r)
}

/// A plain number, symbols from `vars`; None where it isn't defined (domain, division by zero, unknown names).
pub fn eval_f(e: &Expr, vars: &dyn Fn(&str) -> Option<f64>) -> Option<f64> {
    let r = match e {
        Expr::Cases(v) => {
            let mut selected = None;
            for (value, condition) in v {
                if eval_f(condition, vars)? != 0.0 { selected = Some(value); break; }
            }
            return eval_f(selected?, vars);
        }
        Expr::Vector(_) | Expr::Matrix(_) | Expr::Op(..) => return None,
        Expr::C(v) => v.v.f(),
        Expr::Pi => std::f64::consts::PI,
        Expr::E => std::f64::consts::E,
        Expr::I => return None,
        Expr::Const(_, backing) => eval_f(backing, vars)?,
        Expr::Sym(n) => vars(n)?,
        Expr::Add(ts) => {
            let mut s = 0.0;
            for t in ts {
                s += eval_f(t, vars)?;
            }
            s
        }
        Expr::Mul(fs) => {
            let mut s = 1.0;
            for f in fs {
                s *= eval_f(f, vars)?;
            }
            s
        }
        Expr::Pow(b, x) => {
            let (b, x) = (eval_f(b, vars)?, eval_f(x, vars)?);
            if b == 0.0 && x < 0.0 {
                return None;
            }
            if b < 0.0 && x.fract() != 0.0 {
                let r = rat_of(x).filter(|r| r.denom() % 2 == 1)?;
                let m = (-b).powf(x);
                if r.numer() % 2 == 0 { m } else { -m }
            } else if x.fract() == 0.0 && x.abs() < 1e9 {
                b.powi(x as i32)
            } else {
                b.powf(x)
            }
        }
        Expr::F(f, a) => f.eval(eval_f(a, vars)?)?,
        Expr::Cmp(v, rs) => {
            let vals: Vec<f64> = v.iter().map(|x| eval_f(x, vars)).collect::<Option<_>>()?;
            let ok = rs.iter().enumerate().all(|(i, r)| r.holds(vals[i].total_cmp(&vals[i + 1]), close(vals[i], vals[i + 1])));
            if ok { 1.0 } else { 0.0 }
        }
        Expr::Call(..) | Expr::Subs(..) | Expr::Derivative(..) => return None,
        Expr::InverseLaplace(body, var, by) => crate::inverse_laplace::evaluate(body, var, eval_f(by, vars)?, vars).ok()?,
        Expr::Sum(body, var, lo, hi) => {
            let bound = |e: &Expr| if let Expr::C(v) = e { Some(v.v.f()) } else { eval_f(e, vars) };
            let points = crate::calculus::signal_points(body, var, vars);
            crate::calculus::series(&|k| eval_f(body, &|n| if n == var { Some(k) } else { vars(n) }), bound(lo)?, bound(hi)?, &points, crate::calculus::piecewise_affine(body, var)).ok()?
        }
        Expr::Integral(body, var, lo, hi) => {
            let bound = |e: &Expr| if let Expr::C(v) = e { Some(v.v.f()) } else { eval_f(e, vars) };
            let points = crate::calculus::signal_points(body, var, vars);
            crate::calculus::piecewise_quadrature(&|x| eval_f(body, &|n| if n == var { Some(x) } else { vars(n) }), bound(lo)?, bound(hi)?, &points).ok()?
        }
        Expr::Number(a, k, ..) => eval_f(a, vars)? / k.f(),
    };
    r.is_finite().then_some(r)
}

/// The dimension of an expression; symbols' dims from `sd` (None: not known, then the result may be None).
/// Errors where units can't match.
pub fn dim_of(e: &Expr, sd: &dyn Fn(&str) -> Option<Dim>) -> Result<Option<Dim>, String> {
    Ok(match e {
        Expr::Cases(v) => {
            let mut result = None;
            for (value, condition) in v {
                if dim_of(condition, sd)?.is_some_and(|d| !d.is_one()) { return Err("a cases condition must be unitless".into()); }
                if value.is_zero() { continue; }
                if let Some(d) = dim_of(value, sd)? {
                    if result.is_some_and(|previous| previous != d) { return Err("cases branches have different units".into()); }
                    result = Some(d);
                }
            }
            result.or(Some(Dim::ONE))
        }
        Expr::Vector(_) | Expr::Matrix(_) | Expr::Op(..) => {
            for e in e.children() { dim_of(e, sd)?; }
            None
        }
        Expr::C(v) => Some(v.d.clone()),
        Expr::Pi | Expr::E | Expr::I => Some(Dim::ONE),
        Expr::Const(_, backing) => dim_of(backing, sd)?,
        Expr::Sym(n) => sd(n),
        Expr::Call(_, args) => {
            for a in args { dim_of(a, sd)?; }
            None
        }
        Expr::Add(ts) => {
            let mut d: Option<Dim> = None;
            let mut unknown = false;
            for t in ts {
                // a literal 0 fits any unit
                if t.is_zero() {
                    continue;
                }
                match dim_of(t, sd)? {
                    Some(x) => match d {
                        Some(y) if y != x => return Err(format!("adding different units: {} and {}", units_text(&y), units_text(&x))),
                        _ => d = Some(x),
                    },
                    None => unknown = true,
                }
            }
            if unknown && d.is_none() { None } else { d.or(Some(Dim::ONE)) }
        }
        Expr::Mul(fs) => {
            let mut d = Dim::ONE;
            for f in fs {
                match dim_of(f, sd)? {
                    Some(x) => d = d.mul(&x),
                    None => return Ok(None),
                }
            }
            Some(d)
        }
        Expr::Pow(b, x) => {
            if let Some(xd) = dim_of(x, sd)? {
                if !xd.is_one() {
                    return Err("an exponent must be unitless".into());
                }
            }
            let Some(bd) = dim_of(b, sd)? else { return Ok(None) };
            if bd.is_one() {
                Some(Dim::ONE)
            } else {
                match x.as_num() {
                    Some(Num::R(r)) => Some(bd.pow(r).ok_or("a unit to a power that isn't a simple fraction")?),
                    Some(Num::F(f)) => Some(bd.pow(rat_of(f).ok_or("a unit to a power that isn't a simple fraction")?).ok_or("a unit to a power that isn't a simple fraction")?),
                    None => match eval_f(x, &|_| None) {
                        Some(f) => Some(bd.pow(rat_of(f).ok_or("a unit to a power that isn't a simple fraction")?).ok_or("bad power")?),
                        None => return Err("a unit to a power that isn't known".into()),
                    },
                }
            }
        }
        Expr::F(f, a) => {
            let d = dim_of(a, sd)?;
            if matches!(f, Func::Abs | Func::Re | Func::Im | Func::Conj) {
                d
            } else if *f == Func::Arg {
                Some(Dim::ONE)
            } else {
                if let Some(d) = d {
                    if !d.is_one() {
                        return Err(format!("the argument of {} must be unitless, it is in {}", f.name(), units_text(&d)));
                    }
                }
                Some(Dim::ONE)
            }
        }
        Expr::Cmp(v, _) => {
            for x in v {
                dim_of(x, sd)?;
            }
            Some(Dim::ONE)
        }
        Expr::Subs(body, var, by) => {
            let d = dim_of(by, sd)?;
            dim_of(body, &|n| if n == var { d.clone() } else { sd(n) })?
        }
        Expr::Derivative(body, var) => match (dim_of(body, sd)?, sd(var)) {
            (Some(a), Some(b)) => Some(a.mul(&b.powi(-1))),
            _ => None,
        },
        Expr::InverseLaplace(body, var, by) => {
            let body_dim = dim_of(body, &|n| if n == var { Some(Dim::ONE) } else { sd(n) })?;
            let time_dim = dim_of(by, sd)?;
            if [body_dim, time_dim].into_iter().flatten().any(|d| !d.is_one()) {
                return Err("inverse Laplace coefficients and time must be unitless".into());
            }
            Some(Dim::ONE)
        }
        Expr::Sum(body, var, lo, hi) => {
            for bound in [lo, hi] {
                if dim_of(bound, sd)?.is_some_and(|d| !d.is_one()) { return Err("sum bounds must be unitless".into()); }
            }
            dim_of(body, &|n| if n == var { Some(Dim::ONE) } else { sd(n) })?
        }
        Expr::Integral(body, var, lo, hi) => {
            let ld = dim_of(lo, sd)?;
            let hd = dim_of(hi, sd)?;
            if let (Some(l), Some(h)) = (&ld, &hd) { if l != h { return Err("integral bounds have different units".into()); } }
            match (ld.clone().or(hd.clone()), dim_of(body, &|n| if n == var { ld.clone().or(hd.clone()) } else { sd(n) })?) {
                (Some(x), Some(y)) => Some(y.mul(&x)),
                _ => None,
            }
        }
        Expr::Number(a, _, d, _) => {
            if let Some(da) = dim_of(a, sd)?.filter(|da| da != d) {
                return Err(format!("number: the value is in {}, not in {}", units_text(&da), units_text(d)));
            }
            Some(Dim::ONE)
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::*;

    #[test]
    fn values() {
        let amp = Expr::C(Val::new(Num::ONE, Dim::base(3)));
        let e = mul(vec![int(2), amp.clone()]);
        assert_eq!(value(&e).unwrap().v, Num::int(2));
        let e = add(vec![int(1), amp.clone()]);
        assert!(matches!(value(&e), Err(NoValue::Error(m)) if m.contains("different units")));
        let e = add(vec![sym("z"), int(1)]);
        assert_eq!(value(&e), Err(NoValue::Unknown(vec!["z".into()])));
        let e = sqrt(int(2));
        assert!((value(&e).unwrap().v.f() - 2f64.sqrt()).abs() < 1e-15);
    }
}
