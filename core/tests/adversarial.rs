//! Confirmed failures and proposed common-feature contracts. These assert desired behavior,
//! not today's incorrect answers. See docs/adversarial.md.
mod support;

use solveng_core::{
    env::Env,
    eval::value,
    expr::{Expr, simp},
};
use std::{
    process::{Command, Stdio},
    time::{Duration, Instant},
};

fn number(source: &str) -> f64 {
    let mut env = Env::default();
    support::run(&mut env, &format!("answer := {source}"));
    let expr = simp(&support::bind(&env, &env.vars["answer"].expr));
    value(&expr)
        .unwrap_or_else(|e| panic!("{source}: {e:?}"))
        .v
        .f()
}

fn near(source: &str, expected: f64) {
    let got = number(source);
    assert!(
        (got - expected).abs() <= 1e-10 * expected.abs().max(1.0),
        "{source}: got {got}, expected {expected}"
    );
}

fn rejects(source: &str) {
    let mut env = Env::default();
    let out = support::line(&mut env, source);
    assert!(
        !out.errors.is_empty(),
        "{source}: expected a domain/unit error, got {out:?}"
    );
}

#[test]
fn zero_divided_by_zero_is_undefined() {
    rejects("0/0 =");
}

#[test]
fn multiplying_by_zero_preserves_domain_errors() {
    rejects("0 * (1/0) =");
}

#[test]
fn power_zero_preserves_domain_errors() {
    rejects("(1/0)^0 =");
}

#[test]
fn reciprocal_preserves_domain_errors() {
    rejects("1/(1/0) =");
}

#[test]
fn negative_square_root_target_has_no_solution() {
    for target in [-1, -2] {
        let mut env = Env::default();
        let out = support::run(&mut env, &format!("sqrt(x) = {target} -> solve(x)"));
        assert!(
            out.solutions.is_empty() && !env.vars.contains_key("x"),
            "{out:?}"
        );
        assert!(
            out.result
                .as_deref()
                .is_some_and(|s| s.contains("no") && s.contains("solution")),
            "{out:?}"
        );
    }
}

#[test]
fn divergent_integral_at_midpoint_is_rejected() {
    rejects("integrate(1/x^2, x, -1, 1) =");
}

#[test]
fn divergent_integral_away_from_midpoint_is_rejected() {
    rejects("integrate(1/(x - 1/3)^2, x, 0, 1) =");
}

#[test]
fn complex_vector_norm_is_positive_definite() {
    near("norm(vec(3 #i, 4))", 5.0);
    near("norm(vec(#i, 1))", 2.0_f64.sqrt());
}

#[test]
fn complex_matrix_norm_is_positive_definite() {
    near("norm(mat(#i, 0; 0, 1))", 2.0_f64.sqrt());
}

#[test]
fn simplified_log_exp_matches_direct_complex_evaluation() {
    let mut env = Env::default();
    support::run(&mut env, "f(x) := ln(exp(x)) ->");
    let direct = support::result(&mut env, "ln(exp(4 pi #i)) =");
    assert_eq!(direct, "\"0\"");
    assert_eq!(support::result(&mut env, "f(4 pi #i) ="), direct);
}

#[test]
fn zero_with_a_unit_cannot_be_added_to_a_different_dimension() {
    rejects("0 [m] + 1 [s] =");
}

// Explicit operation tokens express the proposed public operation contract without relying
// on the support harness's current operation-name list. These are feature requests, not
// claims that the existing documentation promises these spellings.
#[test]
fn quadrant_aware_atan2() {
    near("⦇atan2⦈(1, -1)", 3.0 * std::f64::consts::PI / 4.0);
    near("⦇atan2⦈(-1, -1)", -3.0 * std::f64::consts::PI / 4.0);
}

#[test]
fn minimum_of_scalar_values() {
    near("⦇min⦈(3, -2, 7)", -2.0);
}

#[test]
fn maximum_of_scalar_values() {
    near("⦇max⦈(3, -2, 7)", 7.0);
}

#[test]
fn logarithm_with_explicit_base() {
    near("log(8, 2)", 3.0);
}

#[test]
fn finite_product() {
    near("⦇prod⦈(k, k, 1, 5)", 120.0);
}

#[test]
fn arithmetic_mean_of_vector() {
    near("⦇mean⦈(vec(1, 2, 6))", 3.0);
}

