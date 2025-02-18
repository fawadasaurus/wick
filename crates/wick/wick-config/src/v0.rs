#![deny(
  warnings,
  missing_debug_implementations,
  trivial_casts,
  trivial_numeric_casts,
  unsafe_code,
  unstable_features,
  unused_import_braces,
  unused_qualifications,
  unreachable_pub,
  type_alias_bounds,
  trivial_bounds,
  mutable_transmutes,
  invalid_value,
  explicit_outlives_requirements,
  deprecated,
  clashing_extern_declarations,
  clippy::expect_used,
  clippy::explicit_deref_methods,
  missing_docs
)]
#![warn(clippy::cognitive_complexity)]

pub(crate) mod conversions;
pub(crate) mod parse;

use std::collections::HashMap;
use std::str::FromStr;

use num_traits::FromPrimitive;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with_expand_env::with_expand_envs;

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A manifest defines the starting state of a Wick host and network.
pub(crate) struct HostManifest {
  /// The configuration manifest format.

  #[serde(deserialize_with = "with_expand_envs")]
  pub(crate) format: u32,
  /// The version of the configuration.
  #[serde(default)]
  #[serde(deserialize_with = "with_expand_envs")]
  pub(crate) version: String,
  /// Additional host configuration.
  #[serde(default)]
  pub(crate) host: HostConfig,
  /// The configuration for a Wick network.
  #[serde(default)]
  pub(crate) network: NetworkManifest,
  /// The default schematic to execute if none is provided.
  #[serde(default)]
  pub(crate) default_schematic: Option<String>,
}

#[allow(non_snake_case)]
pub(crate) fn HOST_CONFIG_TIMEOUT() -> u64 {
  5000
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// Host configuration options.
pub(crate) struct HostConfig {
  /// Whether or not to allow the :latest tag on remote artifacts.
  #[serde(default)]
  #[serde(deserialize_with = "with_expand_envs")]
  pub(crate) allow_latest: bool,
  /// A list of registries to connect to insecurely (over HTTP vs HTTPS).
  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) insecure_registries: Vec<String>,
  /// The timeout for network requests (in ms).
  #[serde(default = "HOST_CONFIG_TIMEOUT")]
  #[serde(deserialize_with = "with_expand_envs")]
  pub(crate) timeout: u64,
  /// The ID for this host, used to identify the host over the mesh.
  #[serde(default)]
  pub(crate) id: Option<String>,
  /// The schematics to expose via RPC or the mesh, if any.
  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) expose: Vec<String>,
  /// The mesh configuration.
  #[serde(default)]
  pub(crate) mesh: Option<MeshConfig>,
  /// Configuration for the GRPC server.
  #[serde(default)]
  pub(crate) rpc: Option<HttpConfig>,
  /// Configuration for the HTTP 1 server (development only).
  #[serde(default)]
  pub(crate) http: Option<HttpConfig>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// Configuration for HTTP/S servers.
