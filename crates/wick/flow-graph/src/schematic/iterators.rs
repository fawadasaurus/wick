mod connection;
mod node;
mod port;

pub use connection::*;
pub use node::*;
pub use port::*;

use crate::node::NodePort;
use crate::port::PortDirection;
use crate::{Connection, ConnectionIndex, PortReference, Schematic};

#[derive(Debug, Clone)]
pub enum SchematicHop<'graph, DATA>
where
  DATA: Clone,
{
  Node(NodeHop<'graph, DATA>),
  Port(Port<'graph, DATA>),
  Ports(Ports<'graph, DATA>),
  Connections(Connections<'graph, DATA>),
  Connection(ConnectionRef<'graph, DATA>),
}

impl<'graph, DATA> std::fmt::Display for SchematicHop<'graph, DATA>
where
  DATA: Clone,
{
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      match self {
        SchematicHop::Node(v) => format!("Node:{}", v.name()),
        SchematicHop::Port(v) => format!("Port:{}", v),
        SchematicHop::Ports(v) => format!("Ports:{}", v),
        SchematicHop::Connections(v) => format!("Connections:{}", v),
        SchematicHop::Connection(v) => format!("Connection:{}", v),
      }
    )
  }
}

impl<'graph, DATA> From<NodeHop<'graph, DATA>> for SchematicHop<'graph, DATA>
where
  DATA: Clone,
{
  fn from(node: NodeHop<'graph, DATA>) -> Self {
    Self::Node(node)
  }
}

#[derive(Debug, Clone, Copy)]
#[must_use]
pub enum WalkDirection {
  Up,
  Down,
}

#[derive(Debug)]
#[must_use]
pub struct SchematicWalker<'graph, DATA>
where
  DATA: Clone,
{
  schematic: &'graph Schematic<DATA>,
  last_hop: Option<SchematicHop<'graph, DATA>>,
  hop_queue: Vec<SchematicHop<'graph, DATA>>,
  direction: WalkDirection,
}

impl<'graph, DATA> SchematicWalker<'graph, DATA>
where
  DATA: Clone,
{
  pub fn new_from_input(schematic: &'graph Schematic<DATA>) -> Self {
    let inputs = NodeHop::new(schematic, schematic.input).into_inputs();
    let hop_queue = vec![SchematicHop::Ports(inputs)];
    Self {
      schematic,
      last_hop: None,
      hop_queue,
      direction: WalkDirection::Down,
    }
  }

  pub fn new_from_output(schematic: &'graph Schematic<DATA>) -> Self {
    let hop_queue = vec![SchematicHop::Node(NodeHop::new(schematic, schematic.output))];
    Self {
      schematic,
      last_hop: None,
      hop_queue,
      direction: WalkDirection::Up,
    }
  }

  pub fn from_port(schematic: &'graph Schematic<DATA>, port: PortReference, direction: WalkDirection) -> Self {
    let port = Port::new(schematic, port);
    let hop_queue = vec![SchematicHop::Port(port)];
    Self {
      schematic,
      last_hop: None,
      hop_queue,
      direction,
    }
  }
}

impl<'graph, DATA> Iterator for SchematicWalker<'graph, DATA>
where
  DATA: Clone,
{
  type Item = SchematicHop<'graph, DATA>;

  fn next(&mut self) -> Option<SchematicHop<'graph, DATA>> {
    let last_hop = self.last_hop.take();
    let (mut next_hop, branch) = walk(self.schematic, last_hop, self.direction);

    if let Some(branch) = branch {
      self.hop_queue.push(branch);
    }
    if next_hop.is_none() {
      next_hop = self.hop_queue.pop();
    }
    self.last_hop = next_hop.clone();
    next_hop
  }
}

#[allow(clippy::too_many_lines)]
fn walk<'graph, DATA>(
  schematic: &'graph Schematic<DATA>,
  hop: Option<SchematicHop<'graph, DATA>>,
  direction: WalkDirection,
) -> (Option<SchematicHop<'graph, DATA>>, Option<SchematicHop<'graph, DATA>>)
where
  DATA: Clone,
{
  match hop {
    Some(hop) => match hop {
      SchematicHop::Node(v) => {
        let ports = match direction {
          WalkDirection::Up => v.into_inputs(),
          WalkDirection::Down => v.into_outputs(),
        };
        if ports.is_empty() {
          (None, None)
        } else {
          (Some(SchematicHop::Ports(ports)), None)
        }
      }
      SchematicHop::Port(v) => match direction {
        WalkDirection::Down => match v.port.direction {
          PortDirection::In => {
            let node = NodeHop::new(schematic, v.port.node_index);
            (Some(node.into()), None)
          }
          PortDirection::Out => {
            let connections = schematic
              .get(v.port.node_index)
              .and_then(|node| node.output_connections(v.port.port_index))
              .map(|indices| Connections::new(schematic, indices.clone()))
              .unwrap();

            if connections.is_empty() {
              (None, None)
            } else {
              (Some(SchematicHop::Connections(connections)), None)
            }
          }
        },
        WalkDirection::Up => match v.port.direction {
          PortDirection::In => {
            let connections = schematic
              .get(v.port.node_index)
              .and_then(|node| node.input_connections(v.port.port_index))
              .map(|indices| Connections::new(schematic, indices.clone()))
              .unwrap();

            if connections.is_empty() {
              (None, None)
            } else {
              (Some(SchematicHop::Connections(connections)), None)
            }
          }
          PortDirection::Out => {
            let node = NodeHop::new(schematic, v.port.node_index);
            (Some(node.into()), None)
          }
        },
      },
      SchematicHop::Ports(mut v) => {
        if v.direction.is_none() {
          return (None, None);
        }
        let port_direction = v.direction.unwrap();
        match direction {
          WalkDirection::Up => match port_direction {
            PortDirection::In => {
              let result = v.next();
              let rest = if result.is_none() {
                None
              } else {
                Some(SchematicHop::Ports(v))
              };
              (result.map(SchematicHop::Port), rest)
            }
            PortDirection::Out => {
              let node = NodeHop::new(schematic, v.node_index);
              (Some(node.into()), None)
            }
          },
          WalkDirection::Down => match port_direction {
            PortDirection::In => {
              let node = NodeHop::new(schematic, v.node_index);
              (Some(node.into()), None)
            }
            PortDirection::Out => {
              let result = v.next();
              let rest = if result.is_none() {
                None
              } else {
                Some(SchematicHop::Ports(v))
              };
              (result.map(SchematicHop::Port), rest)
            }
          },
        }
      }
      SchematicHop::Connection(v) => {
        let connection = &schematic.connections[v.index];
        match direction {
          WalkDirection::Up => {
            let port = Port::new(schematic, connection.from);
            (Some(SchematicHop::Port(port)), None)
          }
          WalkDirection::Down => {
            let port = Port::new(schematic, connection.to);
            (Some(SchematicHop::Port(port)), None)
          }
        }
      }
      SchematicHop::Connections(mut v) => {
        let result = v.next();
        let rest = if result.is_none() {
          None
        } else {
          Some(SchematicHop::Connections(v))
        };

        (result.map(SchematicHop::Connection), rest)
      }
    },
    None => (None, None),
  }
}

fn get_ports_node<'graph, DATA>(schematic: &'graph Schematic<DATA>, port: &PortReference) -> &'graph NodePort
where
  DATA: Clone,
{
  match port.direction {
    PortDirection::In => &schematic.nodes[port.node_index].inputs()[port.port_index],
    PortDirection::Out => &schematic.nodes[port.node_index].outputs()[port.port_index],
  }
}

fn get_port_name<'graph, DATA>(schematic: &'graph Schematic<DATA>, port: &PortReference) -> &'graph str
where
  DATA: Clone,
{
  match port.direction {
    PortDirection::In => schematic.nodes[port.node_index].inputs()[port.port_index].name(),
    PortDirection::Out => schematic.nodes[port.node_index].outputs()[port.port_index].name(),
  }
}

fn get_port_connections<'graph, DATA>(
  schematic: &'graph Schematic<DATA>,
  port: &PortReference,
) -> Connections<'graph, DATA>
where
  DATA: Clone,
{
  let direction = port.direction;
  schematic
    .get(port.node_index)
    .and_then(|node| match direction {
      PortDirection::In => node.input_connections(port.port_index),
      PortDirection::Out => node.output_connections(port.port_index),
    })
    .map(|indices| Connections::new(schematic, indices.clone()))
    .unwrap()
}

fn get_connection<DATA>(schematic: &Schematic<DATA>, index: ConnectionIndex) -> &Connection<DATA> {
  &schematic.connections[index]
}
