//! Dimensions: powers of the SI base units and of base units of your own.

use serde::{Deserialize, Serialize};

use crate::num::Rat;

/// SI base units, then slots for base units of your own (`units: (ü: auto)`).
pub const BASE: [&str; 7] = ["m", "kg", "s", "A", "K", "mol", "cd"];
pub const OWN: usize = 8;
pub const N: usize = BASE.len() + OWN;
/// Exponents are kept as multiples of 1/60, so square, cube, … sixth roots stay exact.
const S: i32 = 60;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Default, Debug, Serialize, Deserialize)]
pub struct Dim(pub [i32; N]);

impl Dim {
    pub const ONE: Dim = Dim([0; N]);

    pub fn base(i: usize) -> Dim {
        let mut d = [0; N];
        d[i] = S;
        Dim(d)
    }
    /// From whole exponents of m, kg, s, A, K, mol, cd.
    pub fn si(e: [i32; 7]) -> Dim {
        let mut d = [0; N];
        for (i, x) in e.iter().enumerate() {
            d[i] = x * S;
        }
        Dim(d)
    }
    pub fn is_one(&self) -> bool {
        self.0.iter().all(|&x| x == 0)
    }
    pub fn mul(&self, o: &Dim) -> Dim {
        let mut d = self.0;
        for i in 0..N {
            d[i] += o.0[i];
        }
        Dim(d)
    }
    pub fn div(&self, o: &Dim) -> Dim {
        let mut d = self.0;
        for i in 0..N {
            d[i] -= o.0[i];
        }
        Dim(d)
    }
    /// self^p, or None if an exponent would leave the 1/60 grid (m^(1/7)).
    pub fn pow(&self, p: Rat) -> Option<Dim> {
        let mut d = [0; N];
        for i in 0..N {
            let x = Rat::int(self.0[i] as i128).mul(p)?;
            if !x.is_int() || x.numer().abs() > 1 << 30 {
                return None;
            }
            d[i] = x.numer() as i32;
        }
        Some(Dim(d))
    }
    pub fn powi(&self, k: i32) -> Dim {
        let mut d = self.0;
        for x in d.iter_mut() {
            *x *= k;
        }
        Dim(d)
    }
    /// The exponent of base unit i.
    pub fn exp(&self, i: usize) -> Rat {
        Rat::new(self.0[i] as i128, S as i128).unwrap()
    }
    /// Whether self is a whole multiple k of o: m^2 of m is 2.
    pub fn ratio(&self, o: &Dim) -> Option<Rat> {
        let i = (0..N).find(|&i| o.0[i] != 0)?;
        let k = Rat::new(self.0[i] as i128, o.0[i] as i128)?;
        (o.pow(k)? == *self).then_some(k)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roots() {
        let m2 = Dim::si([2, 0, 0, 0, 0, 0, 0]);
        let half = Rat::new(1, 2).unwrap();
        assert_eq!(m2.pow(half), Some(Dim::si([1, 0, 0, 0, 0, 0, 0])));
        assert_eq!(m2.ratio(&Dim::base(0)), Some(Rat::int(2)));
        assert!(Dim::base(0).pow(Rat::new(1, 7).unwrap()).is_none());
    }
}
