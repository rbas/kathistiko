use std::{net::SocketAddr, path::PathBuf};

use clap::{builder::PathBufValueParser, Arg, Command};
use dashboard::{
    display::spawn_display_worker,
    inbound::http::{router, AppState},
    settings::Settings,
};
use log::info;
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
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let display_worker = spawn_display_worker(app_state.clone());

    info!("Server running at {addr}");
    let server = axum::serve(listener, router(app_state)).with_graceful_shutdown(async {
        if let Err(error) = signal::ctrl_c().await {
            log::error!("Failed to listen for shutdown signal: {error}");
        }
    });

    server.await?;

    if let Some(display_worker) = display_worker {
        display_worker.abort();
        let _ = display_worker.await;
    }

    Ok(())
}
