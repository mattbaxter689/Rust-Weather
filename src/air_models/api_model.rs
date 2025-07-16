use crate::air_models::{AirQualityHourly, RawAirQuality};
use crate::traits::data_fetcher::DataFetcher;
use async_trait::async_trait;
use chrono::{NaiveDateTime, TimeZone, Utc};
use reqwest::Client;

pub struct APIFetcher {
    pub client: Client,
    pub latitude: f64,
    pub longitude: f64,
}

#[async_trait]
impl DataFetcher for APIFetcher {
    async fn fetch_recent(
        &self,
    ) -> Result<Vec<AirQualityHourly>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!(
            "https://air-quality-api.open-meteo.com/v1/air-quality?latitude={}&longitude={}&past_hours=1&forecas&hourly=pm10,pm2_5,carbon_monoxide,carbon_dioxide,nitrogen_dioxide,sulphur_dioxide,ozone,methane,uv_index,dust,aerosol_optical_depth,us_aqi",
            self.latitude, self.longitude
        );

        let response = self.client.get(url).send().await?;
        let raw_data: RawAirQuality = response.json().await?;
        let hourly_data: Vec<AirQualityHourly> = raw_data.into();

        let now = Utc::now();

        // Filter out future timestamps by parsing time string
        let filtered: Vec<AirQualityHourly> = hourly_data
            .into_iter()
            .filter_map(|record| {
                if let Ok(naive_dt) = NaiveDateTime::parse_from_str(&record.time, "%Y-%m-%dT%H:%M")
                {
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

    async fn fetch_historical(
        &self,
        start_date: &str,
        end_date: &str,
    ) -> Result<Vec<AirQualityHourly>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!(
        "https://air-quality-api.open-meteo.com/v1/air-quality?latitude={}&longitude={}&start_date={}&end_date={}&hourly=pm10,pm2_5,carbon_monoxide,carbon_dioxide,nitrogen_dioxide,sulphur_dioxide,ozone,methane,uv_index,dust,aerosol_optical_depth,us_aqi",
        self.latitude,
        self.longitude,
        start_date,
        end_date
    );

        let response = self.client.get(url).send().await?;
        let raw_data: RawAirQuality = response.json().await?;
        Ok(raw_data.into())
    }
}
