//! Vectors and matrices of expressions: sizes, sums and products, and the operations of `Expr::Op` (det,
//! transpose, lsolve, …). Matrices of numbers are computed exactly where the arithmetic allows, with their
//! units split into a unit per row and per column; matrices with names in them symbolically. The
//! algorithms are in `eliminate`, written once over the `field` interface.
//!
//! Names without a value are scalars: they commute in products, and an operation on one waits until it
//! has a value.

mod eliminate;
mod field;

use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::dim::Dim;
use crate::expr::{self, Expr, Func, Val, int, rat, simp};
use crate::num::Num;
use crate::poly::simplify;
use eliminate::Mat;

/// An operation on arrays or multiple scalars, kept as written until its arguments are known. The names are
/// Typst's: det, tr, norm; A^T, A^top and a times b are written as operators.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Op {
    /// A^T, A^top.
    Transpose,
    Det,
    /// tr(A): the sum of the diagonal.
    Trace,
    Rank,
    /// norm(v), ‖v‖: the length of a vector, the Frobenius norm of a matrix.
    Norm,
    /// a times b: the cross product of vectors of three components.
    Cross,
    Identity,
    Rows,
    Cols,
    /// at(v, i), at(A, i, j): an element, counted from 0.
    At,
    /// A[i, :]: row i as a matrix of one row.
    Row,
    /// A[:, j]: column j as a vector.
    Col,
    /// convolve(x, h): the discrete convolution of two sequences (vectors), of length n + m − 1.
    Convolve,
    Atan2,
    Min,
    Max,
    Mean,
    Lsolve,
    Eigenvals,
    SingularValues,
    Condition,
    Median,
    Variance,
    Stdev,
    NormalCdf,
    NormalQuantile,
    Interpolate,
    LinearFit,
    Dft,
    Idft,
    Gamma,
    Erf,
    BesselJ,
    Celsius,
    Fahrenheit,
    Polyroots,
}

/// The functions on matrices, by name.
pub const FUNCTIONS: &[&str] = &["det", "tr", "rank", "norm", "identity", "rows", "cols", "lsolve", "eigenvals", "singularvals", "cond"];

impl Op {
    /// The operation called by a name (A^T and a times b are operators).
    pub fn of(n: &str) -> Option<Op> {
        Some(match n {
            "lsolve" => Op::Lsolve,
            "eigenvals" => Op::Eigenvals,
            "singularvals" => Op::SingularValues,
            "cond" => Op::Condition,
            "median" => Op::Median,
            "variance" => Op::Variance,
            "stdev" => Op::Stdev,
            "pnorm" => Op::NormalCdf,
            "qnorm" => Op::NormalQuantile,
            "linterp" => Op::Interpolate,
            "linfit" => Op::LinearFit,
            "dft" => Op::Dft,
            "idft" => Op::Idft,
            "gamma" => Op::Gamma,
            "erf" => Op::Erf,
            "besselj" => Op::BesselJ,
            "celsius" => Op::Celsius,
            "fahrenheit" => Op::Fahrenheit,
            "polyroots" => Op::Polyroots,
            "atan2" => Op::Atan2,
            "min" => Op::Min,
            "max" => Op::Max,
            "mean" => Op::Mean,
            "det" => Op::Det,
            "tr" => Op::Trace,
            "rank" => Op::Rank,
            "norm" => Op::Norm,
            "identity" => Op::Identity,
            "rows" => Op::Rows,
            "cols" => Op::Cols,
            "at" => Op::At,
            _ => return None,
        })
    }
    pub fn name(self) -> &'static str {
        match self {
            Op::Transpose => "transpose",
            Op::Det => "det",
            Op::Trace => "tr",
            Op::Rank => "rank",
            Op::Norm => "norm",
            Op::Cross => "cross",
            Op::Identity => "identity",
            Op::Rows => "rows",
            Op::Cols => "cols",
            Op::At => "at",
            Op::Row => "row",
            Op::Col => "column",
            Op::Convolve => "convolve",
            Op::Atan2 => "atan2",
            Op::Min => "min",
            Op::Max => "max",
            Op::Mean => "mean",
            Op::Lsolve => "lsolve",
            Op::Eigenvals => "eigenvals",
            Op::SingularValues => "singularvals",
            Op::Condition => "cond",
            Op::Median => "median",
            Op::Variance => "variance",
            Op::Stdev => "stdev",
            Op::NormalCdf => "pnorm",
            Op::NormalQuantile => "qnorm",
            Op::Interpolate => "linterp",
            Op::LinearFit => "linfit",
            Op::Dft => "dft",
            Op::Idft => "idft",
            Op::Gamma => "gamma",
            Op::Erf => "erf",
            Op::BesselJ => "besselj",
            Op::Celsius => "celsius",
            Op::Fahrenheit => "fahrenheit",
            Op::Polyroots => "polyroots",

        }
    }
    /// What it takes, for a call with the wrong number of arguments.
    pub fn usage(self) -> String {
        match self {
            Op::Atan2 => "atan2 takes two arguments: atan2(y, x)".into(),
            Op::Min | Op::Max => format!("{} takes one or more arguments", self.name()),
            Op::At => "at takes a vector and an index, or a matrix, a row and a column, starting at 0".into(),
            _ => format!("{} takes one argument", self.name()),
        }
    }
    /// How many arguments it takes.
    pub fn arity(self) -> RangeInclusive<usize> {
        match self {
            Op::Lsolve | Op::LinearFit | Op::BesselJ => 2..=2,
            Op::NormalCdf | Op::NormalQuantile | Op::Interpolate => 3..=3,
            Op::Cross | Op::Convolve | Op::Row | Op::Col | Op::Atan2 => 2..=2,
            Op::Min | Op::Max => 1..=usize::MAX,
            Op::At => 2..=3,
            _ => 1..=1,
        }
    }
}

