mod support;
use solveng_core::env::Env;
use support::{fails, line, result, run};

/// An equation after -> is reduced to where it holds: one side zero, in factors, each factor zero.
#[test]
fn an_equation_shows_where_it_holds() {
    let mut env = Env::default();
    run(&mut env, "F(u) := u^2");
    // is F linear? F(a p) = a F(p) only where a² p² = a p²
    assert_eq!(result(&mut env, "F(a p) = a F(p) -> simplify"), "a = \"0\" or a = \"1\" or p = \"0\"");
    assert_eq!(result(&mut env, "F(a p) = a F(p) ->"), "a = \"0\" or a = \"1\" or p = \"0\"");
    assert_eq!(result(&mut env, "F(x + y) = F(x) + F(y) -> simplify"), "x = \"0\" or y = \"0\"");
    // factors of degree one in several names, and exact roots of one name
    let mut env = Env::default();
    assert_eq!(result(&mut env, "x^2 = y^2 ->"), "x = -y or x = y");
    assert_eq!(result(&mut env, "x^2 = 2 ->"), "x = -sqrt(\"2\") or x = sqrt(\"2\")");
    assert_eq!(result(&mut env, "x^3 = x ->"), "x = \"−1\" or x = \"0\" or x = \"1\"");
    assert_eq!(result(&mut env, "6 x^2 - x = 1 ->"), "x = -frac(\"1\", \"3\") or x = frac(\"1\", \"2\")");
    assert_eq!(result(&mut env, "x + y = 3 ->"), "x = -y + \"3\"");
    // a factor no name can be solved for exactly stays as it is
    assert_eq!(result(&mut env, "x^2 + y^2 = 1 ->"), "x^(\"2\") + y^(\"2\") - \"1\" = \"0\"");
    // the denominator's zeros aren't where it holds
    assert_eq!(result(&mut env, "(x^2 - 1)/(x - 1) = 0 ->"), "x = \"−1\"");
}

#[test]
fn an_identity_is_true_and_a_contradiction_false() {
    let mut env = Env::default();
    run(&mut env, "G(u) := 3 u");
    run(&mut env, "H(u) := u + 1");
    assert_eq!(result(&mut env, "G(a p) = a G(p) -> simplify"), "\"true\"");
    assert_eq!(result(&mut env, "G(x + y) = G(x) + G(y) ->"), "\"true\"");
    assert_eq!(result(&mut env, "H(x + y) = H(x) + H(y) ->"), "\"false\"");
    // a factor that is never zero says nothing
    assert_eq!(result(&mut env, "x = x + 1 ->"), "\"false\"");
}

#[test]
fn steps_show_the_way_a_line_each() {
    let mut env = Env::default();
    run(&mut env, "F(u) := u^2");
    assert_eq!(
        result(&mut env, "F(a p) = a F(p) -> steps"),
        "& a^(\"2\") dot p^(\"2\") = a dot p^(\"2\") \\ & <=> a^(\"2\") dot p^(\"2\") - a dot p^(\"2\") = \"0\" \\ & <=> a dot p^(\"2\") dot lr((a - \"1\")) = \"0\" \\ & <=> a = \"0\" or a = \"1\" or p = \"0\""
    );
    assert_eq!(result(&mut env, "F(a p) = a F(p) -> simplify, steps"), result(&mut env, "F(a p) = a F(p) -> steps"));
    // an expression: as written with the definitions put in, then simplified
    assert_eq!(result(&mut env, "(F(x) - 1)/(x - 1) -> steps"), "& frac(x^(\"2\") - \"1\", x - \"1\") \\ & = x + \"1\"");
    // nothing on the way: just the result
    assert_eq!(result(&mut env, "a p -> steps"), "a dot p");
    // inline: in a row
    let inline = solveng_core::format::Cfg { block: false, ..support::cfg() };
    assert_eq!(support::line_cfg(&mut env, "x^2 = x -> steps", &inline).result.unwrap(), "x^(\"2\") = x <=> x^(\"2\") - x = \"0\" <=> x dot lr((x - \"1\")) = \"0\" <=> x = \"0\" or x = \"1\"");
    assert!(fails(&mut env, "x^2 = 4 -> steps, solve(x)").contains("steps"));
    assert!(fails(&mut env, "q := 2 -> steps").contains("steps"));
}

/// solve splits the equation into its factors first: a² p² = a p² for a is a = 0 or a = 1, not 1/2 ± √(p⁴)/(2p²).
#[test]
fn solve_solves_factor_by_factor() {
    let mut env = Env::default();
    run(&mut env, "F(u) := u^2");
    assert_eq!(result(&mut env, "F(a p) = a F(p) -> solve(a)"), "a_(1) = \"0\", quad a_(2) = \"1\"");
    let mut env = Env::default();
    assert_eq!(result(&mut env, "x^3 - x = 0 -> solve(x)"), "x_(1) = \"−1\", quad x_(2) = \"0\", quad x_(3) = \"1\"");
    assert_eq!(result(&mut env, "x^2 - y^2 = 0 -> solve(x)"), "x_(1) = -y, quad x_(2) = y");
}

