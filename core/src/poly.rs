//! Polynomials with exact coefficients in any number of variables: rational functions brought to lowest terms
//! (`(x² − 1)/(x − 1)` → `x + 1`), expressions over one denominator for solving, and factors
//! (`a² p² − a p²` → `a p² (a − 1)`).

use std::collections::BTreeMap;

use crate::dim::Dim;
use crate::expr::{self, Expr, Val, base_exp, int, simp};
use crate::num::{Num, Rat};

type Mono = Vec<u32>;

#[derive(Clone, Debug, PartialEq)]
pub struct Poly {
    n: usize,
    t: BTreeMap<Mono, Rat>,
}

impl Poly {
    fn zero(n: usize) -> Poly {
        Poly { n, t: BTreeMap::new() }
    }
    fn constant(n: usize, c: Rat) -> Poly {
        let mut p = Poly::zero(n);
        if !c.is_zero() {
            p.t.insert(vec![0; n], c);
        }
        p
    }
    fn var(n: usize, i: usize) -> Poly {
        let mut m = vec![0; n];
        m[i] = 1;
        let mut p = Poly::zero(n);
        p.t.insert(m, Rat::ONE);
        p
    }
    fn is_zero(&self) -> bool {
        self.t.is_empty()
    }
    fn is_const(&self) -> bool {
        self.t.keys().all(|m| m.iter().all(|&e| e == 0))
    }
    fn add(&self, o: &Poly) -> Option<Poly> {
        let mut t = self.t.clone();
        for (m, c) in &o.t {
            let v = match t.get(m) {
                Some(a) => a.add(*c)?,
                None => *c,
            };
            if v.is_zero() {
                t.remove(m);
            } else {
                t.insert(m.clone(), v);
            }
        }
        Some(Poly { n: self.n, t })
    }
    fn neg(&self) -> Option<Poly> {
        let mut t = BTreeMap::new();
        for (m, c) in &self.t {
            t.insert(m.clone(), c.neg()?);
        }
        Some(Poly { n: self.n, t })
    }
    fn sub(&self, o: &Poly) -> Option<Poly> {
        self.add(&o.neg()?)
    }
    fn mul(&self, o: &Poly) -> Option<Poly> {
        let mut r = Poly::zero(self.n);
        if self.t.len() * o.t.len() > 20000 {
            return None;
        }
        for (m1, c1) in &self.t {
            for (m2, c2) in &o.t {
                let m: Mono = m1.iter().zip(m2).map(|(a, b)| a + b).collect();
                if m.iter().any(|&e| e > 200) {
                    return None;
                }
                let c = c1.mul(*c2)?;
                let v = match r.t.get(&m) {
                    Some(a) => a.add(c)?,
                    None => c,
                };
                if v.is_zero() {
                    r.t.remove(&m);
                } else {
                    r.t.insert(m, v);
                }
            }
        }
        Some(r)
    }
    fn scale(&self, k: Rat) -> Option<Poly> {
        let mut t = BTreeMap::new();
        if k.is_zero() {
            return Some(Poly::zero(self.n));
        }
        for (m, c) in &self.t {
            t.insert(m.clone(), c.mul(k)?);
        }
        Some(Poly { n: self.n, t })
    }
    fn powi(&self, k: u32) -> Option<Poly> {
        let mut r = Poly::constant(self.n, Rat::ONE);
        for _ in 0..k {
            r = r.mul(self)?;
        }
        Some(r)
    }
    fn deg(&self, v: usize) -> u32 {
        self.t.keys().map(|m| m[v]).max().unwrap_or(0)
    }
    /// Coefficients in v: index k is the polynomial (free of v) at v^k.
    fn coeffs(&self, v: usize) -> Vec<Poly> {
        let mut out = vec![Poly::zero(self.n); self.deg(v) as usize + 1];
        for (m, c) in &self.t {
            let mut m2 = m.clone();
            m2[v] = 0;
            out[m[v] as usize].t.insert(m2, *c);
        }
        out
    }
    fn lc(&self, v: usize) -> Poly {
        self.coeffs(v).pop().unwrap_or(Poly::zero(self.n))
    }
    fn shift(&self, v: usize, k: u32) -> Poly {
        let mut t = BTreeMap::new();
        for (m, c) in &self.t {
            let mut m2 = m.clone();
            m2[v] += k;
            t.insert(m2, *c);
        }
        Poly { n: self.n, t }
    }
    fn diff(&self, v: usize) -> Option<Poly> {
        let mut t = BTreeMap::new();
        for (m, c) in &self.t {
            if m[v] > 0 {
                let mut m2 = m.clone();
                m2[v] -= 1;
                t.insert(m2, c.mul(Rat::int(m[v] as i128))?);
            }
        }
        Some(Poly { n: self.n, t })
    }
    fn vars(&self) -> Vec<usize> {
        (0..self.n).filter(|&i| self.t.keys().any(|m| m[i] > 0)).collect()
    }
    /// Leading term in lex order.
    fn lead(&self) -> Option<(&Mono, &Rat)> {
        self.t.iter().next_back()
    }
    /// a / b when it divides exactly.
    fn div_exact(&self, b: &Poly) -> Option<Poly> {
        let mut r = self.clone();
        let mut q = Poly::zero(self.n);
        let (bm, bc) = b.lead()?;
        let mut steps = 0;
        while let Some((rm, rc)) = r.lead() {
            steps += 1;
            if steps > 5000 || !rm.iter().zip(bm).all(|(a, b)| a >= b) {
                return None;
            }
            let m: Mono = rm.iter().zip(bm).map(|(a, b)| a - b).collect();
            let c = rc.div(*bc)?;
            let mut t = Poly::zero(self.n);
            t.t.insert(m, c);
            q = q.add(&t)?;
            r = r.sub(&t.mul(b)?)?;
        }
        Some(q)
    }
    /// Pseudo-remainder of a by b in v.
    fn prem(&self, b: &Poly, v: usize) -> Option<Poly> {
        let n = b.deg(v);
        let lb = b.lc(v);
        let mut r = self.clone();
        let mut steps = 0;
        while !r.is_zero() && r.deg(v) >= n {
            steps += 1;
            if steps > 200 {
                return None;
            }
            let lr = r.lc(v);
            r = r.mul(&lb)?.sub(&lr.mul(&b.shift(v, r.deg(v) - n))?)?;
        }
        Some(r)
    }
    fn content(&self, v: usize) -> Option<Poly> {
        let mut g: Option<Poly> = None;
        for c in self.coeffs(v) {
            if c.is_zero() {
                continue;
            }
            g = Some(match g {
                None => c,
                Some(g) => gcd(&g, &c)?,
            });
        }
        Some(g.unwrap_or(Poly::zero(self.n)))
    }
    /// The same polynomial scaled to whole coefficients without a common factor, leading one positive.
    fn normalized(&self) -> Option<(Poly, Rat)> {
        if self.is_zero() {
            return Some((self.clone(), Rat::ONE));
        }
        let mut l: i128 = 1;
        for c in self.t.values() {
            let d = c.denom();
            l = l.checked_mul(d / gcd_i(l, d))?;
        }
        let mut g: i128 = 0;
        for c in self.t.values() {
            g = gcd_i(g, c.mul(Rat::int(l))?.numer());
        }
        let mut k = Rat::new(l, g.max(1))?;
        if self.lead()?.1.is_neg() {
            k = k.neg()?;
        }
        Some((self.scale(k)?, k))
    }
}

