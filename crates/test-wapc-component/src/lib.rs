mod generated;
extern crate wapc_guest as guest;
use std::error::Error;
use std::fmt::Display;
use std::usize;

pub use generated::*;
use guest::prelude::*;

#[no_mangle]
pub fn wapc_init() {
  Handlers::register_job(job);
}
#[derive(Debug)]
enum LengthError {
  TooShort,
  TooLong,
}

impl Display for LengthError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      match self {
        LengthError::TooShort => format!("Needs to be longer than {} characters", MINIMUM_LENGTH),
        LengthError::TooLong => format!("Needs to be shorter than {} characters", MAXIMUM_LENGTH),
      }
    )
  }
}

impl Error for LengthError {
  fn source(&self) -> Option<&(dyn Error + 'static)> {
    Some(self)
  }
}

static MINIMUM_LENGTH: usize = 8;
static MAXIMUM_LENGTH: usize = 512;

fn job(input: Inputs, output: Outputs) -> HandlerResult<()> {
  let password = input.input;
  if password.len() < MINIMUM_LENGTH {
    output.output.exception(LengthError::TooShort.to_string())?;
    return Ok(());
  }
  if password.len() > MAXIMUM_LENGTH {
    output.output.exception(LengthError::TooLong.to_string())?;
    return Ok(());
  }
  output.output.send(password)?;

  Ok(())
}