//! Complex numbers: exact ones as `a + b·i` with real constants a and b (the canonical form `simp` keeps), and
//! floating point ones for numeric evaluation.
//!
//! Values are calculated in the complex numbers. Operations keep their real value whenever they have one
//! ((−8)^(1/3) is −2) and take their principal complex value where the real one is missing (√−4 is 2i,
//! ln(−1) is iπ). Graphs and numeric solving accept real results with complex intermediates; ordering still requires real operands.

use crate::dim::Dim;
use crate::expr::{Expr, Val, simp};
use crate::num::{Num, Rat};

mod symbolic;
pub use symbolic::projection;
pub use symbolic::{assuming_real, real_conditions};

/// A complex floating point number.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Z {
    pub re: f64,
    pub im: f64,
}

impl Z {
    pub const I: Z = Z { re: 0.0, im: 1.0 };

    pub fn new(re: f64, im: f64) -> Z {
        Z { re, im }
    }
    pub fn real(re: f64) -> Z {
        Z { re, im: 0.0 }
    }
    pub fn is_real(self) -> bool {
        self.im == 0.0
    }
    pub fn is_finite(self) -> bool {
        self.re.is_finite() && self.im.is_finite()
    }
    pub fn is_nan(self) -> bool {
        self.re.is_nan() || self.im.is_nan()
    }
    pub fn add(self, o: Z) -> Z {
        Z::new(self.re + o.re, self.im + o.im)
    }
    pub fn sub(self, o: Z) -> Z {
        Z::new(self.re - o.re, self.im - o.im)
    }
    pub fn neg(self) -> Z {
        Z::new(-self.re, -self.im)
    }
    pub fn mul(self, o: Z) -> Z {
        if self.is_real() && o.is_real() {
            return Z::real(self.re * o.re);
        }
        Z::new(self.re * o.re - self.im * o.im, self.re * o.im + self.im * o.re)
    }
    pub fn scale(self, k: f64) -> Z {
        Z::new(self.re * k, self.im * k)
    }
    pub fn div(self, o: Z) -> Z {
        if o.is_real() {
            return Z::new(self.re / o.re, self.im / o.re);
        }
        // scaled, so that large and small parts don't overflow
        let s = o.re.abs().max(o.im.abs());
        let (a, b) = (o.re / s, o.im / s);
        let d = a * a + b * b;
        Z::new((self.re * a + self.im * b) / d / s, (self.im * a - self.re * b) / d / s)
    }
    pub fn conj(self) -> Z {
        Z::new(self.re, -self.im)
    }
    pub fn abs(self) -> f64 {
        self.re.hypot(self.im)
    }
    pub fn arg(self) -> f64 {
        self.im.atan2(self.re)
    }
    pub fn exp(self) -> Z {
        let m = self.re.exp();
        if self.im == 0.0 {
            return Z::real(m);
        }
        Z::new(m * self.im.cos(), m * self.im.sin())
    }
    /// The principal logarithm.
    pub fn ln(self) -> Z {
        Z::new(self.abs().ln(), self.arg())
    }
    /// The principal square root.
    pub fn sqrt(self) -> Z {
        if self.im == 0.0 {
            return if self.re >= 0.0 { Z::real(self.re.sqrt()) } else { Z::new(0.0, (-self.re).sqrt()) };
        }
        let m = self.abs();
        let re = ((m + self.re) / 2.0).sqrt();
        let im = ((m - self.re) / 2.0).sqrt();
        Z::new(re, if self.im < 0.0 { -im } else { im })
    }
    /// The principal power; whole exponents by repeated multiplication, so that i² is exactly −1.
    pub fn pow(self, w: Z) -> Z {
        if w.is_real() && w.re.fract() == 0.0 && w.re.abs() <= 64.0 {
            let mut r = Z::real(1.0);
            for _ in 0..w.re.abs() as u32 {
                r = r.mul(self);
            }
            return if w.re < 0.0 { Z::real(1.0).div(r) } else { r };
        }
        if self.re == 0.0 && self.im == 0.0 {
            return Z::real(0.0);
        }
        if w.is_real() && w.re == 0.5 {
            return self.sqrt();
        }
        w.mul(self.ln()).exp()
    }
    pub fn sin(self) -> Z {
        Z::new(self.re.sin() * self.im.cosh(), self.re.cos() * self.im.sinh())
    }
    pub fn cos(self) -> Z {
        Z::new(self.re.cos() * self.im.cosh(), -self.re.sin() * self.im.sinh())
    }
    pub fn sinh(self) -> Z {
        Z::new(self.re.sinh() * self.im.cos(), self.re.cosh() * self.im.sin())
    }
    pub fn cosh(self) -> Z {
        Z::new(self.re.cosh() * self.im.cos(), self.re.sinh() * self.im.sin())
    }
    /// asin z = −i ln(iz + √(1 − z²))
    pub fn asin(self) -> Z {
        let root = Z::real(1.0).sub(self.mul(self)).sqrt();
        Z::I.mul(self).add(root).ln().mul(Z::I).neg()
    }
    /// acos z = π/2 − asin z
    pub fn acos(self) -> Z {
        Z::real(std::f64::consts::FRAC_PI_2).sub(self.asin())
    }
    /// atan z = i/2 · ln((i + z)/(i − z))
    pub fn atan(self) -> Z {
        Z::I.add(self).div(Z::I.sub(self)).ln().mul(Z::new(0.0, 0.5))
    }
    /// Parts much smaller than the whole are rounding noise of the other part's calculation (cos(π/2) for 0):
    /// they become 0.
    pub fn cleaned(self) -> Z {
        let m = self.abs();
        let tiny = |x: f64| x.abs() <= 1e-14 * m;
        Z::new(if tiny(self.re) { 0.0 } else { self.re }, if tiny(self.im) { 0.0 } else { self.im })
    }
}

