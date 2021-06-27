use std::collections::HashMap;

use actix::Recipient;

use crate::component_model::ComponentModel;
use crate::error::{
  ValidationError,
  VinoError,
};
use crate::provider_model::ProviderModel;
use crate::schematic_definition::*;
use crate::{
  Error,
  Invocation,
  Result,
  SchematicDefinition,
};

type ComponentReference = String;
type ComponentId = String;

#[derive(Debug, Default)]
pub(crate) struct SchematicModel {
  pub(crate) definition: SchematicDefinition,
  components: HashMap<ComponentId, ComponentModel>,
  references: HashMap<ComponentReference, String>,
  providers: HashMap<String, ProviderModel>,
}

impl SchematicModel {
  pub(crate) fn new(definition: SchematicDefinition) -> Self {
    let mut references = HashMap::new();
    definition.components.iter().for_each(|(instance, actor)| {
      references.insert(instance.to_string(), actor.id.to_string());
    });
    Self {
      definition,
      references,
      ..SchematicModel::default()
    }
  }
  // Lookup the external component definition for an ID
  pub(crate) fn lookup_external(&self, id: &str) -> Option<ExternalComponentDefinition> {
    for component in &self.definition.external {
      if id == component.key || Some(id.to_string()) == component.alias {
        return Some(component.clone());
      }
    }
    None
  }
  pub(crate) fn get_name(&self) -> String {
    self.definition.get_name()
  }
  pub(crate) fn has_namespace(&self, id: &str) -> bool {
    self.providers.contains_key(id)
  }
  pub(crate) fn has_component(&self, id: &str) -> bool {
    let (ns, name) = parse_namespace(id);
    trace!("ns parts: {:?} and {}", ns, name);
    if let Some(ns) = ns {
      let provider = self.providers.get(&ns);
      if let Some(provider) = provider {
        provider.components.get(&name).is_some()
      } else {
        false
      }
    } else {
      self.components.contains_key(id)
    }
  }
  pub(crate) fn add_provider(&mut self, provider: ProviderModel) -> Result<()> {
    if self.has_namespace(&provider.namespace) {
      Err(Error::SchematicError(format!(
        "Can not add another provider with namespace '{}'",
        provider.namespace
      )))
    } else {
      self.providers.insert(provider.namespace.clone(), provider);
      Ok(())
    }
  }
  pub(crate) fn get_component(&self, reference: &str) -> Option<ComponentModel> {
    self
      .get_component_definition(reference)
      .and_then(|def| self.get_component_model(&def.id))
  }
  /// Gets a ComponentModel by component reference string
  pub(crate) fn get_component_model(&self, id: &str) -> Option<ComponentModel> {
    let (ns, name) = parse_namespace(id);
    trace!("ns parts: {:?} and {}", ns, name);
    if let Some(ns) = ns {
      let provider = self.providers.get(&ns);
      if let Some(provider) = provider {
        provider.components.get(&name).cloned()
      } else {
        None
      }
    } else {
      self.components.get(id).cloned()
    }
  }
  /// Gets a ComponentDefinition by component reference string
  pub(crate) fn get_component_definition(&self, reference: &str) -> Option<ComponentDefinition> {
    self.definition.get_component(reference)
  }
  #[logfn(ok = "TRACE", err = "DEBUG")]
  pub(crate) fn add_component(&mut self, component: ComponentModel) -> Result<()> {
    if self.has_component(&component.id) {
      Err(Error::SchematicError(format!(
        "Can not add another component with id '{}'",
        component.id
      )))
    } else {
      self.components.insert(component.id.to_string(), component);
      Ok(())
    }
  }
  pub(crate) fn get_downstream_connections(&self, reference: &str) -> Vec<ConnectionDefinition> {
    self
      .definition
      .connections
      .iter()
      .filter(|conn| conn.from.instance == reference)
      .cloned()
      .collect()
  }

