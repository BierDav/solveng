//! Numerical operations on measured data, with explicit domain and dimension checks.
//! Numeric arrays use SI coefficients; results restore their physical dimensions.
use crate::{
    complex::CVal,
    dim::Dim,
    expr::{Expr, Val, int, simp, sqrt},
    linalg::{Array, Op},
    num::{Num, Rat},
};
use nalgebra::{
    Complex, DMatrix,
    linalg::{SVD, Schur},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NumericOp {
    Minimize,
    Ode,
}
impl NumericOp {
    pub fn name(self) -> &'static str {
        match self {
            Self::Minimize => "minimize",
            Self::Ode => "odesolve",
        }
    }
}

fn real(e: &Expr) -> Result<Val, String> {
    crate::eval::value(e).map_err(|e| e.message())
}
fn number(x: f64, d: Dim) -> Result<Expr, String> {
    if !x.is_finite() {
        return Err("numerical result is outside the finite range".into());
    }
    Ok(Expr::C(Val::new(Num::F(x), d)))
}
fn vector(e: &Expr) -> Result<Vec<Expr>, String> {
    Array::of(e)
        .filter(Array::vector)
        .filter(|a| a.rows > 0)
        .map(|a| a.a)
        .ok_or("expected a nonempty column vector".into())
}
fn real_vector(e: &Expr) -> Result<(Vec<f64>, Dim), String> {
    let values = vector(e)?.iter().map(real).collect::<Result<Vec<_>, _>>()?;
    let d = common_dimension(&values)?;
    Ok((values.iter().map(|v| v.v.f()).collect(), d))
}
fn common_dimension(values: &[Val]) -> Result<Dim, String> {
    let d = values.iter().find(|v| !(v.is_zero() && v.d.is_one())).map_or(Dim::ONE, |v| v.d.clone());
    if values.iter().any(|v| v.d != d && !(v.is_zero() && v.d.is_one())) {
        return Err("values must have matching units".into());
    }
    Ok(d)
}
// Differences of finite values can overflow even when their ratio is finite.
fn difference_ratio(a: f64, b: f64, c: f64, d: f64) -> f64 {
    let numerator = a - b;
    let denominator = c - d;
    if numerator.is_finite() && denominator.is_finite() {
        numerator / denominator
    } else {
        let scale = a.abs().max(b.abs()).max(c.abs()).max(d.abs());
        (a / scale - b / scale) / (c / scale - d / scale)
    }
}
fn lerp(a: f64, b: f64, t: f64) -> f64 {
    if a.signum() != b.signum() { a * (1.0 - t) + b * t } else { a + (b - a) * t }
}
// Centred observations in a finite scale, retaining close large offsets.
fn centered(values: &[f64]) -> (Vec<f64>, f64, f64) {
    let origin = values[0];
    let scale = values.iter().map(|v| v.abs()).fold(0.0_f64, f64::max);
    if scale == 0.0 { return (vec![0.0; values.len()], 0.0, 0.0); }
    let offsets: Vec<_> = values.iter().map(|v| {
        let delta = v - origin;
        if delta.is_finite() { delta / scale } else { v / scale - origin / scale }
    }).collect();
    let mean = offsets.iter().sum::<f64>() / values.len() as f64;
    let location = origin / scale + mean;
    (offsets.iter().map(|v| v - mean).collect(), scale, location)
}
fn same(a: &Dim, b: &Dim) -> Result<(), String> {
    if a != b { Err("arguments must have matching units".into()) } else { Ok(()) }
}
fn unitless(v: &Val) -> Result<f64, String> {
    if !v.d.is_one() {
        return Err("argument must be unitless".into());
    }
    Ok(v.v.f())
}
fn array(e: &Expr) -> Result<Array, String> {
    let a = Array::of(e).ok_or("expected a vector or matrix")?;
    if a.rows == 0 || a.cols == 0 {
        return Err("matrix must be nonempty".into());
    }
    if a.rows.max(a.cols) > 256 {
        return Err("matrix dimension limit exceeded (256)".into());
    }
    Ok(a)
}
fn complex_values(a: &Array) -> Result<Vec<CVal>, String> {
    a.a.iter().map(|e| crate::eval::complex_value(e).map_err(|e| e.message())).collect()
}
fn uniform_matrix(a: &Array) -> Result<(DMatrix<Complex<f64>>, Dim), String> {
    let v = complex_values(a)?;
    let dims: Vec<Val> = v.iter().map(|z| Val::new(Num::F(z.z().abs()), z.d().clone())).collect();
    let d = common_dimension(&dims)?;
    Ok((DMatrix::from_row_iterator(a.rows, a.cols, v.iter().map(|z| Complex::new(z.re.v.f(), z.im.v.f()))), d))
}
fn complex_number(z: Complex<f64>, d: Dim) -> Result<Expr, String> {
    if !z.re.is_finite() || !z.im.is_finite() {
        return Err("numerical result is outside the finite range".into());
    }
    Ok(crate::complex::expr(CVal { re: Val::new(Num::F(z.re), d.clone()), im: Val::new(Num::F(z.im), d) }))
}
fn svd(m: DMatrix<Complex<f64>>, vectors: bool) -> Result<SVD<Complex<f64>, nalgebra::Dyn, nalgebra::Dyn>, String> {
    SVD::try_new(m, vectors, vectors, 8.0 * f64::EPSILON, 10_000)
        .ok_or("singular-value decomposition did not converge".into())
}