/// Isolate aborts and expensive expressions; always kill and reap a timed-out child.
/// The deadline applies to process startup plus evaluation, excluding compilation.
fn bounded_worker(case: &str) {
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "resource_worker", "--ignored", "--nocapture"])
        .env("SOLVENG_ADVERSARIAL_WORKER", case)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();
    let start = Instant::now();
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert!(
                status.success(),
                "{case}: worker crashed or failed: {status}"
            );
            return;
        }
        if start.elapsed() >= Duration::from_secs(3) {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!(
                "{case}: exceeded 3 seconds; compute within budget or reject with a resource-limit error"
            );
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn deeply_nested_input_returns_or_reports_a_limit() {
    for case in ["nesting", "unary", "powers"] {
        bounded_worker(case);
    }
}

#[test]
fn large_convolution_completes_or_reports_a_limit() {
    bounded_worker("convolution");
}

#[test]
#[ignore = "internal subprocess entry; invoked by the two resource tests"]
fn resource_worker() {
    let Ok(case) = std::env::var("SOLVENG_ADVERSARIAL_WORKER") else {
        return;
    };
    let mut env = Env::default();
    let source = match case.as_str() {
        "nesting" => format!("{}1{} =", "(".repeat(2000), ")".repeat(2000)),
        "unary" => format!("{}1 =", "-".repeat(2000)),
        "powers" => format!("{}1 =", "1^".repeat(2000)),
        "convolution" => "answer := convolve(range(50000), range(50000))".into(),
        _ => panic!("unknown worker {case}"),
    };
    let out = support::line(&mut env, &source);
    if !out.errors.is_empty() {
        let message = out.errors.join(" ").to_lowercase();
        assert!(
            [
                "limit",
                "too long",
                "too large",
                "too deep",
                "budget",
                "nesting"
            ]
            .iter()
            .any(|s| message.contains(s)),
            "expected an explicit resource-limit diagnostic, got {out:?}"
        );
        return;
    }
    if case != "convolution" {
        assert_eq!(out.result.as_deref(), Some("\"1\""));
    } else {
        let expr = simp(&support::bind(&env, &env.vars["answer"].expr));
        let Expr::Vector(entries) = expr else {
            panic!("expected convolution vector")
        };
        assert_eq!(entries.len(), 99_999);
        for (index, expected) in [(0, 0.0), (2, 1.0), (99_998, 2_499_900_001.0)] {
            assert_eq!(
                value(&entries[index]).unwrap().v.f(),
                expected,
                "coefficient {index}"
            );
        }
    }
}

#[test]
fn undefined_operands_survive_cancellation_and_function_identities() {
    for source in ["1/0 - 1/0 =", "0 * ln(0) =", "ln(0)^0 =", "exp(ln(0)) =", "0 * (1/0)^2 ="] {
        rejects(source);
    }
}

#[test]
fn complex_norms_and_dimensioned_zeros() {
    near("norm(vec(3 + 4 #i, 12))", 13.0);
    near("norm(mat(3 #i, 4; 0, 12))", 13.0);
    for source in ["0 [m] + 0 [s] =", "(1 [m] - 1 [m]) + 1 [s] =", "0 [m] + #i [s] ="] {
        rejects(source);
    }
    let mut env = Env::default();
    for source in ["0 [m] + 1 [m] =", "0 + 1 [m] =", "0 [m] * 1 [s] ="] {
        support::run(&mut env, source);
    }
}

#[test]
fn aggregate_operations_validate_arguments_and_keep_local_indices() {
    near("⦇atan2⦈(0, -1)", std::f64::consts::PI);
    near("⦇atan2⦈(1 [m], 0 [m])", std::f64::consts::PI / 2.0);
    near("⦇min⦈(7/3, -1/2, 0)", -0.5);
    near("⦇max⦈(-3)", -3.0);
    near("⦇prod⦈(k, k, 3, 2)", 1.0);
    near("⦇prod⦈(k, k, -3, -1)", -6.0);
    near("log(1/8, 2)", -3.0);
    for source in [
        "⦇atan2⦈(0, 0) =", "⦇atan2⦈(1 [m], 1 [s]) =", "⦇atan2⦈(#i, 1) =",
        "⦇min⦈() =", "⦇max⦈(1, #i) =", "⦇min⦈(0 [m], 1 [s]) =",
        "⦇mean⦈(vec()) =", "⦇mean⦈(3) =", "⦇mean⦈(vec(1 [m], 1 [s])) =",
        "log(8, 1) =", "log(8, 0) =", "⦇prod⦈(k, k, 1, 3/2) =",
        "⦇prod⦈(k, k, 1 [m], 3 [m]) =", "⦇prod⦈(k, k, 1, 10001) =",
    ] {
        rejects(source);
    }
    let mut env = Env::default();
    support::run(&mut env, "k := 99");
    assert_eq!(support::result(&mut env, "⦇prod⦈(k, k, 1, 5) ="), "\"120\"");
    assert_eq!(support::result(&mut env, "k ="), "\"99\"");
}

#[test]
fn pole_detection_preserves_convergent_improper_integrals() {
    near("integrate(x^(-1/2), x, 0, 1)", 2.0);
    near("integrate(1/(x - 2)^2, x, 0, 1)", 0.5);
    rejects("integrate(1/(x^2 - 1/9)^2, x, -1, 1) =");
}
