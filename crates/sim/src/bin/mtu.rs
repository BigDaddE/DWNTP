/// MTU node — peer-to-peer overlay.
///
/// Each MTU:
///   - Listens for peer MTU connections on --listen
///   - Dials out to configured --peers
///   - Listens for its local RTU on --rtu-port
///   - Signs and hash-chains every event it produces
///   - Verifies and re-gossips events from peer MTUs (flood with seen-set dedup)
///
/// Usage examples (three terminals):
///   cargo run -p sim --bin mtu -- --name mtu-a --listen 18001 --rtu-port 17001 --peers 127.0.0.1:18002,127.0.0.1:18003
///   cargo run -p sim --bin mtu -- --name mtu-b --listen 18002 --rtu-port 17002 --peers 127.0.0.1:18001,127.0.0.1:18003
///   cargo run -p sim --bin mtu -- --name mtu-c --listen 18003 --rtu-port 17003 --peers 127.0.0.1:18001,127.0.0.1:18002
use std::collections::HashSet;
use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use sim::{
    crypto::{event_hash, sign_event, verify_event},
    framing::{recv, send},
    types::{ControlEvent, RtuMessage},
};
use tokio::{
    io::split,
    net::{TcpListener, TcpStream},
    sync::{broadcast, mpsc},
    time,
};
use tracing::{info, warn};

#[derive(Parser)]
struct Args {
    #[arg(long)]
    name: String,

    /// Port to listen on for peer MTU connections.
    #[arg(long, default_value = "18001")]
    listen: u16,

    /// Port to listen on for the local RTU connection.
    #[arg(long, default_value = "17001")]
    rtu_port: u16,

    /// Comma-separated peer MTU addresses to dial (host:port).
    #[arg(long, default_value = "")]
    peers: String,
}