pub(crate) fn operation(op: Op, args: &[Expr]) -> Result<Expr, String> {
    let result = (|| match op {
        Op::Median | Op::Variance | Op::Stdev => {
            let (mut values, d) = real_vector(&args[0])?;
            if op == Op::Median {
                values.sort_by(f64::total_cmp);
                let n = values.len();
                return number(
                    if n % 2 == 0 { values[n / 2 - 1] * 0.5 + values[n / 2] * 0.5 } else { values[n / 2] },
                    d,
                );
            }
            if values.len() < 2 {
                return Err("sample statistics need at least two observations".into());
            }
            let (centered, scale, _) = centered(&values);
            let ss = centered.iter().map(|v| v * v).sum::<f64>() / (values.len() - 1) as f64;
            if op == Op::Stdev { number(scale * ss.sqrt(), d) } else { number(scale * (scale * ss), d.powi(2)) }
        }
        Op::Celsius | Op::Fahrenheit => {
            let x = real(&args[0])?;
            unitless(&x)?;
            // Exact affine arithmetic also preserves the absolute-zero boundary
            // (offset-then-rescale avoids cancellation in Fahrenheit readings).
            let kelvin = if op == Op::Celsius {
                x.v.add(Num::R(Rat::new(27315, 100).unwrap()))
            } else {
                x.v.add(Num::R(Rat::new(45967, 100).unwrap())).mul(Num::R(Rat::new(5, 9).unwrap()))
            };
            if kelvin.is_neg() {
                return Err("absolute temperature cannot be below zero kelvin".into());
            }
            if !kelvin.f().is_finite() {
                return Err("absolute temperature is outside the finite range".into());
            }
            Ok(Expr::C(Val::new(kelvin, Dim::base(4))))
        }
        Op::Gamma | Op::Erf | Op::BesselJ => {
            let x_val = real(args.last().unwrap())?;
            let x = unitless(&x_val)?;
            if op == Op::Erf && x_val.v.is_exact() && x_val.v.is_zero() {
                return Ok(int(0));
            }
            if op == Op::Gamma && x_val.v.is_exact() {
                if let Some(r) = x_val.v.rat() {
                    if r == Rat::ONE || r == Rat::int(2) {
                        return Ok(int(1));
                    } else if r == Rat::int(3) {
                        return Ok(int(2));
                    } else if r == Rat::new(1, 2).unwrap() {
                        return Ok(simp(&sqrt(Expr::Pi)));
                    }
                }
            }
            let y = match op {
                Op::Gamma => {
                    if x <= 0.0 && x.fract() == 0.0 {
                        return Err("gamma has a pole at nonpositive integers".into());
                    }
                    libm::tgamma(x)
                }
                Op::Erf => libm::erf(x),
                _ => {
                    let n = unitless(&real(&args[0])?)?;
                    if n.fract() != 0.0 || n.abs() > 10_000.0 {
                        return Err("Bessel order must be an integer between -10000 and 10000".into());
                    }
                    if x_val.v.is_exact() && x_val.v.is_zero() {
                        return Ok(int(if n == 0.0 { 1 } else { 0 }));
                    }
                    libm::jn(n as i32, x)
                }
            };
            number(y, Dim::ONE)
        }
        Op::Pnorm | Op::Qnorm => {
            let (x, mu, sigma) = (real(&args[0])?, real(&args[1])?, real(&args[2])?);
            same(&mu.d, &sigma.d)?;
            if sigma.v.f() <= 0.0 {
                return Err("standard deviation must be positive".into());
            }
            if op == Op::Pnorm {
                same(&x.d, &mu.d)?;
                number(normal_cdf(difference_ratio(x.v.f(), mu.v.f(), sigma.v.f(), 0.0)), Dim::ONE)
            } else {
                let p = unitless(&x)?;
                if !(0.0 < p && p < 1.0) {
                    return Err("probability must be strictly between zero and one".into());
                }
                // Invert the smaller tail, avoiding cancellation near p = 1.
                let (p, sign) = if p > 0.5 { (1.0 - p, -1.0) } else { (p, 1.0) };
                let (mut lo, mut hi) = (-40.0, 0.0);
                // An exactly represented endpoint root should remain exact;
                // bisection into a rounded CDF plateau would move it needlessly.
                if normal_cdf(hi) == p {
                    return number(sigma.v.f().mul_add(sign * hi, mu.v.f()), mu.d);
                }
                for _ in 0..160 {
                    let mid = (lo + hi) * 0.5;
                    if normal_cdf(mid) < p {
                        lo = mid;
                    } else {
                        hi = mid;
                    }
                }
                number(sigma.v.f().mul_add(sign * ((lo + hi) * 0.5), mu.v.f()), mu.d)
            }
        }
        Op::Interpolate | Op::LinearFit => {
            let (x, dx) = real_vector(&args[0])?;
            let (y, dy) = real_vector(&args[1])?;
            if x.len() != y.len() || x.len() < 2 {
                return Err("x and y need the same length, at least two".into());
            }
            if op == Op::Interpolate {
                if x.windows(2).any(|v| v[0] >= v[1]) {
                    return Err("x knots must be strictly increasing".into());
                }
                let at = real(&args[2])?;
                same(&at.d, &dx)?;
                let at = at.v.f();
                if at < x[0] || at > x[x.len() - 1] {
                    return Err("interpolation point is outside the data range".into());
                }
                let i = x.partition_point(|v| *v < at);
                if i < x.len() && x[i] == at {
                    return Ok(vector(&args[1])?[i].clone());
                }
                let weight = difference_ratio(at, x[i - 1], x[i], x[i - 1]);
                number(lerp(y[i - 1], y[i], weight), dy)
            } else {
                let (xc, sx, mx) = centered(&x);
                let (yc, sy, my) = centered(&y);
                let xx = xc.iter().map(|v| v * v).sum::<f64>();
                if xx == 0.0 { return Err("regression needs distinct x observations".into()); }
                let xy = xc.iter().zip(&yc).map(|(x, y)| x * y).sum::<f64>();
                let normalized_slope = xy / xx;
                let slope = (normalized_slope * sy) / sx;
                Ok(Expr::Vector(vec![
                    number((my - normalized_slope * mx) * sy, dy.clone())?,
                    number(slope, dy.div(&dx))?,
                ]))
            }
        }
        Op::Dft | Op::Idft => {
            let entries = vector(&args[0])?;
            let n = entries.len();
            if n > 2048 {
                return Err("direct Fourier transform limit exceeded (2048 samples)".into());
            }
            let a = Array { rows: n, cols: 1, a: entries };
            let (mut m, d) = uniform_matrix(&a)?;
            let amplitude = m.iter().map(|v| v.re.abs().max(v.im.abs())).fold(0.0_f64, f64::max);
            if amplitude > 0.0 { m.iter_mut().for_each(|v| { v.re /= amplitude; v.im /= amplitude; }); }
            let scale = if op == Op::Idft { 1.0 / n as f64 } else { 1.0 };
            let sign = if op == Op::Idft { 1.0 } else { -1.0 };
            let mut out = vec![];
            for k in 0..n {
                let mut sum = Complex::new(0.0, 0.0);
                let mut correction = sum;
                let (mut re_scale, mut im_scale) = (0.0, 0.0);
                for j in 0..n {
                    let angle = sign * std::f64::consts::TAU * ((k * j) % n) as f64 / n as f64;
                    let phase = (k * j) % n;
                    let rotation = if (4 * phase) % n == 0 {
                        match (4 * phase) / n {
                            0 => Complex::new(1.0, 0.0),
                            1 => Complex::new(0.0, sign),
                            2 => Complex::new(-1.0, 0.0),
                            _ => Complex::new(0.0, -sign),
                        }
                    } else {
                        Complex::new(angle.cos(), angle.sin())
                    };
                    let product = m[(j, 0)] * rotation;
                    re_scale += (m[(j, 0)].re * rotation.re).abs() + (m[(j, 0)].im * rotation.im).abs();
                    im_scale += (m[(j, 0)].re * rotation.im).abs() + (m[(j, 0)].im * rotation.re).abs();
                    let term = product - correction;
                    let next = sum + term;
                    correction = (next - sum) - term;
                    sum = next;
                }
                if sum.re.abs() <= 8.0 * f64::EPSILON * re_scale {
                    sum.re = 0.0;
                }
                if sum.im.abs() <= 8.0 * f64::EPSILON * im_scale {
                    sum.im = 0.0;
                }
                out.push(complex_number((sum * scale) * amplitude, d.clone())?);
            }
            Ok(Expr::Vector(out))
        }
        Op::Lsolve => {
            let (a, b) = (array(&args[0])?, array(&args[1])?);
            if a.rows != b.rows || a.rows < a.cols {
                return Err("lsolve needs at least as many rows as columns and a right side with matching rows".into());
            }
            let av = complex_values(&a)?;
            let bv = complex_values(&b)?;
            let column = |v: &[CVal], rows: usize, cols: usize, j: usize| {
                common_dimension(
                    &(0..rows)
                        .map(|i| Val::new(Num::F(v[i * cols + j].z().abs()), v[i * cols + j].d().clone()))
                        .collect::<Vec<_>>(),
                )
            };
            let da = (0..a.cols).map(|j| column(&av, a.rows, a.cols, j)).collect::<Result<Vec<_>, _>>()?;
            let db = (0..b.cols).map(|j| column(&bv, b.rows, b.cols, j)).collect::<Result<Vec<_>, _>>()?;
            let mut m =
                DMatrix::from_row_iterator(a.rows, a.cols, av.iter().map(|v| Complex::new(v.re.v.f(), v.im.v.f())));
            let mut rhs =
                DMatrix::from_row_iterator(b.rows, b.cols, bv.iter().map(|v| Complex::new(v.re.v.f(), v.im.v.f())));
            let scale = m.iter().map(|v| v.re.abs().max(v.im.abs())).fold(0.0_f64, f64::max);
            if scale == 0.0 {
                return Err("matrix does not have full column rank".into());
            }
            m.iter_mut().for_each(|v| { v.re /= scale; v.im /= scale; });
            rhs.iter_mut().for_each(|v| { v.re /= scale; v.im /= scale; });
            let decomp = svd(m, true)?;
            let cutoff = decomp.singular_values[0] * a.rows.max(a.cols) as f64 * 32.0 * f64::EPSILON;
            if decomp.singular_values.iter().any(|v| *v <= cutoff) {
                return Err("matrix does not have full column rank".into());
            }
            let solved = decomp.solve(&rhs, cutoff).map_err(str::to_string)?;
            let mut out = vec![];
            for i in 0..a.cols {
                for j in 0..b.cols {
                    out.push(complex_number(solved[(i, j)], db[j].div(&da[i]))?);
                }
            }
            Ok(Array { rows: a.cols, cols: b.cols, a: out }.expr())
        }
        Op::Eigenvals | Op::SingularValues | Op::Condition => {
            let a = array(&args[0])?;
            let (mut m, d) = uniform_matrix(&a)?;
            let scale = m.iter().map(|v| v.re.abs().max(v.im.abs())).fold(0.0_f64, f64::max);
            if scale > 0.0 {
                m.iter_mut().for_each(|v| { v.re /= scale; v.im /= scale; });
            }
            if op == Op::Eigenvals {
                if a.rows != a.cols {
                    return Err("eigenvalues need a square matrix".into());
                }
                let (_, t) = Schur::try_new(m, 8.0 * f64::EPSILON, 10_000)
                    .ok_or("eigenvalue decomposition did not converge")?
                    .unpack();
                return Ok(Expr::Vector(
                    (0..a.rows).map(|i| complex_number(t[(i, i)] * scale, d.clone())).collect::<Result<_, _>>()?,
                ));
            }
            let values = svd(m, false)?.singular_values;
            if op == Op::Condition {
                let last = values[values.len() - 1];
                if last <= 32.0 * f64::EPSILON * a.rows.max(a.cols) as f64 * values[0] {
                    return Err("condition number is unbounded or unresolved: matrix is numerically singular".into());
                }
                number(values[0] / last, Dim::ONE)
            } else {
                Ok(Expr::Vector(values.iter().map(|v| number(v * scale, d.clone())).collect::<Result<_, _>>()?))
            }
        }
        Op::Polyroots => {
            let entries = vector(&args[0])?;
            if entries.len() < 2 || entries.len() > 61 {
                return Err("polyroots takes 2 to 61 coefficients in ascending power order".into());
            }
            let values = entries
                .iter()
                .map(|e| crate::eval::complex_value(e).map_err(|e| e.message()))
                .collect::<Result<Vec<_>, _>>()?;
            if values.iter().any(|v| !v.d().is_one()) {
                return Err("polyroots currently takes unitless coefficients".into());
            }
            if values.last().unwrap().z().abs() == 0.0 {
                return Err("leading polynomial coefficient must be nonzero".into());
            }
            let roots = crate::solve::complex_roots(&values.iter().map(CVal::z).collect::<Vec<_>>());
            Ok(Expr::Vector(
                roots.iter().map(|v| complex_number(Complex::new(v.re, v.im), Dim::ONE)).collect::<Result<_, _>>()?,
            ))
        }
        _ => unreachable!("not a numerical operation"),
    })();
    result.map_err(|e: String| format!("{}: {e}", op.name()))
}
fn normal_cdf(z: f64) -> f64 {
    0.5 * libm::erfc(-z / std::f64::consts::SQRT_2)
}