/// A vector or matrix, row by row. A vector is a matrix of one column.
#[derive(Clone, Debug, PartialEq)]
pub struct Array {
    pub rows: usize,
    pub cols: usize,
    pub a: Vec<Expr>,
}

impl Array {
    pub fn of(e: &Expr) -> Option<Array> {
        match e {
            Expr::Vector(v) => Some(Array { rows: v.len(), cols: 1, a: v.clone() }),
            Expr::Matrix(rs) if is_array(e) => Some(Array { rows: rs.len(), cols: rs[0].len(), a: rs.concat() }),
            _ => None,
        }
    }
    fn identity(n: usize) -> Array {
        Array { rows: n, cols: n, a: (0..n * n).map(|k| int((k / n == k % n) as i128)).collect() }
    }
    pub fn at(&self, i: usize, j: usize) -> &Expr {
        &self.a[i * self.cols + j]
    }
    /// As an expression: one column is a vector.
    pub fn expr(self) -> Expr {
        if self.cols == 1 { Expr::Vector(self.a) } else { matrix(self.a.chunks(self.cols).map(<[Expr]>::to_vec).collect()) }
    }
    fn square(&self) -> bool {
        self.rows == self.cols
    }
    pub fn vector(&self) -> bool {
        self.cols == 1
    }
    /// For messages: "a vector of 3 components", "a 2×3 matrix".
    pub fn describe(&self) -> String {
        if self.vector() { format!("a vector of {} components", self.rows) } else { format!("a {}×{} matrix", self.rows, self.cols) }
    }
    fn map(&self, f: impl FnMut(&Expr) -> Expr) -> Array {
        Array { a: self.a.iter().map(f).collect(), ..*self }
    }
    fn transpose(&self) -> Array {
        Array { rows: self.cols, cols: self.rows, a: (0..self.cols).flat_map(|j| (0..self.rows).map(move |i| (i, j))).map(|(i, j)| self.at(i, j).clone()).collect() }
    }
    fn mat(&self) -> Mat<Expr> {
        Mat::new(self.rows, self.cols, self.a.clone())
    }
}

/// A matrix from its rows; one column is a vector. Elements that are matrices are blocks: mat(A, b) puts
/// them side by side, mat(A; B) one below the other. Unevaluated where the blocks don't fit.
pub fn matrix(rows: Vec<Vec<Expr>>) -> Expr {
    if rows.iter().flatten().any(is_array) || rows.iter().any(|r| r.len() != rows[0].len()) {
        return blocks(&rows).map(Array::expr).unwrap_or(Expr::Matrix(rows));
    }
    if rows.first().is_some_and(|r| r.len() == 1) { Expr::Vector(rows.into_iter().flatten().collect()) } else { Expr::Matrix(rows) }
}

/// A block matrix put together: in each row of blocks the same number of rows, and the same number of
/// columns in every row of blocks. A scalar is a 1×1 block.
fn blocks(rows: &[Vec<Expr>]) -> Result<Array, String> {
    let mut out: Option<Array> = None;
    for r in rows {
        let parts: Vec<Array> = r.iter().map(|e| Array::of(e).unwrap_or(Array { rows: 1, cols: 1, a: vec![e.clone()] })).collect();
        let n = parts[0].rows;
        if let Some(p) = parts.iter().find(|p| p.rows != n) {
            return Err(format!("mat: blocks side by side need the same number of rows, not {} and {}", n, p.rows));
        }
        let cols = parts.iter().map(|p| p.cols).sum();
        let a = (0..n).flat_map(|i| parts.iter().flat_map(move |p| (0..p.cols).map(move |j| p.at(i, j).clone()))).collect::<Vec<_>>();
        out = Some(match out {
            None => Array { rows: n, cols, a },
            Some(m) if m.cols == cols => Array { rows: m.rows + n, cols, a: [m.a, a].concat() },
            Some(_) if !rows.iter().flatten().any(is_array) => return Err("mat: every row needs the same number of elements".into()),
            Some(m) => return Err(format!("mat: blocks one below the other need the same number of columns, not {} and {cols}", m.cols)),
        });
    }
    out.ok_or_else(|| "mat needs elements".into())
}

