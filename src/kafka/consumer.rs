use crate::model::AirQualityHourly;
use chrono::{DateTime, NaiveDateTime, Utc};
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;
use tokio_stream::StreamExt;

use crate::config::{load_config, AppConfig};

pub async fn run_consumer(broker: &str) {
    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", broker)
        .set("group.id", "hello-group")
        .set("auto.offset.reset", "earliest")
        .create()
        .expect("Failed to create consumer");

    consumer
        .subscribe(&["weather-data"])
        .expect("Failed to subscribe to topic");

    let config: AppConfig = load_config().expect("Failed to load config");
    let db_url = config.database.db_url.as_str();

    let pool: PgPool = PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(20))
        .connect(db_url)
        .await
        .expect("Failed to establish db connection");

    println!("[Consumer] Listening for messages...");

    let mut message_stream = consumer.stream();

    while let Some(result) = message_stream.next().await {
        match result {
            Ok(msg) => {
                if let Some(Ok(payload)) = msg.payload_view::<str>() {
                    match serde_json::from_str::<Vec<AirQualityHourly>>(payload) {
                        Ok(parsed) => {
                            if let Err(e) = insert_air_qualiy_batch(&pool, &parsed).await {
                                eprintln!("[Consumer] failed to insert record: {}", e)
                            }
                        }
                        Err(e) => eprint!("[Consumer] failed to parse JSON: {}", e),
                    }
                }
            }
            Err(e) => eprint!("[Consumer] Kafka Error: {}", e),
        }
    }
}

pub async fn insert_air_qualiy_batch(
    pool: &PgPool,
    data: &[AirQualityHourly],
) -> Result<(), sqlx::Error> {
    if data.is_empty() {
        return Ok(());
    }

    let times: Vec<DateTime<Utc>> = data
        .iter()
        .map(|r| {
            let naive = NaiveDateTime::parse_from_str(&r.time, "%Y-%m-%dT%H:%M")
                .expect("Invalid Time Format");
            naive.and_utc()
        })
        .collect();

    let pm10s: Vec<Option<f64>> = data.iter().map(|r| r.pm10).collect();
    let pm2_5s: Vec<Option<f64>> = data.iter().map(|r| r.pm2_5).collect();
    let carbon_monoxides: Vec<Option<f64>> = data.iter().map(|r| r.carbon_monoxide).collect();
    let carbon_dioxides: Vec<Option<f64>> = data.iter().map(|r| r.carbon_dioxide).collect();
    let nitrogen_dioxides: Vec<Option<f64>> = data.iter().map(|r| r.nitrogen_dioxide).collect();
    let sulphur_dioxides: Vec<Option<f64>> = data.iter().map(|r| r.sulphur_dioxide).collect();
    let ozones: Vec<Option<f64>> = data.iter().map(|r| r.ozone).collect();
    let methanes: Vec<Option<f64>> = data.iter().map(|r| r.methane).collect();
    let uv_indexes: Vec<Option<f64>> = data.iter().map(|r| r.uv_index).collect();
    let dusts: Vec<Option<f64>> = data.iter().map(|r| r.dust).collect();
    let aods: Vec<Option<f64>> = data.iter().map(|r| r.aerosol_optical_depth).collect();
    let us_aqis: Vec<Option<i64>> = data.iter().map(|r| r.us_aqi.map(|v| v as i64)).collect();

    let query = r#"
        INSERT INTO air_quality (
            _time, pm10, pm2_5, carbon_monoxide, carbon_dioxide,
            nitrogen_dioxide, sulphur_dioxide, ozone, methane,
            uv_index, dust, aerosol_optical_depth, us_aqi
        )
        SELECT * FROM UNNEST(
            $1::timestamp[],
            $2::float8[],
            $3::float8[],
            $4::float8[],
            $5::float8[],
            $6::float8[],
            $7::float8[],
            $8::float8[],
            $9::float8[],
            $10::float8[],
            $11::float8[],
            $12::float8[],
            $13::int8[]
        )
    "#;

    sqlx::query(query)
        .bind(&times)
        .bind(&pm10s)
        .bind(&pm2_5s)
        .bind(&carbon_monoxides)
        .bind(&carbon_dioxides)
        .bind(&nitrogen_dioxides)
        .bind(&sulphur_dioxides)
        .bind(&ozones)
        .bind(&methanes)
        .bind(&uv_indexes)
        .bind(&dusts)
        .bind(&aods)
        .bind(&us_aqis)
        .execute(pool)
        .await?;
    Ok(())
}