fn gcd_i(mut a: i128, mut b: i128) -> i128 {
    a = a.abs();
    b = b.abs();
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a.max(1)
}

/// The greatest common divisor (up to a constant factor).
pub fn gcd(a: &Poly, b: &Poly) -> Option<Poly> {
    if a.is_zero() {
        return Some(b.clone());
    }
    if b.is_zero() {
        return Some(a.clone());
    }
    let mut vs = a.vars();
    for v in b.vars() {
        if !vs.contains(&v) {
            vs.push(v);
        }
    }
    vs.sort();
    let Some(&v) = vs.first() else { return Some(Poly::constant(a.n, Rat::ONE)) };
    let (ca, cb) = (a.content(v)?, b.content(v)?);
    let c = gcd(&ca, &cb)?;
    let mut p = a.div_exact(&ca)?;
    let mut q = b.div_exact(&cb)?;
    if p.deg(v) < q.deg(v) {
        std::mem::swap(&mut p, &mut q);
    }
    let mut steps = 0;
    while !q.is_zero() && q.deg(v) > 0 {
        steps += 1;
        if steps > 100 {
            return None;
        }
        let r = p.prem(&q, v)?;
        p = q;
        q = if r.is_zero() { r } else { let cr = r.content(v)?; r.div_exact(&cr)? };
    }
    // a remainder free of v: the primitive parts share nothing in v
    let g = if q.is_zero() { let cp = p.content(v)?; p.div_exact(&cp)? } else { Poly::constant(a.n, Rat::ONE) };
    let g = g.mul(&c)?;
    Some(g.normalized()?.0)
}