/// A complex value with a unit: real and imaginary part, of one dimension.
#[derive(Clone, Debug, PartialEq)]
pub struct CVal {
    pub re: Val,
    pub im: Val,
}

impl CVal {
    pub fn real(v: Val) -> CVal {
        let im = Val::new(Num::ZERO, v.d.clone());
        CVal { re: v, im }
    }
    pub fn is_real(&self) -> bool {
        self.im.v.is_zero()
    }
    pub fn d(&self) -> &Dim {
        &self.re.d
    }
    pub fn mul(&self, o: &CVal) -> CVal {
        let re = self.re.mul(&o.re).add(&self.im.mul(&o.im).neg()).expect("parts of one dimension");
        let im = self.re.mul(&o.im).add(&self.im.mul(&o.re)).expect("parts of one dimension");
        CVal { re: plain(re), im: plain(im) }
    }
    /// 1/z = z̄/|z|²; None for 0.
    pub fn recip(&self) -> Option<CVal> {
        let m2 = self.re.mul(&self.re).add(&self.im.mul(&self.im))?;
        let inv = m2.pow(Num::int(-1))?;
        Some(CVal { re: plain(self.re.mul(&inv)), im: plain(self.im.neg().mul(&inv)) })
    }
    pub fn powi(&self, n: i64) -> Option<CVal> {
        let base = if n < 0 { self.recip()? } else { self.clone() };
        let mut r = CVal::real(Val::int(1));
        let mut b = base;
        let mut k = n.unsigned_abs();
        while k > 0 {
            if k & 1 == 1 {
                r = r.mul(&b);
            }
            k >>= 1;
            if k > 0 {
                b = b.mul(&b);
            }
        }
        Some(r)
    }
    /// The principal square root, where it is exact: √(a + bi) = √((|z| + a)/2) ± i √((|z| − a)/2).
    pub fn sqrt(&self) -> Option<CVal> {
        let m2 = self.re.mul(&self.re).add(&self.im.mul(&self.im))?;
        let m = exact(m2.pow(Num::R(Rat::new(1, 2)?))?)?;
        let half = Val::num(Num::R(Rat::new(1, 2)?));
        let re = exact(m.add(&self.re)?.mul(&half).pow(Num::R(Rat::new(1, 2)?))?)?;
        let im = exact(m.add(&self.re.neg())?.mul(&half).pow(Num::R(Rat::new(1, 2)?))?)?;
        Some(CVal { re: plain(re), im: plain(if self.im.v.is_neg() { im.neg() } else { im }) })
    }
    pub fn z(&self) -> Z {
        Z::new(self.re.v.f(), self.im.v.f())
    }
}

fn exact(v: Val) -> Option<Val> {
    v.v.is_exact().then_some(v)
}

