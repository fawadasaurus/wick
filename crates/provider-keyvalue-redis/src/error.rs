use thiserror::Error;
type BoxedError = Box<dyn std::error::Error + Sync + Send>;

#[derive(Error, Debug)]
pub enum Error {
  #[error("Error during initialization: {0}")]
  Init(String),

  #[error("Can't get lock on context")]
  ContextLock,

  #[error("No connection found for '{0}'")]
  ConnectionNotFound(String),

  #[error("Redis command failed: {0}")]
  RedisError(String),

  #[error("Deserialization error {0}")]
  RpcMessageError(&'static str),

  #[error("Client is shutting down, streams are closing")]
  ShuttingDown,
  #[error(transparent)]
  RpcError(#[from] vino_rpc::Error),
  #[error(transparent)]
  CliError(#[from] vino_provider_cli::Error),
  #[error(transparent)]
  IoError(#[from] std::io::Error),
  #[error(transparent)]
  ProviderSdkError(#[from] vino_provider::native::Error),
  #[error(transparent)]
  ComponentError(#[from] vino_packet::error::DeserializationError),
  #[error(transparent)]
  UpstreamError(#[from] BoxedError),
}

pub(crate) enum Exception {
  KeyNotFound(String),
  NothingToDelete(String, String), // Other(String),
}

impl From<Exception> for String {
  fn from(e: Exception) -> Self {
    e.to_string()
  }
}

impl std::fmt::Display for Exception {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Exception::KeyNotFound(v) => write!(f, "Key '{}' not found", v),
      Exception::NothingToDelete(k, v) => write!(f, "Value '{}' not found in '{}'", v, k),
      // Exception::Other(v) => write!(f, "{}", v),
    }
  }
}
