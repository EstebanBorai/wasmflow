use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Instant;

use parking_lot::RwLock;
use tokio::sync::mpsc::unbounded_channel;
use tokio_stream::wrappers::UnboundedReceiverStream;
use wapc::{WapcHostBuilder, WasiParams};
use wapc_pool::{HostPool, HostPoolBuilder};
use wasmflow_sdk::v1::codec::messagepack::serialize;
use wasmflow_sdk::v1::runtime::HostCommand;
use wasmflow_sdk::v1::transport::{TransportStream, TransportWrapper};
use wasmflow_sdk::v1::types::CollectionSignature;
use wasmflow_wascap::{Claims, CollectionClaims};

use crate::callbacks::{create_link_handler, create_log_handler, create_output_handler};
use crate::collection::HostLinkCallback;
use crate::error::WasmCollectionError;
use crate::transaction::Transaction;
use crate::wapc_module::WapcModule;
use crate::{Error, Result};

#[must_use]
pub struct WasmHostBuilder {
  wasi_params: Option<WasiParams>,
  callback: Option<Box<HostLinkCallback>>,
  min_threads: usize,
  max_threads: usize,
}

impl std::fmt::Debug for WasmHostBuilder {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("WasmHostBuilder")
      .field("wasi_params", &self.wasi_params)
      .finish()
  }
}

impl WasmHostBuilder {
  pub fn new() -> Self {
    Self {
      wasi_params: None,
      callback: None,
      min_threads: 1,
      max_threads: 1,
    }
  }

  pub fn wasi_params(mut self, params: WasiParams) -> Self {
    self.wasi_params = Some(params);
    self
  }

  pub fn link_callback(mut self, callback: Box<HostLinkCallback>) -> Self {
    self.callback = Some(callback);
    self
  }

  pub fn preopened_dirs(mut self, dirs: Vec<String>) -> Self {
    let mut params = self.wasi_params.take().unwrap_or_default();
    params.preopened_dirs = dirs;
    self.wasi_params.replace(params);
    self
  }

  pub fn build(self, module: &WapcModule) -> Result<WasmHost> {
    WasmHost::try_load(
      module,
      self.wasi_params,
      self.callback,
      self.min_threads,
      self.max_threads,
    )
  }

  pub fn max_threads(mut self, max_threads: usize) -> Self {
    self.max_threads = max_threads;
    self
  }

  pub fn min_threads(mut self, min_threads: usize) -> Self {
    self.min_threads = min_threads;
    self
  }
}

impl Default for WasmHostBuilder {
  fn default() -> Self {
    Self::new()
  }
}

#[derive()]
pub struct WasmHost {
  host: HostPool,
  claims: Claims<CollectionClaims>,
  tx_map: Arc<RwLock<HashMap<u32, RwLock<Transaction>>>>,
  rng: seeded_random::Random,
}

impl std::fmt::Debug for WasmHost {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("WasmHost")
      .field("claims", &self.claims)
      .field("tx_map", &self.tx_map)
      .finish()
  }
}

