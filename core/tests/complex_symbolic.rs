mod support;

use solveng_core::env::Env;
use support::{result, run};

#[test]
fn real_projection_is_linear_in_real_coefficients_with_unresolved_signals() {
    let mut env = Env::default();
    run(&mut env, "a := 2");
    run(&mut env, "b := -3");
    run(&mut env, "x(t) := a s_a(t) + b s_b(t)");
    run(&mut env, "y_1(t) := Re(x(t))");
    run(&mut env, "y_a1(t) := Re(s_a(t))");
    run(&mut env, "y_a2(t) := Re(s_b(t))");
    run(&mut env, "y_2(t) := a y_a1(t) + b y_a2(t)");
    assert_eq!(result(&mut env, "y_1(t) = y_2(t) ->"), "\"true\"");
    // The proof must survive supplying complex signals later.
    run(&mut env, "s_a(t) := t + #i t^2");
    run(&mut env, "s_b(t) := 4 #i - 2 t");
    assert_eq!(result(&mut env, "y_1(t) = y_2(t) ->"), "\"true\"");
}

#[test]
fn symbolic_complex_algebra_proves_identities_without_real_symbol_assumptions() {
    let mut env = Env::default();
    for identity in [
        "Re(x + y) = Re(x) + Re(y)",
        "Im(x - y) = Im(x) - Im(y)",
        "Re(Re(a) x + abs(b) y) = Re(a) Re(x) + abs(b) Re(y)",
        "Re(x y) = Re(x) Re(y) - Im(x) Im(y)",
        "Im(x y) = Re(x) Im(y) + Im(x) Re(y)",
        "Re(#i x) = -Im(x)",
        "Im(#i x) = Re(x)",
        "Re(x^2) = Re(x)^2 - Im(x)^2",
        "Im(x^2) = 2 Re(x) Im(x)",
        "Re(1/x) = Re(x)/(Re(x)^2 + Im(x)^2)",
        "Im(1/x) = -Im(x)/(Re(x)^2 + Im(x)^2)",
        "Re(conj(x)) = Re(x)",
        "Im(conj(x)) = -Im(x)",
        "conj(conj(x)) = x",
        "conj(x y) = conj(x) conj(y)",
        "conj(x + y) = conj(x) + conj(y)",
        "conj(x^(-3)) = conj(x)^(-3)",
        "conj(sin(x)) = sin(conj(x))",
        "Re(Im(x)) = Im(x)",
        "Im(Re(x)) = 0",
        "Im(abs(x)) = 0",
        "Re(pi) = pi",
    ] {
        assert_eq!(result(&mut env, &format!("{identity} ->")), "\"true\"", "{identity}");
    }
}

#[test]
fn complex_arguments_and_branch_sensitive_operations_keep_their_meaning() {
    let mut env = Env::default();
    run(&mut env, "f(z) := Re(z)");
    run(&mut env, "g(z) := Im(z)");
    run(&mut env, "h(z) := conj(z)");
    assert_eq!(result(&mut env, "f(3 + 4 #i) ->"), "\"3\"");
    assert_eq!(result(&mut env, "g(3 + 4 #i) ->"), "\"4\"");
    assert_eq!(result(&mut env, "h(3 + 4 #i) = 3 - 4 #i ->"), "\"true\"");
    // No 'contains no i, therefore real' inference, nor conjugation through branch cuts.
    for expression in ["Re(x)", "Im(x)", "Re(ln(x))", "Im(sqrt(x))", "conj(ln(x))", "conj(sqrt(x))"] {
        let output = result(&mut env, &format!("{expression} ->"));
        let operation = if expression.starts_with("conj") { "overline" } else { expression.split('(').next().unwrap() };
        assert!(output.contains(operation), "{expression}: {output}");
    }
    // Check a mixed complex quotient and higher integer powers by actual substitution.
    run(&mut env, "p(z, w) := Re((z/w)^(-3)) + #i Im((z/w)^(-3))");
    assert_eq!(result(&mut env, "p(1 + #i, 2 - #i) = ((1 + #i)/(2 - #i))^(-3) ->"), "\"true\"");
}

#[test]
fn large_powers_remain_bounded_and_symbolic() {
    let mut env = Env::default();
    let output = result(&mut env, "Re(z^1000000) ->");
    assert!(output.contains("Re") && output.len() < 200, "{output}");
}

#[test]
fn projections_preserve_units_and_piecewise_conditions() {
    let mut env = Env::default();
    run(&mut env, "f(z, t) := Re(cases(z, t < 0, #i z))");
    run(&mut env, "g(z, t) := Im(cases(z, t < 0, #i z))");
    assert_eq!(result(&mut env, "f(3 + 4 #i, -1) ->"), "\"3\"");
    assert_eq!(result(&mut env, "f(3 + 4 #i, 1) ->"), "\"−4\"");
    assert_eq!(result(&mut env, "g(3 + 4 #i, -1) ->"), "\"4\"");
    assert_eq!(result(&mut env, "g(3 + 4 #i, 1) ->"), "\"3\"");
    run(&mut env, "r(z) := Re(2 Ω z)");
    assert_eq!(result(&mut env, "r(3 + 4 #i) ->"), "\"6\" thin upright(\"Ω\")");
}
