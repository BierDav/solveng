//! Continuous convolution `(f ∗ g)(t) = ∫ f(τ) g(t − τ) dτ` over the whole real line.
//!
//! Exact closed forms for piecewise-polynomial operands with finitely many numeric breakpoints: polynomials in τ
//! (with any τ-free coefficients, symbolic ones too) multiplied and added with `rect`, `tri`, `sgn`, `abs` and
//! `cases`/comparisons, every one of them with an affine argument `a·τ + b` (numeric, possibly dimensioned a ≠ 0
//! and b). The operands are split into pieces (interval, polynomial); the product f(τ)·g(t − τ) has τ-breakpoints
//! at f's breakpoints a_i and at t − b_j, so the result is a piecewise polynomial in t whose breakpoints are among
//! the a_i + b_j. Between two consecutive candidates the order of all τ-breakpoints is fixed and every pair of
//! pieces is integrated exactly. Unbounded pieces are fine while the overlap of f's and g's supports stays bounded.
//!
//! Everything else falls back to the improper integral, which the engine evaluates numerically: non-polynomial
//! pieces (exp, sin, floor, …), non-affine arguments or conditions, breakpoints that aren't numeric constants
//! (symbolic shifts, π, √2), divergent overlaps, breakpoints of inconsistent dimensions.

use std::cmp::Ordering;

use crate::dim::Dim;
use crate::eval::dim_of;
use crate::expr::{Expr, Func, Val, add, int, mul, neg, pow, rat, simp, sub, sym};
use crate::num::{Num, Rat};
use crate::parse::Rel;
use crate::poly::coeffs;

/// Ascending coefficients, trailing zeros trimmed (the zero polynomial is empty).
type Poly = Vec<Expr>;

/// A polynomial on an open interval; None ends are infinite.
#[derive(Clone, Debug)]
struct Piece {
    lo: Option<Val>,
    hi: Option<Val>,
    p: Poly,
}

/// (f ∗ g)(at) = ∫_{−∞}^{∞} f(var) · g(at − var) d var
pub fn continuous(f: &Expr, g: &Expr, var: &str, at: &Expr) -> Expr {
    let t = fresh("t", var, &[f, g, at]);
    match exact(f, g, var, &t, at) {
        Some(closed) if matches!(at, Expr::Sym(_)) => closed.subst(&t, at),
        Some(closed) => simp(&closed.subst(&t, at)),
        None => fallback(f, g, var, at),
    }
}

/// The improper integral, with a capture-safe integration variable and bounds in var's dimension.
fn fallback(f: &Expr, g: &Expr, var: &str, at: &Expr) -> Expr {
    let v = if at.has(var) { fresh(var, var, &[f, g, at]) } else { var.to_string() };
    let body = mul(vec![f.subst(var, &sym(&v)), g.subst(var, &sub(at.clone(), sym(&v)))]);
    let d = var_dim(&[f, g], var, at).unwrap_or(Dim::ONE);
    let infinity = |sign: f64| Expr::C(Val::new(Num::F(sign * f64::INFINITY), d));
    crate::calculus::integral(simp(&body), v, infinity(-1.0), infinity(1.0))
}

fn fresh(base: &str, var: &str, avoid: &[&Expr]) -> String {
    (0..).map(|i| format!("{base}_conv{i}")).find(|n| n != var && !avoid.iter().any(|e| e.has(n))).unwrap()
}

// ---------- the exact path ----------

