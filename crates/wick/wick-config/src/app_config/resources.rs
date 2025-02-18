#[derive(Debug, Clone, PartialEq)]
/// A definition of a Wick Collection with its namespace, how to retrieve or access it and its configuration.
#[must_use]
pub struct BoundResource {
  /// The id to bind the resource to.
  pub id: String,
  /// The bound resource.
  pub kind: ResourceDefinition,
}

impl BoundResource {
  /// Create a new [CollectionDefinition] with specified name and type.
  pub fn new(name: impl AsRef<str>, kind: ResourceDefinition) -> Self {
    Self {
      id: name.as_ref().to_owned(),
      kind,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Normalized representation of a resource definition.
pub enum ResourceDefinition {
  /// A TCP port.
  TcpPort(TcpPort),
  /// A UDP port.
  UdpPort(UdpPort),
}

impl From<ResourceDefinition> for TcpPort {
  fn from(value: ResourceDefinition) -> Self {
    match value {
      ResourceDefinition::TcpPort(v) => v,
      _ => panic!("Cannot convert non-tcp port to tcp port"),
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Normalized representation of a TCP port configuration.
pub struct TcpPort {
  /// The port number.
  pub port: u16,
  /// The address to bind to.
  pub address: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Normalized representation of a UDP port configuration.
pub struct UdpPort {
  /// The port number.
  pub port: u16,
  /// The address to bind to.
  pub address: String,
}
