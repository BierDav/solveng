//! solveng's calculation engine: reads equations as solveng writes them (math as Typst gives it, code lines),
//! calculates them like a Mathcad worksheet and writes the results as Typst math markup.

pub mod api;
pub mod calculus;
pub mod dim;
pub mod env;
pub mod eval;
pub mod expr;
pub mod format;
pub mod graph;
pub mod num;
pub mod parse;
pub mod poly;
pub mod render;
pub mod sheet;
pub mod solve;
pub mod units;
