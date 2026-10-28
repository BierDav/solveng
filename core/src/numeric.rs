//! Floating complex arithmetic with componentwise absolute forward-error estimates.
//! Estimates describe numerical noise, not interval proofs of correctness.
use std::ops::Deref;
use crate::complex::Z;

#[derive(Clone, Copy, Debug)]
pub(crate) struct N {
    pub z: Z,
    pub error: Z,
}
impl Deref for N { type Target = Z; fn deref(&self) -> &Z { &self.z } }
impl N {
    pub const I: Self = Self { z: Z::I, error: Z { re: 0.0, im: 0.0 } };
    pub fn real(x: f64) -> Self { Self::from(Z::real(x)) }
    pub fn estimated(z: Z, error: Z) -> Self { Self { z, error } }
    // A small safety factor covers several elementary floating operations and
    // library function rounding. No absolute floor: small imaginary signals matter.
    fn rounding(scale: f64) -> f64 { 8.0 * f64::EPSILON * scale }
    pub fn add(self, b: Self) -> Self {
        Self::estimated(self.z.add(b.z), Z::new(
            self.error.re + b.error.re + Self::rounding(self.re.abs() + b.re.abs()),
            self.error.im + b.error.im + Self::rounding(self.im.abs() + b.im.abs())))
    }
    pub fn mul(self, b: Self) -> Self {
        let contribution = |a: f64, b: f64| if a == 0.0 || b == 0.0 { 0.0 } else { a*b };
        let product_error = |a: f64, ae: f64, b: f64, be: f64| contribution(a.abs(),be) + contribution(b.abs(),ae) + contribution(ae,be) + Self::rounding((a*b).abs());
        Self::estimated(self.z.mul(b.z), Z::new(
            product_error(self.re, self.error.re, b.re, b.error.re) + product_error(self.im, self.error.im, b.im, b.error.im),
            product_error(self.re, self.error.re, b.im, b.error.im) + product_error(self.im, self.error.im, b.re, b.error.re)))
    }
    pub fn scale(self, k: f64) -> Self {
        Self::estimated(self.z.scale(k), Z::new(if k == 0.0 { 0.0 } else { self.error.re*k.abs() } + Self::rounding((self.re*k).abs()), if k == 0.0 { 0.0 } else { self.error.im*k.abs() } + Self::rounding((self.im*k).abs())))
    }
    pub fn conj(self) -> Self { Self::estimated(self.z.conj(), self.error) }
    pub fn pow(self, b: Self) -> Self {
        let z = self.z.pow(b.z);
        if self.abs() == 0.0 && self.error.abs() == 0.0 && b.error.abs() == 0.0 { return Self::from(z); }
        let base_derivative = b.z.mul(z).div(self.z);
        let exponent_derivative = z.mul(self.z.ln());
        let a = self.function(z, base_derivative);
        let b = b.function(Z::real(0.0), exponent_derivative);
        Self::estimated(z, Z::new(a.error.re+b.error.re, a.error.im+b.error.im))
    }
    pub fn real_power(self, b: Self, value: f64) -> Self {
        // Real powers retain the real branch, including odd roots of negatives.
        let sensitivity = if self.abs() > 0.0 {
            value.abs() * (b.abs()*self.error.re/self.abs() + self.abs().ln().abs()*b.error.re)
        } else if b.re > 0.0 { self.error.re.powf(b.re) } else if self.error.re == 0.0 && b.error.re == 0.0 { 0.0 } else { f64::INFINITY };
        Self::estimated(Z::real(value), Z::new(sensitivity + Self::rounding(value.abs()), 0.0))
    }
    pub fn function(self, z: Z, derivative: Z) -> Self {
        let propagate = |a: f64, e: f64| if e == 0.0 { 0.0 } else { a.abs()*e };
        Self::estimated(z, Z::new(
            propagate(derivative.re,self.error.re) + propagate(derivative.im,self.error.im) + Self::rounding(z.re.abs()),
            propagate(derivative.im,self.error.re) + propagate(derivative.re,self.error.im) + Self::rounding(z.im.abs())))
    }
    pub fn projected(self, x: f64, error: f64) -> Self { Self::estimated(Z::real(x), Z::new(error, 0.0)) }
    pub fn is_real_result(self) -> bool { self.im == 0.0 || (self.error.im.is_finite() && self.im.abs() <= self.error.im) }
}
impl From<Z> for N { fn from(z: Z) -> Self { Self { z, error: Z::real(0.0) } } }