/// The factors of a product, simplified, with A^(-1) b solved as a system instead of inverting A.
pub fn factors(v: &[Expr]) -> Vec<Expr> {
    let mut out = vec![];
    let mut i = 0;
    while i < v.len() {
        let f = simp(&v[i]);
        if let (Expr::Pow(b, x), Some(next)) = (&f, v.get(i + 1)) {
            if x.as_num().is_some_and(|k| k.as_int() == Some(-1)) {
                let rhs = simp(next);
                if let (Some(a), Some(b)) = (Array::of(b), Array::of(&rhs)) {
                    if a.square() && b.rows == a.rows {
                        if let Ok(x) = lsolve(&a, &b) {
                            out.push(x.expr());
                            i += 2;
                            continue;
                        }
                    }
                }
            }
        }
        out.push(f);
        i += 1;
    }
    out
}

/// A vector, or a matrix that is put together (not one with rows of different lengths or blocks in it).
pub fn is_array(e: &Expr) -> bool {
    match e {
        Expr::Vector(_) => true,
        Expr::Matrix(rs) => !rs.iter().any(|r| r.len() != rs[0].len()) && !rs.iter().flatten().any(is_array),
        _ => false,
    }
}

/// Surely a scalar: no array, and no name that may still turn out to be one.
fn scalar(e: &Expr) -> bool {
    !e.contains(&|x| matches!(x, Expr::Vector(_) | Expr::Matrix(_) | Expr::Sym(_) | Expr::Call(..) | Expr::Op(..)))
}

const SINGULAR: &str = "the matrix is singular (its determinant is 0), so it has no inverse";

// ---------- arithmetic ----------

/// A sum with arrays in it: elementwise, scalars added to every element. Unevaluated where the sizes differ.
pub fn add(terms: Vec<Expr>) -> Expr {
    sum(&terms).unwrap_or(Expr::Add(terms))
}

fn sum(terms: &[Expr]) -> Result<Expr, String> {
    let mut acc: Option<Array> = None;
    let mut scalars = vec![];
    for t in terms {
        let Some(b) = Array::of(t) else {
            scalars.push(t.clone());
            continue;
        };
        acc = Some(match acc {
            None => b,
            Some(a) if (a.rows, a.cols) == (b.rows, b.cols) => Array { a: a.a.iter().zip(&b.a).map(|(x, y)| expr::add(vec![x.clone(), y.clone()])).collect(), ..a },
            Some(a) => return Err(format!("adding {} and {}: their sizes must match", a.describe(), b.describe())),
        });
    }
    let a = acc.expect("a sum with an array in it");
    let s = Expr::Add(scalars);
    Ok(a.map(|x| simp(&expr::add(vec![x.clone(), s.clone()]))).expr())
}

/// A product with arrays in it: the arrays multiplied in their order (matrix products, the dot product of
/// two vectors), the scalars into every element. Unevaluated where the sizes don't fit.
pub fn mul(factors: Vec<Expr>) -> Expr {
    product(&factors).unwrap_or(Expr::Mul(factors))
}

fn product(fs: &[Expr]) -> Result<Expr, String> {
    let mut acc: Option<Array> = None;
    let mut scalars = vec![];
    for f in fs {
        let Some(b) = Array::of(f) else {
            scalars.push(f.clone());
            continue;
        };
        acc = match acc {
            None => Some(b),
            Some(a) => match times(&a, &b)? {
                Ok(m) => Some(m),
                Err(s) => {
                    scalars.push(s);
                    None
                }
            },
        };
    }
    let s = simp(&Expr::Mul(scalars));
    Ok(match acc {
        None => s,
        Some(a) => a.map(|x| simp(&expr::mul(vec![s.clone(), x.clone()]))).expr(),
    })
}

/// a·b: the matrix product, or a scalar (the dot product of two vectors, a 1×1 product).
fn times(a: &Array, b: &Array) -> Result<Result<Array, Expr>, String> {
    if a.vector() && b.vector() && a.rows == b.rows && a.rows > 1 {
        return Ok(Err(simp(&Expr::Add(a.a.iter().zip(&b.a).map(|(x, y)| expr::mul(vec![x.clone(), y.clone()])).collect()))));
    }
    if a.vector() && b.vector() && a.rows > 1 && b.rows > 1 {
        return Err(format!("the dot product takes two vectors of the same length, not {} and {}", a.describe(), b.describe()));
    }
    if a.cols != b.rows {
        return Err(format!("multiplying {} by {}: the columns of the first must match the rows of the second", a.describe(), b.describe()));
    }
    let entry = |i: usize, j: usize| simp(&Expr::Add((0..a.cols).map(|k| expr::mul(vec![a.at(i, k).clone(), b.at(k, j).clone()])).collect()));
    if a.rows == 1 && b.cols == 1 {
        return Ok(Err(entry(0, 0)));
    }
    Ok(Ok(Array { rows: a.rows, cols: b.cols, a: (0..a.rows).flat_map(|i| (0..b.cols).map(move |j| (i, j))).map(|(i, j)| entry(i, j)).collect() }))
}

