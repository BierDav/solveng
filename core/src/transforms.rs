//! Unilateral Laplace transforms. Parameters are formal constants; convergence is
//! understood on the right half-plane where the defining integral exists.
use crate::expr::{Expr, Func, add, div, func, int, mul, neg, pow, simp, sqrt, sub, sym};
use crate::poly::{coeffs, simplify, together};

pub fn transform(body: &Expr, input: &str, output: &str, inverse: bool) -> Result<Expr, String> {
    if input == output { return Err("transform variables must be different".into()); }
    if body.has(output) { return Err(format!("transform output variable '{output}' already occurs in the expression")); }
    let body = simplify(body);
    let result = if inverse { backward(&body, input, &sym(output)) } else { forward(&body, input, &sym(output), 0) };
    result.map(|e| simplify(&e)).ok_or_else(|| format!("{}: this expression is not supported by the symbolic transform engine", if inverse { "invlaplace" } else { "laplace" }))
}

fn factorial(n: i64) -> Option<Expr> {
    if !(0..=33).contains(&n) { return None; }
    Some(int((1..=n as i128).product()))
}

fn affine(e: &Expr, x: &str) -> Option<(Expr, Expr)> {
    let c = coeffs(e, x)?;
    if c.len() != 2 || c[1].is_zero() { return None; }
    Some((c[1].clone(), c[0].clone()))
}

fn forward(e: &Expr, x: &str, s: &Expr, depth: usize) -> Option<Expr> {
    if depth > 40 { return None; }
    if let Expr::InverseLaplace(body, variable, time) = e {
        if **time == sym(x) { return Some(body.subst(variable, s)); }
    }
    if e.is_zero() { return Some(int(0)); }
    if !e.has(x) { return Some(div(e.clone(), s.clone())); }
    if let Expr::Add(ts) = e {
        return Some(add(ts.iter().map(|t| forward(t, x, s, depth + 1)).collect::<Option<_>>()?));
    }
    if let Some(cs) = coeffs(e, x) {
        return Some(add(cs.into_iter().enumerate().map(|(n, c)| Some(div(mul(vec![c, factorial(n as i64)?]), pow(s.clone(), int(n as i128 + 1))))).collect::<Option<_>>()?));
    }
    if let Expr::Mul(fs) = e {
        let (constant, mut varying): (Vec<_>, Vec<_>) = fs.iter().cloned().partition(|f| !f.has(x));
        if !constant.is_empty() {
            return Some(mul(vec![mul(constant), forward(&simp(&mul(varying)), x, s, depth + 1)?]));
        }
        // exp(a t + b) f(t) shifts the transform by a.
        for i in 0..varying.len() {
            if let Some((a, b)) = exponential(&varying[i], x) {
                varying.remove(i);
                return Some(mul(vec![func(Func::Exp, b), forward(&simp(&mul(varying)), x, &simp(&sub(s.clone(), a)), depth + 1)?]));
            }
        }
        // t^n f(t) gives (-d/ds)^n F(s). Use a fresh symbol so
        // a previously shifted s remains independent while differentiating.
        for i in 0..varying.len() {
            let n = if varying[i] == sym(x) { Some(1) } else if let Expr::Pow(a, b) = &varying[i] { if **a == sym(x) { b.as_num().and_then(|n| n.as_int()) } else { None } } else { None };
            if let Some(n) = n.filter(|n| (1..=20).contains(n)) {
                varying.remove(i);
                let mut fresh = "laplace_internal".to_string();
                while e.has(&fresh) || s.has(&fresh) { fresh.push('_'); }
                let mut f = forward(&simp(&mul(varying)), x, &sym(&fresh), depth + 1)?;
                for _ in 0..n { f = simp(&neg(f.diff(&fresh))); }
                return Some(f.subst(&fresh, s));
            }
        }
    }
    if let Some((a, b)) = exponential(e, x) {
        return Some(div(func(Func::Exp, b), sub(s.clone(), a)));
    }
    if let Expr::F(f, argument) = e {
        let (a, b) = affine(argument, x)?;
        if matches!(f, Func::Rect | Func::Tri | Func::Sgn) {
            return signal(*f, a, b, s);
        }
        let s2 = pow(s.clone(), int(2));
        let a2 = pow(a.clone(), int(2));
        let (numerator, denominator) = match f {
            Func::Sin => (add(vec![mul(vec![s.clone(), func(Func::Sin, b.clone())]), mul(vec![a, func(Func::Cos, b)])]), add(vec![s2, a2])),
            Func::Cos => (sub(mul(vec![s.clone(), func(Func::Cos, b.clone())]), mul(vec![a, func(Func::Sin, b)])), add(vec![s2, a2])),
            Func::Sinh => (add(vec![mul(vec![s.clone(), func(Func::Sinh, b.clone())]), mul(vec![a, func(Func::Cosh, b)])]), sub(s2, a2)),
            Func::Cosh => (add(vec![mul(vec![s.clone(), func(Func::Cosh, b.clone())]), mul(vec![a, func(Func::Sinh, b)])]), sub(s2, a2)),
            _ => return None,
        };
        return Some(div(numerator, denominator));
    }
    None
}

