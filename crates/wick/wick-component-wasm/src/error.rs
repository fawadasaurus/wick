use thiserror::Error;
use wick_rpc::error::RpcError;

#[derive(Error, Debug)]
pub enum WasmCollectionError {
  #[error("Could not extract claims signature from WASM module : {0}")]
  ClaimsError(String),

  #[error("Could not validate claims : {0}")]
  ClaimsInvalid(String),

  #[error("Component error : {0}")]
  ComponentError(String),

  #[error("WASM collection requested data for a nonexistent call.")]
  TxNotFound,

  #[error(transparent)]
  WasmRS(#[from] wasmrs::Error),

  #[error(transparent)]
  IoError(#[from] std::io::Error),

  #[error("Could not load reference: {0}")]
  Loading(String),

  #[error("JSON Serialization/Deserialization error : {0}")]
  JsonError(String),

  #[error("WebAssembly engine failed: {0}")]
  EngineFailure(String),

  #[error("Could not extract claims from component. Is it a signed WebAssembly module?")]
  ClaimsExtraction,

  #[error("Error sending output to stream.")]
  SendError,

  #[error("{0}")]
  Other(String),

  #[error("Internal Error : {0}")]
  InternalError(u32),

  #[error("Operation '{0}' not found. Valid operations are: {}", .1.join(", "))]
  OperationNotFound(String, Vec<String>),
}

impl From<serde_json::error::Error> for WasmCollectionError {
  fn from(e: serde_json::error::Error) -> Self {
    WasmCollectionError::JsonError(e.to_string())
  }
}

impl From<WasmCollectionError> for Box<RpcError> {
  fn from(e: WasmCollectionError) -> Self {
    Box::new(RpcError::Component(e.to_string()))
  }
}

impl From<wick_loader_utils::Error> for WasmCollectionError {
  fn from(e: wick_loader_utils::Error) -> Self {
    WasmCollectionError::Loading(e.to_string())
  }
}

#[derive(Error, Debug)]
pub enum LinkError {
  #[error("{0}")]
  EntityFailure(String),
  #[error("Component '{0}' can't call a link to itself.")]
  Circular(String),
  #[error("{0}")]
  CallFailure(String),
}
