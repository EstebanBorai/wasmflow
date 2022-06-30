use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

use anyhow::Result;
use clap::Args;
use nkeys::KeyPairType;
use wasmflow_sdk::v1::types::CollectionSignature;
use wasmflow_wascap::{sign_buffer_with_claims, ClaimsOptions};

use crate::keys::{extract_keypair, GenerateCommon};
#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
pub(crate) struct Options {
  #[clap(flatten)]
  pub(crate) logging: logger::LoggingOptions,

  /// WebAssembly module location.
  #[clap(action)]
  pub(crate) source: String,

  /// File path to the JSON representation of the module's interface.
  #[clap(action)]
  pub(crate) interface: String,

  /// Destination for signed module. If omitted, the signed module will have a .signed.wasm extension.
  #[clap(short = 'd', long = "destination", action)]
  destination: Option<String>,

  #[clap(flatten)]
  common: GenerateCommon,

  /// Version to embed in the module.
  #[clap(long, action)]
  ver: Option<String>,

  /// Revision number to embed in the module.
  #[clap(long, action)]
  rev: Option<u32>,
}

#[allow(clippy::unused_async)]
pub(crate) async fn handle(opts: Options) -> Result<()> {
  let _guard = crate::utils::init_logger(&opts.logging)?;
  debug!("Signing module");

  debug!("Reading from {}", opts.interface);
  let json = std::fs::read_to_string(opts.interface)?;
  debug!("Read {} bytes", json.len());

  let interface: CollectionSignature = serde_json::from_str(&json)?;

  let mut sfile = File::open(&opts.source).unwrap();
  let mut buf = Vec::new();
  sfile.read_to_end(&mut buf).unwrap();

  let issuer = extract_keypair(
    Some(opts.source.clone()),
    opts.common.directory.clone(),
    KeyPairType::Account,
  )
  .await?;

  let subject = extract_keypair(
    Some(opts.source.clone()),
    opts.common.directory.clone(),
    KeyPairType::Module,
  )
  .await?;

  debug!("Signing module with interface : {:?}", interface);
  let signed = sign_buffer_with_claims(
    &buf,
    interface,
    &subject,
    &issuer,
    ClaimsOptions {
      revision: opts.rev,
      version: opts.ver,
      expires_in_days: opts.common.expires_in_days,
      not_before_days: opts.common.not_before,
    },
  )?;

  let destination = match opts.destination.clone() {
    Some(d) => d,
    None => {
      let path = PathBuf::from(opts.source.clone())
        .parent()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();
      let module_name = PathBuf::from(opts.source)
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();
      // If path is empty, user supplied module in current directory
      if path.is_empty() {
        format!("./{}.signed.wasm", module_name)
      } else {
        format!("{}/{}.signed.wasm", path, module_name)
      }
    }
  };
  debug!("Destination : {}", destination);

  let mut outfile = File::create(&destination).unwrap();
  match outfile.write(&signed) {
    Ok(_) => {
      info!("Successfully signed {}", destination,);
    }
    Err(e) => {
      error!("Error signing: {}", e);
    }
  }

  Ok(())
}