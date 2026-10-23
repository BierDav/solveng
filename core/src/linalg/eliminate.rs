//! Dense elimination over any field: determinants, inverses, ranks and linear systems by Gaussian
//! elimination with pivoting (Gauss–Jordan for systems), and fraction-free Bareiss elimination for symbolic
//! determinants.

use super::field::Field;

/// A dense matrix, row by row.
#[derive(Clone, Debug, PartialEq)]
pub struct Mat<F> {
    pub rows: usize,
    pub cols: usize,
    pub a: Vec<F>,
}

impl<F: Field> Mat<F> {
    pub fn new(rows: usize, cols: usize, a: Vec<F>) -> Mat<F> {
        debug_assert_eq!(a.len(), rows * cols);
        Mat { rows, cols, a }
    }
    pub fn identity(n: usize) -> Mat<F> {
        Mat::new(n, n, (0..n * n).map(|k| if k / n == k % n { F::one() } else { F::zero() }).collect())
    }
    pub fn at(&self, i: usize, j: usize) -> &F {
        &self.a[i * self.cols + j]
    }
    fn set(&mut self, i: usize, j: usize, x: F) {
        self.a[i * self.cols + j] = x;
    }
    fn swap_rows(&mut self, i: usize, k: usize) {
        for j in 0..self.cols {
            self.a.swap(i * self.cols + j, k * self.cols + j);
        }
    }
    /// The best pivot in column j at or below row r.
    fn pivot_row(&self, r: usize, j: usize, tol: f64) -> Option<usize> {
        (r..self.rows).filter_map(|i| self.at(i, j).pivot(tol).map(|s| (i, s))).max_by(|a, b| a.1.total_cmp(&b.1)).map(|(i, _)| i)
    }
}

/// Row echelon form of the first `pivot_cols` columns (the others are carried along): the pivot columns and
/// whether the rows were swapped an odd number of times.
fn echelon<F: Field>(m: &mut Mat<F>, pivot_cols: usize, tol: f64) -> (Vec<usize>, bool) {
    let mut pivots = vec![];
    let mut odd = false;
    let mut r = 0;
    for j in 0..pivot_cols {
        if r == m.rows {
            break;
        }
        let Some(p) = m.pivot_row(r, j, tol) else { continue };
        if p != r {
            m.swap_rows(p, r);
            odd = !odd;
        }
        let pv = m.at(r, j).clone();
        for i in r + 1..m.rows {
            if m.at(i, j).pivot(tol).is_none() {
                m.set(i, j, F::zero());
                continue;
            }
            let l = m.at(i, j).div(&pv).expect("a pivot is nonzero");
            m.set(i, j, F::zero());
            for k in j + 1..m.cols {
                let x = m.at(i, k).sub(&l.mul(m.at(r, k)));
                m.set(i, k, x);
            }
        }
        pivots.push(j);
        r += 1;
    }
    (pivots, odd)
}

/// The determinant of a square matrix: the product of the pivots.
pub fn det<F: Field>(mut m: Mat<F>, tol: f64) -> F {
    let n = m.rows;
    let (pivots, odd) = echelon(&mut m, n, tol);
    if pivots.len() < n {
        return F::zero();
    }
    let d = (0..n).fold(F::one(), |d, i| d.mul(m.at(i, i)));
    if odd { d.neg() } else { d }
}

/// The determinant by fraction-free elimination: every intermediate entry is a minor, so each division is
/// exact and symbolic entries stay polynomial.
pub fn det_bareiss<F: Field>(mut m: Mat<F>, tol: f64) -> F {
    let n = m.rows;
    let mut prev = F::one();
    let mut odd = false;
    for k in 0..n {
        let Some(p) = m.pivot_row(k, k, tol) else { return F::zero() };
        if p != k {
            m.swap_rows(p, k);
            odd = !odd;
        }
        for i in k + 1..n {
            for j in k + 1..n {
                let x = m.at(k, k).mul(m.at(i, j)).sub(&m.at(i, k).mul(m.at(k, j)));
                m.set(i, j, x.div(&prev).expect("the previous pivot is nonzero"));
            }
            m.set(i, k, F::zero());
        }
        prev = m.at(k, k).clone();
    }
    if odd { prev.neg() } else { prev }
}