fn exact(f: &Expr, g: &Expr, var: &str, t: &str, at: &Expr) -> Option<Expr> {
    let mut bf = vec![];
    let mut bg = vec![];
    breakpoints(f, var, &mut bf)?;
    breakpoints(g, var, &mut bg)?;
    // var's dimension: the one of its (nonzero) breakpoints and of a numeric `at`; a unitless 0 fits any.
    let mut d: Option<Dim> = None;
    let at_dim = if at.syms().is_empty() { dim_of(at, &|_| None).ok().flatten() } else { None };
    for candidate in bf.iter().chain(&bg).filter(|b| !(b.is_zero() && b.d.is_one())).map(|b| b.d).chain(at_dim) {
        if d.is_some_and(|known| known != candidate) { return None; }
        d = Some(candidate);
    }
    let d = d.unwrap_or(Dim::ONE);
    let pf = pieces(f, var, bf, d)?;
    let pg = pieces(g, var, bg, d)?;
    if pf.iter().all(|p| p.p.is_empty()) || pg.iter().all(|p| p.p.is_empty()) {
        return Some(int(0));
    }
    let edges = |ps: &[Piece]| ps.iter().filter_map(|p| p.hi.clone()).collect::<Vec<_>>();
    let mut candidates = vec![];
    for a in edges(&pf) {
        for b in edges(&pg) { candidates.push(a.add(&b)?); }
    }
    let candidates = sorted(candidates);
    let tau = fresh("tau", var, &[f, g, at, &sym(t)]);
    let mut intervals: Vec<Piece> = vec![];
    for (lo, hi) in spans(&candidates) {
        let sample = inside(lo.as_ref(), hi.as_ref(), d)?;
        let p = integrate_pairs(&pf, &pg, t, &tau, &sample)?;
        match intervals.last_mut() {
            Some(last) if last.p == p => last.hi = hi,
            _ => intervals.push(Piece { lo, hi, p }),
        }
    }
    Some(assemble(intervals, t, d))
}

/// ∫ f(τ) g(t − τ) dτ as a polynomial in t, for the t-interval around `sample` (all τ-breakpoints keep their order there).
fn integrate_pairs(pf: &[Piece], pg: &[Piece], t: &str, tau: &str, sample: &Val) -> Option<Poly> {
    // A τ-bound: a constant c (Some(c), false) or t − c (Some(c), true); None is infinite.
    let at = |b: &(Option<Val>, bool)| -> Option<Option<Val>> {
        Some(match b {
            (None, _) => None,
            (Some(c), false) => Some(c.clone()),
            (Some(c), true) => Some(sample.add(&c.neg())?),
        })
    };
    let bound = |b: &(Option<Val>, bool)| match b {
        (Some(c), false) => Expr::C(c.clone()),
        (Some(c), true) => sub(sym(t), Expr::C(c.clone())),
        (None, _) => unreachable!(),
    };
    let mut terms = vec![];
    for p in pf.iter().filter(|p| !p.p.is_empty()) {
        for q in pg.iter().filter(|q| !q.p.is_empty()) {
            // τ ∈ (p.lo, p.hi) and t − τ ∈ (q.lo, q.hi)
            let lows = [(p.lo.clone(), false), (q.hi.clone(), true)];
            let highs = [(p.hi.clone(), false), (q.lo.clone(), true)];
            let (l0, l1) = (at(&lows[0])?, at(&lows[1])?);
            let (h0, h1) = (at(&highs[0])?, at(&highs[1])?);
            let lower = match (&l0, &l1) {
                (Some(a), Some(b)) => if a.v.cmp(b.v) == Ordering::Less { 1 } else { 0 },
                (None, _) => 1,
                (_, None) => 0,
            };
            let upper = match (&h0, &h1) {
                (Some(a), Some(b)) => if a.v.cmp(b.v) == Ordering::Greater { 1 } else { 0 },
                (None, _) => 1,
                (_, None) => 0,
            };
            let (lv, hv) = ([l0, l1][lower].clone(), [h0, h1][upper].clone());
            match (&lv, &hv) {
                (Some(l), Some(h)) => if l.v.cmp(h.v) != Ordering::Less { continue; },
                // a nonzero polynomial over an unbounded overlap: the integral diverges
                _ => return None,
            }
            let (lb, hb) = (bound(&lows[lower]), bound(&highs[upper]));
            let product = mul(vec![poly_at(&p.p, &sym(tau)), poly_at(&q.p, &sub(sym(t), sym(tau)))]);
            let cs = coeffs(&product, tau)?;
            let primitive = |x: &Expr| add(cs.iter().enumerate().map(|(k, c)| {
                let n = k as i128 + 1;
                mul(vec![c.clone(), pow(x.clone(), int(n)), rat(1, n)])
            }).collect());
            terms.push(sub(primitive(&hb), primitive(&lb)));
        }
    }
    Some(trim(coeffs(&add(terms), t)?))
}

