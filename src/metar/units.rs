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
// WxFetch - metar/units.rs

use serde_json::Value;

#[derive(PartialEq, Eq, Debug, Default, Clone, Copy)]
pub struct Units {
    pub pressure: PressureUnit,
    pub altitude: AltitudeUnit,
    pub wind_speed: SpeedUnit,
    pub temperature: TemperatureUnit,
    pub distance: DistanceUnit,
}

impl Units {
    pub fn from_json(json: &Value) -> Self {
        match json.get("units") {
            Some(units_json) => {
                let pressure: PressureUnit = units_json
                    .get("altimeter")
                    .and_then(|v| v.as_str())
                    .map(PressureUnit::from)
                    .unwrap_or_default();
                let altitude: AltitudeUnit = units_json
                    .get("altitude")
                    .and_then(|v| v.as_str())
                    .map(AltitudeUnit::from)
                    .unwrap_or_default();
                let wind_speed: SpeedUnit = units_json
                    .get("wind_speed")
                    .and_then(|v| v.as_str())
                    .map(SpeedUnit::from)
                    .unwrap_or_default();
                let temperature: TemperatureUnit = units_json
                    .get("temperature")
                    .and_then(|v| v.as_str())
                    .map(TemperatureUnit::from)
                    .unwrap_or_default();
                let distance: DistanceUnit = units_json
                    .get("visibility")
                    .and_then(|d| d.as_str())
                    .map(DistanceUnit::from)
                    .unwrap_or_default();
                Units {
                    pressure,
                    altitude,
                    wind_speed,
                    temperature,
                    distance,
                }
            }
            None => Self::default(),
        }
    }
}

#[derive(Default, PartialEq, Eq, Debug, Clone, Copy)]
pub enum PressureUnit {
    #[default]
    Hpa,
    Inhg,
}

impl From<&str> for PressureUnit {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "hpa" => Self::Hpa,
            "inhg" => Self::Inhg,
            _ => Self::default(),
        }
    }
}

#[derive(Default, PartialEq, Eq, Debug, Clone, Copy)]
pub enum AltitudeUnit {
    #[default]
    Ft,
    M,
}

impl From<&str> for AltitudeUnit {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "ft" => Self::Ft,
            "m" => Self::M,
            _ => Self::default(),
        }
    }
}

#[derive(Default, PartialEq, Eq, Debug, Clone, Copy)]
pub enum SpeedUnit {
    #[default]
    Kt,
    Kph,
    Mph,
}

impl From<&str> for SpeedUnit {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "kt" => Self::Kt,
            "kph" => Self::Kph,
            "mph" => Self::Mph,
            _ => Self::default(),
        }
    }
}

#[derive(Default, PartialEq, Eq, Debug, Clone, Copy)]
pub enum TemperatureUnit {
    #[default]
    C,
    F,
}

impl From<&str> for TemperatureUnit {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "c" => Self::C,
            "f" => Self::F,
            _ => Self::default(),
        }
    }
}

#[derive(Default, PartialEq, Eq, Debug, Clone, Copy)]
pub enum DistanceUnit {
    #[default]
    M,
    Nm,
    Mi,
    Km,
}

impl From<&str> for DistanceUnit {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "m" => Self::M,
            "nm" => Self::Nm,
            "mi" => Self::Mi,
            "km" => Self::Km,
            _ => Self::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[tokio::test]
    async fn test_units_from_json_empty() {
        let json: Value = Value::from_str("{}").unwrap();
        let expected: Units = Units {
            pressure: PressureUnit::Hpa,
            altitude: AltitudeUnit::Ft,
            wind_speed: SpeedUnit::Kt,
            temperature: TemperatureUnit::C,
            distance: DistanceUnit::M,
        };
        let actual = Units::from_json(&json);
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_units_from_json_invalid() {
        let json: Value = Value::from_str("{\"units\":{\"altimeter\": \"aa\",\"altitude\":\"bb\",\"temperature\":\"cc\",\"wind_speed\": \"dd\", \"visibility\":\"ee\"}}").unwrap();
        let expected: Units = Units {
            pressure: PressureUnit::Hpa,
            altitude: AltitudeUnit::Ft,
            wind_speed: SpeedUnit::Kt,
            temperature: TemperatureUnit::C,
            distance: DistanceUnit::M,
        };
        let actual = Units::from_json(&json);
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_units_from_json_us_units() {
        let json: Value = Value::from_str("{\"units\":{\"altimeter\": \"inHg\",\"altitude\":\"ft\",\"temperature\":\"F\",\"wind_speed\": \"mph\", \"visibility\":\"mi\"}}").unwrap();
        let expected: Units = Units {
            pressure: PressureUnit::Inhg,
            altitude: AltitudeUnit::Ft,
            wind_speed: SpeedUnit::Mph,
            temperature: TemperatureUnit::F,
            distance: DistanceUnit::Mi,
        };
        let actual = Units::from_json(&json);
        assert_eq!(actual, expected);
    }
}
