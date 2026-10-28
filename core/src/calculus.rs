use crate::expr::{Expr, Func, add, div, func, int, mul, neg, pow, simp, sub, sym};
use crate::num::Num;

pub fn integral(body: Expr, var: String, lo: Expr, hi: Expr) -> Expr {
    let (lo, hi) = unit_bounds(lo, hi);
    if lo == hi && !lo.contains(&infinite) { return int(0); }
    if let Some(value) = crate::spectral::integral(&body, &var, &lo, &hi) { return value; }
    match closed(&body, &var, &lo, &hi) {
        Some((value, nonzero)) => guarded(value, nonzero, &body, &var, &lo, &hi),
        None => Expr::Integral(Box::new(body), var, Box::new(lo), Box::new(hi)),
    }
}

/// The definite integral in closed form, with the expressions it divided by (each must be ≠ 0 for it to hold),
/// or None.
fn closed(body: &Expr, var: &str, lo: &Expr, hi: &Expr) -> Option<(Expr, Vec<Expr>)> {
    if let Some(found) = piecewise(body, var, lo, hi) { return Some(found); }
    let bounds_match = match (crate::eval::dim_of(&lo, &|_| None), crate::eval::dim_of(&hi, &|_| None)) {
        (Ok(Some(a)), Ok(Some(b))) => a == b,
        _ => true,
    };
    // where an antiderivative jumps or is undefined between the bounds: 1/u and ln u at u = 0, tan u at cos u = 0
    let normalized = simp(body);
    let crosses_pole = normalized.contains(&|e| {
        let at = |a: &Expr| (crate::eval::eval_f(&a.subst(&var, &lo), &|_| None), crate::eval::eval_f(&a.subst(&var, &hi), &|_| None));
        match e {
            Expr::Pow(base, power) if power.as_num().is_some_and(|n| n.is_neg()) => {
                // An affine root with exponent in (-1, 0) is an integrable
                // algebraic singularity; its continuous primitive can be used.
                if power.as_num().is_some_and(|n| n.f() > -1.0)
                    && crate::poly::coeffs(base, var).is_some_and(|cs| cs.len() == 2) {
                    return false;
                }
                let crosses = matches!(at(base), (Some(a), Some(b)) if a * b <= 0.0);
                let roots_inside = (|| {
                    let l = crate::eval::eval_f(lo, &|_| None)?;
                    let h = crate::eval::eval_f(hi, &|_| None)?;
                    let cs = crate::poly::coeffs(base, var)?;
                    let cs = cs.iter().map(|c| crate::eval::eval_f(c, &|_| None)).collect::<Option<Vec<_>>>()?;
                    Some(crate::solve::roots(&cs).iter().any(|r| *r >= l.min(h) && *r <= l.max(h)))
                })().unwrap_or(false);
                crosses || roots_inside
            },
            Expr::F(Func::Ln, a) => matches!(at(a), (Some(a), Some(b)) if a * b <= 0.0),
            Expr::F(Func::Tan, a) => matches!(at(a), (Some(a), Some(b)) if {
                let pole = |u: f64| (u / std::f64::consts::PI - 0.5).floor();
                pole(a) != pole(b) || (a / std::f64::consts::PI - 0.5).fract() == 0.0
            }),
            _ => false,
        }
    });
    // the antiderivative at a numeric bound must have a value (ln u at u = 0 has none, the integral may converge)
    let defined_at = |f: &Expr, bound: &Expr| !crate::eval::eval_f(&f.subst(var, bound), &|_| None).is_some_and(|v| !v.is_finite());
    if !bounds_match || crosses_pole || lo.contains(&infinite) || hi.contains(&infinite) { return None; }
    let mut nonzero = vec![];
    let f = primitive(&simp(body), var, &mut nonzero)?;
    (defined_at(&f, lo) && defined_at(&f, hi)).then(|| {
        let lower = simp(&f.subst(var, lo));
        let upper = simp(&f.subst(var, hi));
        // The primitive has the same dimension at both endpoints. A zero endpoint
        // must not constrain the dimension of a bound that is still symbolic.
        let value = if lower.is_zero() { upper } else if upper.is_zero() { neg(lower) } else { sub(upper, lower) };
        (simp(&value), nonzero)
    })
}

