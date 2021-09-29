#[path = "./runtime_utils/mod.rs"]
mod utils;
use std::env;

use utils::*;
use vino_entity::Entity;
use vino_invocation_server::{
  bind_new_socket,
  make_rpc_server,
};
use vino_runtime::prelude::TransportWrapper;

#[test_logger::test(actix_rt::test)]
async fn native_component() -> Result<()> {
  let (network, _) = init_network_from_yaml("./manifests/v0/native-component.yaml").await?;

  let data = hashmap! {
      "left" => 42,
      "right" => 302309,
  };

  let mut result = network
    .request("native_component", Entity::test("native component"), &data)
    .await?;

  println!("Result: {:?}", result);
  let mut messages: Vec<TransportWrapper> = result.collect_port("output").await;
  assert_eq!(result.buffered_size(), (0, 0));
  assert_eq!(messages.len(), 1);

  let msg: TransportWrapper = messages.pop().unwrap();
  println!("Output: {:?}", msg);
  let output: i64 = msg.payload.try_into()?;

  equals!(output, 42 + 302309 + 302309);
  Ok(())
}

#[test_logger::test(actix_rt::test)]
async fn global_providers() -> Result<()> {
  let (network, _) = init_network_from_yaml("./manifests/v0/global-provider-def.yaml").await?;

  let data = hashmap! {
      "input" => "some input",
  };

  let mut result = network
    .request("first_schematic", Entity::test("global providers"), &data)
    .await?;

  let mut messages: Vec<TransportWrapper> = result.collect_port("output").await;
  assert_eq!(messages.len(), 1);

  let output: String = messages.pop().unwrap().payload.try_into()?;

  equals!(output, "some input");

  let data = hashmap! {
      "input" => "other input",
  };

  let mut result = network
    .request("second_schematic", Entity::test("global providers"), &data)
    .await?;
  let mut messages: Vec<TransportWrapper> = result.collect_port("output").await;
  assert_eq!(messages.len(), 1);

  let output: String = messages.pop().unwrap().payload.try_into()?;
  println!("Output: {:?}", output);
  equals!(output, "other input");
  Ok(())
}

#[test_logger::test(actix_rt::test)]
async fn subnetworks() -> Result<()> {
  let (network, _) = init_network_from_yaml("./manifests/v0/sub-network-parent.yaml").await?;

  let data = hashmap! {
      "input" => "some input",
  };

  let mut result = network
    .request("parent", Entity::test("subnetworks"), &data)
    .await?;

  let mut messages: Vec<TransportWrapper> = result.collect_port("output").await;
  assert_eq!(messages.len(), 1);

  let output: String = messages.pop().unwrap().payload.try_into()?;

  equals!(output, "some input");

  Ok(())
}

#[test_logger::test(actix_rt::test)]
async fn grpc() -> Result<()> {
  let socket = bind_new_socket()?;
  let port = socket.local_addr()?.port();
  let _ = make_rpc_server(socket, Box::new(test_vino_provider::Provider::default()));
  env::set_var("TEST_PORT", port.to_string());

  let (network, _) = init_network_from_yaml("./manifests/v0/providers/grpc-provider.yaml").await?;
  let user_data = "Hello world";

  let data = hashmap! {
      "input" => user_data,
  };

  let mut result = network.request("grpc", Entity::test("grpc"), &data).await?;

  let mut messages: Vec<TransportWrapper> = result.collect_port("output").await;
  assert_eq!(messages.len(), 1);

  let output: String = messages.pop().unwrap().payload.try_into()?;

  equals!(output, format!("TEST: {}", user_data));

  Ok(())
}