impl WasmHost {
  pub fn try_load(
    module: &WapcModule,
    wasi_options: Option<WasiParams>,
    callback: Option<Box<HostLinkCallback>>,
    min_threads: usize,
    max_threads: usize,
  ) -> Result<Self> {
    let jwt = &module.token.jwt;

    wasmflow_wascap::validate_token::<CollectionClaims>(jwt).map_err(|e| Error::ClaimsInvalid(e.to_string()))?;

    let time = Instant::now();

    let tx_map: Arc<RwLock<HashMap<u32, RwLock<Transaction>>>> = Arc::new(RwLock::new(HashMap::new()));
    let link_callback = Arc::new(callback);

    #[cfg(feature = "wasmtime")]
    let engine = {
      let engine = wasmtime_provider::WasmtimeEngineProvider::new_with_cache(&module.bytes, wasi_options, None)
        .map_err(|e| WasmCollectionError::EngineFailure(e.to_string()))?;
      trace!(duration_μs = %time.elapsed().as_micros(), "wasmtime instance loaded");
      engine
    };

    let engine = Box::new(engine);

    let tx_map_inner = tx_map.clone();

    let pool = HostPoolBuilder::new()
      .name(module.name().clone().unwrap_or_else(|| "wasmtime".to_owned()))
      .factory(move || {
        let handle_port_output = create_output_handler(tx_map_inner.clone());
        let handle_link_call = create_link_handler(link_callback.clone());
        let handle_log_call = create_log_handler();

        let async_callback: Box<wapc::AsyncHostCallback> = Box::new(move |_id, command, arg1, arg2, payload| {
          trace!("wapc callback");
          let handle_link_call = handle_link_call.clone();
          Box::pin(async move {
            trace!(%command, %arg1, %arg2, len = payload.len(), "wapc callback");

            let now = Instant::now();
            let result = match HostCommand::from_str(&command) {
              Ok(HostCommand::Output) => panic!("output is a synchronous function"),
              Ok(HostCommand::LinkCall) => handle_link_call(&arg1, &arg2, &payload).await,
              Ok(HostCommand::Log) => panic!("logging is synchronous"),
              Err(_) => Err(format!("Invalid command: {}", command).into()),
            };
            trace!(
              %command, %arg1, %arg2, duration_μs = %now.elapsed().as_micros(),
              "wapc callback done",
            );
            result
          })
        });

        let sync_callback: Box<wapc::HostCallback> = Box::new(move |_id, command, arg1, arg2, payload| {
          trace!("wapc callback");
          let handle_port_output = handle_port_output.clone();
          let handle_log_call = handle_log_call.clone();

          trace!(%command, %arg1, %arg2, len = payload.len(), "wapc callback");

          let now = Instant::now();
          let result = match HostCommand::from_str(&command) {
            Ok(HostCommand::Output) => handle_port_output(&arg1, &arg2, &payload),
            Ok(HostCommand::LinkCall) => panic!("external calls are asynchronous"),
            Ok(HostCommand::Log) => handle_log_call(&arg1, &arg2, &payload),
            Err(_) => Err(format!("Invalid command: {}", command).into()),
          };
          trace!(
            %command, %arg1, %arg2, duration_μs = %now.elapsed().as_micros(),
            "wapc callback done",
          );
          result
        });

        WapcHostBuilder::new()
          .callback(Arc::new(sync_callback))
          .async_callback(Arc::new(async_callback))
          .build(engine.clone())
          .unwrap()
      })
      .min_threads(min_threads)
      .max_threads(max_threads)
      .build();

    debug!(duration_μs = ?time.elapsed().as_micros(), "wasmtime initialize");

    Ok(Self {
      claims: module.claims().clone(),
      host: pool,
      tx_map,
      rng: seeded_random::Random::new(),
    })
  }

  fn new_tx(&self) -> u32 {
    let mut id = self.rng.u32();
    while self.tx_map.read().contains_key(&id) {
      id = self.rng.u32();
    }
    self.tx_map.write().insert(id, RwLock::new(Transaction::default()));
    id
  }

  fn take_tx(&self, id: u32) -> Result<RwLock<Transaction>> {
    self.tx_map.write().remove(&id).ok_or(WasmCollectionError::TxNotFound)
  }

  pub async fn call(
    &self,
    component_name: &str,
    input_map: &HashMap<String, Vec<u8>>,
    config: Option<Vec<u8>>,
  ) -> Result<TransportStream> {
    let id = self.new_tx();

    debug!(component = component_name, id, payload = ?input_map, "wasm invoke");

    let payload = serialize(&(id, &input_map, config)).map_err(|e| Error::SdkError(e.into()))?;

    let now = Instant::now();
    let result = self.host.call(component_name, payload).await;
    trace!(
      component = component_name,
      id,
      duration_μs = ?now.elapsed().as_micros(),
      "wasm call finished"
    );
    trace!(component = component_name, id, ?result, "wasm call result");
    let transaction = self.take_tx(id)?;
    if let Err(e) = result {
      return Err(e.into());
    };
    let (tx, rx) = unbounded_channel();
    let mut locked = transaction.write();
    while let Some((port, payload)) = locked.buffer.pop_front() {
      let transport = TransportWrapper {
        port,
        payload: payload.into(),
      };
      tx.send(transport).map_err(|_| Error::SendError)?;
    }
    Ok(TransportStream::new(UnboundedReceiverStream::new(rx)))
  }

  pub fn get_components(&self) -> &CollectionSignature {
    let claims = &self.claims;
    &claims.metadata.as_ref().unwrap().interface
  }
}
