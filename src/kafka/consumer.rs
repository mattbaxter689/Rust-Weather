use crate::air_models::AirQualityHourly;
use crate::traits::data_loader::Persistable;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;
use tokio_stream::StreamExt;
use tracing::{error, info};

pub async fn run_consumer(broker: &str, db_url: &str) {
    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", broker)
        .set("group.id", "hello-group")
        .set("auto.offset.reset", "earliest")
        .create()
        .expect("Failed to create consumer");

    consumer
        .subscribe(&["weather-data"])
        .expect("Failed to subscribe to topic");

    // let config: AppConfig = load_config().expect("Failed to load config");
    // let db_url = config.database.db_url.as_str();

    let pool: PgPool = PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(20))
        .connect(db_url)
        .await
        .expect("Failed to establish db connection");

    info!(target: "consumer", "[Consumer] Listening for messages...");

    let mut message_stream = consumer.stream();

    while let Some(result) = message_stream.next().await {
        match result {
            Ok(msg) => {
                if let Some(Ok(payload)) = msg.payload_view::<str>() {
                    match serde_json::from_str::<Vec<AirQualityHourly>>(payload) {
                        Ok(parsed) => {
                            if let Err(e) = parsed.save_to_db(&pool).await {
                                error!(target: "consumer", "[Consumer] failed to insert record: {}", e)
                            }
                        }
                        Err(e) => {
                            error!(target: "consumer", "[Consumer] failed to parse JSON: {}", e)
                        }
                    }
                }
            }
            Err(e) => error!(target: "consumer", "[Consumer] Kafka Error: {}", e),
        }
        info!(target: "consumer", "[Consumer] Waiting for next message ...")
    }
}