/// An array to a power: whole powers of a square matrix, negative ones of its inverse. Unevaluated otherwise.
pub fn pow(b: Expr, x: Expr) -> Expr {
    power(&b, &x).unwrap_or_else(|_| expr::pow(b, x))
}

fn power(b: &Expr, x: &Expr) -> Result<Expr, String> {
    let a = Array::of(b).expect("a power of an array");
    if !a.square() {
        return Err(format!("{} has no powers – only a square matrix does (A^T transposes)", a.describe()));
    }
    let Some(k) = x.as_num().and_then(Num::as_int) else { return Err("a matrix power must be a whole number".into()) };
    let mut base = if k < 0 { inverse(&a)? } else { a.clone() };
    let mut r = Array::identity(a.rows);
    let mut k = k.unsigned_abs();
    while k > 0 {
        if k & 1 == 1 {
            r = times(&r, &base)?.map_err(|_| "a matrix power".to_string())?;
        }
        k >>= 1;
        if k > 0 {
            base = times(&base, &base)?.map_err(|_| "a matrix power".to_string())?;
        }
    }
    Ok(r.expr())
}

/// |v| is the length of a vector, |A| the determinant of a square matrix; no other function takes an array.
pub fn func(f: Func, a: Expr) -> Expr {
    function(f, &a).unwrap_or_else(|_| expr::func(f, a))
}

fn function(f: Func, a: &Expr) -> Result<Expr, String> {
    let m = Array::of(a).expect("a function of an array");
    match f {
        Func::Abs if m.vector() => Ok(simp(&expr::sqrt(Expr::Add(m.a.iter().map(|x| expr::pow(expr::func(Func::Abs, x.clone()), int(2))).collect())))),
        Func::Abs if m.square() => det(&m),
        Func::Abs => Err(format!("|…| of {}: it's the length of a vector or the determinant of a square matrix", m.describe())),
        _ => Err(format!("{} takes a number, not {}", f.name(), m.describe())),
    }
}

// ---------- operations ----------

/// An operation evaluated where its arguments allow; as it is while a name may still be a matrix.
pub fn apply(op: Op, args: Vec<Expr>) -> Expr {
    match eval(op, &args) {
        Ok(Some(e)) => e,
        _ => Expr::Op(op, args),
    }
}

