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

use std::collections::HashMap;
use std::str::FromStr;

use enum_primitive_derive::Primitive;
use num_traits::FromPrimitive;
use serde::{
  Deserialize,
  Serialize,
};
use serde_with_expand_env::with_expand_envs;

#[allow(non_snake_case)]
fn HOST_MANIFEST_DEFAULT_SCHEMATIC() -> String {
  "default".to_owned()
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
/// The Host Manifest defines the starting state of a Vino host
pub struct HostManifest {
  /// The manifest version

  #[serde(deserialize_with = "with_expand_envs")]
  pub version: String,
  /// The configuration for a Vino network
  #[serde(default)]
  pub network: NetworkManifest,
  /// The default schematic to execute if none is provided
  #[serde(default = "HOST_MANIFEST_DEFAULT_SCHEMATIC")]
  #[serde(deserialize_with = "with_expand_envs")]
  pub default_schematic: String,
  /// The NATS configuration to connect hosts
  #[serde(default)]
  pub nats: NatsConfiguration,
}

#[allow(non_snake_case)]
fn NATS_CONFIGURATION_RPC_HOST() -> String {
  "0.0.0.0".to_owned()
}
#[allow(non_snake_case)]
fn NATS_CONFIGURATION_RPC_PORT() -> String {
  "4222".to_owned()
}
#[allow(non_snake_case)]
fn NATS_CONFIGURATION_CONTROL_HOST() -> String {
  "0.0.0.0".to_owned()
}
#[allow(non_snake_case)]
fn NATS_CONFIGURATION_CONTROL_PORT() -> String {
  "4222".to_owned()
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
/// The NATS configuration
pub struct NatsConfiguration {
  /// The host for the RPC layer
  #[serde(default = "NATS_CONFIGURATION_RPC_HOST")]
  #[serde(deserialize_with = "with_expand_envs")]
  pub rpc_host: String,
  /// The port for the RPC layer
  #[serde(default = "NATS_CONFIGURATION_RPC_PORT")]
  #[serde(deserialize_with = "with_expand_envs")]
  pub rpc_port: String,
  /// The NATS credsfile for the RPC layer
  #[serde(default)]
  pub rpc_credsfile: Option<String>,
  /// The JSON web token to use for authentication to the RPC layer
  #[serde(default)]
  pub rpc_jwt: Option<String>,
  /// The seed to use for authentication to the RPC layer
  #[serde(default)]
  pub rpc_seed: Option<String>,
  /// The host for the RPC layer
  #[serde(default = "NATS_CONFIGURATION_CONTROL_HOST")]
  #[serde(deserialize_with = "with_expand_envs")]
  pub control_host: String,
  /// The port for the RPC layer
  #[serde(default = "NATS_CONFIGURATION_CONTROL_PORT")]
  #[serde(deserialize_with = "with_expand_envs")]
  pub control_port: String,
  /// The NATS credsfile for the RPC layer
  #[serde(default)]
  pub control_credsfile: Option<String>,
  /// The JSON web token to use for authentication to the RPC layer
  #[serde(default)]
  pub control_jwt: Option<String>,
  /// The seed to use for authentication to the RPC layer
  #[serde(default)]
  pub control_seed: Option<String>,
  /// Enable :latest tags for OCI references
  #[serde(default)]
  #[serde(deserialize_with = "with_expand_envs")]
  pub allow_oci_latest: bool,
  /// A list of insecure registries to allow
  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub allowed_insecure: Vec<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
/// A Vino network definition
pub struct NetworkManifest {
  /// The labels that apply to this host
  #[serde(default)]
  #[serde(skip_serializing_if = "HashMap::is_empty")]
  pub labels: HashMap<String, String>,
  /// The links between capabilities and components
  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub schematics: Vec<SchematicManifest>,
  /// A list of providers and component collections
  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub providers: Vec<ProviderDefinition>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
/// A definition for an individual Vino schematic
pub struct SchematicManifest {
  /// Schematic name

  #[serde(deserialize_with = "with_expand_envs")]
  pub name: String,
  /// A list of providers and component collections
  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub providers: Vec<ProviderDefinition>,
  /// A map from component reference to its target
  #[serde(default)]
  #[serde(skip_serializing_if = "HashMap::is_empty")]
  pub components: HashMap<String, ComponentDefinition>,
  /// A list of connections from component to component
  #[serde(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  #[serde(deserialize_with = "vec_connection")]
  pub connections: Vec<ConnectionDefinition>,
  /// A map of constraints and values that limit where this schematic can run
  #[serde(default)]
  #[serde(skip_serializing_if = "HashMap::is_empty")]
  pub constraints: HashMap<String, String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
/// A provider definition
pub struct ProviderDefinition {
  /// The namespace to reference the provider&#x27;s components on
  #[serde(default)]
  #[serde(deserialize_with = "with_expand_envs")]
  pub namespace: String,
  /// The kind/type of the provider
  #[serde(default)]
  pub kind: ProviderKind,
  /// The reference/location of the provider
  #[serde(default)]
  #[serde(deserialize_with = "with_expand_envs")]
  pub reference: String,
  /// Data or configuration to pass to the provider initialization
  #[serde(default)]
  #[serde(skip_serializing_if = "HashMap::is_empty")]
  pub data: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy, Eq, PartialEq, Primitive)]
#[serde(deny_unknown_fields)]
/// Kind of provider,
pub enum ProviderKind {
  /// Native providers included at compile-time in a Vino host
  Native = 0,
  /// The URL for a separately managed GRPC endpoint
  GrpcUrl = 1,
  /// A WaPC WebAssembly provider
  WaPC = 2,
}

impl Default for ProviderKind {
  fn default() -> Self {
    Self::from_u16(0).unwrap()
  }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
/// A single component definition
pub struct ComponentDefinition {
  /// The ID of the component (i.e. the alias, key, or namespace)

  #[serde(deserialize_with = "with_expand_envs")]
  pub id: String,
  /// Unused (reserved)
  #[serde(default)]
  pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
/// A connection between components
pub struct ConnectionDefinition {
  /// The originating component (upstream)
  #[serde(default)]
  #[serde(deserialize_with = "connection_target_shortform")]
  pub from: Option<ConnectionTargetDefinition>,
  /// The destination component (downstream)
  #[serde(default)]
  #[serde(deserialize_with = "connection_target_shortform")]
  pub to: Option<ConnectionTargetDefinition>,
  /// The default value to provide in the event of an upstream Error or Exception
  #[serde(default)]
  pub default: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
/// A connection target
pub struct ConnectionTargetDefinition {
  /// The component reference

  #[serde(deserialize_with = "with_expand_envs")]
  pub reference: String,
  /// The component&#x27;s port

  #[serde(deserialize_with = "with_expand_envs")]
  pub port: String,
}

impl FromStr for ConnectionDefinition {
  type Err = crate::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    crate::parse::parse_connection_v0(s)
  }
}

impl FromStr for ConnectionTargetDefinition {
  type Err = crate::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    crate::parse::parse_connection_target_v0(s)
  }
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

    fn visit_seq<A: serde::de::SeqAccess<'de>>(
      self,
      mut seq: A,
    ) -> Result<Vec<ConnectionDefinition>, A::Error> {
      let mut v = vec![];
      while let Some(thing) = seq.next_element::<serde_value::Value>()? {
        let result = match thing {
          serde_value::Value::String(s) => ConnectionDefinition::from_str(&s)
            .map_err(|e| serde::de::Error::custom(e.to_string()))?,
          serde_value::Value::Map(map) => ConnectionDefinition::deserialize(
            serde_value::ValueDeserializer::new(serde_value::Value::Map(map)),
          )?,
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

fn connection_target_shortform<'de, D>(
  deserializer: D,
) -> Result<Option<ConnectionTargetDefinition>, D::Error>
where
  D: serde::Deserializer<'de>,
{
  struct ConnectionTargetVisitor;

  impl<'de> serde::de::Visitor<'de> for ConnectionTargetVisitor {
    type Value = Option<ConnectionTargetDefinition>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
      formatter.write_str("a connection target definition")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
      E: serde::de::Error,
    {
      ConnectionTargetDefinition::from_str(s)
        .map(Some)
        .map_err(|e| serde::de::Error::custom(e.to_string()))
    }

    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
      A: serde::de::MapAccess<'de>,
    {
      Ok(Some(ConnectionTargetDefinition::deserialize(
        serde::de::value::MapAccessDeserializer::new(map),
      )?))
    }
  }

  deserializer.deserialize_any(ConnectionTargetVisitor)
}
