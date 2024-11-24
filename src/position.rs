// Copyright 2024 Robin Arnold
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
// http://www.apache.org/licenses/LICENSE-2.0
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
// WxFetch - position.rs

use serde_json::Value;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, PartialEq, Clone)]
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

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_display_latlong() {
        let latlon = LatLong(51.4, 8.5);
        let expected = "51.4,8.5";
        let actual = latlon.to_string();
        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn test_display_latlong_neg() {
        let latlon = LatLong(-51.4, -8.5);
        let expected = "-51.4,-8.5";
        let actual = latlon.to_string();
        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn test_get_location_str_latlong() {
        let latlon = Position::LatLong(LatLong(51.4, 8.5));
        let expected = "51.4,8.5";
        let actual = latlon.get_location_str().await;
        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn test_get_location_str_airfield() {
        let airport = Position::Airfield("EDRK".to_string());
        let expected = "EDRK";
        let actual = airport.get_location_str().await;
        assert_eq!(expected, actual);
    }
}
