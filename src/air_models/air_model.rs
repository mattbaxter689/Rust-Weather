use serde::{Deserialize, Serialize};
use std::fmt;

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
    pub latitude: f64,
    pub longitude: f64,
    pub hourly: Vec<AirQualityHourly>,
}

impl From<RawAirQuality> for Vec<AirQualityHourly> {
    fn from(raw: RawAirQuality) -> Self {
        let len = raw.hourly.time.len();

        (0..len)
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
            .collect()
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
