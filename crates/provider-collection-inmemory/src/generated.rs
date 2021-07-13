/**********************************************
***** This file is generated, do not edit *****
***********************************************/

use vino_provider::{
  ComponentSignature,
  VinoProviderComponent,
};

use crate::generated;

pub(crate) fn get_component(
  name: &str,
) -> Option<Box<dyn VinoProviderComponent<Context = crate::State> + Sync + Send>> {
  match name {
    "add-item" => Some(Box::new(generated::add_item::Component::default())),
    "get-item" => Some(Box::new(generated::get_item::Component::default())),
    "list-items" => Some(Box::new(generated::list_items::Component::default())),
    _ => None,
  }
}

pub(crate) fn get_all_components() -> Vec<ComponentSignature> {
  vec![
    ComponentSignature {
      name: "add-item".to_owned(),
      inputs: generated::add_item::inputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
      outputs: generated::add_item::outputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
    },
    ComponentSignature {
      name: "get-item".to_owned(),
      inputs: generated::get_item::inputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
      outputs: generated::get_item::outputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
    },
    ComponentSignature {
      name: "list-items".to_owned(),
      inputs: generated::list_items::inputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
      outputs: generated::list_items::outputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
    },
  ]
}

pub(crate) mod add_item {

  use std::collections::HashMap;
  use std::sync::{
    Arc,
    Mutex,
  };

  use async_trait::async_trait;
  use serde::{
    Deserialize,
    Serialize,
  };
  use vino_codec::messagepack::deserialize;
  use vino_provider::error::ProviderComponentError;
  use vino_provider::{
    Context as ProviderContext,
    VinoProviderComponent,
  };
  pub(crate) use vino_rpc::port::Sender;
  use vino_rpc::port::{
    Port,
    PortStream,
  };

  #[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
  pub(crate) struct Inputs {
    pub(crate) document_id: String,
    pub(crate) collection_id: String,
    pub(crate) document: String,
  }

  pub(crate) fn inputs_list() -> Vec<(&'static str, &'static str)> {
    vec![
      ("document_id", "string"),
      ("collection_id", "string"),
      ("document", "string"),
    ]
  }

  #[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
  pub(crate) struct InputEncoded {
    #[serde(rename = "document_id")]
    pub(crate) document_id: Vec<u8>,
    #[serde(rename = "collection_id")]
    pub(crate) collection_id: Vec<u8>,
    #[serde(rename = "document")]
    pub(crate) document: Vec<u8>,
  }

  pub(crate) fn deserialize_inputs(
    map: &HashMap<String, Vec<u8>>,
  ) -> Result<Inputs, Box<dyn std::error::Error + Send + Sync>> {
    Ok(Inputs {
      document_id: deserialize(map.get("document_id").unwrap())?,
      collection_id: deserialize(map.get("collection_id").unwrap())?,
      document: deserialize(map.get("document").unwrap())?,
    })
  }

  #[derive(Default)]
  pub(crate) struct Outputs {
    pub(crate) document_id: DocumentIdSender,
  }