// ---------- factors ----------

/// A polynomial as k · Π fᵉ: each f with whole coefficients without a common factor, its leading one positive, and
/// as far as split: the factors free of a variable (its content), repeated factors, and factors of degree one in a
/// variable. A factor left over may still split (x⁴ + 4 is (x² − 2x + 2)(x² + 2x + 2)); it isn't searched further.
fn factor_poly(p: &Poly) -> Option<(Rat, Vec<(Poly, u32)>)> {
    if p.is_zero() {
        return None;
    }
    let (mut q, _) = p.normalized()?;
    let mut out = vec![];
    // the variables every term has
    let low: Mono = (0..q.n).map(|i| q.t.keys().map(|m| m[i]).min().unwrap_or(0)).collect();
    if low.iter().any(|&e| e > 0) {
        for (i, &e) in low.iter().enumerate() {
            if e > 0 {
                out.push((Poly::var(q.n, i), e));
            }
        }
        let mut m = Poly::zero(q.n);
        m.t.insert(low, Rat::ONE);
        q = q.div_exact(&m)?;
    }
    merge(&mut out, split(&q, 0));
    // the constant: what the factors don't give of p
    let mut prod = Poly::constant(p.n, Rat::ONE);
    for (f, e) in &out {
        prod = prod.mul(&f.powi(*e)?)?;
    }
    let k = p.lead()?.1.div(*prod.lead()?.1)?;
    Some((k, out))
}

/// Adds factors to a list, a factor already there by its multiplicity.
fn merge(out: &mut Vec<(Poly, u32)>, more: Vec<(Poly, u32)>) {
    for (f, e) in more {
        match out.iter_mut().find(|g| g.0 == f) {
            Some(g) => g.1 += e,
            None => out.push((f, e)),
        }
    }
}

/// The factors of q (primitive, whole coefficients, no variable dividing it); q itself where it can't be split.
fn split(q: &Poly, depth: u32) -> Vec<(Poly, u32)> {
    if q.is_const() {
        return vec![];
    }
    if depth < 16 {
        if let Some(fs) = try_split(q, depth) {
            return fs;
        }
    }
    vec![(q.normalized().map_or_else(|| q.clone(), |n| n.0), 1)]
}

fn try_split(q: &Poly, depth: u32) -> Option<Vec<(Poly, u32)>> {
    let q = q.normalized()?.0;
    let both = |a: &Poly, b: &Poly| -> Option<Vec<(Poly, u32)>> {
        let mut out = split(a, depth + 1);
        merge(&mut out, split(b, depth + 1));
        Some(out)
    };
    // the part free of a variable: its content in it
    for v in q.vars() {
        let c = q.content(v)?;
        if !c.is_const() {
            return both(&c, &q.div_exact(&c)?);
        }
    }
    // now every factor has every variable: a repeated one is shared with the derivative in any of them
    let vs = q.vars();
    let g = gcd(&q, &q.diff(vs[0])?)?;
    if !g.is_const() {
        return both(&g, &q.div_exact(&g)?);
    }
    // of degree one in a variable it has no factor free of: irreducible
    if vs.iter().any(|&v| q.deg(v) == 1) {
        return Some(vec![(q, 1)]);
    }
    for &v in &vs {
        if let Some(f) = linear_factor(&q, v)? {
            let mut out = vec![(f.clone(), 1)];
            merge(&mut out, split(&q.div_exact(&f)?, depth + 1));
            return Some(out);
        }
    }
    Some(vec![(q, 1)])
}

