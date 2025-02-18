use std::collections::HashMap;
use std::convert::TryInto;
use std::fmt::Display;
use std::hash::Hash;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use wick_interface_types::Field;
use wick_packet::PacketPayload;

use crate::{Error, Result};

#[derive(Debug, Clone, Default)]
/// The SchematicDefinition struct is a normalized representation of a Wick [SchematicManifest].
/// It handles the job of translating manifest versions into a consistent data structure.
#[must_use]
pub struct FlowOperation {
  /// The name of the schematic.
  pub name: String,

  /// A list of the input types for the operation.
  pub inputs: Vec<Field>,

  /// A list of the input types for the operation.
  pub outputs: Vec<Field>,

  /// A mapping of instance names to the components they refer to.
  pub instances: HashMap<String, InstanceReference>,

  /// A list of connections from and to ports on instances defined in the instance map.
  pub connections: Vec<ConnectionDefinition>,

  /// A list of collection namespaces to expose to this schematic.
  pub collections: Vec<String>,

  /// Reserved.
  pub constraints: HashMap<String, String>,
}

impl FlowOperation {
  /// Get the name as an owned [String].
  #[must_use]
  pub fn get_name(&self) -> String {
    self.name.clone()
  }
  /// Get a [ComponentDefinition] by instance name.
  #[must_use]
  pub fn get_component(&self, instance: &str) -> Option<InstanceReference> {
    self.instances.get(instance).cloned()
  }

  /// Get a reference to the [ComponentDefinition] map.
  #[must_use]
  pub fn instances(&self) -> &HashMap<String, InstanceReference> {
    &self.instances
  }
}

#[derive(Debug, Clone, PartialEq)]
/// A definition of a component used to reference a component registered under a collection.
/// Note: [InstanceReference] include embed the concept of a namespace so two identical.
/// components registered on different namespaces will not be equal.
pub struct InstanceReference {
  /// The operation's name.
  pub name: String,
  /// The id of the component.
  pub component_id: String,
  /// Data associated with the component instance.
  pub data: Option<Value>,
}

impl InstanceReference {
  /// Returns the fully qualified ID for the component, i.e. namespace::name.
  #[must_use]
  pub fn id(&self) -> String {
    format!("{}::{}", self.component_id, self.name)
  }
}

impl Display for InstanceReference {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.id())
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// A [ConnectionDefinition] defines the link between an upstream and downstream port as well as.
/// the default value to use in the case of an exception.
#[must_use]
pub struct ConnectionDefinition {
  /// The upstream [ConnectionTargetDefinition] (port).
  pub from: ConnectionTargetDefinition,
  /// The downstream [ConnectionTargetDefinition] (port).
  pub to: ConnectionTargetDefinition,
}

impl Hash for ConnectionDefinition {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.from.hash(state);
    self.to.hash(state);
  }
}

impl PartialEq for ConnectionDefinition {
  fn eq(&self, other: &Self) -> bool {
    self.from == other.from && self.to == other.to
  }
}

impl Eq for ConnectionDefinition {}

impl ConnectionDefinition {
  /// Constructor for a [ConnectionDefinition]. This is mostly used in tests,.
  /// the most common way to get a [ConnectionDefinition] is by parsing a manifest.
  pub fn new(from: ConnectionTargetDefinition, to: ConnectionTargetDefinition) -> Self {
    Self { from, to }
  }

  /// Generate a [ConnectionDefinition] from short form syntax.
  pub fn from_v0_str(s: &str) -> Result<Self> {
    let parsed = crate::v0::parse::parse_connection(s)?;
    (&parsed).try_into()
  }
}

/// Configuration specific to a [ConnectionTargetDefinition].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SenderData {
  pub(crate) inner: Value,
}

impl SenderData {}

impl From<SenderData> for PacketPayload {
  fn from(v: SenderData) -> Self {
    PacketPayload::serialize(v)
  }
}

impl FromStr for SenderData {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self> {
    let value: Value = serde_json::from_str(s).map_err(|e| Error::InvalidSenderData(e.to_string()))?;
    Ok(SenderData { inner: value })
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// A [ConnectionTargetDefinition] is a wrapper around an [Option<PortReference>].
#[must_use]
pub struct ConnectionTargetDefinition {
  pub(crate) target: PortReference,
  pub(crate) data: Option<SenderData>,
}

impl Hash for ConnectionTargetDefinition {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.target.hash(state);
  }
}

impl PartialEq for ConnectionTargetDefinition {
  fn eq(&self, other: &Self) -> bool {
    self.target == other.target
  }
}

impl Eq for ConnectionTargetDefinition {}

impl ConnectionTargetDefinition {
  /// Constructor for a [PortReference]. Used mostly in test code.
  pub fn new<T: AsRef<str>, U: AsRef<str>>(instance: T, port: U) -> Self {
    Self {
      target: PortReference {
        instance: instance.as_ref().to_owned(),
        port: port.as_ref().to_owned(),
      },
      data: None,
    }
  }

