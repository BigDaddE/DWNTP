use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use log::{debug, error, info};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Parser)]
#[command(name = "dwntp-client")]
#[command(author, version, about = "CLI client for the DWNTP Hyperledger Fabric network", long_about = None)]
struct Cli {
    /// User identity to use (e.g., "Admin", "User1", "User2"). Defaults to "Admin".
    #[arg(long, default_value = "Admin")]
    user: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Logs a new RTU control event to the ledger
    LogEvent {
        /// Target RTU identifier (e.g. "RTU-999")
        #[arg(long)]
        rtu_id: String,

        /// Event name (e.g. "SwitchBreaker")
        #[arg(long)]
        event_name: String,

        /// Event description (e.g. "Turn off breaker 5")
        #[arg(long)]
        event_desc: String,
    },
    /// Queries an event from the ledger by its ID
    QueryEvent {
        /// Event ID string (SHA-256 hash returned from LogEvent)
        #[arg(long)]
        id: String,
    },
    /// Retrieves all events from the ledger
    GetAllEvents,
}

fn main() -> Result<()> {
    // Initialize the logger if RUST_LOG is set, otherwise default to "info"
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    let cli = Cli::parse();
    let runtime = detect_container_runtime()?;

    match &cli.command {
        Commands::LogEvent {
            rtu_id,
            event_name,
            event_desc,
        } => {
            let event_timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .context("Time went backwards")?
                .as_millis() as u64;

            // Format the chaincode arguments using Fabric's expected JSON format
            let args_json = serde_json::json!({
                "function": "LogEvent",
                "Args": [
                    rtu_id,
                    event_name,
                    event_desc,
                    event_timestamp.to_string()
                ]
            });
            let args_string = serde_json::to_string(&args_json)?;

            info!("Invoking LogEvent on chaincode...");
            debug!("Payload: {}", args_string);

            let msp_env = format!("CORE_PEER_MSPCONFIGPATH=/opt/gopath/src/github.com/hyperledger/fabric/peer/crypto/peerOrganizations/org1.dwntp.com/users/{}@org1.dwntp.com/msp", cli.user);

            // Using podman to execute the transaction from the 'cli' container
            let podman_cmd = format!(
                "{} peer chaincode invoke -o orderer.dwntp.com:7050 --tls --cafile $ORDERER_CA -C dwntpchannel -n dwntp -c '{}'",
                msp_env,
                args_string.replace("'", "'\\''")
            );

            let output = exec_in_cli(runtime, &podman_cmd)?;

            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            if output.status.success() {
                println!("--- Transaction Successful ---");
                if !stdout.trim().is_empty() {
                    println!("{}", stdout.trim());
                }

                // Fabric peer cli usually prints invoke result to stderr
                if stderr.contains("Chaincode invoke successful") {
                    // Try to extract the payload (event ID)
                    if let Some(payload_idx) = stderr.find("payload:\"") {
                        let sub = &stderr[payload_idx + 9..];
                        if let Some(end_idx) = sub.find("\"") {
                            println!("\nEvent ID: {}", &sub[..end_idx]);
                        }
                    } else {
                        println!("\nLogs:\n{}", stderr.trim());
                    }
                }
            } else {
                error!("Transaction failed: {}", stderr.trim());
                std::process::exit(1);
            }
        }
        Commands::GetAllEvents => {
            let args_json = serde_json::json!({
                "function": "GetAllEvents",
                "Args": []
            });
            let args_string = serde_json::to_string(&args_json)?;

            info!("Querying all events...");

            let msp_env = format!("CORE_PEER_MSPCONFIGPATH=/opt/gopath/src/github.com/hyperledger/fabric/peer/crypto/peerOrganizations/org1.dwntp.com/users/{}@org1.dwntp.com/msp", cli.user);

            let podman_cmd = format!(
                "{} peer chaincode query -C dwntpchannel -n dwntp -c '{}'",
                msp_env,
                args_string.replace("'", "'\\''")
            );

            let output = exec_in_cli(runtime, &podman_cmd)?;

            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            if output.status.success() {
                println!("--- All Events ---");
                if !stdout.trim().is_empty() {
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(stdout.trim()) {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&parsed)
                                .unwrap_or_else(|_| stdout.trim().to_string())
                        );
                    } else {
                        println!("{}", stdout.trim());
                    }
                } else {
                    println!("No events found.");
                }
            } else {
                error!("Query failed: {}", stderr.trim());
                std::process::exit(1);
            }
        }
        Commands::QueryEvent { id } => {
            let args_json = serde_json::json!({
                "function": "QueryEvent",
                "Args": [id]
            });
            let args_string = serde_json::to_string(&args_json)?;

            info!("Querying event ID: {}", id);

            let msp_env = format!("CORE_PEER_MSPCONFIGPATH=/opt/gopath/src/github.com/hyperledger/fabric/peer/crypto/peerOrganizations/org1.dwntp.com/users/{}@org1.dwntp.com/msp", cli.user);

            let podman_cmd = format!(
                "{} peer chaincode query -C dwntpchannel -n dwntp -c '{}'",
                msp_env,
                args_string.replace("'", "'\\''")
            );

            let output = exec_in_cli(runtime, &podman_cmd)?;

            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            if output.status.success() {
                println!("--- Query Result ---");

                // Try to pretty-print if it's JSON
                if let Ok(parsed_json) = serde_json::from_str::<serde_json::Value>(stdout.trim()) {
                    let pretty = serde_json::to_string_pretty(&parsed_json).unwrap();
                    println!("{}", pretty);
                } else {
                    println!("{}", stdout.trim());
                }
            } else {
                error!("Query failed: {}", stderr.trim());
                std::process::exit(1);
            }
        }
    }

    Ok(())
}

fn detect_container_runtime() -> Result<&'static str> {
    if runtime_available("docker") {
        return Ok("docker");
    }

    if runtime_available("podman") {
        return Ok("podman");
    }

    anyhow::bail!("Neither Docker nor Podman is reachable")
}

fn runtime_available(runtime: &str) -> bool {
    Command::new(runtime)
        .arg("info")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn exec_in_cli(runtime: &str, command: &str) -> Result<Output> {
    Command::new(runtime)
        .args(["exec", "cli", "bash", "-c", command])
        .output()
        .with_context(|| format!("Failed to execute {} command", runtime))
}