/// Cases over the t-intervals: nonzero pieces `lo < t ≤ hi`, zeros gathered in a final default branch.
/// The convolution of bounded locally integrable pieces over bounded overlaps is continuous, so the
/// choice of closed ends doesn't change any value.
fn assemble(intervals: Vec<Piece>, t: &str, d: Dim) -> Expr {
    if intervals.len() == 1 {
        return poly_at(&intervals[0].p, &sym(t));
    }
    let zero = intervals.iter().find(|i| !i.p.is_empty()).and_then(|i| {
        let sample = inside(i.lo.as_ref(), i.hi.as_ref(), d)?;
        dim_of(&simp(&poly_at(&i.p, &Expr::C(sample))), &|_| None).ok().flatten()
    }).map(|rd| Expr::C(Val::new(Num::ZERO, rd))).unwrap_or(int(0));
    let any_zero = intervals.iter().any(|i| i.p.is_empty());
    let n = intervals.len();
    let mut branches = vec![];
    for (k, i) in intervals.into_iter().enumerate() {
        if i.p.is_empty() { continue; }
        let condition = if k == n - 1 && !any_zero {
            int(1)
        } else {
            match (i.lo, i.hi) {
                (Some(lo), Some(hi)) => Expr::Cmp(vec![Expr::C(lo), sym(t), Expr::C(hi)], vec![Rel::Lt, Rel::Le]),
                (None, Some(hi)) => Expr::Cmp(vec![sym(t), Expr::C(hi)], vec![Rel::Le]),
                (Some(lo), None) => Expr::Cmp(vec![Expr::C(lo), sym(t)], vec![Rel::Lt]),
                (None, None) => int(1),
            }
        };
        branches.push((poly_at(&i.p, &sym(t)), condition));
    }
    if any_zero { branches.push((zero, int(1))); }
    Expr::Cases(branches)
}

fn poly_at(p: &Poly, x: &Expr) -> Expr {
    simp(&add(p.iter().enumerate().map(|(k, c)| mul(vec![c.clone(), pow(x.clone(), int(k as i128))])).collect()))
}

fn trim(mut p: Poly) -> Poly {
    while p.last().is_some_and(Expr::is_zero) { p.pop(); }
    p
}

fn sorted(mut v: Vec<Val>) -> Vec<Val> {
    v.sort_by(|a, b| a.v.cmp(b.v));
    v.dedup_by(|a, b| a.v.same(b.v));
    v
}

/// The open intervals between sorted points, infinite at both ends.
fn spans(points: &[Val]) -> Vec<(Option<Val>, Option<Val>)> {
    let mut ends: Vec<Option<Val>> = vec![None];
    ends.extend(points.iter().cloned().map(Some));
    ends.push(None);
    ends.windows(2).map(|w| (w[0].clone(), w[1].clone())).collect()
}

/// A point strictly inside an interval, in dimension d.
fn inside(lo: Option<&Val>, hi: Option<&Val>, d: Dim) -> Option<Val> {
    let unit = Val::new(Num::ONE, d);
    match (lo, hi) {
        (Some(l), Some(h)) => Some(l.add(h)?.mul(&Val::num(Num::R(Rat::new(1, 2)?)))),
        (Some(l), None) => l.add(&unit),
        (None, Some(h)) => h.add(&unit.neg()),
        (None, None) => Some(Val::new(Num::ZERO, d)),
    }
}

/// The operand as polynomials between its breakpoints, adjacent equal pieces merged.
fn pieces(e: &Expr, var: &str, points: Vec<Val>, d: Dim) -> Option<Vec<Piece>> {
    let points: Vec<Val> = points.into_iter().map(|b| if b.is_zero() && b.d.is_one() { Val::new(Num::ZERO, d) } else { b }).collect();
    let mut out: Vec<Piece> = vec![];
    for (lo, hi) in spans(&sorted(points)) {
        let sample = inside(lo.as_ref(), hi.as_ref(), d)?;
        let p = trim(coeffs(&simp(&resolve(e, var, &sample)?), var)?);
        match out.last_mut() {
            Some(last) if last.p == p => last.hi = hi,
            _ => out.push(Piece { lo, hi, p }),
        }
    }
    Some(out)
}

/// (slope, intercept) of an affine expression with constant coefficients.
fn affine(e: &Expr, var: &str) -> Option<(Val, Val)> {
    let cs = trim(coeffs(e, var)?);
    let val = |e: Option<&Expr>| match e {
        None => Some(Val::int(0)),
        Some(Expr::C(v)) => Some(v.clone()),
        Some(_) => None,
    };
    if cs.len() > 2 { return None; }
    Some((val(cs.get(1))?, val(cs.first())?))
}

