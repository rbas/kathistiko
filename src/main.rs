use std::{net::SocketAddr, path::PathBuf};

use clap::{builder::PathBufValueParser, Arg, Command};
use dashboard::{
    inbound::http::{spawn_web_server, AppState},
    settings::Settings,
};
use tokio::signal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let matches = Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .long_about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .help("Path to configuration file")
                .value_parser(PathBufValueParser::default()),
        )
        .get_matches();

    let default_file = PathBuf::from("config.local.toml");
    let config_path = matches.get_one("config").unwrap_or(&default_file);

    let settings = Settings::new(config_path.as_path())?;

    let addr = settings.server_listen_at.parse::<SocketAddr>().expect(
        "Expecting configuration `server_listen_at` to be in the format like `127.0.0.1:8042`.",
    );

    let app_state = AppState::new(settings);

    // Define the address and port to listen on
    tokio::spawn(spawn_web_server(addr, app_state));

    signal::ctrl_c().await?;

    Ok(())
}
