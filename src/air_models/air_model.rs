use async_trait::async_trait;
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::fmt;
use tracing::info;

use crate::traits::data_loader::Persistable;

#[derive(Serialize, Deserialize, Debug)]
pub struct RawAirQuality {
    pub latitude: f64,
    pub longitude: f64,
    pub elevation: f64,
    pub generationtime_ms: f64,
    pub utc_offset_seconds: i32,
    pub timezone: String,
    pub timezone_abbreviation: String,
    pub hourly: RawHourlyData,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RawHourlyData {
    pub time: Vec<String>,
    pub pm10: Vec<Option<f64>>,
    pub pm2_5: Vec<Option<f64>>,
    pub carbon_monoxide: Vec<Option<f64>>,
    pub carbon_dioxide: Vec<Option<f64>>,
    pub nitrogen_dioxide: Vec<Option<f64>>,
    pub sulphur_dioxide: Vec<Option<f64>>,
    pub ozone: Vec<Option<f64>>,
    pub methane: Vec<Option<f64>>,
    pub uv_index: Vec<Option<f64>>,
    pub dust: Vec<Option<f64>>,
    pub aerosol_optical_depth: Vec<Option<f64>>,
    pub us_aqi: Vec<Option<f64>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AirQualityHourly {
    pub time: String,
    pub pm10: Option<f64>,
    pub pm2_5: Option<f64>,
    pub carbon_monoxide: Option<f64>,
    pub carbon_dioxide: Option<f64>,
    pub nitrogen_dioxide: Option<f64>,
    pub sulphur_dioxide: Option<f64>,
    pub ozone: Option<f64>,
    pub methane: Option<f64>,
    pub uv_index: Option<f64>,
    pub dust: Option<f64>,
    pub aerosol_optical_depth: Option<f64>,
    pub us_aqi: Option<f64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AirQuality {
    pub hourly: Vec<AirQualityHourly>,
}

impl From<RawAirQuality> for AirQuality {
    fn from(raw: RawAirQuality) -> Self {
        let len = raw.hourly.time.len();

        let hourly = (0..len)
            .map(|i| AirQualityHourly {
                time: raw.hourly.time.get(i).cloned().unwrap_or_default(),
                pm10: raw.hourly.pm10.get(i).copied().flatten(),
                pm2_5: raw.hourly.pm2_5.get(i).copied().flatten(),
                carbon_monoxide: raw.hourly.carbon_monoxide.get(i).copied().flatten(),
                carbon_dioxide: raw.hourly.carbon_dioxide.get(i).copied().flatten(),
                nitrogen_dioxide: raw.hourly.nitrogen_dioxide.get(i).copied().flatten(),
                sulphur_dioxide: raw.hourly.sulphur_dioxide.get(i).copied().flatten(),
                ozone: raw.hourly.ozone.get(i).copied().flatten(),
                methane: raw.hourly.methane.get(i).copied().flatten(),
                uv_index: raw.hourly.uv_index.get(i).copied().flatten(),
                dust: raw.hourly.dust.get(i).copied().flatten(),
                aerosol_optical_depth: raw.hourly.aerosol_optical_depth.get(i).copied().flatten(),
                us_aqi: raw.hourly.us_aqi.get(i).copied().flatten(),
            })
            .collect();

        AirQuality { hourly: hourly }
    }
}

#[async_trait]
impl Persistable for Vec<AirQualityHourly> {
    async fn save_to_db(&self, pool: &PgPool) -> Result<(), sqlx::Error> {
        if self.is_empty() {
            return Ok(());
        }
        info!(target: "consumer", "Parsing time series data");
        let times: Vec<DateTime<Utc>> = self
            .iter()
            .map(|r| {
                let naive = NaiveDateTime::parse_from_str(&r.time, "%Y-%m-%dT%H:%M")
                    .expect("Invalid Time Format");
                naive.and_utc()
            })
            .collect();

        let pm10s: Vec<Option<f64>> = self.iter().map(|r| r.pm10).collect();
        let pm2_5s: Vec<Option<f64>> = self.iter().map(|r| r.pm2_5).collect();
        let carbon_monoxides: Vec<Option<f64>> = self.iter().map(|r| r.carbon_monoxide).collect();
        let carbon_dioxides: Vec<Option<f64>> = self.iter().map(|r| r.carbon_dioxide).collect();
        let nitrogen_dioxides: Vec<Option<f64>> = self.iter().map(|r| r.nitrogen_dioxide).collect();
        let sulphur_dioxides: Vec<Option<f64>> = self.iter().map(|r| r.sulphur_dioxide).collect();
        let ozones: Vec<Option<f64>> = self.iter().map(|r| r.ozone).collect();
        let methanes: Vec<Option<f64>> = self.iter().map(|r| r.methane).collect();
        let uv_indexes: Vec<Option<f64>> = self.iter().map(|r| r.uv_index).collect();
        let dusts: Vec<Option<f64>> = self.iter().map(|r| r.dust).collect();
        let aods: Vec<Option<f64>> = self.iter().map(|r| r.aerosol_optical_depth).collect();
        let us_aqis: Vec<Option<i64>> = self.iter().map(|r| r.us_aqi.map(|v| v as i64)).collect();

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
        info!(target: "consumer", "Data ingested");
        Ok(())
    }
}

impl fmt::Display for AirQualityHourly {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Air Quality at {}", self.time)?;
        writeln!(f, "-----------------------------------------")?;
        writeln!(f, "PM10:                 {:?}", self.pm10)?;
        writeln!(f, "PM2.5:                {:?}", self.pm2_5)?;
        writeln!(f, "Carbon Monoxide:      {:?}", self.carbon_monoxide)?;
        writeln!(f, "Carbon Dioxide:       {:?}", self.carbon_dioxide)?;
        writeln!(f, "Nitrogen Dioxide:     {:?}", self.nitrogen_dioxide)?;
        writeln!(f, "Sulphur Dioxide:      {:?}", self.sulphur_dioxide)?;
        writeln!(f, "Ozone:                {:?}", self.ozone)?;
        writeln!(f, "Methane:              {:?}", self.methane)?;
        writeln!(f, "UV Index:             {:?}", self.uv_index)?;
        writeln!(f, "Dust:                 {:?}", self.dust)?;
        writeln!(f, "Aerosol Optical Depth:{:?}", self.aerosol_optical_depth)?;
        writeln!(f, "US AQI:               {:?}", self.us_aqi)?;
        Ok(())
    }
}
