use anyhow::{Context, Result};
use clap::Args;
use wick_component_cli::options::DefaultCliOptions;
mod manifest;
mod wasm;

#[derive(Debug, Clone, Args)]
pub(crate) struct ServeCommand {
  #[clap(flatten)]
  pub(crate) cli: DefaultCliOptions,

  #[clap(flatten)]
  pub(crate) fetch: super::FetchOptions,

  #[clap(flatten)]
  wasi: crate::wasm::WasiOptions,

  /// The path or OCI URL to a wick manifest or wasm file.
  #[clap(action)]
  pub(crate) location: String,
}

pub(crate) async fn handle_command(opts: ServeCommand) -> Result<()> {
  let _guard = logger::init(&opts.cli.logging.name(crate::BIN_NAME));

  let bytes = wick_loader_utils::get_bytes(&opts.location, opts.fetch.allow_latest, &opts.fetch.insecure_registries)
    .await
    .context("Could not load from location")?;

  if wick_loader_utils::is_wasm(&bytes) {
    wasm::handle_command(opts, bytes).await
  } else {
    manifest::handle_command(opts, bytes).await
  }
}
