//! Rational delay inversion by formal power-series division and causal shifts.
//!
//! With q = exp(-d*s), write F = N(s,q)/D(s,q). Its coefficients satisfy
//! C_k = (N_k - sum_{j=1..k} D_j C_{k-j}) / D_0. At time t only
//! k <= floor(t/d) contribute. No truncation of an infinite time tail is needed.
use crate::{expr::{Expr, Func, add, div, func, int, mul, neg, pow, simp, sub, sym}, num::{Num, Rat}, poly::{coeffs, simplify, together}};

pub(crate) struct DelayForm {
    pub delay: Expr,
    pub numerator: Vec<Expr>,
    pub denominator: Vec<Expr>,
}

fn gcd(mut a: i128, mut b: i128) -> i128 {
    while b != 0 { (a, b) = (b, a % b); }
    a.abs()
}

fn expand_hyperbolic(e: &Expr, s: &str) -> Expr {
    let e = e.map(&mut |e| expand_hyperbolic(e, s));
    if let Expr::F(f @ (Func::Sinh | Func::Cosh | Func::Tanh), arg) = &e {
        if arg.has(s) {
            let positive = func(Func::Exp, (**arg).clone());
            let negative = func(Func::Exp, neg((**arg).clone()));
            return match f {
                Func::Sinh => div(sub(positive, negative), int(2)),
                Func::Cosh => div(add(vec![positive, negative]), int(2)),
                _ => div(sub(positive.clone(), negative.clone()), add(vec![positive, negative])),
            };
        }
    }
    e
}

fn exponential(e: &Expr, s: &str) -> Option<(Expr, Expr)> {
    let arg = match e {
        Expr::F(Func::Exp, arg) => arg,
        Expr::Pow(base, arg) if **base == Expr::E => arg,
        _ => return None,
    };
    let c = coeffs(arg, s)?;
    (c.len() == 2).then(|| (c[1].clone(), c[0].clone()))
}

/// Normalize commensurate rational delays, including products, powers, e^x,
/// hyperbolic functions and advances that cancel in the complete fraction.
pub(crate) fn normalize(e: &Expr, s: &str) -> Option<DelayForm> {
    fn collect(e: &Expr, s: &str, delays: &mut Vec<Rat>) -> Option<()> {
        if let Some((a, _)) = exponential(e, s) {
            if let Some(Num::R(a)) = a.as_num() {
                if !a.is_zero() { delays.push(if a.is_neg() { a.neg()? } else { a }); }
                return Some(());
            }
            return None;
        }
        for child in e.children() { collect(child, s, delays)?; }
        Some(())
    }
    fn replace(e: &Expr, s: &str, delay: Rat, q: &Expr) -> Option<Expr> {
        if let Some((a, b)) = exponential(e, s) {
            let Num::R(a) = a.as_num()? else { return None; };
            let multiple = a.neg()?.div(delay)?;
            if !multiple.is_int() { return None; }
            return Some(mul(vec![func(Func::Exp, b), pow(q.clone(), crate::expr::c(Num::R(multiple)))]));
        }
        let mut valid = true;
        let result = e.map(&mut |e| match replace(e, s, delay, q) {
            Some(e) => e,
            None => { valid = false; e.clone() },
        });
        valid.then_some(result)
    }
    let body = expand_hyperbolic(e, s);
    let mut delays = vec![];
    collect(&body, s, &mut delays)?;
    let mut delay = delays.first().copied().unwrap_or(Rat::ONE);
    for d in delays.iter().skip(1) {
        let denominator = delay.denom().checked_div(gcd(delay.denom(), d.denom()))?.checked_mul(d.denom())?;
        let a = delay.numer().checked_mul(denominator / delay.denom())?;
        let b = d.numer().checked_mul(denominator / d.denom())?;
        delay = Rat::new(gcd(a, b), denominator)?;
    }
    let mut q = "delay_internal".to_string();
    while body.has(&q) || q == s { q.push('_'); }
    let rational = simplify(&replace(&body, s, delay, &sym(&q))?);
    let (num, den) = together(&rational);
    let mut numerator = coeffs(&num, &q)?;
    let mut denominator = coeffs(&den, &q)?;
    // Algebraic cancellation can remove intermediate delays, e.g. tanh(s/4)
    // initially uses quarter steps but simplifies to a half-step sequence.
    let stride = numerator.iter().enumerate().chain(denominator.iter().enumerate())
        .filter(|(i, c)| *i > 0 && !c.is_zero()).fold(0, |g, (i, _)| gcd(g, i as i128));
    if stride > 1 {
        numerator = numerator.into_iter().step_by(stride as usize).collect();
        denominator = denominator.into_iter().step_by(stride as usize).collect();
        delay = delay.mul(Rat::int(stride))?;
    }
    if denominator[0].is_zero() { return None; }
    let degree = coeffs(&denominator[0], s)?.len() - 1;
    if degree == 0 { return None; } // a regular function, not Dirac distributions
    for coefficient in &denominator {
        if coeffs(coefficient, s)?.len() - 1 > degree { return None; }
    }
    for coefficient in &numerator {
        if !coefficient.is_zero() && coeffs(coefficient, s)?.len() > degree { return None; }
    }
    Some(DelayForm { delay: crate::expr::c(Num::R(delay)), numerator, denominator })
}