/// The result of an operation: None while it waits for names, an error where it can't be done.
fn eval(op: Op, args: &[Expr]) -> Result<Option<Expr>, String> {
    let waits = |e: &Expr| !is_array(e) && !scalar(e);
    if !op.arity().contains(&args.len()) {
        return Err(op.usage());
    }
    if op == Op::Norm {
        if let Some(a) = Array::of(&args[0]) {
            let values = a.a.iter().map(crate::eval::complex_value).collect::<Result<Vec<_>, _>>();
            if let Ok(values) = values {
                if values.iter().any(|v| !v.re.v.is_exact() || !v.im.v.is_exact() || v.z().abs() > 1e150 || (v.z().abs() > 0.0 && v.z().abs() < 1e-150)) {
                    let d = values.iter().find(|v| v.z().abs() != 0.0).map(|v| v.d().clone()).unwrap_or(crate::dim::Dim::ONE);
                    if values.iter().any(|v| *v.d() != d && !(v.z().abs() == 0.0 && v.d().is_one())) { return Err("norm: entries must have matching units".into()); }
                    let norm = values.iter().fold(0.0_f64, |acc, v| acc.hypot(v.re.v.f()).hypot(v.im.v.f()));
                    if !norm.is_finite() { return Err("norm: result is outside finite range".into()); }
                    return Ok(Some(Expr::C(Val::new(crate::num::Num::F(norm), d))));
                }
            }
        }
    }
    if matches!(op, Op::Lsolve | Op::Eigenvals | Op::SingularValues | Op::Condition | Op::Median | Op::Variance | Op::Stdev | Op::NormalCdf | Op::NormalQuantile | Op::Interpolate | Op::LinearFit | Op::Dft | Op::Idft | Op::Gamma | Op::Erf | Op::BesselJ | Op::Celsius | Op::Fahrenheit | Op::Polyroots) {
        if args.iter().any(|e| !e.syms().is_empty()) { return Ok(None); }
        return crate::numerics::operation(op, args).map(Some);
    }
    if matches!(op, Op::Atan2 | Op::Min | Op::Max | Op::Mean) {
        if args.iter().any(|e| !e.syms().is_empty()) { return Ok(None); }
        let reduction;
        let entries = if matches!(op, Op::Min | Op::Max) && args.len() == 1 && Array::of(&args[0]).is_some() {
            reduction = Array::of(&args[0]).filter(Array::vector).ok_or("extrema require a vector")?.a;
            if reduction.is_empty() { return Err("extrema require a nonempty vector".into()); }
            &reduction
        } else if op == Op::Mean {
            let a = Array::of(&args[0]).filter(Array::vector).ok_or("mean takes a vector")?;
            if a.a.is_empty() { return Err("mean takes a nonempty vector".into()); }
            return Ok(Some(simp(&expr::div(expr::add(a.a.clone()), int(a.a.len() as i128)))));
        } else { args };
        let values = entries.iter().map(|e| crate::eval::value(e).map_err(|e| e.message())).collect::<Result<Vec<_>, _>>()?;
        let d = &values[0].d;
        if values.iter().any(|v| v.d != *d) { return Err(format!("{}: arguments must have matching units", op.name())); }
        if op == Op::Atan2 {
            if values[0].v.is_zero() && values[1].v.is_zero() {
                return Err("atan2 is undefined at (0, 0)".into());
            }
            if let (Some(yr), Some(xr)) = (values[0].v.rat(), values[1].v.rat()) {
                if yr.is_zero() {
                    return Ok(Some(if !xr.is_neg() { int(0) } else { Expr::Pi }));
                }
                if xr.is_zero() {
                    return Ok(Some(simp(&expr::mul(vec![rat(if !yr.is_neg() { 1 } else { -1 }, 2), Expr::Pi]))));
                }
                if yr.abs() == xr.abs() {
                    let (y_pos, x_pos) = (!yr.is_neg(), !xr.is_neg());
                    let q = match (y_pos, x_pos) {
                        (true, true) => rat(1, 4),
                        (true, false) => rat(3, 4),
                        (false, true) => rat(-1, 4),
                        (false, false) => rat(-3, 4),
                    };
                    return Ok(Some(simp(&expr::mul(vec![q, Expr::Pi]))));
                }
            }
            let (y, x) = (values[0].v.f(), values[1].v.f());
            return Ok(Some(expr::c(Num::F(y.atan2(x)))));
        }
        let selected = values.iter().reduce(|a, b| {
            if (op == Op::Min && b.v.cmp(a.v).is_lt()) || (op == Op::Max && b.v.cmp(a.v).is_gt()) { b } else { a }
        }).unwrap();
        return Ok(Some(Expr::C(selected.clone())));
    }
    if op == Op::Cross {
        // × of scalars is the product
        if scalar(&args[0]) || scalar(&args[1]) {
            return Ok(Some(simp(&expr::mul(args.to_vec()))));
        }
        if args.iter().any(waits) {
            return Ok(None);
        }
        let (a, b) = (Array::of(&args[0]).unwrap(), Array::of(&args[1]).unwrap());
        if !(a.vector() && b.vector() && a.rows == 3 && b.rows == 3) {
            return Err(format!("the cross product takes two vectors of 3 components, not {} and {}", a.describe(), b.describe()));
        }
        let c = |i: usize, j: usize| simp(&expr::sub(expr::mul(vec![a.a[i].clone(), b.a[j].clone()]), expr::mul(vec![a.a[j].clone(), b.a[i].clone()])));
        return Ok(Some(Expr::Vector(vec![c(1, 2), c(2, 0), c(0, 1)])));
    }
    if args.iter().any(waits) {
        return Ok(None);
    }
    // x[i] with a vector of indices (a range variable): the elements at each
    if op == Op::At && args.len() >= 2 {
        if let Some(k) = (1..args.len()).find(|&k| Array::of(&args[k]).is_some_and(|a| a.vector())) {
            let idx = Array::of(&args[k]).unwrap();
            let mut out = vec![];
            for i in idx.a {
                let mut a = args.to_vec();
                a[k] = i;
                match eval(op, &a)? {
                    Some(e) => out.push(e),
                    None => return Ok(None),
                }
            }
            return Ok(Some(Expr::Vector(out)));
        }
    }
    if op == Op::Convolve {
        let seq = |e: &Expr| -> Result<Vec<Expr>, String> {
            match Array::of(e) {
                Some(a) if a.vector() => Ok(a.a),
                Some(a) => Err(format!("convolve takes two sequences (vectors), not {}", a.describe())),
                None => Ok(vec![e.clone()]),
            }
        };
        let (x, h) = (seq(&args[0])?, seq(&args[1])?);
        if x.is_empty() || h.is_empty() {
            return Err("convolve: a sequence without elements".into());
        }
        if x.len().saturating_mul(h.len()) > 1_000_000 {
            return Err("convolve: work limit exceeded (at most 1000000 coefficient products)".into());
        }
        let n = x.len() + h.len() - 1;
        // sequences of values (the usual case, sampled signals): computed with their numbers directly
        let values = |v: &[Expr]| -> Option<(Vec<Num>, Dim)> {
            let d = v.first()?.as_c()?.d.clone();
            v.iter().map(|e| e.as_c().filter(|c| c.d == d).map(|c| c.v)).collect::<Option<Vec<_>>>().map(|ns| (ns, d))
        };
        if let (Some((xs, dx)), Some((hs, dh))) = (values(&x), values(&h)) {
            let d = dx.mul(&dh);
            // whole numbers exactly; other samples (measured, sampled signals) as floats, which a sum of
            // products of many fractions would only turn into anyway, slowly
            let whole = xs.iter().chain(&hs).all(|v| v.as_int().is_some_and(|i| i.abs() < 1 << 40));
            let (xf, hf): (Vec<f64>, Vec<f64>) = (xs.iter().map(|v| v.f()).collect(), hs.iter().map(|v| v.f()).collect());
            let out = (0..n)
                .map(|k| {
                    let lo = k.saturating_sub(hs.len() - 1);
                    let hi = k.min(xs.len() - 1);
                    let v = if whole {
                        (lo..=hi).fold(Num::ZERO, |acc, i| acc.add(xs[i].mul(hs[k - i])))
                    } else {
                        Num::F((lo..=hi).map(|i| xf[i] * hf[k - i]).sum())
                    };
                    Expr::C(Val::new(v, d.clone()))
                })
                .collect();
            return Ok(Some(Expr::Vector(out)));
        }
        let out = (0..n)
            .map(|k| {
                let lo = k.saturating_sub(h.len() - 1);
                let hi = k.min(x.len() - 1);
                simp(&Expr::Add((lo..=hi).map(|i| expr::mul(vec![x[i].clone(), h[k - i].clone()])).collect()))
            })
            .collect();
        return Ok(Some(Expr::Vector(out)));
    }
    let arrays: Vec<Option<Array>> = args.iter().map(Array::of).collect();
    let need = |i: usize| arrays[i].clone().ok_or_else(|| format!("{} takes a vector or matrix", op.name()));
    let square = |a: &Array| if a.square() { Ok(()) } else { Err(format!("{} takes a square matrix, not {}", op.name(), a.describe())) };
    Ok(Some(match op {
        Op::Transpose | Op::Det | Op::Trace => {
            // a scalar is its own transpose, determinant and trace
            let Some(a) = &arrays[0] else { return Ok(Some(args[0].clone())) };
            match op {
                Op::Transpose => a.transpose().expr(),
                Op::Det => {
                    square(a)?;
                    det(a)?
                }
                _ => {
                    square(a)?;
                    simp(&Expr::Add((0..a.rows).map(|i| a.at(i, i).clone()).collect()))
                }
            }
        }
        Op::Rank => int(rank(&need(0)?) as i128),
        Op::Norm => match &arrays[0] {
            Some(a) => simp(&expr::sqrt(Expr::Add(a.a.iter().map(|x| expr::pow(expr::func(Func::Abs, x.clone()), int(2))).collect()))),
            None => simp(&expr::func(Func::Abs, args[0].clone())),
        },
        Op::Rows => int(need(0)?.rows as i128),
        Op::Cols => int(need(0)?.cols as i128),
        Op::Identity => {
            let n = args[0].as_num().and_then(Num::as_int).filter(|n| (1..=10_000).contains(n)).ok_or("identity takes a whole number of rows, 1 or more")?;
            Array::identity(n as usize).expr()
        }
        Op::At => {
            let a = arrays[0].clone().ok_or("at takes a vector or matrix and indices starting at 0")?;
            let index = |e: &Expr| e.as_num().and_then(Num::as_int).filter(|i| *i >= 0).map(|i| i as usize).ok_or("at needs nonnegative whole indices, starting at 0");
            let (i, j) = match args.len() {
                2 if a.vector() => (index(&args[1])?, 0),
                2 => return Err(format!("at(matrix, row, column) selects an element of {}", a.describe())),
                _ => (index(&args[1])?, index(&args[2])?),
            };
            if i >= a.rows || j >= a.cols {
                return Err(format!("index ({i}, {j}) is outside {}", a.describe()));
            }
            a.at(i, j).clone()
        }
        Op::Row | Op::Col => {
            let a = arrays[0].clone().ok_or("A[i, :] and A[:, j] take a row or a column of a vector or matrix")?;
            let (row, n) = (op == Op::Row, if op == Op::Row { a.rows } else { a.cols });
            let k = args[1].as_num().and_then(Num::as_int).filter(|i| *i >= 0).map(|i| i as usize)
                .ok_or("a row or column is selected by a nonnegative whole index, starting at 0 – A[i, :], A[:, j]")?;
            if k >= n {
                return Err(format!("{} {k} is outside {}, which has {n} {}s, counted from 0", op.name(), a.describe(), op.name()));
            }
            let picked = if row {
                Array { rows: 1, cols: a.cols, a: (0..a.cols).map(|j| a.at(k, j).clone()).collect() }
            } else {
                Array { rows: a.rows, cols: 1, a: (0..a.rows).map(|i| a.at(i, k).clone()).collect() }
            };
            // a row of a vector, a column of a row: one element
            if picked.a.len() == 1 { picked.a[0].clone() } else { picked.expr() }
        }
        Op::Lsolve | Op::Eigenvals | Op::SingularValues | Op::Condition | Op::Median | Op::Variance | Op::Stdev | Op::NormalCdf | Op::NormalQuantile | Op::Interpolate | Op::LinearFit | Op::Dft | Op::Idft | Op::Gamma | Op::Erf | Op::BesselJ | Op::Celsius | Op::Fahrenheit | Op::Polyroots => unreachable!("handled above"),
        Op::Cross | Op::Convolve | Op::Atan2 | Op::Min | Op::Max | Op::Mean => unreachable!("handled above"),
    }))
}

