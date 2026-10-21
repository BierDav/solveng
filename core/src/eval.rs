//! Values: numbers with units out of expressions, exactly where the arithmetic allows.

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

/// The value of a bound expression (defined names put in): exact if it can be.
pub fn value(e: &Expr) -> Result<Val, NoValue> {
    let s = simp(e);
    if let Expr::C(v) = s {
        return Ok(v);
    }
    let free = s.syms();
    if !free.is_empty() {
        return Err(NoValue::Unknown(free));
    }
    let (x, d) = eval_d(&s).map_err(NoValue::Error)?;
    Ok(Val::new(Num::F(x), d))
}

fn units_text(d: &Dim) -> String {
    if d.is_one() { "1".into() } else { crate::units::Units::default().auto(d).text() }
}

/// Numeric value with dimension; problems as messages.
pub fn eval_d(e: &Expr) -> Result<(f64, Dim), String> {
    let r = match e {
        Expr::C(v) => (v.v.f(), v.d),
        Expr::Pi => (std::f64::consts::PI, Dim::ONE),
        Expr::E => (std::f64::consts::E, Dim::ONE),
        Expr::Sym(n) => return Err(format!("'{n}' isn't defined")),
        Expr::Add(ts) => {
            let mut acc: Option<(f64, Dim)> = None;
            for t in ts {
                let (v, d) = eval_d(t)?;
                acc = Some(match acc {
                    None => (v, d),
                    Some((a, ad)) => {
                        if ad != d {
                            return Err(format!("adding different units: {} and {}", units_text(&ad), units_text(&d)));
                        }
                        (a + v, d)
                    }
                });
            }
            acc.unwrap_or((0.0, Dim::ONE))
        }
        Expr::Mul(fs) => {
            let mut acc = (1.0, Dim::ONE);
            for f in fs {
                let (v, d) = eval_d(f)?;
                acc = (acc.0 * v, acc.1.mul(&d));
            }
            acc
        }
        Expr::Pow(b, x) => {
            let (bv, bd) = eval_d(b)?;
            let (xv, xd) = eval_d(x)?;
            if !xd.is_one() {
                return Err("an exponent must be unitless".into());
            }
            if bv == 0.0 && xv < 0.0 {
                return Err("division by zero".into());
            }
            if bv < 0.0 && xv.fract() != 0.0 {
                // odd roots of negative numbers are real: (−8)^(1/3)
                let odd = rat_of(xv).is_some_and(|r| r.denom() % 2 == 1);
                if !odd {
                    return Err(if (xv - 0.5).abs() < 1e-12 { "square root of a negative number".into() } else { "a negative number to a fractional power is not real".into() });
                }
            }
            let d = if bd.is_one() {
                Dim::ONE
            } else {
                let r = rat_of(xv).ok_or("a unit to a power that isn't a simple fraction")?;
                bd.pow(r).ok_or("a unit to a power that isn't a simple fraction")?
            };
            let v = if bv < 0.0 && xv.fract() != 0.0 {
                let r = rat_of(xv).unwrap();
                let m = (-bv).powf(xv);
                if r.numer() % 2 == 0 { m } else { -m }
            } else if xv.fract() == 0.0 && xv.abs() < 1e9 {
                bv.powi(xv as i32)
            } else {
                bv.powf(xv)
            };
            (v, d)
        }
        Expr::F(f, a) => {
            let (v, d) = eval_d(a)?;
            if *f == Func::Abs {
                return Ok((v.abs(), d));
            }
            if !d.is_one() {
                return Err(format!("the argument of {} must be unitless, it is in {}", f.name(), units_text(&d)));
            }
            let r = f.eval(v).ok_or_else(|| match f {
                Func::Ln | Func::Log => format!("{} of a number ≤ 0", f.name()),
                Func::Asin | Func::Acos => format!("{} of a number outside −1…1", f.name()),
                _ => format!("{} is out of range here", f.name()),
            })?;
            (r, Dim::ONE)
        }
        Expr::Cmp(v, rs) => {
            let vals: Vec<(f64, Dim)> = v.iter().map(eval_d).collect::<Result<_, _>>()?;
            let ok = rs.iter().enumerate().all(|(i, r)| {
                let (a, b) = (vals[i], vals[i + 1]);
                if a.1 != b.1 {
                    return *r == Rel::Ne;
                }
                r.holds(a.0.total_cmp(&b.0), close(a.0, b.0))
            });
            (if ok { 1.0 } else { 0.0 }, Dim::ONE)
        }
        Expr::Cast(a, k, d, _) => {
            let (v, _) = eval_d(a)?;
            (v * k.f(), *d)
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
        Expr::C(v) => v.v.f(),
        Expr::Pi => std::f64::consts::PI,
        Expr::E => std::f64::consts::E,
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
        Expr::Cast(a, k, ..) => eval_f(a, vars)? * k.f(),
    };
    r.is_finite().then_some(r)
}

/// The dimension of an expression; symbols' dims from `sd` (None: not known, then the result may be None).
/// Errors where units can't match.
pub fn dim_of(e: &Expr, sd: &dyn Fn(&str) -> Option<Dim>) -> Result<Option<Dim>, String> {
    Ok(match e {
        Expr::C(v) => Some(v.d),
        Expr::Pi | Expr::E => Some(Dim::ONE),
        Expr::Sym(n) => sd(n),
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
            if *f == Func::Abs {
                d
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
        Expr::Cast(a, _, d, _) => {
            dim_of(a, sd)?;
            Some(*d)
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