pub(crate) fn deferred(e: &Expr, s: &str, t: &Expr) -> Option<Expr> {
    normalize(e, s)?;
    Some(Expr::InverseLaplace(Box::new(e.clone()), s.into(), Box::new(t.clone())))
}

/// In the separable case D(s,q)=D_0(s) A(q), extract the coefficients
/// of 1/A(q) by exact partial fractions (the inverse Z-transform algorithm).
/// This produces an ordinary finite sum rather than retaining an inverse node.
pub(crate) fn symbolic_delays(e: &Expr, s: &str, t: &Expr) -> Option<Expr> {
    let form = normalize(e, s)?;
    if form.numerator.len() == 1 && form.denominator.len() == 1 { return None; }
    let mut q = "delay_internal".to_string();
    let mut index = "k".to_string();
    while e.has(&q) || t.has(&q) || q == s { q.push('_'); }
    while e.has(&index) || t.has(&index) || index == s { index.push('_'); }
    let k = sym(&index);
    let mut coefficients = vec![];
    for d in &form.denominator {
        let ratio = simplify(&div(d.clone(), form.denominator[0].clone()));
        if ratio.has(s) { return None; }
        coefficients.push(ratio);
    }
    let weights = if coefficients.len() > 1 {
        let denominator = add(coefficients.into_iter().enumerate().map(|(i, c)| mul(vec![c, pow(sym(&q), int(i as i128))])).collect());
        let poles = crate::transforms::partial_fractions(&int(1), &denominator, &q)?;
        Some(add(poles.into_iter().map(|(root, order, residue)| {
            // binomial(k+order-1, order-1), expressed as a fixed-degree
            // polynomial so large k never needs an overflowing factorial(k).
            let mut binomial = int(1);
            for j in 1..order {
                binomial = mul(vec![binomial, div(add(vec![k.clone(), int(j as i128)]), int(j as i128))]);
            }
            mul(vec![residue, pow(neg(root.clone()), int(-(order as i128))), pow(root, neg(k.clone())), binomial])
        }).collect()))
    } else { None };
    let mut terms = vec![];
    for (j, numerator) in form.numerator.iter().enumerate() {
        if numerator.is_zero() { continue; }
        let start = simp(&sub(t.clone(), mul(vec![int(j as i128), form.delay.clone()])));
        let shifted = if weights.is_some() { sub(start.clone(), mul(vec![k.clone(), form.delay.clone()])) } else { start.clone() };
        let kernel = crate::transforms::backward(&simplify(&div(numerator.clone(), form.denominator[0].clone())), s, &shifted)?;
        let value = if let Some(weights) = &weights {
            Expr::Sum(Box::new(mul(vec![weights.clone(), kernel])), index.clone(), Box::new(int(0)), Box::new(func(Func::Floor, div(start.clone(), form.delay.clone()))))
        } else { kernel };
        terms.push(Expr::Cases(vec![
            (value, Expr::Cmp(vec![start, int(0)], vec![crate::parse::Rel::Ge])),
            (int(0), int(1)),
        ]));
    }
    Some(add(terms))
}

fn trim(p: &mut Vec<f64>) {
    while p.len() > 1 && p.last() == Some(&0.0) { p.pop(); }
}
fn product(a: &[f64], b: &[f64]) -> Vec<f64> {
    let mut p = vec![0.0; a.len()+b.len()-1];
    for (i, a) in a.iter().enumerate() { for (j, b) in b.iter().enumerate() { p[i+j] += a*b; } }
    trim(&mut p);
    p
}
fn accumulate(a: &mut Vec<f64>, b: &[f64], scale: f64) {
    a.resize(a.len().max(b.len()), 0.0);
    for (a, b) in a.iter_mut().zip(b) { *a += scale*b; }
    trim(a);
}