/// Integrate each affine signal segment on t >= 0. Endpoint values have zero
/// measure, including rect's half-height endpoints and sgn's value at zero.
fn signal(f: Func, a: Expr, b: Expr, s: &Expr) -> Option<Expr> {
    if b.is_zero() {
        if crate::eval::complex_value(&a).is_ok_and(|value| !value.is_real()) { return None; }
        let scale = simp(&func(Func::Abs, a.clone()));
        let decay = |end: Expr| func(Func::Exp, neg(mul(vec![s.clone(), end])));
        let result = match f {
            Func::Rect => div(sub(int(1), decay(div(int(1), mul(vec![int(2), scale.clone()])))), s.clone()),
            Func::Tri => sub(div(int(1), s.clone()), div(mul(vec![scale.clone(), sub(int(1), decay(div(int(1), scale.clone())))]), pow(s.clone(), int(2)))),
            Func::Sgn => div(div(a.clone(), scale.clone()), s.clone()),
            _ => return None,
        };
        let result = if scale.syms().is_empty() { result } else {
            Expr::Cases(vec![(result, Expr::Cmp(vec![scale, int(0)], vec![crate::parse::Rel::Ne])),
                (if f == Func::Sgn { int(0) } else { div(int(1), s.clone()) }, int(1))])
        };
        return Some(if a.syms().is_empty() { result } else {
            Expr::Cases(vec![(result, Expr::Cmp(vec![func(Func::Im, a), int(0)], vec![crate::parse::Rel::Eq]))])
        });
    }
    // Shifted signals need ordered corners. Only use known real numeric
    // coefficients here; unknown parameter signs require piecewise assumptions.
    let av = a.as_num()?.f();
    let bv = b.as_num()?.f();
    if !av.is_finite() || !bv.is_finite() || av == 0.0 { return None; }
    let levels: &[Expr] = match f {
        Func::Rect => &[crate::expr::rat(-1, 2), crate::expr::rat(1, 2)],
        Func::Tri => &[int(-1), int(0), int(1)],
        Func::Sgn => &[int(0)],
        _ => return None,
    };
    let mut corners = vec![(0.0, int(0))];
    for level in levels {
        let corner = simp(&div(sub(level.clone(), b.clone()), a.clone()));
        let value = corner.as_num()?.f();
        if value > 0.0 && value.is_finite() { corners.push((value, corner)); }
    }
    corners.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut terms = vec![];
    for (i, (lo_value, lo)) in corners.iter().enumerate() {
        let hi = corners.get(i + 1);
        let sample = hi.map_or(lo_value + lo_value.abs().max(1.0), |(end, _)| lo_value + (end - lo_value) / 2.0);
        let argument = av * sample + bv;
        let (slope, intercept) = match f {
            Func::Rect if argument.abs() < 0.5 => (int(0), int(1)),
            Func::Tri if (-1.0..0.0).contains(&argument) => (a.clone(), add(vec![int(1), b.clone()])),
            Func::Tri if (0.0..1.0).contains(&argument) => (neg(a.clone()), sub(int(1), b.clone())),
            Func::Sgn => (int(0), int(if argument > 0.0 { 1 } else { -1 })),
            _ => continue,
        };
        // Integral from u to infinity of (slope*t + intercept)*exp(-s*t).
        let tail = |u: &Expr| mul(vec![
            func(Func::Exp, neg(mul(vec![s.clone(), u.clone()]))),
            add(vec![div(add(vec![mul(vec![slope.clone(), u.clone()]), intercept.clone()]), s.clone()), div(slope.clone(), pow(s.clone(), int(2)))]),
        ]);
        terms.push(match hi { Some((_, end)) => sub(tail(lo), tail(end)), None => tail(lo) });
    }
    Some(add(terms))
}