  pub(crate) fn get_schematic_outputs(&self) -> Vec<String> {
    self.definition.get_output_names()
  }
  pub(crate) fn get_schematic_inputs(&self) -> Vec<String> {
    self.definition.get_input_names()
  }
  #[logfn(ok = "TRACE", err = "DEBUG")]
  pub(crate) fn get_component_metadata(&self, id: &str) -> Result<ComponentModel> {
    trace!("Getting component metadata {}", id);
    let opt = match self.definition.get_component(id) {
      Some(comp) => {
        let (ns, name) = comp.parse_namespace();
        trace!("{} parsed into {:?} | {}", id, ns, name);
        match ns {
          Some(ns) => self
            .providers
            .get(&ns)
            .and_then(|p| p.components.get(&name)),
          None => self.components.get(&comp.id),
        }
      }
      None => None,
    };
    opt.cloned().ok_or_else(|| {
      Error::SchematicError(format!("Could not found component metadata for {}", id))
    })
  }
  pub(crate) fn get_downstream_recipient(&self, reference: &str) -> Option<Recipient<Invocation>> {
    trace!("Getting downstream recipient {}", reference);
    self.get_component_metadata(reference).map(|c| c.addr).ok()
  }
  pub(crate) fn get_outputs(&self, reference: &str) -> Vec<String> {
    match self.references.get(reference) {
      Some(id) => match self.get_component_model(id) {
        Some(component) => component.outputs,
        None => vec![],
      },
      None => vec![],
    }
  }
  pub(crate) fn get_connections(&self, reference: &str, port: &str) -> Vec<ConnectionDefinition> {
    let references = &self.references;
    let connections: Vec<ConnectionDefinition> = self
      .definition
      .connections
      .iter()
      .filter(|connection| connection.from.instance == reference && connection.from.port == port)
      .filter_map(|connection| {
        let from_actor = if connection.from.instance == crate::SCHEMATIC_INPUT {
          Some(&connection.from.instance)
        } else {
          references.get(&connection.from.instance)
        };

        let to_actor = if connection.to.instance == crate::SCHEMATIC_OUTPUT {
          Some(&connection.to.instance)
        } else {
          references.get(&connection.to.instance)
        };
        if from_actor.is_none() || to_actor.is_none() {
          return None;
        }
        Some(connection.clone())
      })
      .collect();
    connections
  }
}

type ValidationResult<T> = std::result::Result<T, ValidationError>;
pub(crate) struct Validator<'a> {
  model: &'a SchematicModel,
}

impl<'a> Validator<'a> {
  pub(crate) fn new(model: &'a SchematicModel) -> Self {
    Validator { model }
  }
  pub(crate) fn validate_early_errors(model: &'a SchematicModel) -> Result<()> {
    let validator = Validator::new(model);
    let name = model.get_name();
    let results: Vec<ValidationError> = vec![
      validator.assert_schematic_outputs(),
      validator.assert_schematic_inputs(),
    ]
    .into_iter()
    .filter_map(|r| r.err())
    .collect();
    if results.is_empty() {
      Ok(())
    } else {
      Err(VinoError::ValidationError(ValidationError::EarlyError(
        name, results,
      )))
    }
  }
  fn assert_schematic_outputs(&self) -> ValidationResult<()> {
    let ports = self.model.get_schematic_outputs();
    if ports.is_empty() {
      Err(ValidationError::NoOutputs)
    } else {
      Ok(())
    }
  }
  fn assert_schematic_inputs(&self) -> ValidationResult<()> {
    let ports = self.model.get_schematic_inputs();
    if ports.is_empty() {
      Err(ValidationError::NoInputs)
    } else {
      Ok(())
    }
  }
}

#[cfg(test)]
mod tests {

  use super::*;
  use crate::prelude::*;
  #[test_env_log::test]
  fn test_basics() -> Result<()> {
    let schematic_name = "Test";
    let mut schematic_def = SchematicDefinition::new(schematic_name.to_string());
    schematic_def.providers.push(ProviderDefinition {
      namespace: "test-namespace".to_string(),
      kind: ProviderKind::Native,
      reference: "internal".to_string(),
      data: HashMap::new(),
    });
    schematic_def.components.insert(
      "logger".to_string(),
      ComponentDefinition {
        metadata: None,
        id: "test-namespace::log".to_string(),
      },
    );
    schematic_def.connections.push(ConnectionDefinition {
      from: ConnectionTargetDefinition {
        instance: SCHEMATIC_INPUT.to_string(),
        port: "input".to_string(),
      },
      to: ConnectionTargetDefinition {
        instance: "logger".to_string(),
        port: "input".to_string(),
      },
    });
    schematic_def.connections.push(ConnectionDefinition {
      from: ConnectionTargetDefinition {
        instance: "logger".to_string(),
        port: "output".to_string(),
      },
      to: ConnectionTargetDefinition {
        instance: SCHEMATIC_OUTPUT.to_string(),
        port: "output".to_string(),
      },
    });
    let model = SchematicModel::new(schematic_def);
    assert_eq!(model.get_name(), schematic_name);
    println!("{:?}", model);
    // let test = model.get_component("logger").unwrap();
    // assert_eq!(test.id, "test-namespace::logger");

    Ok(())
  }
}