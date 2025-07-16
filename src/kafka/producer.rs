use chrono::{Duration as TimeDuration, NaiveDate, Utc};
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info};

use crate::traits::data_fetcher::DataFetcher;

const MAX_DAYS: i64 = 91;
const FETCH_INTERVAL_SECS: u64 = 5;
const EARLIEST_DATE: &str = "2023-01-01";

pub async fn run_historical_producer<F: DataFetcher + Sync>(broker: &str, fetcher: &F) {
    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", broker)
        .create()
        .expect("Error connecting to kafka client");

    let mut current_end = Utc::now().date_naive();
    let earliest =
        NaiveDate::parse_from_str(EARLIEST_DATE, "%Y-%m-%d").expect("Invalid hardcoded date");

    while current_end > earliest {
        let current_start = (current_end - TimeDuration::days(MAX_DAYS)).max(earliest);

        let start_date = current_start.format("%Y-%m-%d").to_string();
        let end_date = current_end.format("%Y-%m-%d").to_string();

        info!(target: "producer",
            "[Producer] Fetching data from {} to {}",
            start_date, end_date
        );
        match fetcher.fetch_historical(&start_date, &end_date).await {
            Ok(hourly) => match serde_json::to_string(&hourly) {
                Ok(batch_payload) => {
                    let record = FutureRecord::to("weather-data")
                        .payload(&batch_payload)
                        .key("key");
                    match producer.send(record, Duration::from_secs(0)).await {
                        Ok(delivery) => {
                            info!(target: "producer", "[Producer] Delivered: {:?}", delivery)
                        }
                        Err((e, _)) => error!(target: "producer", "[Producer] Error: {:?}", e),
                    }
                }
                Err(e) => {
                    error!(target: "producer", "Failed to serialize air quality batch: {}", e)
                }
            },
            Err(e) => {
                error!(target: "producer",
                    "[Producer] Failed to fetch air quality data from {} to {}: {}",
                    start_date, end_date, e
                );
            }
        }
        current_end = current_start - TimeDuration::days(1);
        sleep(Duration::from_secs(FETCH_INTERVAL_SECS)).await;
    }
    info!(target: "producer", "[Producer] Fetching data complete")
}

pub async fn run_recent_producer<F: DataFetcher + Sync>(broker: &str, fetcher: &F) {
    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", broker)
        .create()
        .expect("Error connecting to kafka client");

    loop {
        match fetcher.fetch_recent().await {
            Ok(hourly) => match serde_json::to_string(&hourly) {
                Ok(batch_payload) => {
                    let record = FutureRecord::to("weather-data")
                        .payload(&batch_payload)
                        .key("key");
                    info!(target: "producer", "[Producer] Sent : {:?}", batch_payload);
                    match producer.send(record, Duration::from_secs(0)).await {
                        Ok(delivery) => {
                            info!(target: "producer", "[Producer] Delivered: {:?}", delivery)
                        }
                        Err((e, _)) => error!(target: "producer", "[Producer] Error: {:?}", e),
                    }
                }
                Err(e) => {
                    error!(target: "producer", "Failed to serialize air quality batch: {}", e)
                }
            },
            Err(e) => {
                error!(target: "producer",
                    "[Producer] Failed to fetch air quality data for past hour {}",
                    e
                );
            }
        }
        sleep(Duration::from_secs(3600)).await;
    }
}
