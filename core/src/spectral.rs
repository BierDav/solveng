//! Compact, entire forms for pulse transforms and exponential moments.
//! These rules preserve the continuation at zero instead of introducing removable poles.
use crate::expr::{self, Expr, Func, Val, add, div, func, int, mul, neg, pow, rat, simp, sub, sym};
use crate::num::Num;
use crate::poly;

fn affine(e: &Expr, x: &str) -> Option<(Expr, Expr)> {
    let cs = poly::coeffs(e, x)?;
    match cs.as_slice() {
        [b] => Some((int(0), b.clone())),
        [b, a] => Some((a.clone(), b.clone())),
        _ => None,
    }
}

fn exponential(e: &Expr) -> Option<&Expr> {
    match e {
        Expr::Pow(b, a) if **b == Expr::E => Some(a),
        Expr::F(Func::Exp, a) => Some(a),
        _ => None,
    }
}

fn real_constant(e: &Expr) -> Option<Val> {
    let z = crate::eval::complex_value(e).ok()?;
    (z.is_real() && z.re.v.f().is_finite()).then_some(z.re)
}

fn infinity(e: &Expr, positive: bool) -> bool {
    matches!(e, Expr::C(v) if v.v.f() == if positive { f64::INFINITY } else { f64::NEG_INFINITY })
}

/// A whole-line pulse transform, or a finite-interval exponential moment.
pub(crate) fn integral(body: &Expr, x: &str, lo: &Expr, hi: &Expr) -> Option<Expr> {
    if body.size() > 1024 { return None; }
    let forward = infinity(lo, false) && infinity(hi, true);
    let reverse = infinity(lo, true) && infinity(hi, false);
    let finite = match (real_constant(lo), real_constant(hi)) {
        (Some(l), Some(h)) if l.d == h.d => Some((lo.clone(), hi.clone())),
        _ => None,
    };
    if !forward && !reverse && finite.is_none() { return None; }
    // Keep exact rules exact; numeric bounds and coefficients use the existing integrator.
    if body.contains(&|e| matches!(e, Expr::C(v) if !v.v.is_exact())) { return None; }
    let mut budget = 128;
    let value = linear(body, x, finite.as_ref(), &mut budget)?;
    Some(compact(&poly::simplify(&if reverse { neg(value) } else { value })))
}

