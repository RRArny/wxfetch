use serde_json::Value;
use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum Position {
    Airfield(String),
    GeoIP,
    LatLong(LatLong),
}

impl Position {
    pub async fn get_location_str(&self) -> String {
        match self {
            Self::Airfield(icao_code) => icao_code.to_string(),
            Self::LatLong(latlong) => latlong.to_string(),
            Self::GeoIP => get_geoip()
                .await
                .expect("Could not get location based on IP. Try supplying position instead or check your internet connection.")
                .to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LatLong(pub f64, pub f64);

impl Display for LatLong {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{},{}", self.0, self.1)
    }
}

async fn get_geoip() -> Option<LatLong> {
    let response = reqwest::get("http://ip-api.com/json/").await.ok()?;
    let json: Value = response.json().await.ok()?;

    let success = json.get("status")?;
    if success != "success" {
        return None;
    }

    let lat = json.get("lat")?.as_f64()?;
    let long = json.get("lon")?.as_f64()?;

    Some(LatLong(lat, long))
}
