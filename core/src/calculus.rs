use crate::expr::{Expr, Func, add, div, func, int, mul, neg, pow, simp, sub, sym};
use crate::num::Num;

pub fn integral(body: Expr, var: String, lo: Expr, hi: Expr) -> Expr {
    let (lo, hi) = unit_bounds(lo, hi);
    if lo == hi && !lo.contains(&infinite) { return int(0); }
    let bounds_match = match (crate::eval::dim_of(&lo, &|_| None), crate::eval::dim_of(&hi, &|_| None)) {
        (Ok(Some(a)), Ok(Some(b))) => a == b,
        _ => true,
    };
    // where an antiderivative jumps or is undefined between the bounds: 1/u and ln u at u = 0, tan u at cos u = 0
    let crosses_pole = body.contains(&|e| {
        let at = |a: &Expr| (crate::eval::eval_f(&a.subst(&var, &lo), &|_| None), crate::eval::eval_f(&a.subst(&var, &hi), &|_| None));
        match e {
            Expr::Pow(base, power) if power.as_num().is_some_and(|n| n.is_neg()) => matches!(at(base), (Some(a), Some(b)) if a * b <= 0.0),
            Expr::F(Func::Ln, a) => matches!(at(a), (Some(a), Some(b)) if a * b <= 0.0),
            Expr::F(Func::Tan, a) => matches!(at(a), (Some(a), Some(b)) if {
                let pole = |u: f64| (u / std::f64::consts::PI - 0.5).floor();
                pole(a) != pole(b) || (a / std::f64::consts::PI - 0.5).fract() == 0.0
            }),
            _ => false,
        }
    });
    // the antiderivative at a numeric bound must have a value (ln u at u = 0 has none, the integral may converge)
    let defined_at = |f: &Expr, bound: &Expr| !crate::eval::eval_f(&f.subst(&var, bound), &|_| None).is_some_and(|v| !v.is_finite());
    if let Some(f) = primitive(&simp(&body), &var) {
        if bounds_match && !crosses_pole && !lo.contains(&infinite) && !hi.contains(&infinite) && defined_at(&f, &lo) && defined_at(&f, &hi) {
            return simp(&sub(f.subst(&var, &hi), f.subst(&var, &lo)));
        }
    }
    Expr::Integral(Box::new(body), var, Box::new(lo), Box::new(hi))
}

/// 0 and ±∞ fit every unit: such a bound without one takes the other bound's (integral_0^(2 s)).
fn unit_bounds(lo: Expr, hi: Expr) -> (Expr, Expr) {
    let dim = |e: &Expr| crate::eval::dim_of(e, &|_| None).ok().flatten().filter(|d| !d.is_one());
    let with = |free: Expr, other: &Expr| match (crate::eval::unit_free_bound(&free), dim(other)) {
        (true, Some(d)) => match free {
            Expr::C(v) => Expr::C(crate::expr::Val::new(v.v, d)),
            free => crate::eval::unitless_number(&free).map_or(free, |v| Expr::C(crate::expr::Val::new(Num::F(v), d))),
        },
        _ => free,
    };
    let lo2 = with(lo.clone(), &hi);
    let hi2 = with(hi, &lo);
    (lo2, hi2)
}

fn infinite(e: &Expr) -> bool {
    matches!(e, Expr::C(v) if !v.v.f().is_finite())
}

/// An antiderivative of `body` in `var`, without the constant of integration.
pub fn antiderivative(body: &Expr, var: &str) -> Result<Expr, String> {
    primitive(&simp(body), var).map(|f| simp(&f)).ok_or_else(|| {
        "integral: no antiderivative found for this integrand – with bounds, integral_a^b f dif x is evaluated numerically".into()
    })
}