pub(crate) struct HttpConfig {
  /// Enable/disable the server.
  #[serde(default)]
  #[serde(deserialize_with = "with_expand_envs")]
  pub(crate) enabled: bool,
  /// The port to bind to.
  #[serde(default)]
  pub(crate) port: Option<u16>,
  /// The address to bind to.
  #[serde(default)]
  pub(crate) address: Option<String>,
  /// Path to pem file for TLS.
  #[serde(default)]
  pub(crate) pem: Option<String>,
  /// Path to key file for TLS.
  #[serde(default)]
  pub(crate) key: Option<String>,
  /// Path to CA file.
  #[serde(default)]
  pub(crate) ca: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// Configuration used to connect to the mesh.
pub(crate) struct MeshConfig {
  /// Enable/disable the mesh connection.
  #[serde(default)]
  #[serde(deserialize_with = "with_expand_envs")]
  pub(crate) enabled: bool,
  /// The address of the NATS server.
  #[serde(default)]
  #[serde(deserialize_with = "with_expand_envs")]
  pub(crate) address: String,
  /// The path to the NATS credsfile.
  #[serde(default)]
  pub(crate) creds_path: Option<String>,
  /// The NATS token.
  #[serde(default)]
  pub(crate) token: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A Wick network definition.
pub(crate) struct NetworkManifest {
  /// The unique identifier for this Network.
  #[serde(default)]
  pub(crate) name: Option<String>,
  /// The labels that apply to this network.
  #[serde(default)]
  #[serde(skip_serializing_if = "HashMap::is_empty")]
  pub(crate) labels: HashMap<String, String>,
  /// The collection to use as the entrypoint when running as a standalone process.
  #[serde(default)]
  pub(crate) triggers: Option<EntrypointDefinition>,
  /// The links between capabilities and components.
  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) schematics: Vec<SchematicManifest>,
  /// A list of component collections.
  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) collections: Vec<CollectionDefinition>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A collection definition for the main entrypoint.
pub(crate) struct EntrypointDefinition {
  /// The reference/location of the collection.
  #[serde(default)]
  #[serde(deserialize_with = "with_expand_envs")]
  pub(crate) reference: String,
  /// The component to use as the entrypoint.
  #[serde(default)]
  #[serde(deserialize_with = "with_expand_envs")]
  pub(crate) component: String,
  /// Data or configuration used to initialize the collection.
  #[serde(default)]
  #[serde(deserialize_with = "crate::helpers::deserialize_json_env")]
  pub(crate) data: Value,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A collection definition.
pub(crate) struct CollectionDefinition {
  /// The local namespace for the collection.
  #[serde(default)]
  #[serde(deserialize_with = "with_expand_envs")]
  pub(crate) namespace: String,
  /// The kind/type of the collection.
  #[serde(default)]
  pub(crate) kind: CollectionKind,
  /// The reference/location of the collection.
  #[serde(default)]
  #[serde(deserialize_with = "with_expand_envs")]
  pub(crate) reference: String,
  /// Data or configuration used to initialize the collection.
  #[serde(default)]
  #[serde(deserialize_with = "crate::helpers::deserialize_json_env")]
  pub(crate) data: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy, PartialEq)]
#[serde(deny_unknown_fields)]
/// Kind of collection.
pub(crate) enum CollectionKind {
  /// Native collections included at compile-time in a Wick host.
  Native = 0,
  /// The URL for a separately managed GRPC endpoint.
  GrpcUrl = 1,
  /// A WaPC WebAssembly collection.
  WaPC = 2,
  /// A local or remote Network definition.
  Network = 4,
}

impl Default for CollectionKind {
  fn default() -> Self {
    Self::from_u16(0).unwrap()
  }
}

impl FromPrimitive for CollectionKind {
  fn from_i64(n: i64) -> Option<Self> {
    Some(match n {
      0 => Self::Native,
      1 => Self::GrpcUrl,
      2 => Self::WaPC,
      4 => Self::Network,
      _ => {
        return None;
      }
    })
  }

