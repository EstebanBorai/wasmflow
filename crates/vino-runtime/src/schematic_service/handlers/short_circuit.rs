use crate::dev::prelude::*;
use crate::schematic_service::handlers::output_message::OutputMessage;

#[derive(Message, Clone)]
#[rtype(result = "Result<(), SchematicError>")]
pub(crate) struct ShortCircuit {
  pub(crate) tx_id: String,
  pub(crate) reference: String,
  pub(crate) payload: MessageTransport,
}

impl Handler<ShortCircuit> for SchematicService {
  type Result = ResponseActFuture<Self, Result<(), SchematicError>>;

  fn handle(&mut self, msg: ShortCircuit, ctx: &mut Context<Self>) -> Self::Result {
    trace!("Short circuiting component {}", msg.reference);
    let reference = msg.reference;
    let tx_id = msg.tx_id;
    let payload = msg.payload;

    let outputs = self.get_outputs(&reference);

    trace!("Output ports for {} : {:?}", reference, outputs);

    let downstreams: Vec<Connection> = outputs
      .iter()
      .flat_map(|port| self.get_port_connections(port))
      .collect();

    trace!(
      "Connections to short {:?}",
      Connection::print_all(&downstreams)
    );

    let outputs: Vec<OutputMessage> = downstreams
      .into_iter()
      .map(|conn| OutputMessage {
        tx_id: tx_id.clone(),
        port: conn.from,
        payload: payload.clone(),
      })
      .collect();

    let schematic_host = ctx.address();

    let futures = outputs
      .into_iter()
      .map(move |message| schematic_host.send(message));

    Box::pin(
      async move {
        join_or_err(futures, 6002).await?;
        Ok(())
      }
      .into_actor(self),
    )
  }
}