fn exponential(e: &Expr, x: &str) -> Option<(Expr, Expr)> {
    match e {
        Expr::F(Func::Exp, a) => affine(a, x),
        Expr::Pow(a, b) if **a == Expr::E => affine(b, x),
        _ => None,
    }
}

pub(crate) fn backward(e: &Expr, s: &str, t: &Expr) -> Option<Expr> {
    backward_symbolic(e, s, t)
        .or_else(|| crate::inverse_laplace::symbolic_delays(e, s, t))
        .or_else(|| crate::inverse_laplace::deferred(e, s, t))
}

pub(crate) fn backward_symbolic(e: &Expr, s: &str, t: &Expr) -> Option<Expr> {
    if e.is_zero() { return Some(int(0)); }
    if let Expr::Add(ts) = e {
        return Some(add(ts.iter().map(|e| backward(e, s, t)).collect::<Option<_>>()?));
    }
    // Repeated affine poles, including 1/s^n.
    if let Expr::Mul(fs) = e {
        let (constant, varying): (Vec<_>, Vec<_>) = fs.iter().cloned().partition(|f| !f.has(s));
        if !constant.is_empty() { return Some(mul(vec![mul(constant), backward(&simp(&mul(varying)), s, t)?])); }
    }
    if let Expr::Pow(a, b) = e {
        if let Some(n) = b.as_num().and_then(|n| n.as_int()).filter(|n| (-171..=-1).contains(n)) {
            if let Some((slope, offset)) = affine(a, s) {
                return Some(div(mul(vec![pow(t.clone(), int((-n - 1) as i128)), func(Func::Exp, mul(vec![neg(div(offset, slope.clone())), t.clone()]))]), mul(vec![pow(slope, int((-n) as i128)), func(Func::Factorial, int((-n - 1) as i128))])));
            }
        }
    }
    let (num, den) = together(e);
    let n = coeffs(&num, s)?;
    let d = coeffs(&den, s)?;
    if n.len() >= d.len() { return None; } // would require Dirac distributions
    if d.len() == 2 {
        return Some(mul(vec![div(n[0].clone(), d[1].clone()), func(Func::Exp, mul(vec![neg(div(d[0].clone(), d[1].clone())), t.clone()]))]));
    }
    if d.len() == 3 && n.len() <= 2 {
        let a = d[2].clone();
        let shift = simp(&div(d[1].clone(), mul(vec![int(2), a.clone()])));
        let w2 = simplify(&sub(div(d[0].clone(), a.clone()), pow(shift.clone(), int(2))));
        let p = simp(&div(n.get(1).cloned().unwrap_or(int(0)), a.clone()));
        let q = simp(&sub(div(n[0].clone(), a), mul(vec![p.clone(), shift.clone()])));
        let inner = if w2.is_zero() {
            add(vec![p.clone(), mul(vec![q.clone(), t.clone()])])
        } else {
            let hyperbolic = w2.as_num().is_some_and(|n| n.is_neg())
                || matches!(&w2, Expr::Mul(fs) if fs.first().and_then(Expr::as_num).is_some_and(|n| n.is_neg()));
            let square = simp(&if hyperbolic { neg(w2.clone()) } else { w2.clone() });
            // Both cos(w t) and sin(w t)/w are invariant under w -> -w,
            // so a symbolic square needs no absolute value or sign assumption.
            let w = match &square {
                Expr::Pow(base, exponent) if exponent.as_num().and_then(|n| n.as_int()) == Some(2) => (**base).clone(),
                _ => simp(&sqrt(square)),
            };
            let wt = mul(vec![w.clone(), t.clone()]);
            add(vec![mul(vec![p.clone(), func(if hyperbolic { Func::Cosh } else { Func::Cos }, wt.clone())]), mul(vec![div(q.clone(), w), func(if hyperbolic { Func::Sinh } else { Func::Sin }, wt)])])
        };
        let inner = if !w2.syms().is_empty() {
            Expr::Cases(vec![(inner, Expr::Cmp(vec![w2, int(0)], vec![crate::parse::Rel::Ne])),
                (add(vec![p.clone(), mul(vec![q.clone(), t.clone()])]), int(1))])
        } else { inner };
        return Some(mul(vec![func(Func::Exp, neg(mul(vec![shift, t.clone()]))), inner]));
    }
    Some(add(partial_fractions(&num, &den, s)?.into_iter().map(|(root, j, coefficient)| {
        Some(mul(vec![coefficient, div(pow(t.clone(), int(j as i128 - 1)), factorial(j as i64 - 1)?), func(Func::Exp, mul(vec![root, t.clone()]))]))
    }).collect::<Option<_>>()?))
}

