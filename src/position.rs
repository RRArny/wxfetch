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
            Self::Airfield(icao_code) => icao_code.clone(),
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

    if json.get("status")? != "success" {
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

    #[test]
    #[allow(clippy::float_cmp)]
    fn test_latlong_creation() {
        let latlong = LatLong(40.7128, -74.0060);
        assert_eq!(latlong.0, 40.7128);
        assert_eq!(latlong.1, -74.0060);
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn test_latlong_clone() {
        let original = LatLong(51.5074, -0.1278);
        let cloned = original.clone();
        assert_eq!(original, cloned);
        assert_eq!(original.0, cloned.0);
        assert_eq!(original.1, cloned.1);
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn test_position_equality() {
        let pos1 = Position::Airfield("KJFK".to_string());
        let pos2 = Position::Airfield("KJFK".to_string());
        let pos3 = Position::Airfield("KLAX".to_string());

        assert_eq!(pos1, pos2);
        assert_ne!(pos1, pos3);
    }

    #[test]
    fn test_position_latlong_equality() {
        let pos1 = Position::LatLong(LatLong(40.7, -74.0));
        let pos2 = Position::LatLong(LatLong(40.7, -74.0));
        let pos3 = Position::LatLong(LatLong(51.5, -0.1));

        assert_eq!(pos1, pos2);
        assert_ne!(pos1, pos3);
    }

    #[test]
    fn test_position_geoip_equality() {
        let pos1 = Position::GeoIP;
        let pos2 = Position::GeoIP;

        assert_eq!(pos1, pos2);
    }

    #[test]
    fn test_position_different_types() {
        let airfield = Position::Airfield("KJFK".to_string());
        let latlong = Position::LatLong(LatLong(40.7, -74.0));
        let geoip = Position::GeoIP;

        assert_ne!(airfield, latlong);
        assert_ne!(airfield, geoip);
        assert_ne!(latlong, geoip);
    }

    #[test]
    fn test_position_clone() {
        let original = Position::Airfield("EHAM".to_string());
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_latlong_zero_coordinates() {
        let latlong = LatLong(0.0, 0.0);
        let expected = "0,0";
        let actual = latlong.to_string();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_latlong_extreme_coordinates() {
        let latlong = LatLong(90.0, 180.0);
        let expected = "90,180";
        let actual = latlong.to_string();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_latlong_extreme_negative_coordinates() {
        let latlong = LatLong(-90.0, -180.0);
        let expected = "-90,-180";
        let actual = latlong.to_string();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_latlong_precision() {
        let latlong = LatLong(51.4769, 0.0005);
        let expected = "51.4769,0.0005";
        let actual = latlong.to_string();
        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn test_get_location_str_latlong_zero() {
        let pos = Position::LatLong(LatLong(0.0, 0.0));
        let expected = "0,0";
        let actual = pos.get_location_str().await;
        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn test_get_location_str_airfield_lowercase() {
        let airport = Position::Airfield("kjfk".to_string());
        let expected = "kjfk";
        let actual = airport.get_location_str().await;
        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn test_get_location_str_airfield_special_chars() {
        let airport = Position::Airfield("ED-99".to_string());
        let expected = "ED-99";
        let actual = airport.get_location_str().await;
        assert_eq!(expected, actual);
    }
}