  /// Constructor for a [PortReference]. Used mostly in test code.
  pub fn new_with_data<T: AsRef<str>, U: AsRef<str>>(instance: T, port: U, data: Value) -> Self {
    Self {
      target: PortReference {
        instance: instance.as_ref().to_owned(),
        port: port.as_ref().to_owned(),
      },
      data: Some(SenderData { inner: data }),
    }
  }

  /// Create a [ConnectionTargetDefinition] that points nowhere.
  pub fn sender(config: Option<SenderData>) -> Self {
    Self {
      target: PortReference {
        instance: crate::SENDER_ID.to_owned(),
        port: crate::SENDER_PORT.to_owned(),
      },
      data: config,
    }
  }

  /// Getter for the target's [SenderData].
  #[must_use]
  pub fn get_data(&self) -> Option<&SenderData> {
    self.data.as_ref()
  }

  /// Create a [ConnectionTargetDefinition] with the target set to the passed port.
  pub fn from_port(port: PortReference) -> Self {
    Self {
      target: port,
      data: None,
    }
  }

  #[must_use]
  /// Convenience method to test if the target's instance matches the passed string.
  pub fn matches_instance(&self, instance: &str) -> bool {
    self.target.instance == instance
  }

  #[must_use]
  /// Convenience method to test if the target's port matches the passed string.
  pub fn matches_port(&self, port: &str) -> bool {
    self.target.port == port
  }

  /// Get the target's instance.
  #[must_use]
  pub fn get_instance(&self) -> &str {
    &self.target.instance
  }

  /// Get the target's instance as an owned String.
  #[must_use]
  pub fn get_instance_owned(&self) -> String {
    self.target.instance.clone()
  }

  /// Get the target's port.
  #[must_use]
  pub fn get_port(&self) -> &str {
    &self.target.port
  }

  /// Get the target's port as an owned String if it exists.
  #[must_use]
  pub fn get_port_owned(&self) -> String {
    self.target.port.clone()
  }

  /// Generate a [ConnectionTargetDefinition] from short form syntax.
  pub fn from_v0_str(s: &str) -> Result<Self> {
    let parsed = crate::v0::parse::parse_connection_target(s)?;
    parsed.try_into()
  }
}

impl Display for ConnectionDefinition {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{} => {}", self.from, self.to)
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]

/// A [PortReference] is the link to a port for a specific reference of a component.
pub struct PortReference {
  /// A schematic-wide unique reference that maps to a [ComponentDefinition].
  pub instance: String,
  /// A port on the referenced [ComponentDefinition].
  pub port: String,
}

impl PortReference {
  /// Constructor for a [PortReference]. Used mostly in test code.
  pub fn new<T: AsRef<str>, U: AsRef<str>>(instance: T, port: U) -> Self {
    Self {
      instance: instance.as_ref().to_owned(),
      port: port.as_ref().to_owned(),
    }
  }
}

impl Default for PortReference {
  fn default() -> Self {
    Self {
      instance: "<None>".to_owned(),
      port: "<None>".to_owned(),
    }
  }
}

impl<T, U> From<(T, U)> for PortReference
where
  T: AsRef<str>,
  U: AsRef<str>,
{
  fn from((instance, port): (T, U)) -> Self {
    PortReference {
      instance: instance.as_ref().to_owned(),
      port: port.as_ref().to_owned(),
    }
  }
}

impl Display for ConnectionTargetDefinition {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}[{}]", self.target.instance, self.target.port)
  }
}

impl Display for PortReference {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}[{}]", self.instance, self.port)
  }
}

#[cfg(test)]
mod tests {
  use flow_expression_parser::parse_id;

  use super::*;
  #[test_logger::test]
  fn test_parse_id() -> Result<()> {
    let id = "namespace::component_name";
    let (ns, name) = parse_id(id)?;
    assert_eq!(ns, "namespace");
    assert_eq!(name, "component_name");
    let id = "namespace::subns::component_name";
    let (ns, name) = parse_id(id)?;
    assert_eq!(ns, "namespace");
    assert_eq!(name, "subns::component_name");
    Ok(())
  }
}