/// A bounded minimizer for a continuous unimodal objective. Endpoints participate
/// in the comparison, so a constrained boundary minimum is a normal result.
pub(crate) fn minimize(f: &impl Fn(f64) -> Option<f64>, lo: f64, hi: f64) -> Result<f64, String> {
    if !lo.is_finite() || !hi.is_finite() || lo > hi {
        return Err("minimize: finite bounds must be ordered lower <= upper".into());
    }
    let sample = |x| f(lerp(lo, hi, x)).filter(|y| y.is_finite()).ok_or("minimize: objective is undefined in the interval");
    let (fl, fh) = (sample(0.0)?, sample(1.0)?);
    if lo == hi {
        return Ok(lo);
    }
    let (mut a, mut b) = (0.0_f64, 1.0_f64);
    let ratio = (5.0_f64.sqrt() - 1.0) * 0.5;
    let (mut c, mut d) = (b - ratio * (b - a), a + ratio * (b - a));
    let (mut fc, mut fd) = (sample(c)?, sample(d)?);
    for _ in 0..256 {
        if (b - a).abs() <= 1e-12 {
            break;
        }
        if fc < fd {
            b = d;
            d = c;
            fd = fc;
            c = b - ratio * (b - a);
            fc = sample(c)?;
        } else {
            a = c;
            c = d;
            fc = fd;
            d = a + ratio * (b - a);
            fd = sample(d)?;
        }
    }
    let at = a * 0.5 + b * 0.5;
    let value = sample(at)?;
    Ok(lerp(lo, hi, if fl <= value && fl <= fh { 0.0 } else if fh <= value { 1.0 } else { at }))
}

