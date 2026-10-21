use crate::expr::{Expr, Func, add, div, func, int, mul, neg, pow, simp, sub, sym};
use crate::num::Num;

pub fn integral(body: Expr, var: String, lo: Expr, hi: Expr) -> Expr {
    if lo == hi && !lo.contains(&infinite) { return int(0); }
    let bounds_match = match (crate::eval::dim_of(&lo, &|_| None), crate::eval::dim_of(&hi, &|_| None)) {
        (Ok(Some(a)), Ok(Some(b))) => a == b,
        _ => true,
    };
    let crosses_pole = body.contains(&|e| {
        let Expr::Pow(base, power) = e else { return false; };
        if !power.as_num().is_some_and(|n| n.is_neg()) { return false; }
        let at = |bound: &Expr| crate::eval::eval_f(&base.subst(&var, bound), &|_| None);
        matches!((at(&lo), at(&hi)), (Some(a), Some(b)) if a * b <= 0.0)
    });
    if let Some(f) = primitive(&simp(&body), &var) {
        if bounds_match && !crosses_pole && !lo.contains(&infinite) && !hi.contains(&infinite) {
            return simp(&sub(f.subst(&var, &hi), f.subst(&var, &lo)));
        }
    }
    Expr::Integral(Box::new(body), var, Box::new(lo), Box::new(hi))
}

fn infinite(e: &Expr) -> bool {
    matches!(e, Expr::C(v) if !v.v.f().is_finite())
}

fn primitive(e: &Expr, x: &str) -> Option<Expr> {
    if !e.has(x) { return Some(mul(vec![e.clone(), sym(x)])); }
    match e {
        Expr::Sym(n) if n == x => Some(div(pow(sym(x), int(2)), int(2))),
        Expr::Add(ts) => Some(add(ts.iter().map(|t| primitive(t, x)).collect::<Option<_>>()?)),
        Expr::Mul(fs) => {
            let (constant, varying): (Vec<_>, Vec<_>) = fs.iter().cloned().partition(|f| !f.has(x));
            if varying.len() != 1 { return None; }
            Some(mul(vec![mul(constant), primitive(&varying[0], x)?]))
        }
        Expr::Pow(a, b) if !b.has(x) => {
            let slope = a.diff(x);
            if slope.has(x) || slope.is_zero() { return None; }
            let f = if b.as_num().is_some_and(|n| n.same(Num::int(-1))) {
                func(Func::Ln, func(Func::Abs, (**a).clone()))
            } else {
                let next = simp(&add(vec![(**b).clone(), int(1)]));
                div(pow((**a).clone(), next.clone()), next)
            };
            Some(div(f, slope))
        }
        Expr::Pow(a, b) if !a.has(x) => {
            let slope = b.diff(x);
            if slope.has(x) || slope.is_zero() { return None; }
            Some(div(e.clone(), mul(vec![slope, func(Func::Ln, (**a).clone())])))
        }
        Expr::F(f, a) | Expr::Op(_, f, a) => {
            let slope = a.diff(x);
            if slope.has(x) || slope.is_zero() { return None; }
            let a = (**a).clone();
            let result = match f {
                Func::Exp => func(Func::Exp, a),
                Func::Sin => neg(func(Func::Cos, a)),
                Func::Cos => func(Func::Sin, a),
                Func::Sinh => func(Func::Cosh, a),
                Func::Cosh => func(Func::Sinh, a),
                _ => return None,
            };
            Some(div(result, slope))
        }
        _ => None,
    }
}

