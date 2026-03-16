use anyhow::Result;
use sim::{framing::{recv, send}, types::ControlEvent};
use std::time::Duration;
use tokio::{io::split, net::TcpStream, time};
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().init();

    let stream = TcpStream::connect("127.0.0.1:17777").await?;
    info!("RTU connected to MTU");

    let (mut r, mut w) = split(stream);
    let mut seq = 0u64;
    let mut ticker = time::interval(Duration::from_secs(3));

    loop {
        tokio::select! {
            _ = ticker.tick() => {
                let event = ControlEvent { sender: "rtu-01".into(), seq, payload: format!("status:{seq}") };
                seq += 1;
                info!(seq = event.seq, payload = %event.payload, "RTU → MTU");
                send(&mut w, &event).await?;
            }
            result = recv::<_, ControlEvent>(&mut r) => {
                let event = result?;
                info!(from = %event.sender, seq = event.seq, payload = %event.payload, "RTU ← MTU");
            }
        }
    }
}
