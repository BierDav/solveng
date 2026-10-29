//! Supported worksheet facade. Other engine modules expose lower-level implementation details.
use crate::{api, env::Env, format::Cfg, sheet::Out};

pub use crate::api::{Arg as Argument, Compiled as CompiledExpression, CompileReq as CompileRequest, Row as Definition};
pub use crate::format::Cfg as FormatOptions;

/// A diagnostic with a stable data shape, suitable for an editor or host application.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Diagnostic {
    pub message: String,
}
impl From<String> for Diagnostic {
    fn from(message: String) -> Self { Self { message } }
}
impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str(&self.message) }
}
impl std::error::Error for Diagnostic {}

/// An owned worksheet. Evaluating `:=` changes definitions; `=>` and `-> solve(...)` do not.
#[derive(Clone, Debug, Default)]
pub struct Worksheet {
    environment: Env,
    options: Cfg,
}
impl Worksheet {
    pub fn new(options: FormatOptions) -> Self { Self { environment: Env::default(), options } }
    pub fn environment(&self) -> &Env { &self.environment }
    pub fn options(&self) -> &FormatOptions { &self.options }
    /// Evaluate engine source with explicit lexical binding tokens.
    pub fn evaluate(&mut self, source: impl Into<String>) -> Out {
        api::line(&mut self.environment, &api::LineReq { src: source.into(), cfg: self.options.clone(), listed: true })
    }
    pub fn value(&self, source: impl Into<String>) -> Result<String, Diagnostic> {
        api::show_value(&self.environment, &api::LineReq { src: source.into(), cfg: self.options.clone(), listed: false }).map_err(Into::into)
    }
    pub fn definitions(&self) -> Result<Vec<Definition>, Diagnostic> {
        api::defined(&self.environment, &api::DefinedReq { cfg: self.options.clone() }).map_err(Into::into)
    }
    pub fn compile(&self, request: &CompileRequest) -> Result<CompiledExpression, Diagnostic> {
        api::compile(&self.environment, request).map_err(Into::into)
    }
    pub fn clear(&mut self, names: Vec<String>) -> Vec<Diagnostic> {
        api::clear(&mut self.environment, &api::ClearReq { names }).into_iter().map(Into::into).collect()
    }
    pub fn reset(&mut self) { self.environment = Env::default(); }
}