/// Where a·var + b = level.
fn root(slope: &Val, intercept: &Val, level: &Val) -> Option<Val> {
    if slope.is_zero() { return None; }
    let difference = if level.is_zero() { intercept.neg() } else { level.add(&intercept.neg())? };
    Some(difference.mul(&slope.pow(Num::int(-1))?))
}

/// Collect var's breakpoints; None where the operand isn't piecewise polynomial with numeric affine corners.
fn breakpoints(e: &Expr, var: &str, out: &mut Vec<Val>) -> Option<()> {
    if !e.has(var) { return Some(()); }
    match e {
        Expr::Sym(_) => {}
        Expr::Add(v) | Expr::Mul(v) => for x in v { breakpoints(x, var, out)?; },
        Expr::Pow(b, x) => {
            if !x.as_num().and_then(Num::as_int).is_some_and(|k| k >= 0) { return None; }
            breakpoints(b, var, out)?;
        }
        Expr::F(f @ (Func::Rect | Func::Tri | Func::Sgn | Func::Abs), a) => {
            let (slope, intercept) = affine(a, var)?;
            if slope.is_zero() { return Some(()); }
            let levels: &[(i128, i128)] = match f {
                Func::Rect => &[(-1, 2), (1, 2)],
                Func::Tri => &[(-1, 1), (0, 1), (1, 1)],
                _ => &[(0, 1)],
            };
            for &(n, m) in levels { out.push(root(&slope, &intercept, &Val::num(Num::R(Rat::new(n, m)?)))?); }
        }
        Expr::Cmp(v, _) => {
            for pair in v.windows(2) {
                let (slope, intercept) = affine(&sub(pair[0].clone(), pair[1].clone()), var)?;
                if !slope.is_zero() { out.push(root(&slope, &intercept, &Val::int(0))?); }
            }
        }
        Expr::Cases(branches) => for (value, condition) in branches {
            breakpoints(value, var, out)?;
            breakpoints(condition, var, out)?;
        },
        _ => return None,
    }
    Some(())
}

/// The operand's polynomial on the piece around `sample` (a point that is no breakpoint).
fn resolve(e: &Expr, var: &str, sample: &Val) -> Option<Expr> {
    if !e.has(var) { return Some(e.clone()); }
    let value = |a: &Expr| match simp(&a.subst(var, &Expr::C(sample.clone()))) {
        Expr::C(v) => Some(v),
        _ => None,
    };
    Some(match e {
        Expr::Sym(_) => e.clone(),
        Expr::Add(v) => Expr::Add(v.iter().map(|x| resolve(x, var, sample)).collect::<Option<_>>()?),
        Expr::Mul(v) => Expr::Mul(v.iter().map(|x| resolve(x, var, sample)).collect::<Option<_>>()?),
        Expr::Pow(b, x) if !x.has(var) => pow(resolve(b, var, sample)?, (**x).clone()),
        Expr::F(f, a) => {
            let x = value(a)?.v;
            let a = (**a).clone();
            match f {
                Func::Rect => int((x.abs().cmp(Num::R(Rat::new(1, 2)?)) == Ordering::Less) as i128),
                Func::Tri if x.abs().cmp(Num::ONE) != Ordering::Less => int(0),
                Func::Tri if x.is_neg() => add(vec![int(1), a]),
                Func::Tri => sub(int(1), a),
                Func::Sgn => int(if x.is_neg() { -1 } else { 1 }),
                Func::Abs if x.is_neg() => neg(a),
                Func::Abs => a,
                _ => return None,
            }
        }
        Expr::Cmp(..) => Expr::C(value(e)?),
        Expr::Cases(branches) => {
            for (v, condition) in branches {
                let n = simp(&resolve(condition, var, sample)?).as_num()?;
                if !n.is_zero() { return resolve(v, var, sample); }
            }
            return None;
        }
        _ => return None,
    })
}