/// Elements selected by constant indices taken out without simplifying the whole vector or matrix first:
/// the body of x[i] := … for each value of i reads one element of t[i].
pub fn pick(e: &Expr) -> Expr {
    if let Expr::Op(Op::At, args) = e {
        if let Some(a) = Array::of(&args[0]) {
            let index = |e: &Expr| simp(&pick(e)).as_num().and_then(Num::as_int).filter(|i| *i >= 0).map(|i| i as usize);
            let at = match args.len() {
                2 if a.vector() => index(&args[1]).map(|i| (i, 0)),
                3 => index(&args[1]).zip(index(&args[2])),
                _ => None,
            };
            if let Some((i, j)) = at.filter(|(i, j)| *i < a.rows && *j < a.cols) {
                return pick(a.at(i, j));
            }
        }
    }
    e.map(&mut pick)
}

/// Why an expression with arrays in it has no value: the first operation that can't be done.
pub fn problem(e: &Expr) -> Option<String> {
    if let Some(m) = e.children().into_iter().find_map(problem) {
        return Some(m);
    }
    let has = |v: &[Expr]| v.iter().any(is_array);
    match e {
        Expr::Add(ts) if has(ts) => sum(ts).err(),
        Expr::Mul(fs) if has(fs) => product(fs).err(),
        Expr::Pow(b, x) if is_array(b) => power(b, x).err(),
        Expr::F(f, a) if is_array(a) => function(*f, a).err(),
        Expr::Op(op, args) => eval(*op, args).err(),
        Expr::Matrix(rs) if rs.iter().flatten().any(is_array) || rs.iter().any(|r| r.len() != rs[0].len()) => blocks(rs).err(),
        _ => None,
    }
}