fn linear(e: &Expr, x: &str, bounds: Option<&(Expr, Expr)>, budget: &mut usize) -> Option<Expr> {
    if *budget == 0 { return None; }
    *budget -= 1;
    if e.is_zero() { return Some(int(0)); }
    if let Expr::Add(ts) = e {
        return Some(add(ts.iter().map(|t| linear(t, x, bounds, budget)).collect::<Option<_>>()?));
    }
    let fs = match e { Expr::Mul(fs) => fs.clone(), e => vec![e.clone()] };
    // Distribute only sums needed to expose the transform's linearity.
    if let Some((i, ts)) = fs.iter().enumerate().find_map(|(i, f)| match f { Expr::Add(ts) if f.has(x) => Some((i, ts)), _ => None }) {
        return Some(add(ts.iter().map(|t| {
            let mut factors = fs.clone();
            factors[i] = t.clone();
            linear(&simp(&mul(factors)), x, bounds, budget)
        }).collect::<Option<_>>()?));
    }
    // The same table covers sine/cosine and hyperbolic kernels by Euler's formulas.
    for (i, f) in fs.iter().enumerate() {
        if let Expr::F(kind @ (Func::Sin | Func::Cos | Func::Sinh | Func::Cosh), a) = f {
            if !f.has(x) { continue; }
            affine(a, x)?;
            let circular = matches!(kind, Func::Sin | Func::Cos);
            let odd = matches!(kind, Func::Sin | Func::Sinh);
            let z = if circular { mul(vec![Expr::I, (**a).clone()]) } else { (**a).clone() };
            let plus = pow(Expr::E, z.clone());
            let minus = pow(Expr::E, neg(z));
            let numerator = if odd { sub(plus, minus) } else { add(vec![plus, minus]) };
            let denominator = if circular && odd { mul(vec![int(2), Expr::I]) } else { int(2) };
            let mut factors = fs.clone();
            factors[i] = simp(&div(numerator, denominator));
            return linear(&simp(&mul(factors)), x, bounds, budget);
        }
    }
    let mut k = int(0);
    let mut offset = int(0);
    let mut polynomial = vec![];
    let mut pulse = None;
    for f in fs {
        if let Some(a) = exponential(&f) {
            let (slope, intercept) = affine(a, x)?;
            k = simp(&add(vec![k, slope]));
            offset = simp(&add(vec![offset, intercept]));
        } else if let Expr::F(kind @ (Func::Rect | Func::Tri | Func::Dirac), a) = &f {
            if pulse.is_some() || bounds.is_some() { return None; }
            let (slope, intercept) = affine(a, x)?;
            let slope_value = real_constant(&slope)?;
            real_constant(&intercept)?;
            if slope_value.is_zero() { return None; }
            let width = simp(&div(int(1), func(Func::Abs, slope.clone())));
            let centre = poly::simplify(&neg(div(intercept, slope)));
            pulse = Some((width, centre, match kind { Func::Tri => 2, Func::Dirac => 0, _ => 1 }));
        } else {
            polynomial.push(f);
        }
    }
    let cs = poly::coeffs(&simp(&mul(polynomial)), x)?;
    if cs.len() > 7 { return None; }
    let (width, centre, order) = match (pulse, bounds) {
        (Some(p), None) => p,
        (None, Some((l, h))) if !k.is_zero() && !k.syms().is_empty() => (
            simp(&sub(h.clone(), l.clone())),
            simp(&mul(vec![rat(1, 2), add(vec![l.clone(), h.clone()])])),
            1,
        ),
        _ => return None,
    };
    // Multiplication by t^n is the nth derivative of the moment-generating function.
    let mut fresh = "spectral_moment".to_string();
    while e.has(&fresh) { fresh.push('_'); }
    let q = sym(&fresh);
    let argument = simp(&mul(vec![rat(1, 2), Expr::I, q.clone(), width.clone()]));
    let base = |cardinal| simp(&mul(vec![width.clone(), pow(Expr::E, mul(vec![q.clone(), centre.clone()])), pow(cardinal, int(order))]));
    let entire = base(func(Func::Sinc, argument.clone()));
    // Differentiate an analytic quotient away from zero, then supply its exact moments
    // at zero. Differentiating the guarded first derivative of sinc would leave an
    // unresolved derivative of cases on the second pass.
    let mut moment = base(div(func(Func::Sin, argument.clone()), argument));
    let mut values = vec![];
    for (n, coefficient) in cs.into_iter().enumerate() {
        if n > 0 { moment = simp(&moment.diff(&fresh)); }
        if moment.size() > 2048 { return None; }
        if !coefficient.is_zero() {
            let value = if n == 0 { simp(&entire.subst(&fresh, &k)) }
            else if order == 0 { simp(&moment.subst(&fresh, &k)) } else {
                let mut central = vec![];
                let mut choose = 1_i128;
                for j in 0..=n {
                    if j > 0 { choose = choose * (n-j+1) as i128 / j as i128; }
                    if j % 2 != 0 { continue; }
                    let weight = if order == 1 { rat(1, (1_i128 << j)*(j+1) as i128) }
                        else { rat(2, ((j+1)*(j+2)) as i128) };
                    central.push(mul(vec![int(choose), weight, pow(centre.clone(), int((n-j) as i128)), pow(width.clone(), int(j as i128))]));
                }
                let at_zero = simp(&mul(vec![width.clone(), add(central)]));
                let mut condition = Expr::Cmp(vec![k.clone(), int(0)], vec![crate::parse::Rel::Ne]);
                if let Some(((name, root), _)) = crate::identity::guard(&condition) {
                    condition = Expr::Cmp(vec![sym(&name), root], vec![crate::parse::Rel::Ne]);
                }
                simp(&Expr::Cases(vec![(simp(&moment.subst(&fresh, &k)), condition), (at_zero, int(1))]))
            };
            values.push(mul(vec![coefficient, value]));
        }
    }
    Some(mul(vec![pow(Expr::E, offset), add(values)]))
}

fn hyperbolic(a: Expr, odd: bool) -> Expr {
    let (mut coefficient, base) = expr::strip_coef(&a);
    let mut a = a;
    let mut sign = int(1);
    if coefficient.v.is_neg() {
        coefficient.v = coefficient.v.neg();
        a = simp(&mul(vec![Expr::C(coefficient), base.unwrap_or(int(1))]));
        if odd { sign = int(-1); }
    }
    let fs = match &a { Expr::Mul(fs) => fs.clone(), a => vec![a.clone()] };
    if let Some(i) = fs.iter().position(|f| *f == Expr::I) {
        let mut rest = fs;
        rest.remove(i);
        let f = func(if odd { Func::Sin } else { Func::Cos }, simp(&mul(rest)));
        simp(&mul(if odd { vec![sign, Expr::I, f] } else { vec![f] }))
    } else {
        simp(&mul(vec![sign, func(if odd { Func::Sinh } else { Func::Cosh }, a)]))
    }
}

fn exp_term(e: &Expr) -> Option<(Expr, Expr)> {
    let fs = match e { Expr::Mul(fs) => fs.clone(), e => vec![e.clone()] };
    let mut phase = vec![];
    let mut envelope = vec![];
    for f in fs {
        if let Some(a) = exponential(&f) { phase.push(a.clone()); }
        else { envelope.push(f); }
    }
    (!phase.is_empty()).then(|| (simp(&add(phase)), simp(&mul(envelope))))
}

