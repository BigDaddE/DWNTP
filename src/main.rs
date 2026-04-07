use axum::{routing::get, routing::post, Json, Router};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::{sleep, Duration};

#[derive(Debug, Deserialize)]
struct SendRequestBody {
    request_type: String,
    target: Option<String>,
    value: Option<f64>,
}

#[derive(Debug, Serialize)]
struct SendRequestResponse {
    status: &'static str,
    message: &'static str,
    request_type: String,
    target: Option<String>,
    value: Option<f64>,
    processing_delay_seconds: f64,
    processed_at_unix_ms: u128,
}

#[derive(Debug, Serialize)]
struct SensorDataResponse {
    status: &'static str,
    processing_delay_seconds: f64,
    sensor_data: SensorData,
}

#[derive(Debug, Serialize)]
struct SensorData {
    voltage_v: f64,
    current_a: f64,
    temperature_c: f64,
    breaker_closed: bool,
    timestamp_unix_ms: u128,
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/send_request", post(send_request))
        .route("/get_sensor_data", get(get_sensor_data));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("failed to bind API listener");

    println!("RTU API running on http://0.0.0.0:8080");
    axum::serve(listener, app)
        .await
        .expect("failed to start API server");
}

async fn send_request(Json(body): Json<SendRequestBody>) -> Json<SendRequestResponse> {
    // Simulate real RTU behavior with random processing delay in [0.0, 1.0] seconds.
    let processing_delay = random_processing_delay();
    sleep(Duration::from_secs_f64(processing_delay)).await;

    Json(SendRequestResponse {
        status: "ok",
        message: "request processed",
        request_type: body.request_type,
        target: body.target,
        value: body.value,
        processing_delay_seconds: processing_delay,
        processed_at_unix_ms: now_unix_ms(),
    })
}

async fn get_sensor_data() -> Json<SensorDataResponse> {
    // Simulate sensor read latency with random delay.
    let processing_delay = random_processing_delay();
    sleep(Duration::from_secs_f64(processing_delay)).await;

    let mut rng = rand::thread_rng();

    let sensor_data = SensorData {
        voltage_v: rng.gen_range(219.0..=241.0),
        current_a: rng.gen_range(2.5..=28.0),
        temperature_c: rng.gen_range(18.0..=80.0),
        breaker_closed: rng.gen_bool(0.88),
        timestamp_unix_ms: now_unix_ms(),
    };

    Json(SensorDataResponse {
        status: "ok",
        processing_delay_seconds: processing_delay,
        sensor_data,
    })
}

fn random_processing_delay() -> f64 {
    rand::thread_rng().gen_range(0.0..=1.0)
}

fn now_unix_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::random_processing_delay;

    #[test]
    fn random_processing_delay_is_in_expected_range() {
        for _ in 0..1000 {
            let delay = random_processing_delay();
            assert!((0.0..=1.0).contains(&delay));
        }
    }
}
