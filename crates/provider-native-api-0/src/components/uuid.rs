use crate::generated::uuid::*;

pub(crate) async fn job(
  _input: Inputs,
  output: Outputs,
  _context: Context<crate::State>,
) -> JobResult {
  output.output.done(&uuid::Uuid::new_v4().to_string())?;
  Ok(())
}