// ---------- elimination ----------

/// Numbers with units in a pattern of a unit per row and per column (entry (i, j) in rᵢ·cⱼ), the pattern
/// physical matrices have and that determinants, inverses and systems need.
struct Numeric {
    m: Mat<Num>,
    r: Vec<Dim>,
    c: Vec<Dim>,
    /// What counts as zero: 0 for exact numbers, rounding for floats.
    tol: f64,
}

fn numeric(a: &Array) -> Result<Option<Numeric>, String> {
    let Some(vals) = a.a.iter().map(Expr::as_c).collect::<Option<Vec<&Val>>>() else { return Ok(None) };
    let (r, c) = pattern(a.rows, a.cols, &vals).ok_or("the units of the matrix don't fit a unit per row and per column, which a determinant, an inverse or a system needs")?;
    Ok(Some(Numeric { m: Mat::new(a.rows, a.cols, vals.iter().map(|v| v.v).collect()), r, c, tol: tolerance(a, &vals) }))
}

/// What counts as zero in elimination: nothing but 0 for exact numbers, rounding relative to the largest
/// element for floats.
fn tolerance(a: &Array, vals: &[&Val]) -> f64 {
    if vals.iter().all(|v| v.v.is_exact()) {
        return 0.0;
    }
    vals.iter().map(|v| v.v.f().abs()).fold(0.0, f64::max) * a.rows.max(a.cols) as f64 * 1e-13
}

