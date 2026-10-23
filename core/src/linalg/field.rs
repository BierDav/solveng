//! What elimination computes with: numbers (exact while they can be, floats once they can't) and symbolic
//! expressions, behind one interface so every algorithm is written once.

use crate::expr::{self, Expr, simp};
use crate::num::Num;
use crate::poly::simplify;

pub trait Field: Clone {
    fn zero() -> Self;
    fn one() -> Self;
    fn add(&self, o: &Self) -> Self;
    fn sub(&self, o: &Self) -> Self;
    fn mul(&self, o: &Self) -> Self;
    /// None for a division by zero.
    fn div(&self, o: &Self) -> Option<Self>;
    fn neg(&self) -> Self;
    /// How well it serves as a pivot, larger is better; None where it is zero (within `tol` for floats).
    fn pivot(&self, tol: f64) -> Option<f64>;
}

impl Field for Num {
    fn zero() -> Num {
        Num::ZERO
    }
    fn one() -> Num {
        Num::ONE
    }
    fn add(&self, o: &Num) -> Num {
        Num::add(*self, *o)
    }
    fn sub(&self, o: &Num) -> Num {
        Num::sub(*self, *o)
    }
    fn mul(&self, o: &Num) -> Num {
        Num::mul(*self, *o)
    }
    fn div(&self, o: &Num) -> Option<Num> {
        Num::div(*self, *o)
    }
    fn neg(&self) -> Num {
        Num::neg(*self)
    }
    /// Partial pivoting: the largest magnitude.
    fn pivot(&self, tol: f64) -> Option<f64> {
        let m = self.f().abs();
        if self.is_zero() || (!self.is_exact() && m <= tol) { None } else { Some(m) }
    }
}

impl Field for Expr {
    fn zero() -> Expr {
        expr::int(0)
    }
    fn one() -> Expr {
        expr::int(1)
    }
    fn add(&self, o: &Expr) -> Expr {
        simplify(&expr::add(vec![self.clone(), o.clone()]))
    }
    fn sub(&self, o: &Expr) -> Expr {
        simplify(&expr::sub(self.clone(), o.clone()))
    }
    fn mul(&self, o: &Expr) -> Expr {
        simp(&expr::mul(vec![self.clone(), o.clone()]))
    }
    fn div(&self, o: &Expr) -> Option<Expr> {
        if o.is_zero() {
            return None;
        }
        Some(simplify(&expr::div(self.clone(), o.clone())))
    }
    fn neg(&self) -> Expr {
        simp(&expr::neg(self.clone()))
    }
    /// The simplest nonzero entry: numbers first, then the smallest expression. Zero is what simplifies to
    /// zero, so entries that are zero only by identities simplify doesn't know (sin² + cos² − 1) count as
    /// nonzero.
    fn pivot(&self, tol: f64) -> Option<f64> {
        if let Some(v) = self.as_c() {
            return v.v.pivot(tol).map(|m| 2.0 + m.min(1.0));
        }
        if self.is_zero() { None } else { Some(1.0 / self.size() as f64) }
    }
}