  fn from_u64(n: u64) -> Option<Self> {
    Some(match n {
      0 => Self::Native,
      1 => Self::GrpcUrl,
      2 => Self::WaPC,
      4 => Self::Network,
      _ => {
        return None;
      }
    })
  }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A definition for an individual Wick schematic.
pub(crate) struct SchematicManifest {
  /// Schematic name.
  #[serde(deserialize_with = "with_expand_envs")]
  pub(crate) name: String,
  /// A list of collections the schematic uses.
  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) collections: Vec<String>,
  /// A map from component reference to its target.
  #[serde(default)]
  #[serde(skip_serializing_if = "HashMap::is_empty")]
  #[serde(deserialize_with = "map_component_def")]
  pub(crate) instances: HashMap<String, ComponentDefinition>,
  /// A list of connections from component to component.
  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  #[serde(deserialize_with = "vec_connection")]
  pub(crate) connections: Vec<ConnectionDefinition>,
  /// A map of constraints and values that limit where this schematic can run.
  #[serde(default)]
  #[serde(skip_serializing_if = "HashMap::is_empty")]
  pub(crate) constraints: HashMap<String, String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A single component definition.
pub(crate) struct ComponentDefinition {
  /// The ID of the component (i.e. the alias, key, or namespace).
  #[serde(deserialize_with = "with_expand_envs")]
  pub(crate) id: String,
  /// Data to associate with the reference.
  #[serde(default)]
  pub(crate) data: Option<Value>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A connection between components. This can be specified in short-form syntax (where applicable).
pub(crate) struct ConnectionDefinition {
  /// The originating component from upstream.
  #[serde(default)]
  #[serde(deserialize_with = "connection_target_shortform")]
  pub(crate) from: ConnectionTargetDefinition,
  /// The destination component (downstream).
  #[serde(default)]
  #[serde(deserialize_with = "connection_target_shortform")]
  pub(crate) to: ConnectionTargetDefinition,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
/// A connection target e.g. a port on a reference. This can be specified in short-form syntax (where applicable).
pub(crate) struct ConnectionTargetDefinition {
  /// The instance name of the referenced component.
  #[serde(deserialize_with = "with_expand_envs")]
  pub(crate) instance: String,
  /// The component&#x27;s port.
  #[serde(deserialize_with = "with_expand_envs")]
  pub(crate) port: String,
  /// Data to associate with a connection.
  #[serde(default)]
  pub(crate) data: Option<Value>,
}

impl FromStr for ComponentDefinition {
  type Err = crate::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(Self {
      id: s.to_owned(),
      data: None,
    })
  }
}

impl FromStr for ConnectionDefinition {
  type Err = crate::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    crate::v0::parse::parse_connection(s)
  }
}

impl FromStr for ConnectionTargetDefinition {
  type Err = crate::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    crate::v0::parse::parse_connection_target(s)
  }
}

fn map_component_def<'de, D>(deserializer: D) -> Result<HashMap<String, ComponentDefinition>, D::Error>
where
  D: serde::Deserializer<'de>,
{
  struct ComponentDefinitionVisitor;
  impl<'de> serde::de::Visitor<'de> for ComponentDefinitionVisitor {
    type Value = HashMap<String, ComponentDefinition>;
    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
      write!(f, "a map of instances to their components")
    }

    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where
      M: serde::de::MapAccess<'de>,
    {
      let mut map = HashMap::with_capacity(access.size_hint().unwrap_or(0));

      while let Some((key, value)) = access.next_entry::<String, serde_value::Value>()? {
        let result = match value {
          serde_value::Value::String(s) => {
            ComponentDefinition::from_str(&s).map_err(|e| serde::de::Error::custom(e.to_string()))?
          }
          serde_value::Value::Map(map) => {
            ComponentDefinition::deserialize(serde_value::ValueDeserializer::new(serde_value::Value::Map(map)))?
          }
          _ => {
            return Err(serde::de::Error::invalid_type(
              serde::de::Unexpected::Other("other"),
              &self,
            ))
          }
        };

        map.insert(key, result);
      }

      Ok(map)
    }
  }

  deserializer.deserialize_map(ComponentDefinitionVisitor)
}

fn vec_connection<'de, D>(deserializer: D) -> Result<Vec<ConnectionDefinition>, D::Error>
where
  D: serde::Deserializer<'de>,
{
  struct ConnectionDefVisitor;
  impl<'de> serde::de::Visitor<'de> for ConnectionDefVisitor {
    type Value = Vec<ConnectionDefinition>;
    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
      write!(f, "a list of connections")
    }

    fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<Vec<ConnectionDefinition>, A::Error> {
      let mut v = vec![];
      while let Some(thing) = seq.next_element::<serde_value::Value>()? {
        let result = match thing {
          serde_value::Value::String(s) => {
            ConnectionDefinition::from_str(&s).map_err(|e| serde::de::Error::custom(e.to_string()))?
          }
          serde_value::Value::Map(map) => {
            ConnectionDefinition::deserialize(serde_value::ValueDeserializer::new(serde_value::Value::Map(map)))?
          }
          _ => {
            return Err(serde::de::Error::invalid_type(
              serde::de::Unexpected::Other("other"),
              &self,
            ))
          }
        };
        v.push(result);
      }
      Ok(v)
    }
  }

  deserializer.deserialize_seq(ConnectionDefVisitor)
}

fn connection_target_shortform<'de, D>(deserializer: D) -> Result<ConnectionTargetDefinition, D::Error>
where
  D: serde::Deserializer<'de>,
{
  struct ConnectionTargetVisitor;

  impl<'de> serde::de::Visitor<'de> for ConnectionTargetVisitor {
    type Value = ConnectionTargetDefinition;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
      formatter.write_str("a connection target definition")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
      E: serde::de::Error,
    {
      ConnectionTargetDefinition::from_str(s).map_err(|e| serde::de::Error::custom(e.to_string()))
    }

    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
      A: serde::de::MapAccess<'de>,
    {
      ConnectionTargetDefinition::deserialize(serde::de::value::MapAccessDeserializer::new(map))
    }
  }

  deserializer.deserialize_any(ConnectionTargetVisitor)
}