/// Adaptive RK4 step doubling with a fifth-order extrapolated accepted step.
/// Non-stiff scalar initial-value problems, including explicit time dependence.
pub(crate) fn ode(f: &impl Fn(f64, f64) -> Option<f64>, initial: f64, start: f64, end: f64) -> Result<f64, String> {
    if !initial.is_finite() || !start.is_finite() || !end.is_finite() {
        return Err("odesolve: initial value and times must be finite".into());
    }
    if start == end {
        return Ok(initial);
    }
    let step = |t: f64, y: f64, h: f64| -> Option<f64> {
        let a = f(t, y)?;
        let b = f(t + h * 0.5, y + h * 0.5 * a)?;
        let c = f(t + h * 0.5, y + h * 0.5 * b)?;
        let d = f(t + h, y + h * c)?;
        let next = y + h * (a + 2.0 * b + 2.0 * c + d) / 6.0;
        next.is_finite().then_some(next)
    };
    let sign = if end > start { 1.0 } else { -1.0 };
    let delta = end - start;
    let mut max_step = if delta.is_finite() { delta / 31.0 } else { end / 31.0 - start / 31.0 };
    // A subnormal interval may be shorter than one nominal subdivision.
    if max_step == 0.0 { max_step = delta; }
    let (mut t, mut y, mut h) = (start, initial, max_step);
    for _ in 0..100_000 {
        if sign * (t + h - end) > 0.0 {
            h = end - t;
        }
        if t + h == t {
            return Err("odesolve: step is below floating-point resolution".into());
        }
        let whole = step(t, y, h).ok_or("odesolve: right side is undefined or overflows")?;
        let first = step(t, y, h * 0.5).ok_or("odesolve: right side is undefined or overflows")?;
        let half = step(t + h * 0.5, first, h * 0.5).ok_or("odesolve: right side is undefined or overflows")?;
        let error = (half - whole).abs() / 15.0;
        let tolerance = 1e-10 * y.abs().max(half.abs()) + 1e-12 * initial.abs().max(1e-12);
        if error <= tolerance {
            y = half + (half - whole) / 15.0;
            t += h;
            if sign * (end - t) <= 0.0 {
                return Ok(y);
            }
        }
        h *= if error == 0.0 { 4.0 } else { (0.9 * (tolerance / error).powf(0.2)).clamp(0.1, 4.0) };
        h = sign * h.abs().min(max_step.abs());
    }
    Err("odesolve: step limit exceeded (100000); the problem may be stiff".into())
}