/// Signals (rect, tri, sgn, abs, conditions) with numeric affine corners: between two corners each is the
/// polynomial or constant it is there, so the integral is the sum of the pieces' integrals. Pieces where the body
/// is 0 drop out, which makes infinite bounds finite for signals of bounded support (Fourier integrals of pulses).
fn piecewise(body: &Expr, var: &str, lo: &Expr, hi: &Expr) -> Option<(Expr, Vec<Expr>)> {
    use crate::convolution::{breakpoints, inside, resolve, sorted};
    // distributions aren't functions of pieces (δ(0) has no value); the convolution sifts them
    if body.contains(&|e| matches!(e, Expr::F(Func::Dirac, _))) { return None; }
    let mut points = vec![];
    breakpoints(body, var, &mut points)?;
    if points.is_empty() { return None; }
    let (Expr::C(l), Expr::C(h)) = (lo, hi) else { return None };
    if h.v.cmp(l.v) == std::cmp::Ordering::Less {
        let (value, nonzero) = piecewise(body, var, hi, lo)?;
        return Some((neg(value), nonzero));
    }
    // var's dimension: its corners' (a unitless 0 fits any), else the bounds'
    let d = points.iter().find(|p| !(p.is_zero() && p.d.is_one())).map_or_else(|| l.d.clone(), |p| p.d.clone());
    let finite = |v: &crate::expr::Val| v.v.f().is_finite();
    let mut ends = vec![l.clone()];
    ends.extend(sorted(points).into_iter().filter(|p| p.v.cmp(l.v).is_gt() && p.v.cmp(h.v).is_lt()).map(|p| {
        if p.is_zero() { crate::expr::Val::new(Num::ZERO, d.clone()) } else { p }
    }));
    ends.push(h.clone());
    let (mut values, mut nonzero) = (vec![], vec![]);
    for w in ends.windows(2) {
        let sample = inside(Some(&w[0]).filter(|v| finite(v)), Some(&w[1]).filter(|v| finite(v)), d.clone())?;
        let piece = simp(&resolve(body, var, &sample)?);
        if piece.is_zero() { continue; }
        if !finite(&w[0]) || !finite(&w[1]) { return None; }
        let (value, conditions) = closed(&piece, var, &Expr::C(w[0].clone()), &Expr::C(w[1].clone()))?;
        values.push(value);
        nonzero.extend(conditions);
    }
    Some((simp(&add(values)), nonzero))
}

/// The closed form where what it divided by isn't 0, and the integral at the parameter values where it is.
/// A condition is solved for a parameter it holds linearly (f ≠ 0 from −2πi f, a ≠ −b from a + b): that branch is
/// the integral with the parameter put in. Conditions with no such parameter stay unchecked.
fn guarded(value: Expr, nonzero: Vec<Expr>, body: &Expr, var: &str, lo: &Expr, hi: &Expr) -> Expr {
    let mut seen: Vec<(Expr, String, Expr)> = vec![];
    for condition in nonzero {
        let condition = essential(&condition);
        if seen.iter().any(|(c, ..)| *c == condition) { continue; }
        let Some((name, root)) = parameter(&condition) else { continue };
        seen.push((condition, name, root));
    }
    seen.into_iter().rev().fold(value, |inner, (condition, name, root)| {
        let at = |e: &Expr| simp(&e.subst(&name, &root));
        let otherwise = integral(at(body), var.to_string(), at(lo), at(hi));
        let test = Expr::Cmp(vec![condition, int(0)], vec![crate::parse::Rel::Ne]);
        simp(&Expr::Cases(vec![(inner, test), (otherwise, int(1))]))
    })
}