#[test]
fn a_parameter_is_a_value_not_a_function() {
    let mut env = Env::default();
    let out = line(&mut env, "F(u) := u(t)^2");
    assert!(out.errors.iter().any(|e| e.contains("parameter u is called like a function")), "{:?}", out.errors);
}

#[test]
fn a_negated_sum_keeps_its_parentheses() {
    let mut env = Env::default();
    assert_eq!(result(&mut env, "-(y - 3) ->"), "-lr((y - \"3\"))");
}

#[test]
fn expand_multiplies_out_and_factor_splits() {
    let mut env = Env::default();
    assert_eq!(result(&mut env, "(x + 1)^2 -> expand"), "x^(\"2\") + \"2\" x + \"1\"");
    assert_eq!(result(&mut env, "x^3 - x -> factor"), "x dot lr((x - \"1\")) dot lr((x + \"1\"))");
    assert_eq!(result(&mut env, "x^2 + 2 x + 1 -> factor"), "lr((x + \"1\"))^(\"2\")");
    assert_eq!(result(&mut env, "2 x + 2 -> factor"), "\"2\" lr((x + \"1\"))");
    assert_eq!(result(&mut env, "x^2 - y^2 -> factor"), "lr((x - y)) dot lr((x + y))");
    // a fraction: both parts in factors; expand cancels first
    assert_eq!(result(&mut env, "(x^2 - 1)/(x^2 + 2 x) -> factor"), "frac(lr((x - \"1\")) dot lr((x + \"1\")), x dot lr((x + \"2\")))");
    assert_eq!(result(&mut env, "(x^2 - 1)/(x - 1) -> expand"), "x + \"1\"");
    // a number isn't split into primes; an equation side by side
    assert_eq!(result(&mut env, "12 -> factor"), "\"12\"");
    assert_eq!(result(&mut env, "(a + b)(a - b) = c -> expand"), "a^(\"2\") - b^(\"2\") = c");
    assert_eq!(result(&mut env, "x^2 - x = 0 -> factor"), "x dot lr((x - \"1\")) = \"0\"");
    // definitions keep the form
    assert_eq!(result(&mut env, "f(x) := (x + 1)^2 -> expand"), "x^(\"2\") + \"2\" x + \"1\"");
    assert_eq!(result(&mut env, "x^3 - x -> factor, steps"), "& x^(\"3\") - x \\ & = x dot lr((x - \"1\")) dot lr((x + \"1\"))");
    assert!(fails(&mut env, "x^2 = 1 -> expand, factor").contains("one of them"));
    assert!(fails(&mut env, "(x + 1)^2 = expand").contains("after ->"));
}

/// After -> come modifiers and a unit; a calculation followed by anything else is a mistake, not prose.
#[test]
fn only_modifiers_and_units_follow_the_arrow() {
    let mut env = Env::default();
    for src in ["x^2 -> 2", "x^2 -> x + 1", "x^2 -> sin(x)", "a + b -> 2 m", "x^2 -> simplify 2"] {
        assert!(fails(&mut env, src).contains("neither a modifier nor a unit"), "{src}");
    }
    assert!(fails(&mut env, "x^2 -> y").contains("is a name, not a unit"));
    // prose with an arrow stays prose
    for src in ["x -> 0", "n -> infinity", "f: X -> Y", "A -> B"] {
        let out = line(&mut env, src);
        assert!(out.prose && out.errors.is_empty(), "{src}: {out:?}");
    }
}

/// A statement is checked when everything in it is known: a function its definitions call but that isn't defined
/// yet leaves it a statement. One written in it undefined is a mistake.
#[test]
fn a_statement_with_an_undefined_function_is_prose() {
    let mut env = Env::default();
    run(&mut env, "x := a s_1(t) + b s_2(t)");
    run(&mut env, "y_1(t) := (x)^2");
    run(&mut env, "y_2(t) := s_1(t)^2 + s_2(t)^2");
    for src in ["y_1(t) = y_2(t)", "y_1(t) = 3"] {
        let out = line(&mut env, src);
        assert!(out.prose && out.errors.is_empty(), "{src}: {out:?}");
    }
    assert!(fails(&mut env, "f(2) = y_1(t)").contains("f isn't defined"));
    // evaluating still asks for it
    assert!(fails(&mut env, "y_1(1) =").contains("isn't defined"));
}