/// A factor a·v + b of q: a divides q's leading coefficient in v, b its coefficient free of v (the rational root
/// test, with polynomials for the numbers when there are more variables).
fn linear_factor(q: &Poly, v: usize) -> Option<Option<Poly>> {
    let cs = q.coeffs(v);
    let (lc, tc) = (cs.last()?, &cs[0]);
    if tc.is_zero() {
        return Some(Some(Poly::var(q.n, v)));
    }
    let (da, db) = (divisors(lc)?, divisors(tc)?);
    if da.len() * db.len() > 4000 {
        return None;
    }
    for a in &da {
        for b in &db {
            for b in [b.clone(), b.neg()?] {
                let f = a.shift(v, 1).add(&b)?;
                if q.div_exact(&f).is_some() {
                    return Some(Some(f.normalized()?.0));
                }
            }
        }
    }
    Some(None)
}

/// The divisors of a polynomial up to sign: products of its factors and of the divisors of its whole constant.
fn divisors(p: &Poly) -> Option<Vec<Poly>> {
    // free of a variable of q: its factors take fewer variables, so this ends
    let (mut k, fs) = if p.is_const() { (*p.t.values().next()?, vec![]) } else { factor_poly(p)? };
    if !k.is_int() {
        return None;
    }
    k = k.abs();
    let n = k.numer();
    let mut ints = vec![];
    if n > 10_000_000_000 {
        ints.extend([1, n]);
    } else {
        let mut i = 1;
        while i * i <= n {
            if n % i == 0 {
                ints.push(i);
                if i * i != n {
                    ints.push(n / i);
                }
            }
            i += 1;
        }
    }
    let mut out: Vec<Poly> = ints.into_iter().map(|i| Poly::constant(p.n, Rat::int(i))).collect();
    for (f, e) in fs {
        let mut more = vec![];
        for d in &out {
            let mut x = d.clone();
            for _ in 0..e {
                x = x.mul(&f)?;
                more.push(x.clone());
            }
        }
        out.extend(more);
        if out.len() > 2000 {
            return None;
        }
    }
    Some(out)
}

/// Atoms of a polynomial: symbols, constants like π, functions, powers that aren't whole.
struct Atoms(Vec<Expr>);

impl Atoms {
    fn index(&mut self, e: &Expr) -> usize {
        match self.0.iter().position(|a| a == e) {
            Some(i) => i,
            None => {
                self.0.push(e.clone());
                self.0.len() - 1
            }
        }
    }
}

const MAXV: usize = 24;

