use rdkafka::{
    consumer::{Consumer, StreamConsumer},
    producer::{FutureProducer, FutureRecord},
    ClientConfig, Message,
};
use sha2::{Digest, Sha256};
use shared::{config::Service2Config, HashRequest, HashResponse};
use std::time::Duration;
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let config = Service2Config::from_env();
    
    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", &config.config.kafka_brokers)
        .set("group.id", &config.group_id)
        .create()
        .expect("Consumer creation error");

    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", &config.config.kafka_brokers)
        .create()
        .expect("Producer creation error");

    consumer
        .subscribe(&[&config.config.kafka_request_topic])
        .expect("Can't subscribe to topic");

    println!("Service 2 started - processing hash requests");

    let mut stream = consumer.stream();
    while let Some(message) = stream.next().await {
        if let Ok(msg) = message {
            if let Some(payload) = msg.payload_view::<str>() {
                if let Ok(request) = serde_json::from_str::<HashRequest>(payload.unwrap()) {
                    println!("Processing request: {}", request.id);
                    
                    // Generate hash
                    let mut hasher = Sha256::new();
                    hasher.update(request.data.as_bytes());
                    let hash = format!("{:x}", hasher.finalize());

                    let response = HashResponse {
                        id: request.id,
                        hash,
                    };

                    // Send response back
                    let payload = serde_json::to_string(&response).unwrap();
                    let record = FutureRecord::to(&config.config.kafka_response_topic)
                        .payload(&payload)
                        .key(&response.id);

                    if let Err(e) = producer.send(record, Duration::from_secs(config.config.kafka_timeout)).await {
                        eprintln!("Failed to send response: {:?}", e);
                    } else {
                        println!("Response sent for: {}", response.id);
                    }
                }
            }
        }
    }
}