  pub(crate) fn outputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("document_id", "string")]
  }

  pub(crate) struct DocumentIdSender {
    port: Arc<Mutex<Port>>,
  }
  impl Default for DocumentIdSender {
    fn default() -> Self {
      Self {
        port: Arc::new(Mutex::new(Port::new("document_id".into()))),
      }
    }
  }
  impl Sender for DocumentIdSender {
    type PayloadType = String;

    fn get_port(&self) -> Arc<Mutex<Port>> {
      self.port.clone()
    }
  }

  pub(crate) fn get_outputs() -> (Outputs, PortStream) {
    let outputs = Outputs::default();
    let ports = vec![outputs.document_id.port.clone()];
    let stream = PortStream::new(ports);
    (outputs, stream)
  }

  pub(crate) struct Component {}
  impl Default for Component {
    fn default() -> Self {
      Self {}
    }
  }

  #[async_trait]
  impl VinoProviderComponent for Component {
    type Context = crate::State;

    fn get_name(&self) -> String {
      format!("vino::{}", "add-item")
    }
    fn get_input_ports(&self) -> Vec<(&'static str, &'static str)> {
      inputs_list()
    }
    fn get_output_ports(&self) -> Vec<(&'static str, &'static str)> {
      outputs_list()
    }
    async fn job_wrapper(
      &self,
      context: ProviderContext<Self::Context>,
      data: HashMap<String, Vec<u8>>,
    ) -> Result<PortStream, Box<ProviderComponentError>> {
      let inputs = deserialize_inputs(&data).map_err(|e| {
        ProviderComponentError::new(format!("Input deserialization error: {}", e.to_string()))
      })?;
      let (outputs, stream) = get_outputs();
      let result = crate::components::add_item::job(inputs, outputs, context).await;
      match result {
        Ok(_) => Ok(stream),
        Err(e) => Err(Box::new(ProviderComponentError::new(format!(
          "Job failed: {}",
          e.to_string()
        )))),
      }
    }
  }
}
pub(crate) mod get_item {

  use std::collections::HashMap;
  use std::sync::{
    Arc,
    Mutex,
  };

  use async_trait::async_trait;
  use serde::{
    Deserialize,
    Serialize,
  };
  use vino_codec::messagepack::deserialize;
  use vino_provider::error::ProviderComponentError;
  use vino_provider::{
    Context as ProviderContext,
    VinoProviderComponent,
  };
  pub(crate) use vino_rpc::port::Sender;
  use vino_rpc::port::{
    Port,
    PortStream,
  };

  #[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
  pub(crate) struct Inputs {
    pub(crate) collection_id: String,
    pub(crate) document_id: String,
  }

  pub(crate) fn inputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("collection_id", "string"), ("document_id", "string")]
  }

  #[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
  pub(crate) struct InputEncoded {
    #[serde(rename = "collection_id")]
    pub(crate) collection_id: Vec<u8>,
    #[serde(rename = "document_id")]
    pub(crate) document_id: Vec<u8>,
  }

  pub(crate) fn deserialize_inputs(
    map: &HashMap<String, Vec<u8>>,
  ) -> Result<Inputs, Box<dyn std::error::Error + Send + Sync>> {
    Ok(Inputs {
      collection_id: deserialize(map.get("collection_id").unwrap())?,
      document_id: deserialize(map.get("document_id").unwrap())?,
    })
  }

  #[derive(Default)]
  pub(crate) struct Outputs {
    pub(crate) document: DocumentSender,
  }

  pub(crate) fn outputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("document", "string")]
  }

  pub(crate) struct DocumentSender {
    port: Arc<Mutex<Port>>,
  }
  impl Default for DocumentSender {
    fn default() -> Self {
      Self {
        port: Arc::new(Mutex::new(Port::new("document".into()))),
      }
    }
  }
  impl Sender for DocumentSender {
    type PayloadType = String;

    fn get_port(&self) -> Arc<Mutex<Port>> {
      self.port.clone()
    }
  }

  pub(crate) fn get_outputs() -> (Outputs, PortStream) {
    let outputs = Outputs::default();
    let ports = vec![outputs.document.port.clone()];
    let stream = PortStream::new(ports);
    (outputs, stream)
  }

  pub(crate) struct Component {}
  impl Default for Component {
    fn default() -> Self {
      Self {}
    }
  }

  #[async_trait]
  impl VinoProviderComponent for Component {
    type Context = crate::State;

    fn get_name(&self) -> String {
      format!("vino::{}", "get-item")
    }
    fn get_input_ports(&self) -> Vec<(&'static str, &'static str)> {
      inputs_list()
    }
    fn get_output_ports(&self) -> Vec<(&'static str, &'static str)> {
      outputs_list()
    }
    async fn job_wrapper(
      &self,
      context: ProviderContext<Self::Context>,
      data: HashMap<String, Vec<u8>>,
    ) -> Result<PortStream, Box<ProviderComponentError>> {
      let inputs = deserialize_inputs(&data).map_err(|e| {
        ProviderComponentError::new(format!("Input deserialization error: {}", e.to_string()))
      })?;
      let (outputs, stream) = get_outputs();
      let result = crate::components::get_item::job(inputs, outputs, context).await;
      match result {
        Ok(_) => Ok(stream),
        Err(e) => Err(Box::new(ProviderComponentError::new(format!(
          "Job failed: {}",
          e.to_string()
        )))),
      }
    }
  }
}
pub(crate) mod list_items {

