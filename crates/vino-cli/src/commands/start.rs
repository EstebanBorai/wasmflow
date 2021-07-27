use std::path::PathBuf;

use structopt::StructOpt;
use vino_host::{
  HostBuilder,
  HostDefinition,
};
use vino_provider_cli::cli::DefaultCliOptions;

use crate::utils::merge_runconfig;
use crate::Result;
#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct StartCommand {
  #[structopt(flatten)]
  pub logging: super::LoggingOptions,

  #[structopt(flatten)]
  pub host: super::HostOptions,

  /// Specifies a manifest file to apply to the host once started
  #[structopt(parse(from_os_str))]
  pub manifest: Option<PathBuf>,

  #[structopt(flatten)]
  pub server_options: DefaultCliOptions,
}

pub async fn handle_command(command: StartCommand) -> Result<String> {
  crate::utils::init_logger(&command.logging)?;

  let config = match command.manifest {
    Some(file) => vino_host::HostDefinition::load_from_file(&file)?,
    None => HostDefinition::default(),
  };

  let config = merge_runconfig(config, command.host);

  let host_builder = HostBuilder::new();

  let mut host = host_builder.build();

  debug!("Starting host");
  match host.start().await {
    Ok(_) => {
      debug!("Applying manifest");
      host.start_network(config.network).await?;
      info!("Manifest applied");
      let metadata = host
        .start_rpc_server(Some(command.server_options.into()))
        .await?;
      println!("Server bound to {}", metadata.rpc_addr.unwrap());
    }
    Err(e) => {
      error!("Failed to start host: {}", e);
    }
  }

  info!("Waiting for Ctrl-C");
  actix_rt::signal::ctrl_c().await.unwrap();
  info!("Ctrl-C received, shutting down");
  host.stop().await;
  Ok("Done".to_owned())
}
