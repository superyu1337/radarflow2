use std::sync::Arc;

use clap::Parser;
use cli::Cli;
use comms::RadarData;
use tokio::sync::RwLock;

mod comms;
mod cli;

mod dma;
mod websocket;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    simple_logger::SimpleLogger::new()
        .with_level(cli.loglevel.into())
        .init()
        .expect("Initializing logger");

    let rwlock = Arc::new(
        RwLock::new(
            RadarData::empty()
        )
    );

    let rwlock_clone = rwlock.clone();
    let dma_handle = tokio::spawn(async move {
        if let Err(err) = dma::run(cli.connector, cli.pcileech_device, rwlock_clone).await {
            log::error!("Error in dma thread: [{}]", err.to_string());
        }
    });

    tokio::spawn(async move {
        let future = websocket::run(cli.web_path, cli.port, rwlock);

        if let Ok(my_local_ip) = local_ip_address::local_ip() {
            let address = format!("http://{}:{}", my_local_ip, cli.port);
            println!("Launched webserver at {}", address);
        } else {
            let address = format!("http://0.0.0.0:{}", cli.port);
            println!("launched webserver at! {}", address);
        }

        if let Err(err) = future.await {
            log::error!("Error in websocket server: [{}]", err.to_string());
        }
    });

    if let Err(err) = dma_handle.await {
        log::error!("Error when waiting for dma thread: {}", err.to_string());
    }

    Ok(())
}
