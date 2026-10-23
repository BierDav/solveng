//! Numbers: exact rationals where the arithmetic allows, floats otherwise.

use std::cmp::Ordering;
use std::fmt;

use serde::{Deserialize, Serialize};

/// A reduced fraction `n/d` with `d > 0`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Rat {
    n: i128,
    d: i128,
}

fn gcd(mut a: i128, mut b: i128) -> i128 {
    a = a.abs();
    b = b.abs();
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

impl Rat {
    pub const ZERO: Rat = Rat { n: 0, d: 1 };
    pub const ONE: Rat = Rat { n: 1, d: 1 };

    pub fn new(n: i128, d: i128) -> Option<Rat> {
        if d == 0 {
            return None;
        }
        let g = gcd(n, d);
        let g = if g == 0 { 1 } else { g };
        let (mut n, mut d) = (n / g, d / g);
        if d < 0 {
            n = n.checked_neg()?;
            d = d.checked_neg()?;
        }
        Some(Rat { n, d })
    }
    pub fn int(n: i128) -> Rat {
        Rat { n, d: 1 }
    }
    pub fn numer(self) -> i128 {
        self.n
    }
    pub fn denom(self) -> i128 {
        self.d
    }
    pub fn is_zero(self) -> bool {
        self.n == 0
    }
    pub fn is_int(self) -> bool {
        self.d == 1
    }
    pub fn is_neg(self) -> bool {
        self.n < 0
    }
    pub fn to_f64(self) -> f64 {
        self.n as f64 / self.d as f64
    }
    pub fn add(self, o: Rat) -> Option<Rat> {
        let g = gcd(self.d, o.d);
        let l = (self.d / g).checked_mul(o.d)?;
        let a = self.n.checked_mul(l / self.d)?;
        let b = o.n.checked_mul(l / o.d)?;
        Rat::new(a.checked_add(b)?, l)
    }
    pub fn sub(self, o: Rat) -> Option<Rat> {
        self.add(o.neg()?)
    }
    pub fn neg(self) -> Option<Rat> {
        Some(Rat { n: self.n.checked_neg()?, d: self.d })
    }
    pub fn mul(self, o: Rat) -> Option<Rat> {
        let g1 = gcd(self.n, o.d).max(1);
        let g2 = gcd(o.n, self.d).max(1);
        Rat::new((self.n / g1).checked_mul(o.n / g2)?, (self.d / g2).checked_mul(o.d / g1)?)
    }
    pub fn recip(self) -> Option<Rat> {
        Rat::new(self.d, self.n)
    }
    pub fn div(self, o: Rat) -> Option<Rat> {
        self.mul(o.recip()?)
    }
    pub fn abs(self) -> Rat {
        Rat { n: self.n.abs(), d: self.d }
    }
    pub fn powi(self, e: i64) -> Option<Rat> {
        if e < 0 {
            return self.recip()?.powi(e.checked_neg()?);
        }
        let mut r = Rat::ONE;
        let mut b = self;
        let mut e = e as u64;
        while e > 0 {
            if e & 1 == 1 {
                r = r.mul(b)?;
            }
            e >>= 1;
            if e > 0 {
                b = b.mul(b)?;
            }
        }
        Some(r)
    }
    /// The exact k-th root, if there is one: 4/9 → 2/3.
    pub fn root(self, k: u32) -> Option<Rat> {
        if k == 1 {
            return Some(self);
        }
        if self.n < 0 && k % 2 == 0 {
            return None;
        }
        let r = |x: i128| -> Option<i128> {
            let neg = x < 0;
            let x = x.abs();
            let g = (x as f64).powf(1.0 / k as f64).round() as i128;
            for c in [g - 1, g, g + 1] {
                if c >= 0 && c.checked_pow(k) == Some(x) {
                    return Some(if neg { -c } else { c });
                }
            }
            None
        };
        Rat::new(r(self.n)?, r(self.d)?)
    }
    /// A decimal written out: "7.20" → 36/5, "0,5" → 1/2. No e-notation.
    pub fn parse_decimal(s: &str) -> Option<Rat> {
        let mant = s.trim().replace(',', ".");
        let (ip, fp) = match mant.find('.') {
            Some(i) => (&mant[..i], &mant[i + 1..]),
            None => (mant.as_str(), ""),
        };
        if ip.is_empty() && fp.is_empty() {
            return None;
        }
        if !ip.chars().chain(fp.chars()).all(|c| c.is_ascii_digit()) {
            return None;
        }
        let digits = format!("{ip}{fp}");
        let digits = digits.trim_start_matches('0');
        if digits.len() > 30 {
            return None;
        }
        let n: i128 = if digits.is_empty() { 0 } else { digits.parse().ok()? };
        let e = -(fp.len() as i64);
        if e.abs() > 36 {
            return None;
        }
        let p = 10i128.checked_pow(e.unsigned_abs() as u32)?;
        Rat::new(n, p)
    }
    /// The closest fraction with a denominator ≤ `max` that equals `x` (within 1e-12), by continued fractions.
    pub fn approx(x: f64, max: i128) -> Option<Rat> {
        if !x.is_finite() || x.abs() >= 1e15 {
            return None;
        }
        if x.fract() == 0.0 {
            return Some(Rat::int(x as i128));
        }
        let (mut h1, mut h2, mut k1, mut k2) = (1.0f64, 0.0f64, 0.0f64, 1.0f64);
        let mut y = x.abs();
        for _ in 0..40 {
            let a = y.floor();
            (h1, h2) = (a * h1 + h2, h1);
            (k1, k2) = (a * k1 + k2, k1);
            if k1 > max as f64 || h1 > 1e15 {
                return None;
            }
            if (h1 / k1 - x.abs()).abs() <= 1e-12 * x.abs() {
                let n = h1 as i128;
                return Rat::new(if x < 0.0 { -n } else { n }, k1 as i128);
            }
            let f = y - a;
            if f == 0.0 {
                break;
            }
            y = 1.0 / f;
        }
        None
    }
}

impl fmt::Debug for Rat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.d == 1 { write!(f, "{}", self.n) } else { write!(f, "{}/{}", self.n, self.d) }
    }
}

