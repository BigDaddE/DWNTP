use base64::Engine;
use log::{error, info, warn};
use prost::Message;
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response as TonicResponse, Status};

pub mod common {
    tonic::include_proto!("common");
}

pub mod pb {
    tonic::include_proto!("protos");
}

pub mod queryresult {
    tonic::include_proto!("queryresult");
}

use pb::chaincode_server::{Chaincode, ChaincodeServer};
use pb::{
    ChaincodeId, ChaincodeInput, ChaincodeMessage, GetState, GetStateByRange, PutState,
    QueryResponse, Response as PbResponse,
};

use dwntp_events::RtuControlEvent;

#[derive(Default)]
pub struct DwntpChaincode;

#[derive(Deserialize)]
struct CreateEventInput {
    source_mtu: String, // base64 string
    rtu_id: String,
    event_name: String,
    event_description: String,
    event_timestamp: u64,
}

#[tonic::async_trait]
impl Chaincode for DwntpChaincode {
    type ConnectStream = ReceiverStream<Result<ChaincodeMessage, Status>>;

    async fn connect(
        &self,
        request: Request<tonic::Streaming<ChaincodeMessage>>,
    ) -> Result<TonicResponse<Self::ConnectStream>, Status> {
        info!("Received gRPC connection request from Fabric Peer");

        let (tx, rx) = tokio::sync::mpsc::channel(100);
        let mut stream = request.into_inner();

        let pending_requests: Arc<
            Mutex<HashMap<String, tokio::sync::mpsc::Sender<ChaincodeMessage>>>,
        > = Arc::new(Mutex::new(HashMap::new()));

        let chaincode_id = env::var("CHAINCODE_ID").unwrap_or_else(|_| "dwntp_1.0".to_string());

        let cc_id = ChaincodeId {
            path: "".to_string(),
            name: chaincode_id.clone(),
            version: "".to_string(),
        };

        let register_msg = ChaincodeMessage {
            r#type: 1, // REGISTER
            timestamp: None,
            payload: cc_id.encode_to_vec(),
            txid: "".to_string(),
            channel_id: "".to_string(),
            chaincode_event: None,
            proposal: None,
        };

        if let Err(e) = tx.send(Ok(register_msg)).await {
            error!("Failed to send REGISTER message: {}", e);
        } else {
            info!("Sent REGISTER message for chaincode: {}", chaincode_id);
        }

        tokio::spawn(async move {
            while let Ok(Some(message)) = stream.message().await {
                let msg_type = message.r#type;

                match msg_type {
                    1 => info!("REGISTER received"),
                    2 => info!("REGISTERED with Peer"),
                    3 => {
                        info!("Handling INIT from peer");

                        let init_res = PbResponse {
                            status: 200,
                            message: "".to_string(),
                            payload: vec![],
                        };

                        let reply = ChaincodeMessage {
                            r#type: 6, // COMPLETED
                            timestamp: None,
                            payload: init_res.encode_to_vec(),
                            txid: message.txid.clone(),
                            channel_id: message.channel_id.clone(),
                            chaincode_event: None,
                            proposal: None,
                        };
                        let _ = tx.send(Ok(reply)).await;
                    }
                    4 => info!("Chaincode is READY"),
                    5 => {
                        let tx_clone = tx.clone();
                        let pending_requests_clone = pending_requests.clone();

                        tokio::spawn(async move {
                            info!("Handling TRANSACTION: txid={}", message.txid);

                            let (res_tx, mut res_rx) = tokio::sync::mpsc::channel(10);
                            pending_requests_clone
                                .lock()
                                .await
                                .insert(message.txid.clone(), res_tx);

                            let mut final_status = 200;
                            let mut final_message = "".to_string();
                            let mut final_payload = vec![];

                            match ChaincodeInput::decode(message.payload.as_slice()) {
                                Ok(input) => {
                                    if input.args.is_empty() {
                                        final_status = 500;
                                        final_message = "Missing function name".to_string();
                                    } else {
                                        let func =
                                            String::from_utf8_lossy(&input.args[0]).to_string();
                                        info!("Invoking function: {}", func);

                                        if func == "LogEvent" {
                                            if input.args.len() < 2 {
                                                final_status = 500;
                                                final_message =
                                                    "Missing event JSON payload".to_string();
                                            } else {
                                                let json_str =
                                                    String::from_utf8_lossy(&input.args[1]);

                                                match serde_json::from_str::<CreateEventInput>(
                                                    &json_str,
                                                ) {
                                                    Ok(input_data) => {
                                                        let source_bytes = base64::engine::general_purpose::STANDARD
                                                            .decode(&input_data.source_mtu)
                                                            .unwrap_or_default();

                                                        match RtuControlEvent::new(
                                                            source_bytes,
                                                            input_data.rtu_id,
                                                            input_data.event_name,
                                                            input_data.event_description,
                                                            input_data.event_timestamp,
                                                        ) {
                                                            Ok(mut event) => {
                                                                let now = SystemTime::now()
                                                                    .duration_since(UNIX_EPOCH)
                                                                    .unwrap()
                                                                    .as_millis()
                                                                    as u64;
                                                                event.on_chain_timestamp =
                                                                    Some(now);

                                                                info!("Successfully parsed and validated event: {}", event.id);

                                                                let put_state = PutState {
                                                                    key: format!(
                                                                        "event_{}",
                                                                        event.id
                                                                    ),
                                                                    value: event
                                                                        .to_json()
                                                                        .unwrap()
                                                                        .into_bytes(),
                                                                    collection: String::new(),
                                                                };

                                                                let put_msg = ChaincodeMessage {
                                                                    r#type: 9, // PUT_STATE
                                                                    timestamp: None,
                                                                    payload: put_state
                                                                        .encode_to_vec(),
                                                                    txid: message.txid.clone(),
                                                                    channel_id: message
                                                                        .channel_id
                                                                        .clone(),
                                                                    chaincode_event: None,
                                                                    proposal: None,
                                                                };
                                                                let _ = tx_clone
                                                                    .send(Ok(put_msg))
                                                                    .await;

                                                                if let Some(resp) =
                                                                    res_rx.recv().await
                                                                {
                                                                    if resp.r#type == 13 {
                                                                        final_status = 200;
                                                                        final_payload =
                                                                            event.id.into_bytes();
                                                                    } else if resp.r#type == 7 {
                                                                        final_status = 500;
                                                                        final_message = String::from_utf8_lossy(&resp.payload).to_string();
                                                                    }
                                                                }
                                                            }
                                                            Err(e) => {
                                                                final_status = 500;
                                                                final_message = format!(
                                                                    "Event validation failed: {:?}",
                                                                    e
                                                                );
                                                            }
                                                        }
                                                    }
                                                    Err(e) => {
                                                        final_status = 500;
                                                        final_message = format!(
                                                            "Invalid event JSON schema: {:?}",
                                                            e
                                                        );
                                                    }
                                                }
                                            }
                                        } else if func == "QueryEvent" {
                                            if input.args.len() < 2 {
                                                final_status = 500;
                                                final_message =
                                                    "Missing event ID parameter".to_string();
                                            } else {
                                                let event_id =
                                                    String::from_utf8_lossy(&input.args[1])
                                                        .to_string();

                                                let get_state = GetState {
                                                    key: format!("event_{}", event_id),
                                                    collection: String::new(),
                                                };

                                                let get_msg = ChaincodeMessage {
                                                    r#type: 8, // GET_STATE
                                                    timestamp: None,
                                                    payload: get_state.encode_to_vec(),
                                                    txid: message.txid.clone(),
                                                    channel_id: message.channel_id.clone(),
                                                    chaincode_event: None,
                                                    proposal: None,
                                                };
                                                let _ = tx_clone.send(Ok(get_msg)).await;

                                                if let Some(resp) = res_rx.recv().await {
                                                    if resp.r#type == 13 {
                                                        info!("QueryEvent received RESPONSE from peer. Payload len: {}", resp.payload.len());
                                                        if resp.payload.is_empty() {
                                                            final_status = 404;
                                                            final_message = format!(
                                                                "Event {} not found",
                                                                event_id
                                                            );
                                                        } else {
                                                            final_status = 200;
                                                            final_payload = resp.payload.clone();
                                                        }
                                                    } else if resp.r#type == 7 {
                                                        info!("QueryEvent received ERROR from peer. Payload len: {}", resp.payload.len());
                                                        final_status = 500;
                                                        final_message =
                                                            String::from_utf8_lossy(&resp.payload)
                                                                .to_string();
                                                    }
                                                }
                                            }
                                        } else if func == "GetAllEvents" {
                                            info!("Starting GetAllEvents range query");
                                            let get_state_by_range = GetStateByRange {
                                                start_key: "event_".to_string(),
                                                end_key: "event_g".to_string(),
                                                collection: String::new(),
                                                metadata: vec![],
                                            };

                                            let range_msg = ChaincodeMessage {
                                                r#type: 14, // GET_STATE_BY_RANGE
                                                timestamp: None,
                                                payload: get_state_by_range.encode_to_vec(),
                                                txid: message.txid.clone(),
                                                channel_id: message.channel_id.clone(),
                                                chaincode_event: None,
                                                proposal: None,
                                            };
                                            let _ = tx_clone.send(Ok(range_msg)).await;

                                            if let Some(resp) = res_rx.recv().await {
                                                if resp.r#type == 13 {
                                                    info!("Received range response payload of size: {}", resp.payload.len());
                                                    match QueryResponse::decode(
                                                        resp.payload.as_slice(),
                                                    ) {
                                                        Ok(query_response) => {
                                                            info!("Decoded QueryResponse with {} results", query_response.results.len());
                                                            let mut events = Vec::new();
                                                            for result_bytes in
                                                                query_response.results
                                                            {
                                                                if let Ok(kv) =
                                                                    queryresult::Kv::decode(
                                                                        result_bytes
                                                                            .result_bytes
                                                                            .as_slice(),
                                                                    )
                                                                {
                                                                    let json_str =
                                                                        String::from_utf8_lossy(
                                                                            &kv.value,
                                                                        );
                                                                    info!(
                                                                        "Extracted KV pair: key={}",
                                                                        kv.key
                                                                    );
                                                                    if let Ok(event) =
                                                                        serde_json::from_str::<
                                                                            serde_json::Value,
                                                                        >(
                                                                            &json_str
                                                                        )
                                                                    {
                                                                        events.push(event);
                                                                    } else {
                                                                        info!("Failed to parse JSON for key: {}", kv.key);
                                                                    }
                                                                } else {
                                                                    info!("Failed to decode KV");
                                                                }
                                                            }
                                                            info!(
                                                                "Successfully collected {} events",
                                                                events.len()
                                                            );
                                                            final_status = 200;
                                                            final_payload =
                                                                serde_json::to_string(&events)
                                                                    .unwrap()
                                                                    .into_bytes();
                                                        }
                                                        Err(e) => {
                                                            final_status = 500;
                                                            final_message = format!("Failed to decode QueryResponse: {}", e);
                                                        }
                                                    }
                                                } else if resp.r#type == 7 {
                                                    final_status = 500;
                                                    final_message =
                                                        String::from_utf8_lossy(&resp.payload)
                                                            .to_string();
                                                }
                                            }
                                        } else {
                                            final_status = 500;
                                            final_message = format!("Unknown function: {}", func);
                                        }
                                    }
                                }
                                Err(e) => {
                                    final_status = 500;
                                    final_message =
                                        format!("Failed to decode ChaincodeInput: {}", e);
                                }
                            }

                            pending_requests_clone.lock().await.remove(&message.txid);

                            let msg_type = if final_status >= 400 { 7 } else { 6 }; // ERROR or COMPLETED

                            info!(
                                "Replying to txid={} with status={}, message='{}', payload_len={}",
                                message.txid,
                                final_status,
                                final_message,
                                final_payload.len()
                            );

                            let reply_payload = if msg_type == 7 {
                                final_message.into_bytes()
                            } else {
                                let res = PbResponse {
                                    status: final_status,
                                    message: final_message,
                                    payload: final_payload,
                                };
                                res.encode_to_vec()
                            };

                            let reply = ChaincodeMessage {
                                r#type: msg_type,
                                timestamp: None,
                                payload: reply_payload,
                                txid: message.txid.clone(),
                                channel_id: message.channel_id.clone(),
                                chaincode_event: None,
                                proposal: None,
                            };
                            let _ = tx_clone.send(Ok(reply)).await;
                        });
                    }
                    13 | 7 => {
                        let map = pending_requests.lock().await;
                        if let Some(sender) = map.get(&message.txid) {
                            let _ = sender.send(message).await;
                        } else {
                            if msg_type == 7 && message.txid.is_empty() {
                                warn!(
                                    "Received stream-level ERROR from peer: {}",
                                    String::from_utf8_lossy(&message.payload)
                                );
                            } else {
                                warn!(
                                    "Received {} for unknown/completed txid={}",
                                    if msg_type == 13 { "RESPONSE" } else { "ERROR" },
                                    message.txid
                                );
                            }
                        }
                    }
                    _ => {
                        warn!("Unhandled message type: {}", msg_type);
                    }
                }
            }
            info!("Peer closed stream");
        });

        Ok(TonicResponse::new(ReceiverStream::new(rx)))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    info!("Starting DWNTP Hyperledger Fabric External Chaincode (gRPC)...");

    let server_address =
        env::var("CHAINCODE_SERVER_ADDRESS").unwrap_or_else(|_| "0.0.0.0:9999".to_string());
    let chaincode_id = env::var("CHAINCODE_ID").unwrap_or_else(|_| "dwntp_1.0".to_string());

    let addr: std::net::SocketAddr = server_address.parse()?;

    info!("Initializing chaincode ID: {}", chaincode_id);
    info!("gRPC Server configured to bind to: {}", addr);

    let chaincode_service = DwntpChaincode::default();

    info!("Starting Tonic gRPC server...");
    tonic::transport::Server::builder()
        .add_service(ChaincodeServer::new(chaincode_service))
        .serve(addr)
        .await?;

    Ok(())
}