/// A parameter the expression holds linearly, with a numeric coefficient, and where the expression is 0 for it.
pub(crate) fn parameter(e: &Expr) -> Option<(String, Expr)> {
    e.syms().into_iter().find_map(|n| {
        let cs = crate::poly::coeffs(e, &n)?;
        let [c0, c1] = cs.as_slice() else { return None };
        c1.as_num().filter(|k| !k.is_zero())?;
        Some((n.clone(), simp(&neg(div(c0.clone(), c1.clone())))))
    })
}

/// A condition `g ≠ 0` from [`guarded`]: the parameter and where g is 0.
pub(crate) fn guard(condition: &Expr) -> Option<(String, Expr)> {
    match condition {
        Expr::Cmp(sides, rels) if rels.as_slice() == [crate::parse::Rel::Ne] && sides.len() == 2 && sides[1].is_zero() => parameter(&sides[0]),
        _ => None,
    }
}

/// Whether e holds a closed form from [`guarded`] whose guard is on x.
pub(crate) fn guarded_in(e: &Expr, x: &str) -> bool {
    e.contains(&|c| matches!(c, Expr::Cases(branches) if branches.iter().any(|(_, condition)| guard(condition).is_some_and(|(n, _)| n == x))))
}

/// Whether x = by is where a guard of e on x fails: there the other branch is the value, exactly.
pub(crate) fn at_singularity(e: &Expr, x: &str, by: &Expr) -> bool {
    e.contains(&|c| matches!(c, Expr::Cases(branches) if branches.iter().any(|(_, condition)| {
        guard(condition).is_some_and(|(n, _)| n == x) && simp(&condition.subst(x, by)).as_num().is_some_and(|v| v.is_zero())
    })))
}