impl PartialOrd for Rat {
    fn partial_cmp(&self, o: &Rat) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}
impl Ord for Rat {
    fn cmp(&self, o: &Rat) -> Ordering {
        match (self.n.checked_mul(o.d), o.n.checked_mul(self.d)) {
            (Some(a), Some(b)) => a.cmp(&b),
            _ => self.to_f64().total_cmp(&o.to_f64()),
        }
    }
}

/// A number: exact while it can be, a float once it can't.
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Num {
    R(Rat),
    F(f64),
}

impl fmt::Debug for Num {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Num::R(r) => write!(f, "{r:?}"),
            Num::F(x) => write!(f, "{x}f"),
        }
    }
}

impl From<Rat> for Num {
    fn from(r: Rat) -> Num {
        Num::R(r)
    }
}
impl From<f64> for Num {
    fn from(x: f64) -> Num {
        Num::F(x)
    }
}

impl Num {
    pub const ZERO: Num = Num::R(Rat::ZERO);
    pub const ONE: Num = Num::R(Rat::ONE);

    pub fn int(n: i128) -> Num {
        Num::R(Rat::int(n))
    }
    pub fn f(self) -> f64 {
        match self {
            Num::R(r) => r.to_f64(),
            Num::F(x) => x,
        }
    }
    pub fn rat(self) -> Option<Rat> {
        match self {
            Num::R(r) => Some(r),
            Num::F(_) => None,
        }
    }
    pub fn is_exact(self) -> bool {
        matches!(self, Num::R(_))
    }
    pub fn is_zero(self) -> bool {
        self.f() == 0.0
    }
    pub fn is_one(self) -> bool {
        matches!(self, Num::R(r) if r == Rat::ONE) || self.f() == 1.0
    }
    pub fn is_neg(self) -> bool {
        self.f() < 0.0
    }
    pub fn is_int(self) -> bool {
        match self {
            Num::R(r) => r.is_int(),
            Num::F(x) => x.fract() == 0.0,
        }
    }
    pub fn as_int(self) -> Option<i64> {
        match self {
            Num::R(r) if r.is_int() && r.numer().abs() < (1 << 53) => Some(r.numer() as i64),
            Num::F(x) if x.fract() == 0.0 && x.abs() < 9e15 => Some(x as i64),
            _ => None,
        }
    }
    fn op(self, o: Num, r: impl Fn(Rat, Rat) -> Option<Rat>, f: impl Fn(f64, f64) -> f64) -> Num {
        if let (Num::R(a), Num::R(b)) = (self, o) {
            if let Some(c) = r(a, b) {
                return Num::R(c);
            }
        }
        Num::F(f(self.f(), o.f()))
    }
    pub fn add(self, o: Num) -> Num {
        self.op(o, Rat::add, |a, b| a + b)
    }
    pub fn sub(self, o: Num) -> Num {
        self.op(o, Rat::sub, |a, b| a - b)
    }
    pub fn mul(self, o: Num) -> Num {
        self.op(o, Rat::mul, |a, b| a * b)
    }
    /// None for a division by zero.
    pub fn div(self, o: Num) -> Option<Num> {
        if o.is_zero() {
            return None;
        }
        Some(self.op(o, Rat::div, |a, b| a / b))
    }
    pub fn neg(self) -> Num {
        match self {
            Num::R(r) => r.neg().map(Num::R).unwrap_or(Num::F(-r.to_f64())),
            Num::F(x) => Num::F(-x),
        }
    }
    pub fn abs(self) -> Num {
        if self.is_neg() { self.neg() } else { self }
    }
    /// self^e, exactly when it is a rational: 4^(1/2) = 2, 8^(-2/3) = 1/4; None where it isn't real.
    pub fn pow(self, e: Num) -> Option<Num> {
        if let (Num::R(b), Num::R(x)) = (self, e) {
            if x.denom() <= 64 && x.numer().abs() <= 4096 {
                if b.is_zero() && x.is_neg() {
                    return None;
                }
                if let Some(r) = b.root(x.denom() as u32).and_then(|r| r.powi(x.numer() as i64)) {
                    return Some(Num::R(r));
                }
                if b.is_neg() && x.denom() % 2 == 0 {
                    return None;
                }
                if b.is_neg() {
                    // odd roots of negative numbers are real
                    let v = (-b.to_f64()).powf(x.to_f64());
                    return Some(Num::F(if x.numer() % 2 == 0 { v } else { -v }));
                }
            }
        }
        let (b, x) = (self.f(), e.f());
        if b == 0.0 && x < 0.0 {
            return None;
        }
        if b < 0.0 && x.fract() != 0.0 {
            return None;
        }
        let v = if x.fract() == 0.0 && x.abs() < 1e9 { b.powi(x as i32) } else { b.powf(x) };
        Some(Num::F(v))
    }
    pub fn cmp(self, o: Num) -> Ordering {
        match (self, o) {
            (Num::R(a), Num::R(b)) => a.cmp(&b),
            _ => self.f().total_cmp(&o.f()),
        }
    }
    /// Equal, floats allowing for rounding.
    pub fn same(self, o: Num) -> bool {
        match (self, o) {
            (Num::R(a), Num::R(b)) => a == b,
            _ => close(self.f(), o.f()),
        }
    }
}

