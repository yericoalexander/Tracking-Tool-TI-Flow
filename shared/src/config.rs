use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub kafka_brokers: String,
    pub kafka_request_topic: String,
    pub kafka_response_topic: String,
    pub kafka_timeout: u64,
    pub request_timeout: u64,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            kafka_brokers: env::var("KAFKA_BROKERS").unwrap_or_else(|_| "localhost:9092".to_string()),
            kafka_request_topic: env::var("KAFKA_REQUEST_TOPIC").unwrap_or_else(|_| "hash_requests".to_string()),
            kafka_response_topic: env::var("KAFKA_RESPONSE_TOPIC").unwrap_or_else(|_| "hash_responses".to_string()),
            kafka_timeout: env::var("KAFKA_TIMEOUT").unwrap_or_else(|_| "5".to_string()).parse().unwrap_or(5),
            request_timeout: env::var("REQUEST_TIMEOUT").unwrap_or_else(|_| "10".to_string()).parse().unwrap_or(10),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Service1Config {
    pub host: String,
    pub port: u16,
    pub group_id: String,
    pub config: Config,
}

impl Service1Config {
    pub fn from_env() -> Self {
        Self {
            host: env::var("SERVICE_1_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("SERVICE_1_PORT").unwrap_or_else(|_| "3001".to_string()).parse().unwrap_or(3001),
            group_id: env::var("SERVICE_1_GROUP_ID").unwrap_or_else(|_| "service_1".to_string()),
            config: Config::from_env(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Service2Config {
    pub group_id: String,
    pub config: Config,
}

impl Service2Config {
    pub fn from_env() -> Self {
        Self {
            group_id: env::var("SERVICE_2_GROUP_ID").unwrap_or_else(|_| "service_2".to_string()),
            config: Config::from_env(),
        }
    }
}