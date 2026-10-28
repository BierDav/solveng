//! Values: numbers with units out of expressions, exactly where the arithmetic allows.

use crate::complex::{self, CVal, Z};
use crate::dim::Dim;
use crate::env::rat_of;
use crate::expr::{Expr, Func, Val, simp};
use crate::num::{Num, close};
use crate::parse::Rel;
use crate::numeric::N;

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
    let (v, d) = evaluate::<true>(&s, Domain::Complex, &|_| None).map_err(NoValue::Error)?;
    let z = if v.is_real_result() { Z::real(v.re) } else { v.z };
    Ok(CVal { re: Val::new(Num::F(z.re), d.clone()), im: Val::new(Num::F(z.im), d) })
}

fn units_text(d: &Dim) -> String {
    if d.is_one() { "1".into() } else { crate::units::Units::default().auto(d).text() }
}

/// Whether principal complex continuations are allowed during evaluation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Domain {
    Real,
    Complex,
}

/// Numeric value with dimension, in the real numbers; problems as messages.
pub fn eval_d(e: &Expr) -> Result<(f64, Dim), String> {
    let (v, d) = evaluate::<true>(e, Domain::Complex, &|_| None)?;
    if v.is_real_result() { Ok((v.re, d)) } else { Err("the value is complex – use Re(…), Im(…), abs(…) or arg(…)".into()) }
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

/// Cardinal trigonometric values and derivatives, using convergent series near the origin.
pub(crate) fn cardinal(f: Func, z: Z) -> (Z, Z) {
    let one = Z::real(1.0);
    if z.abs() < 0.5 {
        let cosc = f == Func::Cosc;
        let mut term = Z::real(if cosc { 0.5 } else { 1.0 });
        let mut value = if cosc { term.mul(z) } else { term };
        let mut derivative = if cosc { term } else { Z::real(0.0) };
        let zz = z.mul(z).neg();
        let mut power = one;
        for k in 1..=10 {
            let degree = 2 * k + usize::from(cosc);
            term = term.scale(1.0 / ((degree + 1) * degree) as f64);
            let dterm = term.mul(power).mul(z).neg();
            derivative = derivative.add(if cosc { dterm.mul(z) } else { dterm }.scale(degree as f64));
            power = power.mul(zz);
            value = value.add(term.mul(power).mul(if cosc { z } else { one }));
        }
        if f == Func::Tanc {
            let cos = z.cos();
            return (value.div(cos), derivative.div(cos).add(value.mul(z.sin()).div(cos.mul(cos))));
        }
        return (value, derivative);
    }
    let value = match f {
        Func::Sinc => z.sin().div(z),
        Func::Cosc => z.scale(0.5).sin().mul(z.scale(0.5).sin()).scale(2.0).div(z),
        Func::Tanc => z.sin().div(z.cos()).div(z),
        _ => unreachable!(),
    };
    let leading = match f {
        Func::Sinc => z.cos(),
        Func::Cosc => z.sin(),
        _ => one.div(z.cos().mul(z.cos())),
    };
    (value, leading.sub(value).div(z))
}

/// A function of a complex argument; None for the ones only defined for real arguments.
fn complex_f(f: Func, z: Z) -> Option<Z> {
    Some(match f {
        Func::Sinc | Func::Cosc | Func::Tanc => cardinal(f, z).0,
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

fn numeric_function(f: Func, v: N, z: Z) -> N {
    let one = Z::real(1.0);
    let derivative = match f {
        Func::Sinc | Func::Cosc | Func::Tanc => cardinal(f, v.z).1,
        Func::Exp => z,
        Func::Ln => one.div(v.z),
        Func::Log => one.div(v.z).scale(1.0 / std::f64::consts::LN_10),
        Func::Sin => v.z.cos(), Func::Cos => v.z.sin().neg(),
        Func::Sinh => v.z.cosh(), Func::Cosh => v.z.sinh(),
        Func::Tan => one.div(v.z.cos().mul(v.z.cos())),
        Func::Tanh => one.div(v.z.cosh().mul(v.z.cosh())),
        Func::Asin | Func::Acos => one.div(one.sub(v.z.mul(v.z)).sqrt()),
        Func::Atan => one.div(one.add(v.z.mul(v.z))),
        Func::Tri if v.re.abs() > 1.0 + v.error.re => return N::real(0.0),
        Func::Tri => Z::real(1.0),
        Func::Rect | Func::Sgn | Func::Floor | Func::Ceil => {
            let distance = match f {
                Func::Rect => (v.re.abs() - 0.5).abs(),
                Func::Sgn => v.re.abs(),
                _ => { let fraction = v.re.rem_euclid(1.0); fraction.min(1.0 - fraction) },
            };
            return N::estimated(z, Z::real(if v.error.re > 0.0 && v.error.re >= distance { f64::INFINITY } else { 0.0 }));
        }
        Func::Factorial => Z::real(z.abs() * (v.abs() + 2.0).ln()),
        _ => unreachable!("projections are handled before analytic functions"),
    };
    v.function(z, derivative)
}

fn eval_z(e: &Expr, dom: Domain) -> Result<(Z, Dim), String> {
    evaluate::<true>(e, dom, &|_| None).map(|(v, d)| (v.z, d))
}

fn evaluate<const WITH_UNITS: bool>(e: &Expr, dom: Domain, vars: &dyn Fn(&str) -> Option<(N, Dim)>) -> Result<(N, Dim), String> {
    let real = |e: &Expr, what: &str| -> Result<(f64, Dim), String> {
        let (z, d) = evaluate::<WITH_UNITS>(e, dom, vars)?;
        if !z.is_real() { return Err(format!("{what} must be real, not complex")); }
        Ok((z.re, d))
    };
    let r = match e {
        Expr::Cases(v) => {
            for (k, (value, condition)) in v.iter().enumerate() {
                let (test, dim) = real(condition, "a cases condition")?;
                if !dim.is_one() { return Err("a cases condition must be unitless".into()); }
                if test != 0.0 {
                    let found = evaluate::<WITH_UNITS>(value, dom, vars)?;
                    if crate::calculus::guard(condition).is_none() { return Ok(found); }
                    // the function's size where the guard fails: the next branch, the closed form's limit there
                    let at_root = v.get(k + 1).and_then(|(w, _)| evaluate::<WITH_UNITS>(w, Domain::Complex, vars).ok()).map_or(0.0, |(w, _)| w.abs());
                    return Ok(removable::<WITH_UNITS>(value, condition, &found, at_root, vars).unwrap_or(found));
                }
            }
            return Err("no cases condition matches here".into());
        }
        Expr::Vector(_) => return Err("a vector has several components – select one with at(vector, index), starting at 0".into()),
        Expr::Matrix(_) => return Err("a matrix has several elements – select one with at(matrix, row, column), starting at 0".into()),
        Expr::Op(..) => return Err(crate::linalg::problem(e).unwrap_or_else(|| "this matrix operation has no value here".into())),
        Expr::C(v) => (N::real(v.v.f()), if WITH_UNITS { v.d.clone() } else { Dim::ONE }),
        Expr::Pi => (N::real(std::f64::consts::PI), Dim::ONE),
        Expr::E => (N::real(std::f64::consts::E), Dim::ONE),
        Expr::I if dom == Domain::Complex => (N::I, Dim::ONE),
        Expr::Const(_, backing) => return evaluate::<WITH_UNITS>(backing, dom, vars),
        Expr::I => return Err("i is imaginary, and only real values work here".into()),
        Expr::Call(n, _) => return Err(format!("{n} isn't defined – define it first with :=")),
        Expr::Sym(n) => return vars(n).ok_or_else(|| format!("'{n}' isn't defined")),
        Expr::Add(ts) => {
            let mut acc: Option<(N, Dim)> = None;
            for t in ts {
                let (v, d) = evaluate::<WITH_UNITS>(t, dom, vars)?;
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
            acc.unwrap_or((N::real(0.0), Dim::ONE))
        }
        Expr::Mul(fs) => {
            let mut acc = (N::real(1.0), Dim::ONE);
            for f in fs {
                let (v, d) = evaluate::<WITH_UNITS>(f, dom, vars)?;
                acc = (acc.0.mul(v), acc.1.mul(&d));
            }
            acc
        }
        Expr::Pow(b, x) => {
            let (bv, bd) = evaluate::<WITH_UNITS>(b, dom, vars)?;
            let (xv, xd) = evaluate::<WITH_UNITS>(x, dom, vars)?;
            if !xd.is_one() {
                return Err("an exponent must be unitless".into());
            }
            let d = pow_dim(&bd, xv.z)?;
            if bv.is_real() && xv.is_real() {
                match real_pow(bv.re, xv.re) {
                    Ok(v) => (bv.real_power(xv, v), d),
                    // a real power without a real value has its principal complex one: √−4 = 2i
                    Err(_) if dom == Domain::Complex && bv.re < 0.0 => (bv.pow(xv), d),
                    Err(m) => return Err(m),
                }
            } else {
                if bv.z == Z::real(0.0) && xv.re < 0.0 {
                    return Err("division by zero".into());
                }
                (bv.pow(xv), d)
            }
        }
        Expr::F(f, a) => {
            if *f == Func::Dirac {
                return Err("dirac is a distribution with no pointwise value; use it in convolve".into());
            }
            let (v, d) = evaluate::<WITH_UNITS>(a, dom, vars)?;
            match f {
                Func::Abs => return Ok((v.projected(v.abs(), v.error.abs()), d)),
                Func::Re => return Ok((v.projected(v.re, v.error.re), d)),
                Func::Im => return Ok((v.projected(v.im, v.error.im), d)),
                Func::Conj => return Ok((v.conj(), d)),
                Func::Arg if v.z == Z::real(0.0) => return Err("arg of 0 isn't defined".into()),
                Func::Arg => return Ok((v.projected(v.arg(), v.error.abs() / v.abs()), Dim::ONE)),
                _ => {}
            }
            if !d.is_one() {
                return Err(format!("the argument of {} must be unitless, it is in {}", f.name(), units_text(&d)));
            }
            let real_value = if v.is_real() { f.eval(v.re) } else { None };
            match (real_value, dom) {
                (Some(r), _) => (numeric_function(*f, v, Z::real(r)), Dim::ONE),
                (None, Domain::Complex) if complex_f(*f, v.z).is_some_and(|z| z.is_finite()) => (numeric_function(*f, v, complex_f(*f, v.z).unwrap()), Dim::ONE),
                (None, Domain::Complex) if !v.is_real() => return Err(format!("{} takes a real argument, not a complex one", f.name())),
                (None, _) => return Err(match f {
                    Func::Ln | Func::Log => format!("{} of a number ≤ 0", f.name()),
                    Func::Asin | Func::Acos => format!("{} of a number outside −1…1", f.name()),
                    _ => format!("{} is out of range here", f.name()),
                }),
            }
        }
        Expr::Cmp(v, rs) => {
            let vals: Vec<(N, Dim)> = v.iter().map(|x| evaluate::<WITH_UNITS>(x, dom, vars)).collect::<Result<_, _>>()?;
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
            (N::real(if ok { 1.0 } else { 0.0 }), Dim::ONE)
        }
        Expr::Antiderivative(..) => return Err("integral: no antiderivative found for this integrand – with bounds, integral_a^b f dif x is evaluated numerically".into()),
        // a value put in (a guarded closed form at a number): the body with the variable at that value
        Expr::Subs(body, var, by) if !body.contains(&|e| matches!(e, Expr::Derivative(..))) => {
            let at = evaluate::<WITH_UNITS>(by, dom, vars)?;
            return evaluate::<WITH_UNITS>(body, dom, &|n| if n == var { Some(at.clone()) } else { vars(n) });
        }
        Expr::Subs(..) | Expr::Derivative(..) => return Err("this derivative needs piecewise or distributional calculus; it is kept symbolic".into()),
        Expr::InverseLaplace(body, var, by) => {
            let (t, dimension) = real(by, "the inverse Laplace evaluation time")?;
            if !dimension.is_one() { return Err("inverse Laplace evaluation time must be unitless".into()); }
            if WITH_UNITS { dim_of(e, &|n| vars(n).map(|v| v.1))?; }
            (N::real(crate::inverse_laplace::evaluate(body, var, t, &|n| vars(n).filter(|v| v.0.is_real()).map(|v| v.0.re))?), Dim::ONE)
        }
        Expr::Sum(body, var, lo, hi) | Expr::Integral(body, var, lo, hi) => {
            let sum = matches!(e, Expr::Sum(..));
            let bound = |e: &Expr| if let Expr::C(v) = e { Ok((v.v.f(), if WITH_UNITS { v.d.clone() } else { Dim::ONE })) } else { real(e, "a bound") };
            let (l, ld) = bound(lo)?;
            let (h, hd) = bound(hi)?;
            if sum && (!ld.is_one() || !hd.is_one()) { return Err("sum bounds must be unitless".into()); }
            let ld = if sum { Dim::ONE } else { bound_dim(Some(ld), lo, Some(hd), hi)?.unwrap_or(Dim::ONE) };
            let sd = |n: &str| if n == var { Some(ld.clone()) } else { vars(n).map(|v| v.1) };
            let d = if WITH_UNITS { dim_of(body, &sd).map_err(|e| unitless_variable(e, var, lo, hi))?.ok_or("unknown integrand dimension")? } else { Dim::ONE };
            let real_vars = |n: &str| vars(n).filter(|(v, _)| v.is_real()).map(|(v, _)| v.re);
            let points = crate::calculus::signal_points(body, var, &real_vars);
            let sample = |x| evaluate::<WITH_UNITS>(body, dom, &|n| if n == var { Some((N::real(x), ld.clone())) } else { vars(n) }).ok().filter(|(_, sd)| *sd == d).map(|(v, _)| v);
            let v = if sum {
                let part = |which| crate::calculus::series_estimated(&|k| sample(k).map(|v| (if which == 0 { v.re } else { v.im }, if which == 0 { v.error.re } else { v.error.im })), l, h, &points, crate::calculus::piecewise_affine(body, var));
                let (re, im) = (part(0)?, part(1)?);
                N::estimated(Z::new(re.0, im.0), Z::new(re.1, im.1))
            } else {
                crate::calculus::complex_quadrature(&sample, l, h, &points)?
            };
            (v, if sum { d } else { d.mul(&ld) })
        }
        Expr::Number(a, k, d, _) => {
            let (v, da) = evaluate::<WITH_UNITS>(a, dom, vars)?;
            if WITH_UNITS && da != *d {
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

/// Whether an integral bound is 0 or ±∞ without a unit: these fit every unit.
pub fn unit_free_bound(e: &Expr) -> bool {
    unitless_number(e).is_some_and(|v| v == 0.0 || v.is_infinite())
}

/// A number without a unit, ±∞ included (−∞ is −1 · ∞).
pub fn unitless_number(e: &Expr) -> Option<f64> {
    match e {
        Expr::C(v) if v.d.is_one() => Some(v.v.f()),
        Expr::Mul(fs) => fs.iter().try_fold(1.0, |p, f| Some(p * unitless_number(f)?)),
        _ => None,
    }
}

/// An integrand's unit error when only 0 and ∞ bound it: they leave the variable without a unit.
fn unitless_variable(e: String, var: &str, lo: &Expr, hi: &Expr) -> String {
    if !(unit_free_bound(lo) && unit_free_bound(hi)) { return e; }
    format!("{e} – bounds of 0 and ∞ give {var} no unit: write it on a bound, like integral_(0 s)^infinity")
}

/// The unit of an integral's variable from its bounds' dimensions (None: unknown); a bound of 0 or ±∞ without a
/// unit takes the other's.
fn bound_dim(ld: Option<Dim>, lo: &Expr, hd: Option<Dim>, hi: &Expr) -> Result<Option<Dim>, String> {
    match (ld, hd) {
        (Some(l), Some(h)) if l == h => Ok(Some(l)),
        (Some(l), Some(h)) if h.is_one() && unit_free_bound(hi) => Ok(Some(l)),
        (Some(l), Some(h)) if l.is_one() && unit_free_bound(lo) => Ok(Some(h)),
        (Some(_), Some(_)) => Err("integral bounds have different units".into()),
        (l, h) => Ok(l.or(h)),
    }
}

/// A plain number, symbols from `vars`; None where it isn't defined (domain, division by zero, unknown names).
pub fn eval_f(e: &Expr, vars: &dyn Fn(&str) -> Option<f64>) -> Option<f64> {
    eval_real(e, vars).or_else(|| sample_complex(e, vars).ok().filter(|v| v.is_real_result()).map(|v| v.re))
}

pub(crate) fn sample_complex(e: &Expr, vars: &dyn Fn(&str) -> Option<f64>) -> Result<N, String> {
    // Like the fast real evaluator, this API consumes SI scalars. Unit checks
    // belong to value evaluation or graph axis inference.
    evaluate::<false>(e, Domain::Complex, &|n| vars(n).map(|v| (N::real(v), Dim::ONE))).map(|v| v.0)
}

fn sample_with_dims(e: &Expr, vars: &dyn Fn(&str) -> Option<f64>, dims: &dyn Fn(&str) -> Dim) -> Result<N, String> {
    // SI values with dimensions supplied by the graph's axis inference.
    evaluate::<true>(e, Domain::Complex, &|n| vars(n).map(|v| (N::real(v), dims(n)))).map(|v| v.0)
}

pub(crate) enum SampleError { Undefined, Complex }

/// A graph sample: undefined points stay gaps; complex values remain distinguishable.
pub(crate) fn graph_sample(e: &Expr, x: &str, t: f64, dimension: &Dim) -> Result<f64, SampleError> {
    let vars = |n: &str| (n == x).then_some(t);
    if let Some(v) = eval_real(e, &vars) { return Ok(v); }
    let v = sample_with_dims(e, &vars, &|_| dimension.clone()).map_err(|_| SampleError::Undefined)?;
    if v.is_real_result() { Ok(v.re) } else { Err(SampleError::Complex) }
}

/// A closed form that holds where `condition` (g ≠ 0 for a parameter x) does, (eˣ − 1)/x: near g = 0 its parts
/// cancel and rounding swamps the value, though the function is analytic there (the singularity is removable).
/// An analytic function is the mean of its values on a circle around the point (Cauchy's mean value theorem), and
/// on a circle far enough from g = 0 the closed form is well-conditioned: the trapezoid rule on it converges
/// geometrically. Rounding is measured against the function's size at g = 0 (`at_root`, the other branch), so
/// values near the function's own zeros don't count as swamped. The radius goes up tenfold from the distance to
/// g = 0, probed with a few samples, until their error estimate (rounding plus the difference to half the
/// samples) stops improving; the best radius then gets the full sum. None where the value is accurate already or
/// the closed form isn't analytic in x (abs, signals, non-integer powers of x).
fn removable<const WITH_UNITS: bool>(value: &Expr, condition: &Expr, found: &(N, Dim), at_root: f64, vars: &dyn Fn(&str) -> Option<(N, Dim)>) -> Option<(N, Dim)> {
    let (z, d) = found;
    let error = z.error.abs();
    if error <= 1e-8 * z.abs().max(at_root) { return None; }
    let (x, root) = crate::calculus::guard(condition)?;
    if !analytic(value, &x) { return None; }
    let (center, xd) = vars(&x)?;
    let root = evaluate::<WITH_UNITS>(&root, Domain::Complex, vars).ok()?.0;
    let distance = center.z.add(root.z.scale(-1.0)).abs();
    // the trapezoid sum on the circle of radius rho with n samples, and its error estimate
    let circle = |rho: f64, n: usize| -> Option<(Z, f64)> {
        let mut points = Vec::with_capacity(n);
        for k in 0..n {
            let angle = std::f64::consts::TAU * k as f64 / n as f64;
            let at = N::from(center.z.add(Z::new(rho * angle.cos(), rho * angle.sin())));
            let sample = |name: &str| if name == x { Some((at, xd.clone())) } else { vars(name) };
            let (v, _) = evaluate::<WITH_UNITS>(value, Domain::Complex, &sample).ok()?;
            if !(v.re.is_finite() && v.im.is_finite()) { return None; }
            points.push(v);
        }
        let mean = |step: usize| points.iter().step_by(step).fold(Z::new(0.0, 0.0), |sum, v| sum.add(v.z)).scale(step as f64 / n as f64);
        let (all, half) = (mean(1), mean(2));
        let rounding = points.iter().map(|v| v.error.abs()).fold(0.0, f64::max);
        Some((all, all.add(half.scale(-1.0)).abs() + rounding))
    };
    // near g = 0 the best radius is the function's, not the point's: the last one found for this closed form
    let key = (value as *const Expr as usize, x.clone());
    if let Some(rho) = RADIUS.with(|r| r.borrow().as_ref().filter(|(k, rho)| *k == key && *rho > 4.0 * distance).map(|(_, rho)| *rho)) {
        if let Some((mean, estimate)) = circle(rho, 32).filter(|(_, e)| *e < error && *e <= 1e-9 * at_root.max(1e-300)) {
            return Some((N::estimated(mean, Z::new(estimate, estimate)), d.clone()));
        }
    }
    let mut best: Option<(f64, f64)> = None;
    let mut rho = (2.0 * distance).max(1e-300);
    for _ in 0..40 {
        if let Some((_, estimate)) = circle(rho, 8) {
            match best {
                Some((_, e)) if estimate >= e => break,
                _ => best = Some((rho, estimate)),
            }
        }
        rho *= 10.0;
    }
    let rho = best?.0;
    let (mean, estimate) = circle(rho, 32)?;
    RADIUS.with(|r| *r.borrow_mut() = Some((key, rho)));
    (estimate < error).then(|| (N::estimated(mean, Z::new(estimate, estimate)), d.clone()))
}

thread_local! {
    /// The closed form (by address, with its parameter) [`removable`] last found a radius for, and the radius.
    static RADIUS: std::cell::RefCell<Option<((usize, String), f64)>> = const { std::cell::RefCell::new(None) };
}

/// Whether an expression is analytic in x (poles allowed): sums, products, integer powers, exp, sin, cos, sinh, cosh.
fn analytic(e: &Expr, x: &str) -> bool {
    if !e.has(x) { return true; }
    match e {
        Expr::Sym(_) => true,
        Expr::Add(v) | Expr::Mul(v) => v.iter().all(|a| analytic(a, x)),
        Expr::Pow(b, k) if !k.has(x) => k.as_num().is_some_and(|n| n.is_int()) && analytic(b, x),
        Expr::Pow(b, k) if !b.has(x) => analytic(k, x),
        Expr::F(Func::Exp | Func::Sin | Func::Cos | Func::Sinh | Func::Cosh | Func::Sinc | Func::Cosc, a) | Expr::Number(a, ..) => analytic(a, x),
        _ => false,
    }
}

fn eval_real(e: &Expr, vars: &dyn Fn(&str) -> Option<f64>) -> Option<f64> {
    let r = match e {
        Expr::Cases(v) => {
            let mut selected = None;
            for (value, condition) in v {
                // a closed form guarded against its removable singularity may need the careful evaluation
                if crate::calculus::guard(condition).is_some() { return None; }
                if eval_real(condition, vars)? != 0.0 { selected = Some(value); break; }
            }
            return eval_real(selected?, vars);
        }
        Expr::Vector(_) | Expr::Matrix(_) | Expr::Op(..) => return None,
        Expr::C(v) => v.v.f(),
        Expr::Pi => std::f64::consts::PI,
        Expr::E => std::f64::consts::E,
        Expr::I => return None,
        Expr::Const(_, backing) => eval_real(backing, vars)?,
        Expr::Sym(n) => vars(n)?,
        Expr::Add(ts) => {
            let mut s = 0.0;
            for t in ts {
                s += eval_real(t, vars)?;
            }
            s
        }
        Expr::Mul(fs) => {
            let mut s = 1.0;
            for f in fs {
                s *= eval_real(f, vars)?;
            }
            s
        }
        Expr::Pow(b, x) => {
            let (b, x) = (eval_real(b, vars)?, eval_real(x, vars)?);
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
        Expr::F(f, a) => f.eval(eval_real(a, vars)?)?,
        Expr::Cmp(v, rs) => {
            let vals: Vec<f64> = v.iter().map(|x| eval_real(x, vars)).collect::<Option<_>>()?;
            let ok = rs.iter().enumerate().all(|(i, r)| r.holds(vals[i].total_cmp(&vals[i + 1]), close(vals[i], vals[i + 1])));
            if ok { 1.0 } else { 0.0 }
        }
        Expr::Call(..) | Expr::Subs(..) | Expr::Derivative(..) | Expr::Antiderivative(..) => return None,
        Expr::InverseLaplace(body, var, by) => crate::inverse_laplace::evaluate(body, var, eval_real(by, vars)?, vars).ok()?,
        Expr::Sum(body, var, lo, hi) => {
            let bound = |e: &Expr| if let Expr::C(v) = e { Some(v.v.f()) } else { eval_real(e, vars) };
            let points = crate::calculus::signal_points(body, var, vars);
            crate::calculus::series(&|k| eval_real(body, &|n| if n == var { Some(k) } else { vars(n) }), bound(lo)?, bound(hi)?, &points, crate::calculus::piecewise_affine(body, var)).ok()?
        }
        Expr::Integral(body, var, lo, hi) => {
            let bound = |e: &Expr| if let Expr::C(v) = e { Some(v.v.f()) } else { eval_real(e, vars) };
            let points = crate::calculus::signal_points(body, var, vars);
            crate::calculus::piecewise_quadrature(&|x| eval_real(body, &|n| if n == var { Some(x) } else { vars(n) }), bound(lo)?, bound(hi)?, &points).ok()?
        }
        Expr::Number(a, k, ..) => eval_real(a, vars)? / k.f(),
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
                if t.as_c().is_some_and(|v| v.is_zero() && v.d.is_one()) {
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
            } else if *f == Func::Dirac {
                d.map(|d| d.powi(-1))
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
        Expr::Antiderivative(body, var, by) => {
            let d = dim_of(by, sd)?;
            match (dim_of(body, &|n| if n == var { d.clone() } else { sd(n) })?, d) {
                (Some(a), Some(b)) => Some(a.mul(&b)),
                _ => None,
            }
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
            let ld = bound_dim(dim_of(lo, sd)?, lo, dim_of(hi, sd)?, hi)?;
            match (ld.clone(), dim_of(body, &|n| if n == var { ld.clone() } else { sd(n) }).map_err(|e| unitless_variable(e, var, lo, hi))?) {
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