/// Equal within rounding: 1e-9 relative.
pub fn close(a: f64, b: f64) -> bool {
    a == b || (a.is_finite() && b.is_finite() && (a - b).abs() <= 1e-9 * a.abs().max(b.abs()).max(1e-300))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decimals() {
        assert_eq!(Rat::parse_decimal("7.20"), Rat::new(36, 5));
        assert_eq!(Rat::parse_decimal("0.001"), Rat::new(1, 1000));
        assert_eq!(Rat::parse_decimal("0,5"), Rat::new(1, 2));
        assert_eq!(Rat::parse_decimal(".5"), Rat::new(1, 2));
        assert_eq!(Rat::parse_decimal("1e-3"), None);
        assert_eq!(Rat::parse_decimal("x"), None);
    }

    #[test]
    fn powers() {
        let r = |n, d| Num::R(Rat::new(n, d).unwrap());
        assert_eq!(r(4, 1).pow(r(1, 2)), Some(r(2, 1)));
        assert_eq!(r(8, 1).pow(r(-2, 3)), Some(r(1, 4)));
        assert_eq!(r(-1, 1).pow(r(1, 2)), None);
        assert!(matches!(r(2, 1).pow(r(1, 2)), Some(Num::F(_))));
        assert_eq!(r(-8, 1).pow(r(1, 3)).map(|x| x.f()), Some(-2.0));
    }

    #[test]
    fn approx() {
        assert_eq!(Rat::approx(1.0 / 3.0, 1000), Rat::new(1, 3));
        assert_eq!(Rat::approx(std::f64::consts::PI, 1000), None);
    }
}