/// The units per row and per column, if every entry fits them; a plain 0 fits any unit.
fn pattern(rows: usize, cols: usize, vals: &[&Val]) -> Option<(Vec<Dim>, Vec<Dim>)> {
    let fixed = |i: usize, j: usize| -> Option<Dim> {
        let v = vals[i * cols + j];
        (!(v.is_zero() && v.d.is_one())).then_some(v.d.clone())
    };
    let (mut r, mut c): (Vec<Option<Dim>>, Vec<Option<Dim>>) = (vec![None; rows], vec![None; cols]);
    // each connected group of rows and columns: its first row unitless, the others follow from the entries
    for start in 0..rows {
        if r[start].is_some() {
            continue;
        }
        r[start] = Some(Dim::ONE);
        let mut todo = vec![(true, start)];
        while let Some((row, k)) = todo.pop() {
            let n = if row { cols } else { rows };
            for l in 0..n {
                let (i, j) = if row { (k, l) } else { (l, k) };
                let Some(d) = fixed(i, j) else { continue };
                let (known, other) = if row { (<std::option::Option<Dim> as Clone>::clone(&r[i]), &mut c[j]) } else { (<std::option::Option<Dim> as Clone>::clone(&c[j]), &mut r[i]) };
                let want = d.div(&known.unwrap());
                match other {
                    None => {
                        *other = Some(want);
                        todo.push((!row, l));
                    }
                    Some(x) if *x != want => return None,
                    _ => {}
                }
            }
        }
    }
    Some((r.into_iter().map(|d| d.unwrap_or(Dim::ONE)).collect(), c.into_iter().map(|d| d.unwrap_or(Dim::ONE)).collect()))
}

fn det(a: &Array) -> Result<Expr, String> {
    if let Some(n) = numeric(a)? {
        let d = n.r.iter().chain(&n.c).fold(Dim::ONE, |d, x| d.mul(x));
        return Ok(Expr::C(Val::new(eliminate::det(n.m, n.tol), d)));
    }
    // small matrices expanded (the result reads like the textbook formula), larger ones fraction-free
    Ok(simplify(&if a.rows <= 3 { eliminate::det_expand(&a.mat()) } else { eliminate::det_bareiss(a.mat(), 0.0) }))
}

fn inverse(a: &Array) -> Result<Array, String> {
    let k = a.rows;
    if let Some(n) = numeric(a)? {
        let inv = eliminate::solve(&n.m, &Mat::identity(k), n.tol).ok_or(SINGULAR)?;
        return Ok(Array { rows: k, cols: k, a: (0..k * k).map(|p| Expr::C(Val::new(inv.a[p], n.r[p % k].mul(&n.c[p / k]).powi(-1)))).collect() });
    }
    if k <= 3 {
        let d = det(a)?;
        if d.is_zero() {
            return Err(SINGULAR.into());
        }
        return Ok(Array { rows: k, cols: k, a: eliminate::adjugate(&a.mat()).a.into_iter().map(|x| simplify(&expr::div(x, d.clone()))).collect() });
    }
    let inv = eliminate::solve(&a.mat(), &Mat::identity(k), 0.0).ok_or(SINGULAR)?;
    Ok(Array { rows: k, cols: k, a: inv.a })
}

fn lsolve(a: &Array, b: &Array) -> Result<Array, String> {
    const NO_UNIQUE: &str = "the matrix is singular (its determinant is 0), so the system has no unique solution";
    if let (Some(n), Some(rhs)) = (numeric(a)?, b.a.iter().map(Expr::as_c).collect::<Option<Vec<&Val>>>()) {
        // per column of the right side: its unit over the row units must be one unit
        let mut k = vec![None; b.cols];
        for (p, v) in rhs.iter().enumerate() {
            if v.is_zero() && v.d.is_one() {
                continue;
            }
            let want = v.d.div(&n.r[p / b.cols]);
            match k[p % b.cols].clone() {
                None => k[p % b.cols] = Some(want),
                Some(x) if x != want => return Err("the units of the right side don't fit the rows of the matrix".into()),
                _ => {}
            }
        }
        let x = eliminate::solve(&n.m, &Mat::new(b.rows, b.cols, rhs.iter().map(|v| v.v).collect()), n.tol).ok_or(NO_UNIQUE)?;
        return Ok(Array { rows: b.rows, cols: b.cols, a: x.a.iter().enumerate().map(|(p, v)| Expr::C(Val::new(*v, <std::option::Option<Dim> as Clone>::clone(&k[p % b.cols]).unwrap_or(Dim::ONE).div(&n.c[p / b.cols])))).collect() });
    }
    let x = eliminate::solve(&a.mat(), &b.mat(), 0.0).ok_or(NO_UNIQUE)?;
    Ok(Array { rows: x.rows, cols: x.cols, a: x.a })
}

fn rank(a: &Array) -> usize {
    match a.a.iter().map(Expr::as_c).collect::<Option<Vec<&Val>>>() {
        // units don't change which rows are independent
        Some(vals) => eliminate::rank(Mat::new(a.rows, a.cols, vals.iter().map(|v| v.v).collect()), tolerance(a, &vals)),
        None => eliminate::rank(a.mat(), 0.0),
    }
}
