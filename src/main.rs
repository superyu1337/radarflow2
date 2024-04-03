use std::sync::Arc;

use clap::Parser;
use cli::Cli;
use comms::RadarData;
use tokio::sync::RwLock;

mod cli;
mod structs;
mod enums;
mod comms;

mod dma;
mod websocket;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli: Cli = Cli::parse();

    simple_logger::SimpleLogger::new()
        .with_level(cli.loglevel.into())
        .init()
        .expect("Initializing logger");

    let radar_data = Arc::new(
        RwLock::new(
            RadarData::empty(0)
        )
    );

    let radar_clone = radar_data.clone();
    let dma_handle = tokio::spawn(async move {
        if let Err(err) = dma::run(radar_clone, cli.connector, cli.pcileech_device, cli.skip_version).await {
            log::error!("Error in dma thread: [{}]", err.to_string());
        } else {
            println!("CS2 Process exited, exiting program...")
        }
    });

    let _websocket_handle = tokio::spawn(async move {
        if let Ok(my_local_ip) = local_ip_address::local_ip() {
            let address = format!("http://{}:{}", my_local_ip, cli.port);
            println!("Launched webserver at {}", address);
        } else {
            let address = format!("http://0.0.0.0:{}", cli.port);
            println!("launched webserver at {}", address);
        }

        if let Err(err) = websocket::run(cli.web_path, cli.port, radar_data).await {
            log::error!("Error in ws server: [{}]", err.to_string());
        }
    });

    dma_handle.await?;
    Ok(())
}