/// What decides whether an expression is 0: without nonzero constant factors and positive powers (−2πi f² → f).
fn essential(e: &Expr) -> Expr {
    let base = |f: &Expr| match f {
        Expr::Pow(b, k) if k.as_num().is_some_and(|k| !k.is_neg() && !k.is_zero()) => (**b).clone(),
        f => f.clone(),
    };
    match simp(e) {
        Expr::Mul(fs) => simp(&mul(fs.iter().filter(|f| !f.syms().is_empty()).map(base).collect())),
        e => base(&e),
    }
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
pub fn antiderivative(body: &Expr, var: &str) -> Expr {
    antiderivative_at(simp(body), var.to_string(), sym(var))
}

/// Keep an unresolved primitive with its local variable and a separate evaluation point.
/// This lets later definitions resolve the integrand before substituting a function argument.
pub fn antiderivative_at(body: Expr, var: String, by: Expr) -> Expr {
    match primitive(&body, &var, &mut vec![]) {
        Some(f) => simp(&f.subst(&var, &by)),
        None => Expr::Antiderivative(Box::new(body), var, Box::new(by)),
    }
}

/// An antiderivative found by the rules below, or None: linearity, powers and functions of a linear argument,
/// expanded products and integration by parts of a polynomial times a factor integrated again and again.
/// What it divides by that has parameters in it goes to `nonzero`: the result holds where they aren't 0.
fn primitive(e: &Expr, x: &str, nonzero: &mut Vec<Expr>) -> Option<Expr> {
    if !e.has(x) { return Some(mul(vec![e.clone(), sym(x)])); }
    match e {
        Expr::Sym(n) if n == x => Some(div(pow(sym(x), int(2)), int(2))),
        Expr::Add(ts) => Some(add(ts.iter().map(|t| primitive(t, x, nonzero)).collect::<Option<_>>()?)),
        Expr::Mul(fs) => {
            let (constant, varying): (Vec<_>, Vec<_>) = fs.iter().cloned().partition(|f| !f.has(x));
            let found = match varying.as_slice() {
                [one] => primitive(one, x, nonzero),
                _ => expanded(e, x, nonzero).or_else(|| by_parts(&varying, x, nonzero)),
            };
            Some(mul(vec![mul(constant), found?]))
        }
        Expr::Pow(a, b) if !b.has(x) => {
            let slope = a.diff(x);
            if slope.has(x) || slope.is_zero() { return expanded(e, x, nonzero); }
            divides(&slope, nonzero);
            let f = if b.as_num().is_some_and(|n| n.same(Num::int(-1))) {
                func(Func::Ln, func(Func::Abs, (**a).clone()))
            } else {
                let next = simp(&add(vec![(**b).clone(), int(1)]));
                divides(&next, nonzero);
                div(pow((**a).clone(), next.clone()), next)
            };
            Some(div(f, slope))
        }
        Expr::Pow(a, b) if !a.has(x) => {
            let slope = b.diff(x);
            if slope.has(x) || slope.is_zero() { return None; }
            divides(&slope, nonzero);
            // ln a is 0 at a = 1
            divides(&sub((**a).clone(), int(1)), nonzero);
            Some(div(e.clone(), mul(vec![slope, func(Func::Ln, (**a).clone())])))
        }
        Expr::F(f, a) => {
            let slope = a.diff(x);
            if slope.has(x) || slope.is_zero() { return None; }
            divides(&slope, nonzero);
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

fn divides(by: &Expr, nonzero: &mut Vec<Expr>) {
    if !by.syms().is_empty() { nonzero.push(by.clone()); }
}

/// A product or power that multiplies out into a sum: each term on its own.
fn expanded(e: &Expr, x: &str, nonzero: &mut Vec<Expr>) -> Option<Expr> {
    let terms = crate::expr::expand(e);
    if !matches!(terms, Expr::Add(_)) || terms == *e { return None; }
    primitive(&terms, x, nonzero)
}

/// ∫ p g = p G₁ − p′ G₂ + p″ G₃ − …, for a polynomial p and a factor g whose repeated antiderivatives Gₖ are found
/// (an exponential, sin, cos, sinh or cosh of a linear argument). The sum ends after p's degree.
fn by_parts(factors: &[Expr], x: &str, nonzero: &mut Vec<Expr>) -> Option<Expr> {
    let (polynomial, rest): (Vec<_>, Vec<_>) = factors.iter().cloned().partition(|f| crate::poly::coeffs(f, x).is_some());
    let [g] = rest.as_slice() else { return None };
    let mut p = simp(&mul(polynomial));
    let mut integrated = primitive(g, x, nonzero)?;
    let mut terms = vec![];
    for k in 0..=crate::poly::coeffs(&p, x)?.len() {
        if p.is_zero() { return Some(add(terms)); }
        let term = mul(vec![p.clone(), integrated.clone()]);
        terms.push(if k % 2 == 0 { term } else { neg(term) });
        p = simp(&p.diff(x));
        integrated = primitive(&simp(&integrated), x, nonzero)?;
    }
    None
}

/// Globally adaptive Gauss–Kronrod quadrature avoids sampling improper endpoints.
pub fn quadrature(f: &impl Fn(f64) -> Option<f64>, lo: f64, hi: f64) -> Result<f64, String> {
    integrate(f, lo, hi)
}

trait Quad: Copy {
    fn zero() -> Self;
    fn add(self, rhs: Self) -> Self;
    fn scale(self, k: f64) -> Self;
    fn norm(self) -> f64;
    fn finite(self) -> bool;
    fn accepted(self, discrepancy: Self) -> Self;
}
impl Quad for f64 {
    fn zero() -> Self { 0.0 }
    fn add(self, rhs: Self) -> Self { self + rhs }
    fn scale(self, k: f64) -> Self { self * k }
    fn norm(self) -> f64 { self.abs() }
    fn finite(self) -> bool { self.is_finite() }
    fn accepted(self, _: Self) -> Self { self }
}
impl Quad for crate::numeric::N {
    fn zero() -> Self { Self::real(0.0) }
    fn add(self, rhs: Self) -> Self { self.add(rhs) }
    fn scale(self, k: f64) -> Self { self.scale(k) }
    fn norm(self) -> f64 { self.abs() }
    fn finite(self) -> bool { self.is_finite() }
    fn accepted(mut self, discrepancy: Self) -> Self { self.error.re += discrepancy.re.abs(); self.error.im += discrepancy.im.abs(); self }
}

/// Largest estimated error first; values themselves need no ordering.
struct Panel<T> {
    a: f64,
    b: f64,
    value: T,
    error: f64,
    discrepancy: T,
}
impl<T> PartialEq for Panel<T> { fn eq(&self, other: &Self) -> bool { self.error.total_cmp(&other.error).is_eq() } }
impl<T> Eq for Panel<T> {}
impl<T> PartialOrd for Panel<T> { fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> { Some(self.cmp(other)) } }
impl<T> Ord for Panel<T> { fn cmp(&self, other: &Self) -> std::cmp::Ordering { self.error.total_cmp(&other.error) } }

fn integrate<T: Quad>(f: &impl Fn(f64) -> Option<T>, lo: f64, hi: f64) -> Result<T, String> {
    if lo.is_nan() || hi.is_nan() { return Err("invalid integral bounds".into()); }
    if lo == hi { return if lo.is_finite() { Ok(T::zero()) } else { Err("invalid integral bounds".into()) }; }
    if lo > hi { return integrate(f, hi, lo).map(|v| v.scale(-1.0)); }
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
        let y = f(x)?.scale(jac);
        y.finite().then_some(y)
    };
    // The embedded 7/15 rule adds Kronrod samples to the Gauss nodes. Scale
    // their difference by the sampled variation so unresolved oscillations do
    // not look converged merely because two inaccurate estimates nearly agree.
    fn panel<T: Quad>(f: &impl Fn(f64) -> Option<T>, a: f64, b: f64) -> Option<Panel<T>> {
        let (mid, half) = (a + (b - a) * 0.5, (b - a) * 0.5);
        let centre = f(mid)?;
        let mut kronrod = centre.scale(0.2094821410847278);
        let mut gauss = centre.scale(0.4179591836734694);
        let mut magnitude = centre.norm() * 0.2094821410847278;
        let nodes = [
            (0.9914553711208126, 0.02293532201052922, 0.0),
            (0.9491079123427585, 0.06309209262997855, 0.1294849661688697),
            (0.8648644233597691, 0.1047900103222502, 0.0),
            (0.7415311855993945, 0.1406532597155259, 0.2797053914892766),
            (0.5860872354676911, 0.1690047266392679, 0.0),
            (0.4058451513773972, 0.1903505780647854, 0.3818300505051189),
            (0.2077849550078985, 0.2044329400752989, 0.0),
        ];
        let mut samples = Vec::with_capacity(nodes.len());
        for (x, kw, gw) in nodes {
            let (left, right) = (mid - half * x, mid + half * x);
            if left <= a || right >= b { return None; }
            let (l, r) = (f(left)?, f(right)?);
            let pair = l.add(r);
            kronrod = kronrod.add(pair.scale(kw));
            if gw != 0.0 { gauss = gauss.add(pair.scale(gw)); }
            magnitude += (l.norm() + r.norm()) * kw;
            samples.push((l, r, kw));
        }
        let mean = kronrod.scale(0.5);
        let mut variation = centre.add(mean.scale(-1.0)).norm() * 0.2094821410847278;
        for (l, r, w) in samples {
            variation += (l.add(mean.scale(-1.0)).norm() + r.add(mean.scale(-1.0)).norm()) * w;
        }
        variation *= half;
        let difference = kronrod.add(gauss.scale(-1.0)).scale(half);
        let raw_error = difference.norm();
        let mut error = raw_error;
        if variation > 0.0 && raw_error > 0.0 {
            error = variation * (200.0 * raw_error / variation).powf(1.5).min(1.0);
        }
        error = error.max(50.0 * f64::EPSILON * magnitude * half);
        let value = kronrod.scale(half);
        (value.finite() && error.is_finite()).then_some(Panel {
            a, b, value, error,
            discrepancy: if raw_error > 0.0 { difference.scale(error / raw_error) } else { T::zero() },
        })
    }
    let failed = || "integral did not converge or its integrand is undefined".to_string();
    if mapped(0.5).is_none() { return Err("integrand is undefined inside the interval".into()); }
    let first = panel(&mapped, 0.0, 1.0).ok_or_else(failed)?;
    let (mut total, mut error) = (first.value, first.error);
    let mut panels = std::collections::BinaryHeap::from([first]);
    // Spend the global error budget where it matters most. Local tolerances
    // proportional to interval width cannot converge at logarithmic endpoints.
    while error > 1e-10 * total.norm().max(1.0) {
        if panels.len() >= 20_000 { return Err(failed()); }
        let worst = panels.pop().ok_or_else(failed)?;
        let mid = worst.a + (worst.b - worst.a) * 0.5;
        let left = panel(&mapped, worst.a, mid).ok_or_else(failed)?;
        let right = panel(&mapped, mid, worst.b).ok_or_else(failed)?;
        total = total.add(worst.value.scale(-1.0)).add(left.value).add(right.value);
        error = (error - worst.error).max(0.0) + left.error + right.error;
        if !total.finite() { return Err(failed()); }
        panels.push(left);
        panels.push(right);
    }
    // Rebuild from the accepted leaves: discarded estimates must not inflate
    // the componentwise forward-error estimates carried by complex numbers.
    let result = panels.into_iter().fold(T::zero(), |sum, p| sum.add(p.value.accepted(p.discrepancy)));
    result.finite().then_some(result).ok_or_else(failed)
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
    series_estimated(&|k| f(k).map(|v| (v, 0.0)), lo, hi, points, affine).map(|v| v.0)
}

pub(crate) fn series_estimated(f: &impl Fn(f64) -> Option<(f64, f64)>, lo: f64, hi: f64, points: &[f64], affine: bool) -> Result<(f64, f64), String> {
    let integer = |x: f64| x.is_finite() && x.fract() == 0.0 && x.abs() < 9_007_199_254_740_992.0;
    if !integer(lo) || !(integer(hi) || hi == f64::INFINITY) {
        return Err("sum bounds must be unitless integers (the upper bound may be infinity)".into());
    }
    if hi < lo { return Ok((0.0, 0.0)); }
    let mut uncertainty = 0.0;
    let term_at = |k| f(k).filter(|v| v.0.is_finite() && !v.1.is_nan()).ok_or("a sum term is undefined");
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
        let mut magnitude = 0.0;
        for interval in cuts.windows(2) {
            let (first, fe) = term_at(interval[0])?;
            let (last, le) = term_at(interval[1] - 1.0)?;
            uncertainty += (interval[1] - interval[0]) * (fe + le) * 0.5;
            magnitude += (interval[1] - interval[0]) * (first.abs() + last.abs()) * 0.5;
            total += (interval[1] - interval[0]) * (first * 0.5 + last * 0.5);
        }
        if hi == f64::INFINITY {
            let start = *cuts.last().unwrap();
            if term_at(start)?.0 != 0.0 || term_at(start + 1.0)?.0 != 0.0 {
                return Err("sum did not converge: its tail is a nonzero affine function".into());
            }
        }
        return total.is_finite().then_some((total, uncertainty + 32.0 * f64::EPSILON * magnitude)).ok_or_else(|| "sum overflow".into());
    }
    let last_corner = points.last().copied().unwrap_or(f64::NEG_INFINITY);
    let (mut total, mut correction, mut tail, mut stable, mut previous_tail) = (0.0_f64, 0.0, 0.0, 0, f64::INFINITY);
    let mut magnitude = 0.0;
    let mut dyadic = vec![];
    let mut first_magnitude = 0.0_f64;
    for offset in 0..100_000 {
        let k = lo + offset as f64;
        if k > hi { return Ok((total, uncertainty + 32.0 * f64::EPSILON * magnitude)); }
        let (term, error) = term_at(k)?;
        uncertainty += error;
        let adjusted = term - correction;
        let next = total + adjusted;
        correction = (next - total) - adjusted;
        total = next;
        if !total.is_finite() { return Err("sum overflow".into()); }
        magnitude += term.abs();
        if offset < 32 { first_magnitude = first_magnitude.max(term.abs()); }
        let count = offset+1;
        if hi == f64::INFINITY && count >= 32 && (count as u32).is_power_of_two() && k > last_corner {
            dyadic.push(total);
            if term.abs() < first_magnitude*0.5 {
                if let Some((estimate,error)) = extrapolate(&dyadic) {
                    return Ok((estimate, uncertainty + error + 32.0*f64::EPSILON*magnitude));
                }
            }
        }
        tail += term.abs();
        if offset % 32 == 31 {
            if hi == f64::INFINITY && k > last_corner && (tail == 0.0 || tail < previous_tail * 0.99) && tail <= 1e-12 * total.abs().max(1.0) {
                stable += 1;
                if stable == 4 {
                    let ratio = if previous_tail > 0.0 { tail / previous_tail } else { 0.0 };
                    return Ok((total, uncertainty + 32.0 * f64::EPSILON * magnitude + tail * ratio / (1.0 - ratio)));
                }
            } else { stable = 0; }
            previous_tail = tail;
            tail = 0.0;
        }
        if k == hi { return Ok((total, uncertainty + 32.0 * f64::EPSILON * magnitude)); }
    }
    Err("sum did not converge within 100000 terms".into())
}

/// Wynn's epsilon algorithm on geometrically spaced partial sums. Even
/// columns extrapolate a sequence; three adjacent windows and the preceding
/// even column must agree before accepting an accelerated tail estimate.
fn extrapolate(sequence:&[f64]) -> Option<(f64,f64)> {
    if sequence.len() < 7 { return None; }
    let mut previous=vec![0.0;sequence.len()+1];
    let mut current=sequence.to_vec();
    let mut prior_even=sequence.to_vec();
    for order in 1..sequence.len() {
        let next:Vec<f64>=current.windows(2).enumerate().map(|(i,pair)| {
            let difference=pair[1]-pair[0];
            if difference == 0.0 || !difference.is_finite() { f64::NAN } else { previous[i+1]+1.0/difference }
        }).collect();
        previous=current; current=next;
        if order % 2 == 0 {
            if order >= 4 && current.len() >= 3 && prior_even.len() >= 3 {
                // Later windows discard the early asymptotic transient. This
                // matters for shifted tails, without assuming a power law.
                for i in (0..current.len()-2).rev() {
                    let estimate=current[i];
                    let error=(estimate-current[i+1]).abs().max((estimate-current[i+2]).abs()).max((estimate-prior_even[i+2]).abs())*4.0;
                    if current[i..i+3].iter().all(|v| v.is_finite()) && prior_even[i+2].is_finite() && error <= 1e-11*estimate.abs().max(1.0) {
                        return Some((estimate,error));
                    }
                }
            }
            prior_even=current.clone();
        }
    }
    None
}

/// Refine both components together and accumulate their error across pieces.
pub(crate) fn complex_quadrature(f: &impl Fn(f64) -> Option<crate::numeric::N>, lo: f64, hi: f64, points: &[f64]) -> Result<crate::numeric::N, String> {
    use crate::numeric::N;
    if lo > hi { return complex_quadrature(f, hi, lo, points).map(|v| v.scale(-1.0)); }
    let mut bounds = vec![lo];
    bounds.extend(points.iter().copied().filter(|p| *p > lo && *p < hi));
    bounds.push(hi);
    bounds.windows(2).try_fold(N::real(0.0), |total, pair| integrate(f, pair[0], pair[1]).map(|v| total.add(v)))
}