/// var's dimension as the operands' affine (or polynomial) arguments and comparisons reveal it, or a numeric `at`'s.
fn var_dim(es: &[&Expr], var: &str, at: &Expr) -> Option<Dim> {
    fn scan(e: &Expr, var: &str) -> Option<Dim> {
        if !e.has(var) { return None; }
        let from_argument = |a: &Expr| -> Option<Dim> {
            let cs = trim(coeffs(a, var)?);
            let k = cs.len().checked_sub(1).filter(|&k| k > 0)?;
            let Expr::C(c) = &cs[k] else { return None };
            c.d.pow(Rat::new(-1, k as i128)?)
        };
        let found = match e {
            Expr::F(Func::Abs, _) => None,
            Expr::F(_, a) => from_argument(a),
            Expr::Cmp(v, _) => v.windows(2).find_map(|p| {
                let (slope, intercept) = affine(&sub(p[0].clone(), p[1].clone()), var)?;
                root(&slope, &intercept, &Val::int(0)).filter(|r| !r.d.is_one()).map(|r| r.d)
            }),
            _ => None,
        };
        found.or_else(|| e.children().into_iter().find_map(|c| scan(c, var)))
    }
    es.iter().find_map(|e| scan(e, var)).or_else(|| if at.syms().is_empty() { dim_of(at, &|_| None).ok().flatten() } else { None })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::func;

    fn tau() -> Expr { sym("τ") }
    fn at(e: &Expr, t: Expr) -> Expr { simp(&e.subst("t", &t)) }

    #[test]
    fn rect_with_rect_is_tri() {
        let r = func(Func::Rect, tau());
        let e = continuous(&r, &r, "τ", &sym("t"));
        assert!(matches!(e, Expr::Cases(_)), "{e:?}");
        for k in -12..=12 {
            let x = rat(k, 8);
            let expected = simp(&func(Func::Tri, x.clone()));
            assert_eq!(at(&e, x), expected, "t = {k}/8");
        }
    }

    #[test]
    fn tri_with_tri_is_cubic_spline() {
        let t = func(Func::Tri, tau());
        let e = continuous(&t, &t, "τ", &sym("t"));
        assert_eq!(at(&e, rat(1, 2)), rat(23, 48));
        assert_eq!(at(&e, int(0)), rat(2, 3));
        assert_eq!(at(&e, rat(-3, 2)), rat(1, 48));
        assert_eq!(at(&e, int(2)), int(0));
        assert_eq!(continuous(&t, &t, "τ", &rat(1, 2)), rat(23, 48));
    }

    #[test]
    fn step_with_pulse_is_ramp_then_constant() {
        let step = Expr::Cases(vec![(int(1), Expr::Cmp(vec![tau(), int(0)], vec![Rel::Ge])), (int(0), int(1))]);
        let pulse = func(Func::Rect, sub(tau(), rat(1, 2)));
        let e = continuous(&step, &pulse, "τ", &sym("t"));
        for (x, y) in [(int(-1), int(0)), (rat(1, 4), rat(1, 4)), (int(1), int(1)), (int(7), int(1))] {
            assert_eq!(at(&e, x), y);
        }
        // a step with a step is a ramp
        let ramp = continuous(&step, &step, "τ", &sym("t"));
        assert_eq!(at(&ramp, rat(5, 2)), rat(5, 2));
        assert_eq!(at(&ramp, int(-3)), int(0));
        // with a step backwards the overlap is unbounded: the integral stays
        let back = Expr::Cases(vec![(int(1), Expr::Cmp(vec![tau(), int(0)], vec![Rel::Lt])), (int(0), int(1))]);
        assert!(matches!(continuous(&step, &back, "τ", &sym("t")), Expr::Integral(..)));
        // a step forwards with one backwards overlaps on a bounded interval
        let e = continuous(&back, &back, "τ", &sym("t"));
        assert_eq!(at(&e, int(-2)), int(2));
        assert_eq!(at(&e, int(1)), int(0));
    }

    #[test]
    fn capture_and_symbolic_shift() {
        let r = func(Func::Rect, tau());
        // at = var itself
        let e = continuous(&r, &r, "τ", &tau());
        assert_eq!(simp(&e.subst("τ", &rat(1, 2))), rat(1, 2));
        // a symbolic breakpoint falls back
        let shifted = func(Func::Tri, sub(tau(), sym("c")));
        let e = continuous(&shifted, &r, "τ", &tau());
        let Expr::Integral(_, v, _, _) = &e else { panic!("{e:?}") };
        assert_ne!(v, "τ");
    }
}