/// An expression as a fraction of polynomials (in the atoms), or None if it has floats or gets too big.
fn ratfn(e: &Expr, at: &mut Atoms) -> Option<(Poly, Poly)> {
    let one = |n| Poly::constant(n, Rat::ONE);
    Some(match e {
        Expr::C(v) => {
            let r = v.v.rat()?;
            let (mut n, mut d) = (Poly::constant(MAXV, r), one(MAXV));
            // the unit as powers of base-unit atoms
            for (identity, power) in &v.d.0 {
                let k = Rat::new(*power as i128, 60)?;
                if k.is_zero() {
                    continue;
                }
                if !k.is_int() {
                    return None;
                }
                let ai = at.index(&Expr::C(Val::new(Num::ONE, Dim::named(identity))));
                if ai >= MAXV {
                    return None;
                }
                let p = Poly::var(MAXV, ai).powi(k.numer().unsigned_abs() as u32)?;
                if k.is_neg() { d = d.mul(&p)? } else { n = n.mul(&p)? }
            }
            (n, d)
        }
        Expr::Add(ts) => {
            let mut acc = (Poly::zero(MAXV), one(MAXV));
            for t in ts {
                let (n, d) = ratfn(t, at)?;
                acc = if acc.1 == d { (acc.0.add(&n)?, d) } else { (acc.0.mul(&d)?.add(&n.mul(&acc.1)?)?, acc.1.mul(&d)?) };
                acc = cancel(acc)?;
            }
            acc
        }
        Expr::Mul(fs) => {
            let mut acc = (one(MAXV), one(MAXV));
            for f in fs {
                let (n, d) = ratfn(f, at)?;
                acc = (acc.0.mul(&n)?, acc.1.mul(&d)?);
            }
            cancel(acc)?
        }
        Expr::Pow(b, x) if x.as_num().and_then(|k| k.as_int()).is_some_and(|k| k.abs() <= 30) => {
            let k = x.as_num().unwrap().as_int().unwrap();
            let (n, d) = ratfn(b, at)?;
            let (n, d) = (n.powi(k.unsigned_abs() as u32)?, d.powi(k.unsigned_abs() as u32)?);
            if k < 0 { (d, n) } else { (n, d) }
        }
        _ => {
            if e.contains(&|x| matches!(x, Expr::C(v) if !v.v.is_exact())) {
                return None;
            }
            let i = at.index(e);
            if i >= MAXV {
                return None;
            }
            (Poly::var(MAXV, i), one(MAXV))
        }
    })
}

fn cancel((n, d): (Poly, Poly)) -> Option<(Poly, Poly)> {
    if d.is_zero() {
        return None;
    }
    if n.is_zero() {
        return Some((n, Poly::constant(MAXV, Rat::ONE)));
    }
    let g = gcd(&n, &d)?;
    let (n, d) = if g.is_const() { (n, d) } else { (n.div_exact(&g)?, d.div_exact(&g)?) };
    // whole coefficients below, the leading one positive
    let (d, k) = d.normalized()?;
    Some((n.scale(k)?, d))
}

fn poly_expr(p: &Poly, at: &Atoms) -> Expr {
    let mut ts = vec![];
    for (m, c) in &p.t {
        let mut fs = vec![Expr::C(Val::num(Num::R(*c)))];
        for (i, &e) in m.iter().enumerate() {
            if e > 0 {
                fs.push(expr::pow(at.0[i].clone(), int(e as i128)));
            }
        }
        ts.push(Expr::Mul(fs));
    }
    simp(&Expr::Add(ts))
}

/// The expression with its fractions over one denominator and common factors cancelled, or None where that
/// can't be done exactly.
pub fn cancel_expr(e: &Expr) -> Option<Expr> {
    let mut at = Atoms(vec![]);
    let (n, d) = ratfn(e, &mut at)?;
    // whole coefficients above and below, the constant factor in front: (p + q)/2
    let (n, kn) = n.normalized()?;
    let (d, kd) = d.normalized()?;
    let k = Expr::C(Val::num(Num::R(kd.div(kn)?)));
    let (n, d) = (poly_expr(&n, &at), poly_expr(&d, &at));
    Some(simp(&expr::mul(vec![k, expr::div(n, d)])))
}

/// An expression over one denominator, both in factors: k · Π numᵉ / Π denᵉ.
#[derive(Clone, Debug, PartialEq)]
pub struct Factored {
    pub k: Rat,
    pub num: Vec<(Expr, u32)>,
    pub den: Vec<(Expr, u32)>,
}

impl Factored {
    pub fn is_zero(&self) -> bool {
        self.k.is_zero()
    }
    /// The numerator's factors multiplied, without the constant: what is zero where the expression is.
    pub fn numerator(&self) -> Expr {
        simp(&expr::mul(self.num.iter().map(|(f, e)| expr::pow(f.clone(), int(*e as i128))).collect()))
    }
}

