//! Dimensions: powers of base dimensions, by identity: the SI ones and those of `base-unit` bindings.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::num::Rat;

/// Canonical identities of the SI dimensions.
pub const BASE: [&str; 7] = ["m", "kg", "s", "A", "K", "mol", "cd"];
pub const IDENTITIES: [&str; 7] = ["length", "mass", "time", "electric-current", "temperature", "amount-of-substance", "luminous-intensity"];
const S: i64 = 60;

#[derive(Clone, PartialEq, Eq, Hash, Default, Debug, Serialize, Deserialize)]
pub struct Dim(pub BTreeMap<String, i64>);

impl Dim {
    pub const ONE: Dim = Dim(BTreeMap::new());

    pub fn named(identity: &str) -> Dim {
        Dim(BTreeMap::from([(identity.to_owned(), S)]))
    }
    /// The i-th SI base dimension, in the order of [`BASE`].
    pub fn base(i: usize) -> Dim {
        Self::named(IDENTITIES[i])
    }
    pub fn si(e: [i32; 7]) -> Dim {
        Dim(IDENTITIES.into_iter().zip(e).filter(|(_, e)| *e != 0).map(|(k, e)| (k.into(), i64::from(e) * S)).collect())
    }
    /// Public expression powers are bounded to this many sixtieths. Wider
    /// storage lets intermediate products be diagnosed before narrowing.
    pub fn supported(&self) -> bool { self.0.values().all(|e| e.unsigned_abs() <= 1 << 30) }
    pub fn is_one(&self) -> bool { self.0.is_empty() }
    pub fn mul(&self, o: &Dim) -> Dim {
        let mut d = self.0.clone();
        for (k, e) in &o.0 { *d.entry(k.clone()).or_default() += e; }
        d.retain(|_, e| *e != 0);
        Dim(d)
    }
    pub fn div(&self, o: &Dim) -> Dim { self.mul(&o.powi(-1)) }
    pub fn pow(&self, p: Rat) -> Option<Dim> {
        let mut d = BTreeMap::new();
        for (k, e) in &self.0 {
            let x = Rat::int(*e as i128).mul(p)?;
            if !x.is_int() || x.numer().abs() > 1 << 30 { return None; }
            if !x.is_zero() { d.insert(k.clone(), x.numer() as i64); }
        }
        Some(Dim(d))
    }
    pub fn powi(&self, k: i32) -> Dim {
        if k == 0 { return Dim::ONE; }
        Dim(self.0.iter().map(|(n, e)| (n.clone(), e * i64::from(k))).collect())
    }
    pub fn ratio(&self, o: &Dim) -> Option<Rat> {
        let (n, e) = o.0.first_key_value()?;
        let k = Rat::new(*self.0.get(n).unwrap_or(&0) as i128, *e as i128)?;
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
