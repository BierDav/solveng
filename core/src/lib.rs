//! solveng's calculation engine: reads equations as Typst math gives them (with explicit references for bindings),
//! calculates them like a Mathcad worksheet and writes the results as Typst math markup.

pub mod api;
pub mod bindings;
pub mod builtins;
pub mod calculus;
pub mod complex;
pub mod convolution;
pub mod dim;
pub mod env;
pub mod eval;
pub mod expr;
pub mod format;
pub mod graph;
pub mod linalg;
pub mod num;
pub mod parse;
pub mod poly;
pub mod render;
pub mod sheet;
pub mod solve;
pub mod units;
pub mod transforms;
mod inverse_laplace;
mod identity;
mod spectral;

#[cfg(test)]
extern crate self as solveng_core;
#[cfg(test)]
#[path = "../tests/support/mod.rs"]
mod test_support;

mod numeric;
pub mod numerics;

/// The supported high-level worksheet API.
pub mod worksheet;
pub use worksheet::{Worksheet, Diagnostic};