  use std::collections::HashMap;
  use std::sync::{
    Arc,
    Mutex,
  };

  use async_trait::async_trait;
  use serde::{
    Deserialize,
    Serialize,
  };
  use vino_codec::messagepack::deserialize;
  use vino_provider::error::ProviderComponentError;
  use vino_provider::{
    Context as ProviderContext,
    VinoProviderComponent,
  };
  pub(crate) use vino_rpc::port::Sender;
  use vino_rpc::port::{
    Port,
    PortStream,
  };

  #[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
  pub(crate) struct Inputs {
    pub(crate) collection_id: String,
  }

  pub(crate) fn inputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("collection_id", "string")]
  }

  #[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
  pub(crate) struct InputEncoded {
    #[serde(rename = "collection_id")]
    pub(crate) collection_id: Vec<u8>,
  }

  pub(crate) fn deserialize_inputs(
    map: &HashMap<String, Vec<u8>>,
  ) -> Result<Inputs, Box<dyn std::error::Error + Send + Sync>> {
    Ok(Inputs {
      collection_id: deserialize(map.get("collection_id").unwrap())?,
    })
  }

  #[derive(Default)]
  pub(crate) struct Outputs {
    pub(crate) document_ids: DocumentIdsSender,
  }

  pub(crate) fn outputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("document_ids", "[string]")]
  }

  pub(crate) struct DocumentIdsSender {
    port: Arc<Mutex<Port>>,
  }
  impl Default for DocumentIdsSender {
    fn default() -> Self {
      Self {
        port: Arc::new(Mutex::new(Port::new("document_ids".into()))),
      }
    }
  }
  impl Sender for DocumentIdsSender {
    type PayloadType = Vec<String>;

    fn get_port(&self) -> Arc<Mutex<Port>> {
      self.port.clone()
    }
  }

  pub(crate) fn get_outputs() -> (Outputs, PortStream) {
    let outputs = Outputs::default();
    let ports = vec![outputs.document_ids.port.clone()];
    let stream = PortStream::new(ports);
    (outputs, stream)
  }

  pub(crate) struct Component {}
  impl Default for Component {
    fn default() -> Self {
      Self {}
    }
  }

  #[async_trait]
  impl VinoProviderComponent for Component {
    type Context = crate::State;

    fn get_name(&self) -> String {
      format!("vino::{}", "list-items")
    }
    fn get_input_ports(&self) -> Vec<(&'static str, &'static str)> {
      inputs_list()
    }
    fn get_output_ports(&self) -> Vec<(&'static str, &'static str)> {
      outputs_list()
    }
    async fn job_wrapper(
      &self,
      context: ProviderContext<Self::Context>,
      data: HashMap<String, Vec<u8>>,
    ) -> Result<PortStream, Box<ProviderComponentError>> {
      let inputs = deserialize_inputs(&data).map_err(|e| {
        ProviderComponentError::new(format!("Input deserialization error: {}", e.to_string()))
      })?;
      let (outputs, stream) = get_outputs();
      let result = crate::components::list_items::job(inputs, outputs, context).await;
      match result {
        Ok(_) => Ok(stream),
        Err(e) => Err(Box::new(ProviderComponentError::new(format!(
          "Job failed: {}",
          e.to_string()
        )))),
      }
    }
  }
}