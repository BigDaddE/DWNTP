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
    processed_at_unix_ms: u128,
}

#[derive(Debug, Serialize)]
struct SensorDataResponse {
    status: &'static str,
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
        processed_at_unix_ms: now_unix_ms(),
    })
}

async fn get_sensor_data() -> Json<SensorDataResponse> {
    // Simulate sensor read latency with random delay.
    let processing_delay = random_processing_delay();
    sleep(Duration::from_secs_f64(processing_delay)).await;

    let mut rng = rand::thread_rng();

    // Determine realistic state
    let breaker_closed = rng.gen_bool(0.88);

    // Voltage naturally fluctuates around 230V depending on grid load
    let voltage_v = rng.gen_range(225.0..=235.0);

    // Current is 0 if breaker is open, otherwise between 2.5 and 28.0 Amps
    let current_a = if breaker_closed {
        rng.gen_range(2.5..=28.0)
    } else {
        0.0
    };

    // Calculate a realistic transformer oil/winding temperature.
    // 1. Ambient temperature baseline (simulating a diurnal cycle based on time of day).
    //    We use the current unix timestamp to drive a sine wave representing a 24-hour cycle
    //    fluctuating between 15C and 25C.
    let now_sec = now_unix_ms() as f64 / 1000.0;
    let seconds_in_day = 86400.0;
    // Offset by -6 hours (putting the minimum at ~4am and max at ~4pm)
    let time_of_day_angle =
        ((now_sec - (6.0 * 3600.0)) % seconds_in_day) / seconds_in_day * 2.0 * std::f64::consts::PI;
    let ambient_temp = 20.0 + 5.0 * time_of_day_angle.sin();

    // 2. Load-based heating (I^2 * R losses).
    //    If breaker is closed, current drives temperature up.
    //    Formula: Base load heating factor * (I/I_max)^2
    let max_current = 30.0;
    let load_heating = if breaker_closed {
        // At max load, temperature rises by ~45 degrees above ambient.
        45.0 * ((current_a / max_current) as f64).powi(2)
    } else {
        0.0 // Cooling down towards ambient when open
    };

    // 3. Random noise (sensor jitter)
    let noise = rng.gen_range(-0.5..=0.5);

    let temperature_c = ambient_temp + load_heating + noise;

    let sensor_data = SensorData {
        voltage_v,
        current_a,
        temperature_c,
        breaker_closed,
        timestamp_unix_ms: now_unix_ms(),
    };

    Json(SensorDataResponse {
        status: "ok",
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
