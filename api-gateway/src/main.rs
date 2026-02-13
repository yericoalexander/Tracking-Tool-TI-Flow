use axum::{extract::State, http::StatusCode, response::Json, routing::post, Router};
use rdkafka::{
    consumer::{Consumer, StreamConsumer},
    producer::{FutureProducer, FutureRecord},
    ClientConfig, Message,
};
use serde_json::Value;
use shared::{config::Service1Config, HashRequest, HashResponse};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::sync::RwLock;

type PendingRequests = Arc<RwLock<HashMap<String, tokio::sync::oneshot::Sender<String>>>>;

#[derive(Clone)]
struct AppState {
    producer: FutureProducer,
    pending: PendingRequests,
    config: Service1Config,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let config = Service1Config::from_env();
    
    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", &config.config.kafka_brokers)
        .create()
        .expect("Producer creation error");

    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", &config.config.kafka_brokers)
        .set("group.id", &config.group_id)
        .create()
        .expect("Consumer creation error");

    consumer
        .subscribe(&[&config.config.kafka_response_topic])
        .expect("Can't subscribe to topic");

    let pending = Arc::new(RwLock::new(HashMap::new()));
    let state = AppState {
        producer,
        pending: pending.clone(),
        config: config.clone(),
    };

    // Start response consumer
    tokio::spawn(consume_responses(consumer, pending, config.clone()));

    let app = Router::new()
        .route("/hash", post(create_hash))
        .with_state(state);

    let addr = format!("{}:{}", config.host, config.port);
    println!("Service 1 running on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn create_hash(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    let data = payload["data"]
        .as_str()
        .ok_or(StatusCode::BAD_REQUEST)?
        .to_string();

    let request = HashRequest::new(data);
    let (tx, rx) = tokio::sync::oneshot::channel();

    // Store pending request
    state.pending.write().await.insert(request.id.clone(), tx);

    // Send to Kafka
    let payload = serde_json::to_string(&request).unwrap();
    let record = FutureRecord::to(&state.config.config.kafka_request_topic)
        .payload(&payload)
        .key(&request.id);

    state
        .producer
        .send(record, Duration::from_secs(state.config.config.kafka_timeout))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Wait for response
    match tokio::time::timeout(Duration::from_secs(state.config.config.request_timeout), rx).await {
        Ok(Ok(hash)) => Ok(Json(serde_json::json!({
            "id": request.id,
            "hash": hash
        }))),
        _ => Err(StatusCode::REQUEST_TIMEOUT),
    }
}

async fn consume_responses(consumer: StreamConsumer, pending: PendingRequests, _config: Service1Config) {
    use tokio_stream::StreamExt;

    let mut stream = consumer.stream();
    while let Some(message) = stream.next().await {
        if let Ok(msg) = message {
            if let Some(payload) = msg.payload_view::<str>() {
                if let Ok(response) = serde_json::from_str::<HashResponse>(payload.unwrap()) {
                    if let Some(sender) = pending.write().await.remove(&response.id) {
                        let _ = sender.send(response.hash);
                    }
                }
            }
        }
    }
}