enum Msg {
    FromRtu(RtuMessage),
    FromPeer(ControlEvent),
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()))
        .init();

    let args = Args::parse();

    let signing_key = SigningKey::generate(&mut OsRng);
    let vk_bytes = signing_key.verifying_key().to_bytes().to_vec();

    // bcast_tx: main coordinator → all peer writer tasks
    let (bcast_tx, _) = broadcast::channel::<ControlEvent>(128);
    // incoming_tx: RTU handler + peer reader tasks → main coordinator
    let (incoming_tx, mut incoming_rx) = mpsc::channel::<Msg>(128);

    // ── Peer listener ────────────────────────────────────────────────────────
    {
        let listener = TcpListener::bind(format!("0.0.0.0:{}", args.listen)).await?;
        info!(port = args.listen, "listening for peer MTUs");
        let bcast_tx = bcast_tx.clone();
        let incoming_tx = incoming_tx.clone();
        tokio::spawn(async move {
            loop {
                if let Ok((stream, addr)) = listener.accept().await {
                    info!(%addr, "inbound peer MTU connected");
                    tokio::spawn(handle_peer(stream, bcast_tx.clone(), incoming_tx.clone()));
                }
            }
        });
    }

    // ── RTU listener ─────────────────────────────────────────────────────────
    {
        let listener = TcpListener::bind(format!("0.0.0.0:{}", args.rtu_port)).await?;
        info!(port = args.rtu_port, "listening for RTU");
        let incoming_tx = incoming_tx.clone();
        tokio::spawn(async move {
            // Accept one RTU; re-accept if it reconnects.
            loop {
                if let Ok((stream, addr)) = listener.accept().await {
                    info!(%addr, "RTU connected");
                    tokio::spawn(handle_rtu(stream, incoming_tx.clone()));
                }
            }
        });
    }

    // ── Dial configured peers ────────────────────────────────────────────────
    let peers: Vec<String> = args
        .peers
        .split(',')
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect();

    for peer_addr in peers {
        let bcast_tx = bcast_tx.clone();
        let incoming_tx = incoming_tx.clone();
        tokio::spawn(async move {
            loop {
                match TcpStream::connect(&peer_addr).await {
                    Ok(stream) => {
                        info!(peer = %peer_addr, "outbound peer MTU connected");
                        handle_peer(stream, bcast_tx.clone(), incoming_tx.clone()).await;
                        warn!(peer = %peer_addr, "peer disconnected — retrying in 3s");
                    }
                    Err(e) => {
                        warn!(peer = %peer_addr, error = %e, "dial failed — retrying in 3s");
                    }
                }
                time::sleep(Duration::from_secs(3)).await;
            }
        });
    }

    // ── Main coordinator ─────────────────────────────────────────────────────
    let mut seq: u64 = 0;
    let mut prev_hash = [0u8; 32];
    let mut seen: HashSet<[u8; 32]> = HashSet::new();
    let name = args.name.clone();

    // Heartbeat so the MTU gossips even without an RTU attached.
    let mut heartbeat = time::interval(Duration::from_secs(5));
    heartbeat.tick().await; // consume the immediate first tick

    loop {
        tokio::select! {
            _ = heartbeat.tick() => {
                let event = produce_event(&name, &mut seq, &mut prev_hash, &vk_bytes,
                                          &signing_key, format!("heartbeat from {name}"));
                seen.insert(event_hash(&event));
                info!(seq = event.seq, "heartbeat → gossip");
                let _ = bcast_tx.send(event);
            }

            msg = incoming_rx.recv() => {
                match msg {
                    Some(Msg::FromRtu(rtu_msg)) => {
                        let event = produce_event(&name, &mut seq, &mut prev_hash, &vk_bytes,
                                                   &signing_key, rtu_msg.payload.clone());
                        seen.insert(event_hash(&event));
                        info!(seq = event.seq, payload = %rtu_msg.payload, "RTU event signed → gossip");
                        let _ = bcast_tx.send(event);
                    }
                    Some(Msg::FromPeer(event)) => {
                        let hash = event_hash(&event);
                        if seen.contains(&hash) {
                            continue;
                        }
                        seen.insert(hash);
                        if !verify_event(&event) {
                            warn!(sender = %event.sender, seq = event.seq, "invalid signature — dropped");
                            continue;
                        }
                        info!(
                            from = %event.sender,
                            seq = event.seq,
                            payload = %event.payload,
                            chain = %hex::encode(event.prev_hash),
                            "verified peer event → re-gossip"
                        );
                        let _ = bcast_tx.send(event);
                    }
                    None => break,
                }
            }
        }
    }

    Ok(())
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn produce_event(
    name: &str,
    seq: &mut u64,
    prev_hash: &mut [u8; 32],
    vk_bytes: &[u8],
    signing_key: &SigningKey,
    payload: String,
) -> ControlEvent {
    let mut event = ControlEvent {
        sender: name.to_string(),
        seq: *seq,
        prev_hash: *prev_hash,
        payload,
        verifying_key: vk_bytes.to_vec(),
        signature: vec![],
    };
    sign_event(&mut event, signing_key);
    *prev_hash = event_hash(&event);
    *seq += 1;
    event
}

async fn handle_peer(
    stream: TcpStream,
    bcast_tx: broadcast::Sender<ControlEvent>,
    incoming_tx: mpsc::Sender<Msg>,
) {
    let (r, w) = split(stream);
    let mut bcast_rx = bcast_tx.subscribe();

    // Writer task: forward every broadcast event to this peer.
    tokio::spawn(async move {
        let mut w = w;
        loop {
            match bcast_rx.recv().await {
                Ok(event) => {
                    if send(&mut w, &event).await.is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    // Reader: forward received events to the coordinator.
    let mut r = r;
    loop {
        match recv::<_, ControlEvent>(&mut r).await {
            Ok(event) => {
                if incoming_tx.send(Msg::FromPeer(event)).await.is_err() {
                    break;
                }
            }
            Err(_) => break,
        }
    }
}

async fn handle_rtu(stream: TcpStream, incoming_tx: mpsc::Sender<Msg>) {
    let (mut r, _w) = split(stream);
    loop {
        match recv::<_, RtuMessage>(&mut r).await {
            Ok(msg) => {
                if incoming_tx.send(Msg::FromRtu(msg)).await.is_err() {
                    break;
                }
            }
            Err(_) => break,
        }
    }
}
