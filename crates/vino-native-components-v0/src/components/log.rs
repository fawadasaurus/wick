use vino_provider::Context;

use crate::generated::log::*;

pub(crate) async fn job(
  input: Inputs,
  output: Outputs,
  _context: Context<crate::State>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  println!("Logger: {}", input.input);
  output.output.done(input.input);
  Ok(())
}