/// Local, size-decreasing display rewrites. No proof assumptions or numerical tolerances.
pub(crate) fn compact(e: &Expr) -> Expr {
    if let Some(cardinal) = continued_cardinal(e) { return cardinal; }
    if let Expr::F(Func::Sinc, a) = e {
        // Exact constant calls should still evaluate, e.g. exp(1/2)*sinc(i/2) = e-1.
        if let Some(z) = crate::complex::parts(a) {
            if z.re.is_zero() && z.im.d.is_one() && !z.im.is_zero() && z.im.v.is_exact() {
                let q = Expr::C(z.im);
                return simp(&div(sub(pow(Expr::E, q.clone()), pow(Expr::E, neg(q.clone()))), mul(vec![int(2), q])));
            }
        }
        let sine = simp(&func(Func::Sin, (**a).clone()));
        if sine.is_zero() && crate::identity::nonzero(a) { return int(0); }
    }
    // Even cardinal/cosine functions have a canonical argument sign.
    if let Expr::F(f @ (Func::Sinc | Func::Cos | Func::Cosh), a) = e {
        if expr::strip_coef(a).0.v.is_neg() { return simp(&func(*f, simp(&neg((**a).clone())))); }
    }
    let Expr::Add(ts) = e else { return e.clone() };
    if ts.len() > 128 { return e.clone(); }
    let mut terms = ts.clone();
    let mut changed = false;
    for i in 0..terms.len() {
        let Some((a, envelope)) = exp_term(&terms[i]) else { continue };
        for j in i + 1..terms.len() {
            let Some((b, other)) = exp_term(&terms[j]) else { continue };
            if !simp(&add(vec![a.clone(), b])).is_zero() { continue; }
            let odd = if envelope == other { false }
                else if simp(&add(vec![envelope.clone(), other])).is_zero() { true }
                else { continue };
            let paired = simp(&mul(vec![int(2), envelope.clone(), hyperbolic(a.clone(), odd)]));
            if paired.size() >= terms[i].size() + terms[j].size() + 1 { continue; }
            terms[i] = paired;
            terms[j] = int(0);
            changed = true;
            break;
        }
    }
    if !changed { return e.clone(); }
    let paired = simp(&add(terms));
    // Factoring exposes a common pulse envelope shared by different phase pairs.
    let Some(f) = poly::factor(&paired) else { return paired };
    let mut fs = vec![expr::c(Num::R(f.k))];
    fs.extend(f.num.into_iter().map(|(b, n)| pow(b, int(n as i128))));
    fs.extend(f.den.into_iter().map(|(b, n)| pow(b, int(-(n as i128)))));
    let factored = simp(&mul(fs));
    if factored.size() < paired.size() { factored } else { paired }
}

/// A guarded quotient can become an entire cardinal function only when its exceptional
/// value agrees as well. An unguarded sin(x)/x must retain its excluded point.
fn continued_cardinal(e: &Expr) -> Option<Expr> {
    let Expr::Cases(branches) = e else { return None };
    let [(value, condition), (fallback, otherwise)] = branches.as_slice() else { return None };
    if !otherwise.is_one() || value.size() > 512 { return None; }
    let ((name, root), equal) = crate::identity::guard(condition)?;
    if equal { return None; }
    fn convert(e: &Expr) -> Expr {
        if let Expr::F(Func::Sin, a) = e {
            return simp(&mul(vec![(**a).clone(), func(Func::Sinc, (**a).clone())]));
        }
        if let Expr::Add(ts) = e {
            if ts.len() == 2 {
                for term in ts {
                    let (c, base) = expr::strip_coef(term);
                    if (c == Val::int(-1) || c == Val::int(1)) && ts.iter().any(|t| *t == Expr::C(c.neg())) {
                        if let Some(Expr::F(Func::Cos, a)) = base {
                            return simp(&mul(vec![Expr::C(c.neg()), rat(1, 2), pow((*a).clone(), int(2)), pow(func(Func::Sinc, mul(vec![rat(1, 2), *a])), int(2))]));
                        }
                    }
                }
            }
        }
        // Substitution into opaque or branch-sensitive functions stays opaque.
        if matches!(e, Expr::Add(_) | Expr::Mul(_) | Expr::Pow(..)) {
            return simp(&e.map(&mut convert));
        }
        e.clone()
    }
    let candidate = poly::cancel_expr(&convert(value))?;
    // Do not remove a guard while any pole or an unknown domain remains at that point.
    fn regular(e: &Expr, x: &str, at: &Expr) -> bool {
        match e {
            Expr::C(v) => v.v.f().is_finite(),
            Expr::Sym(_) | Expr::Pi | Expr::E | Expr::I => true,
            Expr::Add(v) | Expr::Mul(v) => v.iter().all(|a| regular(a, x, at)),
            Expr::Pow(b, n) if **b == Expr::E => regular(n, x, at),
            Expr::Pow(b, n) if n.as_num().is_some_and(|n| n.is_int()) => {
                regular(b, x, at) && (!n.as_num().unwrap().is_neg() || crate::identity::nonzero(&simp(&b.subst(x, at))))
            }
            Expr::F(Func::Sin | Func::Cos | Func::Sinh | Func::Cosh | Func::Sinc | Func::Exp, a) => regular(a, x, at),
            _ => false,
        }
    }
    if !regular(&candidate, &name, &root) { return None; }
    let at_root = simp(&candidate.subst(&name, &root));
    let difference = simp(&sub(at_root, fallback.clone()));
    if !poly::factor(&difference)?.is_zero() { return None; }
    (candidate.size() < e.size()).then_some(candidate)
}
