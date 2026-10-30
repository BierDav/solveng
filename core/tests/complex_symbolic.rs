mod support;

use solveng_core::env::Env;
use support::{fails, result, run};

#[test]
fn real_conditions_and_local_assumptions_for_unresolved_coefficients() {
    let mut env = Env::default();
    run(&mut env, "x(t) := a s_a(t) + b s_b(t)");
    run(&mut env, "y_1(t) := Re(x(t))");
    run(&mut env, "y_2(t) := a Re(s_a(t)) + b Re(s_b(t))");
    let equation = "y_1(t) = y_2(t)";
    assert_eq!(result(&mut env, &format!("{equation} ->")), "\"true\" \"for\" a in RR and b in RR");
    let before = env.clone();
    for clause in ["assume(a in RR, b in RR)", "assume(b ∈ ℝ, a ∈ ℝ)", "assume(a in RR), assume(b in RR)"] {
        assert_eq!(result(&mut env, &format!("{equation} -> {clause}")), "\"true\"");
    }
    assert_eq!(env, before, "assumptions must not persist or alter function bodies");
    assert_eq!(result(&mut env, &format!("{equation} ->")), "\"true\" \"for\" a in RR and b in RR");
    run(&mut env, "a := #i");
    assert!(fails(&mut env, &format!("{equation} -> assume(a in RR, b in RR)")).contains("nonreal"));
}

#[test]
fn conditional_proofs_are_sufficient_and_do_not_hide_nonlinearity() {
    let mut env = Env::default();
    assert_eq!(result(&mut env, "Re(a z) = a Re(z) ->"), "\"true\" \"for\" a in RR");
    assert_eq!(result(&mut env, "Im(a z) = a Im(z) -> assume(a in RR)"), "\"true\"");
    assert_eq!(result(&mut env, "conj(a z) = a conj(z) -> assume(a in RR)"), "\"true\"");
    assert_eq!(result(&mut env, "Im(sin(a)) -> assume(a in RR)"), "\"0\"");
    assert_ne!(result(&mut env, "Re(ln(a)) = ln(a) -> assume(a in RR)"), "\"true\"");
    assert!(!result(&mut env, "(a z)^2 = a z^2 ->").contains("\"for\""));
    let steps = result(&mut env, "Re(a z) = a Re(z) -> steps");
    assert!(steps.contains("quad lr((\"true\" \"for\" a in RR))"), "{steps}");
    assert_eq!(result(&mut env, "Re(a z) = a Re(z) -> assume(a in RR), steps").split("<=>").last().unwrap().trim(), "\"true\"");
}

#[test]
fn assumption_syntax_and_supported_contexts_are_explicit() {
    let mut env = Env::default();
    for clause in ["assume()", "assume(a = real)", "assume(a in CC)", "assume(a > 1)", "assume(a + b in RR)"] {
        assert!(fails(&mut env, &format!("Re(a) -> {clause}")).contains("assume"));
    }
    for input in ["a = assume(a in RR)", "f(a) := Re(a) -> assume(a in RR)", "a^2 = -1 -> assume(a in RR), solve(a)"] {
        assert!(fails(&mut env, input).contains("assume"));
    }
    // A formal argument named a must remain complex-capable after a calculation assuming a real.
    run(&mut env, "f(a) := Re(a)");
    result(&mut env, "Re(a) = a -> assume(a in RR)");
    assert_eq!(result(&mut env, "f(1 + #i) ->"), "\"1\"");
    run(&mut env, "r := c + #i");
    assert!(fails(&mut env, "Re(r) -> assume(r in RR)").contains("free scalar"));
}

#[test]
fn local_assumptions_respect_bound_transform_variables() {
    use solveng_core::{complex::assuming_real, expr::{Expr, Func, func, sym}};
    use std::collections::BTreeSet;
    // The transform variable is complex even if an outer scalar has the same spelling.
    let inverse = Expr::InverseLaplace(Box::new(func(Func::Re, sym("a"))), "a".into(), Box::new(func(Func::Re, sym("t"))));
    let actual = assuming_real(&inverse, &BTreeSet::from(["a".into(), "t".into()]));
    let expected = Expr::InverseLaplace(Box::new(func(Func::Re, sym("a"))), "a".into(), Box::new(sym("t")));
    assert_eq!(actual, expected);
}

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
fn absolute_values_of_integer_powers_and_reciprocals_simplify() {
    let mut env = Env::default();
    run(&mut env, "b := 1/T");
    assert_eq!(result(&mut env, "G(t) := 1/abs(b) S(f/b) ->"), result(&mut env, "abs(T) S(f T) ->"));
    for identity in [
        "abs(1/z) = 1/abs(z)",
        "1/abs(1/z) = abs(z)",
        "abs(z^3) = abs(z)^3",
        "abs(z^(-4)) = abs(z)^(-4)",
        "abs((x + #i y)^(-2)) = abs(x + #i y)^(-2)",
    ] {
        assert_eq!(result(&mut env, &format!("{identity} ->")), "\"true\"", "{identity}");
    }
    // Complex exponents cannot be moved outside the absolute value.
    use solveng_core::expr::{Expr, Func, func, pow, simp, sym};
    let complex_power = func(Func::Abs, pow(sym("z"), Expr::I));
    assert_eq!(simp(&complex_power), complex_power);
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
