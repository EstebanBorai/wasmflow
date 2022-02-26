pub use crate::components::generated::math::subtract::*;

pub(crate) async fn job(input: Inputs, output: OutputPorts, _context: crate::Context) -> JobResult {
  let result = input.left - input.right;
  output.output.done(Payload::success(&result))?;
  Ok(())
}