/// Adaptive Gauss quadrature keeps samples away from improper endpoints.
pub fn quadrature(f: &impl Fn(f64) -> Option<f64>, lo: f64, hi: f64) -> Result<f64, String> {
    if lo.is_nan() || hi.is_nan() { return Err("invalid integral bounds".into()); }
    if lo == hi { return if lo.is_finite() { Ok(0.0) } else { Err("invalid integral bounds".into()) }; }
    if lo > hi { return quadrature(f, hi, lo).map(|v| -v); }
    let mapped = |t: f64| {
        let (x, jac) = if lo.is_finite() && hi.is_finite() {
            (lo + (hi - lo) * t, hi - lo)
        } else if lo.is_finite() {
            (lo + t / (1.0 - t), 1.0 / (1.0 - t).powi(2))
        } else if hi.is_finite() {
            (hi - (1.0 - t) / t, 1.0 / t.powi(2))
        } else {
            let a = std::f64::consts::PI * (t - 0.5);
            (a.tan(), std::f64::consts::PI / a.cos().powi(2))
        };
        let y = f(x)? * jac;
        y.is_finite().then_some(y)
    };
    fn gauss(f: &impl Fn(f64) -> Option<f64>, a: f64, b: f64) -> Option<f64> {
        let (mid, half) = ((a + b) * 0.5, (b - a) * 0.5);
        let mut sum = 0.0;
        for (x, w) in [(0.1834346424956498, 0.362683783378362), (0.525532409916329, 0.3137066458778873), (0.7966664774136267, 0.2223810344533745), (0.9602898564975363, 0.1012285362903763)] {
            sum += w * (f(mid - half * x)? + f(mid + half * x)?);
        }
        Some(sum * half)
    }
    fn adapt(f: &impl Fn(f64) -> Option<f64>, a: f64, b: f64, whole: f64, tol: f64, depth: u32, budget: &mut u32) -> Option<f64> {
        if *budget == 0 { return None; }
        *budget -= 1;
        let mid = (a + b) * 0.5;
        let (l, r) = (gauss(f, a, mid)?, gauss(f, mid, b)?);
        if (l + r - whole).abs() <= tol { return Some(l + r); }
        if depth == 0 { return None; }
        Some(adapt(f, a, mid, l, tol * 0.5, depth - 1, budget)? + adapt(f, mid, b, r, tol * 0.5, depth - 1, budget)?)
    }
    if mapped(0.5).is_none() { return Err("integrand is undefined inside the interval".into()); }
    let result = gauss(&mapped, 0.0, 1.0).and_then(|whole| adapt(&mapped, 0.0, 1.0, whole, 1e-10 * whole.abs().max(1.0), 32, &mut 20000));
    result.filter(|v| v.is_finite()).ok_or_else(|| "integral did not converge or its integrand is undefined".into())
}

/// Split affine signal arguments at their corners so quadrature never skips a narrow pulse.
pub fn signal_points(body: &Expr, var: &str, vars: &dyn Fn(&str) -> Option<f64>) -> Vec<f64> {
    let mut points = Vec::new();
    fn collect(e: &Expr, var: &str, vars: &dyn Fn(&str) -> Option<f64>, points: &mut Vec<f64>) {
        if let Expr::F(f, argument) | Expr::Op(_, f, argument) = e {
            let levels: &[f64] = match f { Func::Rect => &[-0.5, 0.5], Func::Tri => &[-1.0, 0.0, 1.0], Func::Sgn => &[0.0], _ => &[] };
            if !levels.is_empty() {
                let slope = argument.diff(var);
                if !slope.has(var) {
                    if let (Some(a), Some(b)) = (crate::eval::eval_f(&slope, vars), crate::eval::eval_f(argument, &|n| if n == var { Some(0.0) } else { vars(n) })) {
                        if a != 0.0 { points.extend(levels.iter().map(|level| (level - b) / a).filter(|point| point.is_finite())); }
                    }
                }
            }
        }
        for child in e.children() { collect(child, var, vars, points); }
    }
    collect(body, var, vars, &mut points);
    points.sort_by(f64::total_cmp);
    points.dedup();
    points
}

pub fn piecewise_quadrature(f: &impl Fn(f64) -> Option<f64>, lo: f64, hi: f64, points: &[f64]) -> Result<f64, String> {
    if lo > hi { return piecewise_quadrature(f, hi, lo, points).map(|v| -v); }
    let mut bounds = vec![lo];
    bounds.extend(points.iter().copied().filter(|point| *point > lo && *point < hi));
    bounds.push(hi);
    bounds.windows(2).try_fold(0.0, |sum, bound| quadrature(f, bound[0], bound[1]).map(|value| sum + value))
}