/// The determinant by expanding the permutations (small symbolic matrices, where it reads best).
pub fn det_expand<F: Field>(m: &Mat<F>) -> F {
    let n = m.rows;
    if n == 0 {
        return F::one();
    }
    if n == 1 {
        return m.at(0, 0).clone();
    }
    let mut d = F::zero();
    for j in 0..n {
        let minor = Mat::new(n - 1, n - 1, (1..n).flat_map(|i| (0..n).filter(move |&k| k != j).map(move |k| (i, k))).map(|(i, k)| m.at(i, k).clone()).collect());
        let t = m.at(0, j).mul(&det_expand(&minor));
        d = if j % 2 == 0 { d.add(&t) } else { d.sub(&t) };
    }
    d
}

pub fn rank<F: Field>(mut m: Mat<F>, tol: f64) -> usize {
    let c = m.cols;
    echelon(&mut m, c, tol).0.len()
}

/// A⁻¹·B by Gauss–Jordan elimination of [A | B]; None if A is singular.
pub fn solve<F: Field>(a: &Mat<F>, b: &Mat<F>, tol: f64) -> Option<Mat<F>> {
    let (n, k) = (a.rows, b.cols);
    let w = n + k;
    let mut m = Mat::new(n, w, (0..n).flat_map(|i| (0..n).map(move |j| (i, j, true)).chain((0..k).map(move |j| (i, j, false)))).map(|(i, j, left)| if left { a.at(i, j).clone() } else { b.at(i, j).clone() }).collect());
    let (pivots, _) = echelon(&mut m, n, tol);
    if pivots.len() < n {
        return None;
    }
    // back substitution, last row first
    for r in (0..n).rev() {
        let pv = m.at(r, r).clone();
        for j in n..w {
            let x = m.at(r, j).div(&pv)?;
            m.set(r, j, x);
        }
        m.set(r, r, F::one());
        for i in 0..r {
            let l = m.at(i, r).clone();
            if l.pivot(tol).is_none() {
                continue;
            }
            for j in n..w {
                let x = m.at(i, j).sub(&l.mul(m.at(r, j)));
                m.set(i, j, x);
            }
            m.set(i, r, F::zero());
        }
    }
    Some(Mat::new(n, k, (0..n).flat_map(|i| (n..w).map(move |j| (i, j))).map(|(i, j)| m.at(i, j).clone()).collect()))
}

/// The cofactor matrix transposed: A⁻¹ = adj(A)/det(A) without dividing inside (small symbolic matrices).
pub fn adjugate<F: Field>(m: &Mat<F>) -> Mat<F> {
    let n = m.rows;
    if n == 1 {
        return Mat::new(1, 1, vec![F::one()]);
    }
    let mut out = vec![F::zero(); n * n];
    for i in 0..n {
        for j in 0..n {
            let minor = Mat::new(n - 1, n - 1, (0..n).filter(|&r| r != i).flat_map(|r| (0..n).filter(move |&c| c != j).map(move |c| (r, c))).map(|(r, c)| m.at(r, c).clone()).collect());
            let c = det_expand(&minor);
            out[j * n + i] = if (i + j) % 2 == 0 { c } else { c.neg() };
        }
    }
    Mat::new(n, n, out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::num::{Num, Rat};

    fn m(rows: usize, v: &[i128]) -> Mat<Num> {
        Mat::new(rows, v.len() / rows, v.iter().map(|&x| Num::int(x)).collect())
    }

    #[test]
    fn determinants() {
        let a = m(3, &[2, -3, 1, 2, 0, -1, 1, 4, 5]);
        assert_eq!(det(a.clone(), 0.0), Num::int(49));
        assert_eq!(det_bareiss(a.clone(), 0.0), Num::int(49));
        assert_eq!(det_expand(&a), Num::int(49));
        // a zero first pivot needs a row swap
        assert_eq!(det(m(2, &[0, 1, 1, 0]), 0.0), Num::int(-1));
        assert_eq!(det_bareiss(m(2, &[0, 1, 1, 0]), 0.0), Num::int(-1));
        assert_eq!(det(m(2, &[1, 2, 2, 4]), 0.0), Num::ZERO);
    }

    #[test]
    fn inverse_and_rank() {
        let a = m(2, &[4, 7, 2, 6]);
        let inv = solve(&a, &Mat::identity(2), 0.0).unwrap();
        let r = |n, d| Num::R(Rat::new(n, d).unwrap());
        assert_eq!(inv.a, vec![r(3, 5), r(-7, 10), r(-1, 5), r(2, 5)]);
        assert_eq!(adjugate(&a).a, m(2, &[6, -7, -2, 4]).a);
        assert!(solve(&m(2, &[1, 2, 2, 4]), &Mat::identity(2), 0.0).is_none());
        assert_eq!(rank(m(3, &[1, 2, 3, 2, 4, 6, 1, 0, 1]), 0.0), 2);
    }
}