/// A computed part keeps no unit as written.
fn plain(v: Val) -> Val {
    Val { u: None, raw: None, ..v }
}

/// The parts of a constant: a real constant, i, a constant times i, or a sum of these. None for anything else,
/// and for infinities and NaN: ∞·i has no real and imaginary part (∞·0 is NaN), so it stays a product.
pub fn parts(e: &Expr) -> Option<CVal> {
    let c = finite_parts(e)?;
    (c.re.v.f().is_finite() && c.im.v.f().is_finite()).then_some(c)
}

fn finite_parts(e: &Expr) -> Option<CVal> {
    match e {
        Expr::C(v) => Some(CVal::real(v.clone())),
        Expr::I => Some(CVal { re: Val::int(0), im: Val::int(1) }),
        Expr::Mul(fs) if fs.len() == 2 => match (&fs[0], &fs[1]) {
            (Expr::C(v), Expr::I) => Some(CVal { re: Val::new(Num::ZERO, v.d.clone()), im: v.clone() }),
            _ => None,
        },
        Expr::Add(ts) => {
            let mut acc: Option<CVal> = None;
            for t in ts {
                let p = parts(t)?;
                acc = Some(match acc {
                    None => p,
                    Some(a) => {
                        // Only a unitless zero can adopt the other operand's dimension.
                        let (re, im) = (unify(&a.re, &p.re)?, unify(&a.im, &p.im)?);
                        CVal { re, im }
                    }
                });
            }
            let mut c = acc?;
            // both parts in the dimension of the one that isn't 0
            if c.re.v.is_zero() && c.re.d != c.im.d { c.re = Val::new(Num::ZERO, c.im.d.clone()); }
            if c.im.v.is_zero() && c.im.d != c.re.d { c.im = Val::new(Num::ZERO, c.re.d.clone()); }
            (c.re.d == c.im.d).then_some(c)
        }
        _ => None,
    }
}

fn unify(a: &Val, b: &Val) -> Option<Val> {
    if a.v.is_zero() && a.d.is_one() && a.d != b.d { return Some(b.clone()); }
    if b.v.is_zero() && b.d.is_one() && a.d != b.d { return Some(a.clone()); }
    a.add(b)
}

/// Whether an expression is a constant with an imaginary part: 2i, 1 + 2i.
pub fn is_complex(e: &Expr) -> bool {
    e.contains(&|x| *x == Expr::I) && parts(e).is_some()
}

/// The canonical expression of a complex constant.
pub fn expr(c: CVal) -> Expr {
    if c.im.v.is_zero() {
        return Expr::C(c.re);
    }
    let im = if c.im.is_one() { Expr::I } else { Expr::Mul(vec![Expr::C(c.im), Expr::I]) };
    if c.re.v.is_zero() {
        return im;
    }
    simp(&Expr::Add(vec![Expr::C(c.re), im]))
}

/// Whether an expression has an imaginary unit in it: then it's calculated in the complex numbers.
pub fn has_i(e: &Expr) -> bool {
    e.contains(&|x| *x == Expr::I)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: Z, b: Z) -> bool {
        (a.re - b.re).abs() < 1e-12 && (a.im - b.im).abs() < 1e-12
    }

    #[test]
    fn functions() {
        let z = Z::new(0.3, -1.2);
        assert!(close(z.sin().asin(), z));
        assert!(close(z.cos().acos(), z));
        assert!(close(z.atan().sin().div(z.atan().cos()), z));
        assert!(close(z.exp().ln(), z));
        assert!(close(z.sqrt().mul(z.sqrt()), z));
        assert!(close(Z::I.mul(Z::I), Z::real(-1.0)));
        assert_eq!(Z::I.pow(Z::real(2.0)), Z::real(-1.0));
        assert!(close(Z::new(0.0, std::f64::consts::PI).exp().cleaned(), Z::real(-1.0)));
    }

    #[test]
    fn exact() {
        let a = CVal { re: Val::int(1), im: Val::int(2) };
        let b = CVal { re: Val::int(3), im: Val::int(-1) };
        assert_eq!(a.mul(&b), CVal { re: Val::int(5), im: Val::int(5) });
        let r = a.recip().unwrap();
        assert_eq!(r.mul(&a), CVal::real(Val::int(1)));
        assert_eq!(Expr::I, expr(CVal { re: Val::int(0), im: Val::int(1) }));
    }
}
