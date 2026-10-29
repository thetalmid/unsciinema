//! Core error type. At the WASM boundary every error becomes a JS exception
//! (wasm-bindgen converts `Result<T, E>` where `E: Into<JsValue>`).

use wasm_bindgen::JsValue;

#[derive(thiserror::Error, Debug, Clone)]
pub enum CoreError {
    #[error("invalid input: {0}")]
    Invalid(String),
    #[error("limit exceeded: {0}")]
    Limit(String),
    #[error("failed to decode image: {0}")]
    Decode(String),
    #[error("font error: {0}")]
    Font(String),
    #[error("export failed: {0}")]
    Export(String),
    #[error("{0}")]
    State(String),
}

impl CoreError {
    pub fn invalid(msg: impl Into<String>) -> Self {
        CoreError::Invalid(msg.into())
    }
    pub fn limit(msg: impl Into<String>) -> Self {
        CoreError::Limit(msg.into())
    }
    pub fn decode(msg: impl Into<String>) -> Self {
        CoreError::Decode(msg.into())
    }
    pub fn font(msg: impl Into<String>) -> Self {
        CoreError::Font(msg.into())
    }
    pub fn export(msg: impl Into<String>) -> Self {
        CoreError::Export(msg.into())
    }
    pub fn state(msg: impl Into<String>) -> Self {
        CoreError::State(msg.into())
    }
}

impl From<CoreError> for JsValue {
    fn from(e: CoreError) -> Self {
        JsValue::from_str(&e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, CoreError>;
