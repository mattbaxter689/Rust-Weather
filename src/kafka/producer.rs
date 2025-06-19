use chrono::{Duration as TimeDuration, NaiveDate, NaiveDateTime, TimeZone, Utc};
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use reqwest::Client;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info};

use crate::config::{load_config, AppConfig};
use crate::model::{AirQualityHourly, RawAirQuality};

const MAX_DAYS: i64 = 91;
const FETCH_INTERVAL_SECS: u64 = 5;
const EARLIEST_DATE: &str = "2023-01-01";

pub async fn run_historical_producer(broker: &str) {
    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", broker)
        .create()
        .expect("Error connecting to kafka client");

    let client = Client::new();

    let config: AppConfig = load_config().expect("Failed to load config");
    let latitude = config.location.latitude;
    let longitude = config.location.longitude;

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
        match get_air_quality_data_historical(&client, &start_date, &end_date, latitude, longitude)
            .await
        {
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

pub async fn get_air_quality_data_historical(
    client: &Client,
    start_date: &str,
    end_date: &str,
    latitude: f64,
    longitude: f64,
) -> Result<Vec<AirQualityHourly>, Box<dyn std::error::Error>> {
    let url = format!(
        "https://air-quality-api.open-meteo.com/v1/air-quality?latitude={}&longitude={}&start_date={}&end_date={}&hourly=pm10,pm2_5,carbon_monoxide,carbon_dioxide,nitrogen_dioxide,sulphur_dioxide,ozone,methane,uv_index,dust,aerosol_optical_depth,us_aqi",
        latitude,
        longitude,
        start_date,
        end_date
    );

    let response = client.get(url).send().await?;
    let raw_data: RawAirQuality = response.json().await?;
    Ok(raw_data.into())
}

// Additionally we want to get recent data, which we can create the endpoint for now, in case we have historical data and
// want to stream recent data
pub async fn get_air_quality_data_recent(
    client: &Client,
    latitude: f64,
    longitude: f64,
) -> Result<Vec<AirQualityHourly>, Box<dyn std::error::Error>> {
    let url = format!(
        "https://air-quality-api.open-meteo.com/v1/air-quality?latitude={}&longitude={}&past_hours=1&forecas&hourly=pm10,pm2_5,carbon_monoxide,carbon_dioxide,nitrogen_dioxide,sulphur_dioxide,ozone,methane,uv_index,dust,aerosol_optical_depth,us_aqi",
        latitude, longitude
    );

    let response = client.get(url).send().await?;
    let raw_data: RawAirQuality = response.json().await?;
    let hourly_data: Vec<AirQualityHourly> = raw_data.into();

    let now = Utc::now();

    // Filter out future timestamps by parsing time string
    let filtered: Vec<AirQualityHourly> = hourly_data
        .into_iter()
        .filter_map(|record| {
            if let Ok(naive_dt) = NaiveDateTime::parse_from_str(&record.time, "%Y-%m-%dT%H:%M") {
                let dt_utc = Utc.from_utc_datetime(&naive_dt);
                if dt_utc <= now {
                    Some((dt_utc, record))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .max_by_key(|(dt, _)| *dt)
        .map(|(_, record)| record)
        .into_iter()
        .collect();

    Ok(filtered)
}

pub async fn run_recent_producer(broker: &str) {
    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", broker)
        .create()
        .expect("Error connecting to kafka client");

    let client = Client::new();

    let config: AppConfig = load_config().expect("Failed to load config");
    let latitude = config.location.latitude;
    let longitude = config.location.longitude;

    loop {
        match get_air_quality_data_recent(&client, latitude, longitude).await {
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
