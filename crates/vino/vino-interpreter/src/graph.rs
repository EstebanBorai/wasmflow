pub mod types {
  pub(crate) static INHERENT_COMPONENT: usize = 2;
  pub(crate) type Network = vino_schematic_graph::Network<serde_json::Value>;
  pub(crate) type Component = vino_schematic_graph::Component<serde_json::Value>;
  pub(crate) type ComponentPort = vino_schematic_graph::ComponentPort;
  pub(crate) type Schematic = vino_schematic_graph::Schematic<serde_json::Value>;
  pub(crate) type Port<'a> = vino_schematic_graph::iterators::Port<'a, serde_json::Value>;
}

use types::*;
use vino_manifest::parse::CORE_ID;
use vino_schematic_graph::{ComponentReference, SCHEMATIC_OUTPUT};

use crate::constants::{INTERNAL_ID_INHERENT, NS_CORE, NS_INTERNAL, NS_PROVIDERS};

#[derive(Debug)]
#[must_use]
pub(crate) struct Reference(ComponentReference);

impl From<&ComponentReference> for Reference {
  fn from(v: &ComponentReference) -> Self {
    Self(v.clone())
  }
}

impl Reference {
  pub(crate) fn name(&self) -> &str {
    self.0.name()
  }
  pub(crate) fn namespace(&self) -> &str {
    self.0.namespace()
  }

  pub(crate) fn is_core_component(&self, name: &str) -> bool {
    self.0.namespace() == NS_CORE && self.0.name() == name
  }

  pub(crate) fn is_schematic_output(&self) -> bool {
    self.0.namespace() == NS_INTERNAL && self.0.name() == SCHEMATIC_OUTPUT
  }

  pub(crate) fn is_static(&self) -> bool {
    self.0.namespace() == NS_PROVIDERS
  }
}

#[instrument(name = "schematic_graph", skip_all)]
pub fn from_def(network_def: &vino_manifest::NetworkDefinition) -> Result<Network, vino_schematic_graph::error::Error> {
  let mut network = Network::new(network_def.name.clone().unwrap_or_default());

  for schem_def in &network_def.schematics {
    let mut schematic = Schematic::new(schem_def.name.clone());

    let index = schematic.add_inherent(
      CORE_ID,
      ComponentReference::new(NS_INTERNAL, INTERNAL_ID_INHERENT),
      None,
    );
    trace!(index, name = INTERNAL_ID_INHERENT, "added inherent component");

    for (name, def) in schem_def.instances.iter() {
      schematic.add_external(
        name,
        ComponentReference::new(&def.namespace, &def.name),
        def.data.clone(),
      );
    }

    for connection in &schem_def.connections {
      let from = &connection.from;
      let to = &connection.to;
      let to_port = schematic
        .find_mut(to.get_instance())
        .map(|component| component.add_input(to.get_port()));

      assert!(
        to_port.is_some(),
        "Could not find downstream instance '{}'",
        to.get_instance(),
      );
      let to_port = to_port.unwrap();

      if let Some(component) = schematic.find_mut(from.get_instance()) {
        let from_port = component.add_output(from.get_port());
        schematic.connect(from_port, to_port, connection.default.clone())?;
      } else {
        panic!("Can't find component {}", from.get_instance());
      }
    }
    network.add_schematic(schematic);
  }
  Ok(network)
}