/// Exact residues at all real poles, with validated multiplicities. Shared by
/// continuous rational inversion and generating-function coefficient extraction.
pub(crate) fn partial_fractions(num: &Expr, den: &Expr, s: &str) -> Option<Vec<(Expr, usize, Expr)>> {
    let n = coeffs(num, s)?;
    let d = coeffs(den, s)?;
    if n.len() >= d.len() || d.len() > 13 { return None; }
    let roots = crate::solve::solve1(&den, s, &|_| false).ok()?;
    // real poles only: a real function with complex poles is inverted in real form, by its state space
    if roots.iter().any(crate::complex::has_i) {
        return None;
    }
    let mut terms = vec![];
    let mut accounted = 0;
    for root in roots {
        let mut reduced = d.clone();
        let mut multiplicity = 0;
        while reduced.len() > 1 {
            let mut quotient = vec![int(0); reduced.len() - 1];
            let last = quotient.len() - 1;
            quotient[last] = reduced.last()?.clone();
            for i in (0..last).rev() {
                quotient[i] = simplify(&add(vec![reduced[i + 1].clone(), mul(vec![root.clone(), quotient[i + 1].clone()])]));
            }
            let remainder = simplify(&add(vec![reduced[0].clone(), mul(vec![root.clone(), quotient[0].clone()])]));
            if !remainder.is_zero() { break; }
            multiplicity += 1;
            reduced = quotient;
        }
        if multiplicity == 0 { return None; }
        accounted += multiplicity;
        let reduced = add(reduced.into_iter().enumerate().map(|(i, c)| mul(vec![c, pow(sym(s), int(i as i128))])).collect());
        let mut h = simplify(&div(num.clone(), reduced));
        for k in 0..multiplicity {
            let j = multiplicity - k;
            let coefficient = simplify(&div(h.subst(s, &root), factorial(k as i64)?));
            terms.push((root.clone(), j, coefficient));
            h = h.diff(s);
        }
    }
    if accounted + 1 != d.len() { return None; }
    Some(terms)
}
