use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

pub(super) mod collection_collection;
pub(super) mod core_collection;
pub(crate) mod internal_collection;
pub(super) mod schematic_collection;

use serde_json::Value;
use wick_interface_types::ComponentSignature;
use wick_packet::{Invocation, PacketStream, StreamMap};

use self::core_collection::CoreCollection;
use self::internal_collection::InternalCollection;
use crate::constants::*;
use crate::graph::types::Network;
use crate::{BoxError, BoxFuture};

pub(crate) type ComponentMap = HashMap<String, ComponentSignature>;

#[derive(Debug)]
#[must_use]
pub struct HandlerMap {
  collections: HashMap<String, NamespaceHandler>,
}

impl Default for HandlerMap {
  fn default() -> Self {
    Self::new(Vec::new())
  }
}

impl HandlerMap {
  pub fn new(components: Vec<NamespaceHandler>) -> Self {
    let mut map = Self {
      collections: Default::default(),
    };
    for collection in components {
      map.add(collection);
    }

    map.add(NamespaceHandler {
      namespace: NS_INTERNAL.to_owned(),
      collection: Arc::new(Box::new(InternalCollection::default())),
    });

    map
  }

  pub fn add_core(&mut self, network: &Network) {
    self.add(NamespaceHandler {
      namespace: NS_CORE.to_owned(),
      collection: Arc::new(Box::new(CoreCollection::new(network))),
    });
  }

  #[must_use]
  pub fn collections(&self) -> &HashMap<String, NamespaceHandler> {
    &self.collections
  }

  #[must_use]
  pub fn collection_signatures(&self) -> ComponentMap {
    self
      .collections
      .iter()
      .map(|(name, p)| (name.clone(), p.collection.list().clone()))
      .collect::<HashMap<String, ComponentSignature>>()
  }

  #[must_use]
  pub fn get(&self, namespace: &str) -> Option<&NamespaceHandler> {
    self.collections.get(namespace)
  }

  pub fn add(&mut self, component: NamespaceHandler) {
    trace!(namespace = %component.namespace, "adding component");
    self.collections.insert(component.namespace.clone(), component);
  }
}

pub(crate) fn dyn_component_id(name: &str, schematic: &str, instance: &str) -> String {
  format!("{}<{}::{}>", name, schematic, instance)
}

pub(crate) fn get_id(ns: &str, name: &str, schematic: &str, instance: &str) -> String {
  if ns == NS_CORE && name == CORE_ID_MERGE {
    dyn_component_id(name, schematic, instance)
  } else {
    name.to_owned()
  }
}

#[derive(Clone)]
#[must_use]
pub struct NamespaceHandler {
  pub(crate) namespace: String,
  pub(crate) collection: Arc<Box<dyn Component + Send + Sync>>,
}

impl NamespaceHandler {
  pub fn new<T: AsRef<str>>(namespace: T, collection: Box<dyn Component + Send + Sync>) -> Self {
    Self {
      namespace: namespace.as_ref().to_owned(),
      collection: Arc::new(collection),
    }
  }
}

impl Debug for NamespaceHandler {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("NamespaceHandler")
      .field("namespace", &self.namespace)
      .field("collection", &self.collection.list())
      .finish()
  }
}

pub trait Component {
  fn handle(
    &self,
    invocation: Invocation,
    stream: PacketStream,
    data: Option<Value>,
  ) -> BoxFuture<Result<PacketStream, BoxError>>;
  fn list(&self) -> &ComponentSignature;
  fn shutdown(&self) -> BoxFuture<Result<(), BoxError>> {
    // Override if you need a more explicit shutdown.
    Box::pin(async move { Ok(()) })
  }
}

pub trait Operation {
  fn handle(&self, payload: StreamMap, data: Option<Value>) -> BoxFuture<Result<PacketStream, BoxError>>;
}
