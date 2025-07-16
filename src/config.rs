use config::Config;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct LocationConfig {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug, Deserialize)]
pub struct DBConfig {
    pub db_url: String,
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub location: LocationConfig,
    pub database: DBConfig,
}

pub fn load_config() -> Result<AppConfig, config::ConfigError> {
    Config::builder()
        .add_source(config::File::with_name("config"))
        .build()?
        .try_deserialize()
}