/// Invert a proper real rational function as c^T exp(A*t) b in controllable
/// companion form. Scaling and squaring with a converged Taylor exponential
/// handles repeated and complex poles without numerical root finding.
fn rational_value(num: &[f64], den: &[f64], t: f64) -> Result<f64, String> {
    if num.iter().all(|v| *v == 0.0) { return Ok(0.0); }
    let n = den.len()-1;
    if n == 0 || num.len() > n { return Err("inverse Laplace coefficient requires Dirac distributions".into()); }
    if n > 128 { return Err("inverse Laplace state dimension exceeds 128".into()); }
    let leading = den[n];
    if leading == 0.0 { return Err("inverse Laplace denominator has zero leading coefficient".into()); }
    let mut a = vec![0.0; n*n];
    for i in 0..n-1 { a[i*n+i+1] = t; }
    for i in 0..n { a[(n-1)*n+i] = -t*den[i]/leading; }
    let norm = a.chunks(n).map(|row| row.iter().map(|x| x.abs()).sum::<f64>()).fold(0.0, f64::max);
    if !norm.is_finite() { return Err("inverse Laplace matrix overflow".into()); }
    let scaling = if norm > 0.5 { (norm/0.5).log2().ceil() as u32 } else { 0 };
    if scaling > 64 { return Err("inverse Laplace matrix scaling exceeds numerical precision".into()); }
    for v in &mut a { *v *= 2.0_f64.powi(-(scaling as i32)); }
    let multiply = |a: &[f64], b: &[f64]| {
        let mut c = vec![0.0; n*n];
        for i in 0..n { for k in 0..n {
            let v = a[i*n+k];
            if v != 0.0 { for j in 0..n { c[i*n+j] += v*b[k*n+j]; } }
        } }
        c
    };
    let mut result = vec![0.0; n*n];
    for i in 0..n { result[i*n+i] = 1.0; }
    let mut term = result.clone();
    let mut converged = false;
    for k in 1..=80 {
        term = multiply(&term, &a);
        for v in &mut term { *v /= k as f64; }
        for (r, v) in result.iter_mut().zip(&term) { *r += v; }
        let error = term.iter().map(|v| v.abs()).fold(0.0, f64::max);
        if error <= 1e-16 { converged = true; break; }
    }
    if !converged { return Err("inverse Laplace matrix exponential did not converge".into()); }
    for _ in 0..scaling { result = multiply(&result, &result); }
    let value = num.iter().enumerate().map(|(i, c)| c*result[i*n+n-1]/leading).sum::<f64>();
    value.is_finite().then_some(value).ok_or_else(|| "inverse Laplace result overflow".into())
}

pub(crate) fn evaluate(e: &Expr, s: &str, t: f64, vars: &dyn Fn(&str) -> Option<f64>) -> Result<f64, String> {
    if !t.is_finite() { return Err("inverse Laplace time must be finite".into()); }
    if t < 0.0 { return Ok(0.0); }
    let form = normalize(e, s).ok_or("unsupported rational delay transform")?;
    let number = |e: &Expr| crate::eval::eval_f(e, vars).ok_or_else(|| "inverse Laplace coefficient is undefined".to_string());
    let polynomial = |e: &Expr| -> Result<Vec<f64>, String> {
        let mut p = coeffs(e, s).ok_or("non-polynomial inverse Laplace coefficient")?.iter().map(number).collect::<Result<Vec<_>, _>>()?;
        trim(&mut p);
        Ok(p)
    };
    let numerator = form.numerator.iter().map(polynomial).collect::<Result<Vec<_>, _>>()?;
    let denominator = form.denominator.iter().map(polynomial).collect::<Result<Vec<_>, _>>()?;
    let delay = number(&form.delay)?;
    let delayed = numerator.len() > 1 || denominator.len() > 1;
    let count = if delayed { (t/delay).floor() as usize } else { 0 };
    if count > 256 { return Err("inverse Laplace evaluation exceeds 256 delay intervals".into()); }
    // When every D_j is proportional to D_0, recurrence coefficients are scalar.
    // This avoids artificial high-order repeated poles and cancellation.
    let d0 = &denominator[0];
    let ratios = form.denominator.iter().map(|d| simplify(&div(d.clone(), form.denominator[0].clone()))).collect::<Vec<_>>();
    let separable = ratios.iter().all(|ratio| !ratio.has(s));
    let mut total = 0.0;
    if separable {
        let ratios = ratios.iter().map(number).collect::<Result<Vec<_>, _>>()?;
        let mut weights = vec![0.0; count+1];
        weights[0] = 1.0;
        for k in 0..=count {
            if k > 0 { weights[k] = -(1..=k.min(ratios.len()-1)).map(|j| ratios[j]*weights[k-j]).sum::<f64>(); }
            for (j, n) in numerator.iter().enumerate().take(count-k+1) {
                total += weights[k]*rational_value(n, d0, t-(j+k) as f64*delay)?;
            }
        }
    } else {
        let mut powers = vec![vec![1.0]];
        let mut coefficients: Vec<Vec<f64>> = vec![];
        for k in 0..=count {
            powers.push(product(&powers[k], d0));
            let mut a = numerator.get(k).map_or(vec![0.0], |n| product(n, &powers[k]));
            for j in 1..=k.min(denominator.len()-1) {
                accumulate(&mut a, &product(&product(&denominator[j], &coefficients[k-j]), &powers[j-1]), -1.0);
            }
            if a.iter().any(|v| !v.is_finite()) { return Err("inverse Laplace recurrence overflow".into()); }
            total += rational_value(&a, &powers[k+1], t-k as f64*delay)?;
            coefficients.push(a);
        }
    }
    total.is_finite().then_some(total).ok_or_else(|| "inverse Laplace result overflow".into())
}
