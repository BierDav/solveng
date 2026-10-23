//! solveng's engine for Typst: every function takes and gives CBOR (`cbor.encode(…)` / `cbor(…)` in Typst).
//! The worksheet's definitions travel as opaque bytes the Typst side keeps in its state.

use ciborium::Value;
use serde::Serialize;
use serde::de::DeserializeOwned;
use solveng_core::api::{self, Arg, ClearReq, DefinedReq, Lam, LamReq, LineReq, UnitReq};
use solveng_core::env::Env;
use solveng_core::format::Cfg;
use solveng_core::parse::Mode;
use wasm_minimal_protocol::*;

initiate_protocol!();

fn de<T: DeserializeOwned>(b: &[u8], what: &str) -> Result<T, String> {
    ciborium::from_reader(b).map_err(|e| format!("solveng: bad {what}: {e}"))
}
fn en<T: Serialize>(v: &T) -> Result<Vec<u8>, String> {
    let mut out = vec![];
    ciborium::into_writer(v, &mut out).map_err(|e| e.to_string())?;
    Ok(out)
}
fn env_of(b: &[u8]) -> Result<Env, String> {
    if b.is_empty() { Ok(Env::default()) } else { de(b, "worksheet state") }
}
fn to_value<T: Serialize>(v: &T) -> Result<Value, String> {
    Value::serialized(v).map_err(|e| e.to_string())
}

/// (env, request) → (env: new env, out: what to show).
#[wasm_func]
pub fn line(env: &[u8], req: &[u8]) -> Result<Vec<u8>, String> {
    let mut e = env_of(env)?;
    let r: LineReq = de(req, "request")?;
    let out = api::line(&mut e, &r);
    // the state only changes when something was defined
    let env = if out.defined.is_empty() { env.to_vec() } else { en(&e)? };
    en(&Value::Map(vec![(Value::Text("env".into()), Value::Bytes(env)), (Value::Text("out".into()), to_value(&out)?)]))
}

/// (env, names) → (env: without them, errors: problems).
#[wasm_func]
pub fn clear(env: &[u8], req: &[u8]) -> Result<Vec<u8>, String> {
    let mut e = env_of(env)?;
    let r: ClearReq = de(req, "request")?;
    let errors = api::clear(&mut e, &r);
    en(&Value::Map(vec![(Value::Text("env".into()), Value::Bytes(en(&e)?)), (Value::Text("errors".into()), to_value(&errors)?)]))
}

#[wasm_func]
pub fn defined(env: &[u8], req: &[u8]) -> Result<Vec<u8>, String> {
    let e = env_of(env)?;
    let r: DefinedReq = de(req, "request")?;
    en(&api::defined(&e, &r)?)
}

/// The markup of a unit, or none if the text isn't one.
#[wasm_func]
pub fn unit(req: &[u8]) -> Result<Vec<u8>, String> {
    let r: UnitReq = de(req, "request")?;
    en(&api::unit_markup(&r))
}

/// Problems with the option units, or none.
#[wasm_func]
pub fn check_units(cfg: &[u8]) -> Result<Vec<u8>, String> {
    let c: Cfg = de(cfg, "options")?;
    en(&api::check_units(&c))
}

/// (env, request) → the prepared function as bytes, or an error message as (err: …).
#[wasm_func]
pub fn lambdify(env: &[u8], req: &[u8]) -> Result<Vec<u8>, String> {
    let e = env_of(env)?;
    let r: LamReq = de(req, "request")?;
    match api::lambdify(&e, &r) {
        Ok(l) => en(&Value::Map(vec![
            (Value::Text("lam".into()), Value::Bytes(en(&l)?)),
            (Value::Text("params".into()), to_value(&l.params)?),
        ])),
        Err(m) => en(&Value::Map(vec![(Value::Text("err".into()), Value::Text(m))])),
    }
}

/// (env, request) → (value: the result's markup) or (err: …); the env stays as it is.
#[wasm_func]
pub fn value(env: &[u8], req: &[u8]) -> Result<Vec<u8>, String> {
    let e = env_of(env)?;
    let r: LineReq = de(req, "request")?;
    en(&Value::Map(vec![match api::show_value(&e, &r) {
        Ok(m) => (Value::Text("value".into()), Value::Text(m)),
        Err(m) => (Value::Text("err".into()), Value::Text(m)),
    }]))
}

/// (prepared, args) → the value, none where undefined, or (err: …).
#[wasm_func]
pub fn lam_eval(lam: &[u8], args: &[u8]) -> Result<Vec<u8>, String> {
    let l: Lam = de(lam, "function")?;
    let a: Vec<Arg> = de(args, "arguments")?;
    match api::lam_eval(&l, &a) {
        Ok(v) => en(&v),
        Err(m) => en(&Value::Map(vec![(Value::Text("err".into()), Value::Text(m))])),
    }
}

#[derive(serde::Deserialize)]
struct QReq {
    cfg: Cfg,
    src: String,
    mode: Mode,
}

/// The value of an expression of defined names: (v: SI value, d: dims) or (err: …).
#[wasm_func]
pub fn quantity(env: &[u8], req: &[u8]) -> Result<Vec<u8>, String> {
    let e = env_of(env)?;
    let r: QReq = de(req, "request")?;
    match api::quantity(&e, &r.cfg, &r.src, r.mode) {
        Ok((v, d)) => en(&Value::Map(vec![(Value::Text("v".into()), Value::Float(v)), (Value::Text("d".into()), to_value(&d)?)])),
        Err(m) => en(&Value::Map(vec![(Value::Text("err".into()), Value::Text(m))])),
    }
}

/// (env, graph request) → (env: with what its curves written with := define, errors: problems of these lines).
/// Nothing is defined when the curves can't be read; the graph itself reports that.
#[wasm_func]
pub fn graph_define(env: &[u8], req: &[u8]) -> Result<Vec<u8>, String> {
    let mut e = env_of(env)?;
    let r: solveng_core::graph::GraphReq = de(req, "request")?;
    let mut errors = vec![];
    for d in solveng_core::graph::defines(&e, &r).unwrap_or_default() {
        errors.extend(api::line(&mut e, &LineReq { src: d.src, mode: d.mode, cfg: r.cfg.clone(), listed: true }).errors);
    }
    en(&Value::Map(vec![(Value::Text("env".into()), Value::Bytes(en(&e)?)), (Value::Text("errors".into()), to_value(&errors)?)]))
}

/// (env, inner env) → the env with what the inner one defined taken over.
#[wasm_func]
pub fn merge(env: &[u8], inner: &[u8]) -> Result<Vec<u8>, String> {
    let mut e = env_of(env)?;
    e.merge(&env_of(inner)?);
    en(&e)
}

/// (env, request) → what to draw, or (err: …).
#[wasm_func]
pub fn graph(env: &[u8], req: &[u8]) -> Result<Vec<u8>, String> {
    let e = env_of(env)?;
    let r: solveng_core::graph::GraphReq = de(req, "request")?;
    match solveng_core::graph::graph(&e, &r) {
        Ok(g) => en(&g),
        Err(m) => en(&Value::Map(vec![(Value::Text("err".into()), Value::Text(m))])),
    }
}

#[wasm_func]
pub fn statements(src: &[u8], mode: &[u8]) -> Result<Vec<u8>, String> {
    en(&solveng_core::parse::statements(&de::<String>(src, "source")?, de(mode, "mode")?))
}

#[wasm_func]
pub fn name(req: &[u8]) -> Result<Vec<u8>, String> {
    let source: String = de(req, "name")?;
    en(&solveng_names::resolve(&source).ok().map(|name| solveng_core::format::name(&name)))
}