/// The factors of an expression (see `factor_poly` for how far it splits), None where it isn't exact.
pub fn factor(e: &Expr) -> Option<Factored> {
    let mut at = Atoms(vec![]);
    let (n, d) = ratfn(e, &mut at)?;
    if n.is_zero() {
        return Some(Factored { k: Rat::int(0), num: vec![], den: vec![] });
    }
    let (kn, num) = factor_poly(&n)?;
    let (kd, den) = factor_poly(&d)?;
    let exprs = |fs: Vec<(Poly, u32)>| fs.into_iter().map(|(f, e)| (poly_expr(&f, &at), e)).collect();
    Some(Factored { k: kn.div(kd)?, num: exprs(num), den: exprs(den) })
}

/// How a symbolic result is shown: the simplest form (`simplify`), multiplied out (`expand`), in factors (`factor`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Form {
    #[default]
    Simplest,
    Expanded,
    Factored,
}

/// e in the form asked for: an equation side by side, a vector or matrix element by element. None where factors
/// can't be found (decimals).
pub fn shaped(e: &Expr, form: Form) -> Option<Expr> {
    if form == Form::Simplest {
        return Some(simplify(e));
    }
    let each = |v: &[Expr]| v.iter().map(|x| shaped(x, form)).collect::<Option<Vec<_>>>();
    match e {
        Expr::Cmp(v, rs) => Some(Expr::Cmp(each(v)?, rs.clone())),
        Expr::Vector(v) => Some(Expr::Vector(each(v)?)),
        Expr::Matrix(rs) => Some(Expr::Matrix(rs.iter().map(|r| each(r)).collect::<Option<_>>()?)),
        _ if form == Form::Expanded => Some(expr::expand(&simplify(e))),
        _ => {
            let s = simplify(e);
            // a number isn't split into primes
            if s.syms().is_empty() || s.contains(&crate::linalg::is_array) {
                return Some(s);
            }
            let f = factor(&s)?;
            let mut fs = vec![Expr::C(Val::num(Num::R(f.k)))];
            fs.extend(f.num.iter().map(|(g, e)| expr::pow(g.clone(), int(*e as i128))));
            fs.extend(f.den.iter().map(|(g, e)| expr::pow(g.clone(), int(-(*e as i128)))));
            Some(simp(&expr::mul(fs)))
        }
    }
}

/// The simplest of the forms solveng knows: as it is (canonical), with fractions cancelled, multiplied out.
pub fn simplify(e: &Expr) -> Expr {
    let a = simp(&e.map(&mut simplify));
    // arrays are simplified element by element (above); as polynomial atoms they would commute
    if a.contains(&crate::linalg::is_array) {
        return a;
    }
    let mut best = a.clone();
    let mut cands = vec![];
    if let Some(b) = cancel_expr(&a) {
        cands.push(b.clone());
        // a cancelled fraction with its numerator factored again: 2x + 2 over 4 → (x + 1)/2
        cands.push(simp(&expr::expand(&b)));
    }
    cands.push(expr::expand(&a));
    for c in cands {
        if c.size() < best.size() {
            best = c;
        }
    }
    best
}