/// Result dimensions for scalar numerical operations before values are known
/// (e.g. a special function inside an integral or an ODE right side).
pub(crate) fn dimension(op: Op, args: &[Expr], sd: &dyn Fn(&str) -> Option<Dim>) -> Result<Option<Dim>, String> {
    let dim = |e: &Expr| crate::eval::dim_of(e, sd);
    let require_unitless = |e: &Expr| -> Result<(), String> {
        if dim(e)?.is_some_and(|d| !d.is_one()) {
            return Err(format!("{}: argument must be unitless", op.name()));
        }
        Ok(())
    };
    let uniform = |e: &Expr| -> Result<Option<Dim>, String> {
        let Some(a) = Array::of(e) else {
            return Ok(None);
        };
        let mut result = None;
        let mut unknown = false;
        for entry in a.a {
            if entry.as_c().is_some_and(|v| v.is_zero() && v.d.is_one()) {
                continue;
            }
            if let Some(d) = dim(&entry)? {
                if result.as_ref().is_some_and(|previous| *previous != d) {
                    return Err(format!("{}: values must have matching units", op.name()));
                }
                result = Some(d);
            } else {
                unknown = true;
            }
        }
        Ok(if unknown { None } else { result.or(Some(Dim::ONE)) })
    };
    match op {
        Op::Gamma | Op::Erf | Op::BesselJ => {
            for arg in args {
                require_unitless(arg)?;
            }
            Ok(Some(Dim::ONE))
        }
        Op::Celsius | Op::Fahrenheit => {
            require_unitless(&args[0])?;
            Ok(Some(Dim::base(4)))
        }
        Op::Pnorm | Op::Qnorm => {
            let (mu, sigma) = (dim(&args[1])?, dim(&args[2])?);
            if let (Some(a), Some(b)) = (&mu, &sigma) {
                same(a, b)?;
            }
            if op == Op::Qnorm {
                require_unitless(&args[0])?;
                Ok(mu.or(sigma))
            } else {
                if let (Some(x), Some(mean)) = (dim(&args[0])?, mu) {
                    same(&x, &mean)?;
                }
                Ok(Some(Dim::ONE))
            }
        }
        Op::Median | Op::Stdev => uniform(&args[0]),
        Op::Variance => Ok(uniform(&args[0])?.map(|d| d.powi(2))),
        Op::Condition => {
            uniform(&args[0])?;
            Ok(Some(Dim::ONE))
        }
        Op::Interpolate => {
            if let (Some(x), Some(at)) = (uniform(&args[0])?, dim(&args[2])?) {
                same(&x, &at)?;
            }
            uniform(&args[1])
        }
        _ => {
            for e in args {
                dim(e)?;
            }
            Ok(None)
        }
    }
}
