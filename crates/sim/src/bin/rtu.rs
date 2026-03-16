/// RTU — sends raw control events to its local MTU.
///
/// Usage:
///   MTU_ADDR=127.0.0.1:17001 cargo run -p sim --bin rtu
use anyhow::Result;
use sim::{framing::send, types::RtuMessage};
use std::time::Duration;
use tokio::{net::TcpStream, time};
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().init();

    let mtu_addr = std::env::var("MTU_ADDR").unwrap_or_else(|_| "127.0.0.1:17001".into());
    let mut stream = TcpStream::connect(&mtu_addr).await?;
    info!(%mtu_addr, "RTU connected to MTU");

    let mut seq = 0u64;
    let mut ticker = time::interval(Duration::from_secs(3));

    loop {
        ticker.tick().await;
        let msg = RtuMessage { payload: format!("sensor-reading:{seq}") };
        info!(payload = %msg.payload, "RTU → MTU");
        send(&mut stream, &msg).await?;
        seq += 1;
    }
}