/// Numerator and denominator of an expression brought over one denominator (symbolically, any factors).
pub fn together(e: &Expr) -> (Expr, Expr) {
    let e = simp(e);
    match &e {
        Expr::Add(ts) => {
            let parts: Vec<(Expr, Expr)> = ts.iter().map(together).collect();
            // the least common multiple of the denominators, factor by factor
            let mut lcm: Vec<(Expr, Expr)> = vec![];
            for (_, d) in &parts {
                for f in factors(d) {
                    let (b, x) = base_exp(&f);
                    match lcm.iter_mut().find(|g| g.0 == b) {
                        Some(g) => {
                            if let (Some(a), Some(c)) = (g.1.as_num(), x.as_num()) {
                                if c.cmp(a).is_gt() {
                                    g.1 = x;
                                }
                            } else if g.1 != x {
                                g.1 = simp(&expr::add(vec![g.1.clone(), x]));
                            }
                        }
                        None => lcm.push((b, x)),
                    }
                }
            }
            let l = simp(&expr::mul(lcm.iter().map(|(b, x)| expr::pow(b.clone(), x.clone())).collect()));
            let n = simp(&expr::add(parts.into_iter().map(|(n, d)| simp(&expr::mul(vec![n, expr::div(l.clone(), d)]))).collect()));
            (n, l)
        }
        Expr::Mul(_) | Expr::Pow(..) => {
            let (n, d) = expr::num_den(&e);
            // sums inside the numerator may have fractions of their own
            let (n2, d2) = match &n {
                Expr::Add(_) => together(&n),
                Expr::Mul(fs) if fs.iter().any(|f| matches!(f, Expr::Add(_))) => {
                    let mut nn = vec![];
                    let mut dd = vec![];
                    for f in fs {
                        let (a, b) = if matches!(f, Expr::Add(_)) { together(f) } else { (f.clone(), int(1)) };
                        nn.push(a);
                        dd.push(b);
                    }
                    (simp(&expr::mul(nn)), simp(&expr::mul(dd)))
                }
                _ => (n, int(1)),
            };
            (n2, simp(&expr::mul(vec![d, d2])))
        }
        _ => (e, int(1)),
    }
}

fn factors(e: &Expr) -> Vec<Expr> {
    match e {
        Expr::Mul(fs) => fs.iter().filter(|f| !matches!(f, Expr::C(_))).cloned().collect(),
        Expr::C(_) => vec![],
        e => vec![e.clone()],
    }
}

