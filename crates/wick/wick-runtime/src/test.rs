pub(crate) mod prelude {
  pub(crate) use anyhow::Result;
  pub(crate) use futures::StreamExt;
  pub(crate) use pretty_assertions::assert_eq;

  pub(crate) use super::*;
}

use wick_config::ComponentConfiguration;

use crate::test::prelude::*;
use crate::{Network, NetworkBuilder};

pub(crate) async fn init_network_from_yaml(path: &str) -> Result<(Network, uuid::Uuid)> {
  let def = ComponentConfiguration::load_from_file(path)?;

  let network = NetworkBuilder::from_definition(def)?.build().await?;

  let network_id = network.uid;
  trace!(network_id = %network_id, "network uid");
  Ok((network, network_id))
}