/// An antiderivative found by the rules below, or None: linearity, powers and functions of a linear argument,
/// expanded products and integration by parts of a polynomial times a factor integrated again and again.
fn primitive(e: &Expr, x: &str) -> Option<Expr> {
    if !e.has(x) { return Some(mul(vec![e.clone(), sym(x)])); }
    match e {
        Expr::Sym(n) if n == x => Some(div(pow(sym(x), int(2)), int(2))),
        Expr::Add(ts) => Some(add(ts.iter().map(|t| primitive(t, x)).collect::<Option<_>>()?)),
        Expr::Mul(fs) => {
            let (constant, varying): (Vec<_>, Vec<_>) = fs.iter().cloned().partition(|f| !f.has(x));
            let found = match varying.as_slice() {
                [one] => primitive(one, x),
                _ => expanded(e, x).or_else(|| by_parts(&varying, x)),
            };
            Some(mul(vec![mul(constant), found?]))
        }
        Expr::Pow(a, b) if !b.has(x) => {
            let slope = a.diff(x);
            if slope.has(x) || slope.is_zero() { return expanded(e, x); }
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
        Expr::F(f, a) => {
            let slope = a.diff(x);
            if slope.has(x) || slope.is_zero() { return None; }
            let a = (**a).clone();
            let result = match f {
                Func::Exp => func(Func::Exp, a),
                Func::Sin => neg(func(Func::Cos, a)),
                Func::Cos => func(Func::Sin, a),
                Func::Sinh => func(Func::Cosh, a),
                Func::Cosh => func(Func::Sinh, a),
                // u ln u − u and −ln|cos u|
                Func::Ln => sub(mul(vec![a.clone(), func(Func::Ln, a.clone())]), a),
                Func::Tan => neg(func(Func::Ln, func(Func::Abs, func(Func::Cos, a)))),
                _ => return None,
            };
            Some(div(result, slope))
        }
        _ => None,
    }
}

/// A product or power that multiplies out into a sum: each term on its own.
fn expanded(e: &Expr, x: &str) -> Option<Expr> {
    let terms = crate::expr::expand(e);
    if !matches!(terms, Expr::Add(_)) || terms == *e { return None; }
    primitive(&terms, x)
}

/// ∫ p g = p G₁ − p′ G₂ + p″ G₃ − …, for a polynomial p and a factor g whose repeated antiderivatives Gₖ are found
/// (an exponential, sin, cos, sinh or cosh of a linear argument). The sum ends after p's degree.
fn by_parts(factors: &[Expr], x: &str) -> Option<Expr> {
    let (polynomial, rest): (Vec<_>, Vec<_>) = factors.iter().cloned().partition(|f| crate::poly::coeffs(f, x).is_some());
    let [g] = rest.as_slice() else { return None };
    let mut p = simp(&mul(polynomial));
    let mut integrated = primitive(g, x)?;
    let mut terms = vec![];
    for k in 0..=crate::poly::coeffs(&p, x)?.len() {
        if p.is_zero() { return Some(add(terms)); }
        let term = mul(vec![p.clone(), integrated.clone()]);
        terms.push(if k % 2 == 0 { term } else { neg(term) });
        p = simp(&p.diff(x));
        integrated = primitive(&simp(&integrated), x)?;
    }
    None
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
        if let Expr::F(f, argument) = e {
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
        if let Expr::Cmp(values, _) = e {
            for pair in values.windows(2) {
                let argument = sub(pair[0].clone(), pair[1].clone());
                let slope = argument.diff(var);
                if !slope.has(var) {
                    if let (Some(a), Some(b)) = (crate::eval::eval_f(&slope, vars), crate::eval::eval_f(&argument, &|n| if n == var { Some(0.0) } else { vars(n) })) {
                        if a != 0.0 && (-b / a).is_finite() { points.push(-b / a); }
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

/// Expand small finite sums exactly; keep larger and infinite sums scoped.
pub fn sum(body: Expr, var: String, lo: Expr, hi: Expr) -> Expr {
    let bounds = lo.as_num().and_then(Num::as_int).zip(hi.as_num().and_then(Num::as_int));
    if let Some((l, h)) = bounds {
        if h < l { return int(0); }
        if h.checked_sub(l).is_some_and(|n| n < 256) {
            return simp(&add((l..=h).map(|k| body.subst(&var, &int(k.into()))).collect()));
        }
    }
    Expr::Sum(Box::new(body), var, Box::new(lo), Box::new(hi))
}

/// These expressions are constant between their affine signal/condition corners.
pub fn piecewise_constant(e: &Expr, var: &str) -> bool {
    piecewise_degree(e, var) == Some(0)
}

/// Linear pieces can be summed as arithmetic progressions between signal corners.
pub fn piecewise_affine(e: &Expr, var: &str) -> bool {
    piecewise_degree(e, var).is_some()
}

fn piecewise_degree(e: &Expr, var: &str) -> Option<u8> {
    if !e.has(var) { return Some(0); }
    let affine = |a: &Expr| {
        let slope = a.diff(var);
        !slope.has(var) && !slope.contains(&|e| matches!(e, Expr::Derivative(..)))
    };
    match e {
        Expr::Sym(n) if n == var => Some(1),
        Expr::Cases(branches) => branches.iter().try_fold(0, |degree, (v, c)| {
            if piecewise_degree(c, var)? != 0 { return None; }
            Some(degree.max(piecewise_degree(v, var)?))
        }),
        Expr::Cmp(v, _) => v.windows(2).all(|p| affine(&sub(p[0].clone(), p[1].clone()))).then_some(0),
        Expr::F(Func::Rect | Func::Sgn, a) => affine(a).then_some(0),
        Expr::F(Func::Tri, a) => affine(a).then_some(1),
        Expr::Add(v) => v.iter().try_fold(0, |degree, a| Some(degree.max(piecewise_degree(a, var)?))),
        Expr::Mul(v) => v.iter().try_fold(0, |degree, a| {
            let next = degree + piecewise_degree(a, var)?;
            (next <= 1).then_some(next)
        }),
        _ => None,
    }
}

/// Bounded numerical series evaluation, using compensated accumulation and a
/// small absolute tail over consecutive blocks as a convergence estimate.
pub fn series(f: &impl Fn(f64) -> Option<f64>, lo: f64, hi: f64, points: &[f64], affine: bool) -> Result<f64, String> {
    let integer = |x: f64| x.is_finite() && x.fract() == 0.0 && x.abs() < 9_007_199_254_740_992.0;
    if !integer(lo) || !(integer(hi) || hi == f64::INFINITY) {
        return Err("sum bounds must be unitless integers (the upper bound may be infinity)".into());
    }
    if hi < lo { return Ok(0.0); }
    // Isolate boundary values, then sum each linear interval using its first
    // and last integer values. Work depends on corners, not the index range.
    if affine {
        let mut cuts = vec![lo];
        for point in points {
            for k in [point.floor(), point.floor() + 1.0] {
                if k > lo && k <= hi { cuts.push(k); }
            }
        }
        if hi.is_finite() { cuts.push(hi + 1.0); }
        cuts.sort_by(f64::total_cmp);
        cuts.dedup();
        let mut total = 0.0;
        for interval in cuts.windows(2) {
            let first = f(interval[0]).filter(|v| v.is_finite()).ok_or("a sum term is undefined")?;
            let last = f(interval[1] - 1.0).filter(|v| v.is_finite()).ok_or("a sum term is undefined")?;
            total += (interval[1] - interval[0]) * (first * 0.5 + last * 0.5);
        }
        if hi == f64::INFINITY {
            let start = *cuts.last().unwrap();
            if f(start) != Some(0.0) || f(start + 1.0) != Some(0.0) {
                return Err("sum did not converge: its tail is a nonzero affine function".into());
            }
        }
        return total.is_finite().then_some(total).ok_or_else(|| "sum overflow".into());
    }
    let last_corner = points.last().copied().unwrap_or(f64::NEG_INFINITY);
    let (mut total, mut correction, mut tail, mut stable, mut previous_tail) = (0.0_f64, 0.0, 0.0, 0, f64::INFINITY);
    for offset in 0..100_000 {
        let k = lo + offset as f64;
        if k > hi { return Ok(total); }
        let term = f(k).filter(|v| v.is_finite()).ok_or("a sum term is undefined")?;
        let adjusted = term - correction;
        let next = total + adjusted;
        correction = (next - total) - adjusted;
        total = next;
        if !total.is_finite() { return Err("sum overflow".into()); }
        tail += term.abs();
        if offset % 32 == 31 {
            if hi == f64::INFINITY && k > last_corner && (tail == 0.0 || tail < previous_tail * 0.99) && tail <= 1e-12 * total.abs().max(1.0) {
                stable += 1;
                if stable == 4 { return Ok(total); }
            } else { stable = 0; }
            previous_tail = tail;
            tail = 0.0;
        }
        if k == hi { return Ok(total); }
    }
    Err("sum did not converge within 100000 terms".into())
}