/// The coefficients of e as a polynomial in x (ascending), None if it isn't one.
pub fn coeffs(e: &Expr, x: &str) -> Option<Vec<Expr>> {
    let e = expr::expand(e);
    let ts = match &e {
        Expr::Add(ts) => ts.clone(),
        t => vec![t.clone()],
    };
    let mut out: Vec<Vec<Expr>> = vec![];
    for t in ts {
        let fs = match &t {
            Expr::Mul(fs) => fs.clone(),
            f => vec![f.clone()],
        };
        let mut k = 0usize;
        let mut rest = vec![];
        for f in fs {
            if !f.has(x) {
                rest.push(f);
                continue;
            }
            let (b, p) = base_exp(&f);
            let n = p.as_num().and_then(|n| n.as_int())?;
            if b != Expr::Sym(x.to_string()) || n < 0 || n > 60 {
                return None;
            }
            k += n as usize;
        }
        if out.len() <= k {
            out.resize(k + 1, vec![]);
        }
        out[k].push(simp(&expr::mul(rest)));
    }
    Some(out.into_iter().map(|v| simp(&expr::add(v))).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::*;

    #[test]
    fn cancelling() {
        let x = sym("x");
        let e = div(sub(pow(x.clone(), int(2)), int(1)), sub(x.clone(), int(1)));
        assert_eq!(simplify(&e), add(vec![x.clone(), int(1)]));
        // (a x + a y)/a = x + y
        let (a, y) = (sym("a"), sym("y"));
        let e = div(add(vec![mul(vec![a.clone(), x.clone()]), mul(vec![a.clone(), y.clone()])]), a.clone());
        assert_eq!(simplify(&e), add(vec![x.clone(), y.clone()]));
        // (x+1)^2 stays
        let e = pow(add(vec![x.clone(), int(1)]), int(2));
        assert_eq!(simplify(&e), e);
    }

    #[test]
    fn polynomial_in_x() {
        let (x, y) = (sym("x"), sym("y"));
        // 1 + 3/x + y^2 = 0 over one denominator: x + 3 + x y² (over x)
        let e = add(vec![int(1), div(int(3), x.clone()), pow(y.clone(), int(2))]);
        let (n, d) = together(&e);
        assert_eq!(d, x.clone());
        let c = coeffs(&n, "x").unwrap();
        assert_eq!(c, vec![int(3), add(vec![pow(y.clone(), int(2)), int(1)])]);
    }

    /// e's factors: the numerator's, as k · Π fᵉ with each f simplified.
    fn factors_of(e: &Expr) -> (Rat, Vec<(Expr, u32)>) {
        let f = factor(&expand(e)).unwrap();
        (f.k, f.num)
    }

    #[test]
    fn factoring() {
        let (x, y, a, p, t) = (sym("x"), sym("y"), sym("a"), sym("p"), sym("t"));
        // a² p² t − a p² t = a p² t (a − 1)
        let e = sub(mul(vec![pow(a.clone(), int(2)), pow(p.clone(), int(2)), t.clone()]), mul(vec![a.clone(), pow(p.clone(), int(2)), t.clone()]));
        let (k, fs) = factors_of(&e);
        assert_eq!(k, Rat::ONE);
        assert_eq!(fs, vec![(a.clone(), 1), (p.clone(), 2), (t.clone(), 1), (simp(&sub(a.clone(), int(1))), 1)]);
        // x² − y² = (x − y)(x + y)
        let (_, fs) = factors_of(&sub(pow(x.clone(), int(2)), pow(y.clone(), int(2))));
        assert_eq!(fs.len(), 2);
        assert!(fs.contains(&(simp(&sub(x.clone(), y.clone())), 1)) && fs.contains(&(simp(&add(vec![x.clone(), y.clone()])), 1)));
        // 2x³ − 2x = 2 x (x − 1)(x + 1)
        let (k, fs) = factors_of(&sub(mul(vec![int(2), pow(x.clone(), int(3))]), mul(vec![int(2), x.clone()])));
        assert_eq!(k, Rat::int(2));
        assert_eq!(fs.len(), 3);
        // repeated: (x − 1)² (x + 2)³ (y + 1)
        let e = mul(vec![pow(sub(x.clone(), int(1)), int(2)), pow(add(vec![x.clone(), int(2)]), int(3)), add(vec![y.clone(), int(1)])]);
        let (_, mut fs) = factors_of(&e);
        fs.sort_by_key(|f| f.1);
        assert_eq!(fs, vec![(simp(&add(vec![y.clone(), int(1)])), 1), (simp(&sub(x.clone(), int(1))), 2), (simp(&add(vec![x.clone(), int(2)])), 3)]);
        // rational roots: 6x² − x − 1 = (2x − 1)(3x + 1); x⁴ − 1 = (x − 1)(x + 1)(x² + 1)
        assert_eq!(factors_of(&add(vec![mul(vec![int(6), pow(x.clone(), int(2))]), neg(x.clone()), int(-1)])).1.len(), 2);
        let (_, fs) = factors_of(&sub(pow(x.clone(), int(4)), int(1)));
        assert_eq!(fs.len(), 3);
        assert!(fs.contains(&(simp(&add(vec![pow(x.clone(), int(2)), int(1)])), 1)));
        // irreducible stays: x² + y², x² − 2
        assert_eq!(factors_of(&add(vec![pow(x.clone(), int(2)), pow(y.clone(), int(2))])).1.len(), 1);
        assert_eq!(factors_of(&sub(pow(x.clone(), int(2)), int(2))).1.len(), 1);
        // over a denominator: (x² − 1)/(2x) has x − 1 and x + 1 above, x below
        let f = factor(&div(sub(pow(x.clone(), int(2)), int(1)), mul(vec![int(2), x.clone()]))).unwrap();
        assert_eq!((f.k, f.num.len(), f.den), (Rat::new(1, 2).unwrap(), 2, vec![(x.clone(), 1)]));
        // zero
        assert!(factor(&sub(x.clone(), x.clone())).unwrap().is_zero());
    }

    #[test]
    fn gcds() {
        let x = sym("x");
        let a = expand(&mul(vec![add(vec![x.clone(), int(1)]), add(vec![x.clone(), int(2)])]));
        let b = expand(&mul(vec![add(vec![x.clone(), int(1)]), add(vec![x.clone(), int(3)])]));
        let e = div(a, b);
        assert_eq!(simplify(&e), simp(&div(add(vec![x.clone(), int(2)]), add(vec![x.clone(), int(3)]))));
    }
}
