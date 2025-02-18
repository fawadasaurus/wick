pub(crate) mod component_service;
pub(crate) mod error;
pub(crate) mod network_component;

use std::str::FromStr;
use std::sync::Arc;

use flow_graph_interpreter::NamespaceHandler;
use seeded_random::{Random, Seed};
use uuid::Uuid;
use wick_component_wasm::collection::HostLinkCallback;
use wick_component_wasm::error::LinkError;
use wick_config::{ManifestComponent, WasmComponent};
use wick_packet::{Entity, Invocation, PacketStream};

use self::component_service::NativeComponentService;
use crate::dev::prelude::*;
use crate::dispatch::network_invoke_async;
use crate::network_service::ComponentInitOptions;
use crate::BoxFuture;

pub(crate) trait InvocationHandler {
  fn get_signature(&self) -> Result<ComponentSignature>;
  fn invoke(&self, msg: Invocation, stream: PacketStream) -> Result<BoxFuture<Result<InvocationResponse>>>;
}

type Result<T> = std::result::Result<T, ComponentError>;

type CollectionInitResult = Result<NamespaceHandler>;

pub(crate) async fn initialize_wasm_component<'a, 'b>(
  kind: &'a WasmComponent,
  namespace: String,
  opts: ComponentInitOptions<'b>,
) -> CollectionInitResult {
  trace!(namespace = %namespace, ?opts, "registering");

  let component =
    wick_component_wasm::helpers::load_wasm(&kind.reference, opts.allow_latest, &opts.allowed_insecure).await?;

  // TODO take max threads from configuration
  let collection = Arc::new(wick_component_wasm::collection::Collection::try_load(
    &component,
    5,
    Some(kind.permissions.clone()),
    None,
    Some(make_link_callback(opts.network_id)),
  )?);

  let service = NativeComponentService::new(collection);

  Ok(NamespaceHandler::new(namespace, Box::new(service)))
}

fn make_link_callback(network_id: Uuid) -> Box<HostLinkCallback> {
  Box::new(move |origin_url, target_url, stream| {
    let origin_url = origin_url.to_owned();
    let target_url = target_url.to_owned();
    Box::pin(async move {
      {
        debug!(
          origin = %origin_url,
          target = %target_url,
          network_id = %network_id,
          "link_call"
        );

        let target = Entity::from_str(&target_url).map_err(|e| LinkError::EntityFailure(e.to_string()))?;
        let origin = Entity::from_str(&origin_url).map_err(|e| LinkError::EntityFailure(e.to_string()))?;
        if let Entity::Operation(origin_ns, _) = &origin {
          if let Entity::Operation(target_ns, _) = &target {
            if target_ns == origin_ns {
              return Err(LinkError::Circular(target_ns.clone()));
            }
          }
        }

        let invocation = Invocation::new(origin, target, None);
        let result = network_invoke_async(network_id, invocation, stream)
          .await
          .map_err(|e| LinkError::CallFailure(e.to_string()))?;
        Ok(result)
      }
    })
  })
}

// #[instrument(parent=opts.span, skip(kind, opts))]
pub(crate) async fn initialize_network_collection<'a, 'b>(
  kind: &'a ManifestComponent,
  namespace: String,
  mut opts: ComponentInitOptions<'b>,
) -> CollectionInitResult {
  trace!(namespace = %namespace, ?opts, "registering");

  let rng = Random::from_seed(opts.rng_seed);
  opts.rng_seed = rng.seed();
  let uuid = rng.uuid();

  let _network = NetworkService::new_from_manifest(uuid, &kind.reference, Some(namespace.clone()), opts)
    .await
    .map_err(|e| ComponentError::SubNetwork(kind.reference.clone(), e.to_string()))?;

  let collection = Arc::new(network_component::Component::new(uuid));

  let service = NativeComponentService::new(collection);

  Ok(NamespaceHandler::new(namespace, Box::new(service)))
}

pub(crate) fn initialize_native_component(
  namespace: String,
  seed: Seed,
  _span: &tracing::Span,
) -> CollectionInitResult {
  trace!("registering");
  let collection = Arc::new(wick_stdlib::Collection::new(seed));
  let service = NativeComponentService::new(collection);

  Ok(NamespaceHandler::new(namespace, Box::new(service)))
}
