use crate::air_models::AirQualityHourly;
use async_trait::async_trait;

#[async_trait]
pub trait DataFetcher {
    async fn fetch_historical(
        &self,
        start_date: &str,
        end_date: &str,
    ) -> Result<Vec<AirQualityHourly>, Box<dyn std::error::Error + Send + Sync>>;

    async fn fetch_recent(
        &self,
    ) -> Result<Vec<AirQualityHourly>, Box<dyn std::error::Error + Send + Sync>>;
}
