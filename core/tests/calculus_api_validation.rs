//! Public calculus operations reject ambiguous binders and unsupported orders.
mod support;
use solveng_core::env::Env;

#[test]
fn derivative_matrix_binders_are_distinct_names() {
    for source in [
        "jacobian(vec(x),vec(x,x)) =>",
        "jacobian(vec(x),vec(2)) =>",
        "jacobian(x,vec(x)) =>",
        "hessian(vec(x),vec(x)) =>",
    ] {
        let out = support::line(&mut Env::default(), source);
        assert!(!out.errors.is_empty(), "{source}: {out:?}");
    }
}

#[test]
fn taylor_requires_a_bounded_integer_degree() {
    for degree in ["-1", "33", "1/2", "1 m"] {
        let source = format!("taylor(exp(x),x,0,{degree}) =>");
        let out = support::line(&mut Env::default(), &source);
        assert!(!out.errors.is_empty(), "{source}: {out:?}");
    }
}

#[test]
fn cancellation_preserves_negative_power_restrictions() {
    let mut env = Env::default();
    support::run(&mut env, "f(x) := x^(-2)*x^2");
    assert!(!support::line(&mut env, "f(0) =>").errors.is_empty());
    assert!(support::line(&mut env, "f(2) =>").errors.is_empty());
}
