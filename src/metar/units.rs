use serde_json::Value;

#[derive(PartialEq, Eq, Debug, Default, Clone, Copy)]
pub struct Units {
    pub pressure: PressureUnit,
    pub altitude: AltitudeUnit,
    pub wind_speed: SpeedUnit,
    pub temperature: TemperatureUnit,
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
                Units {
                    pressure,
                    altitude,
                    wind_speed,
                    temperature,
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

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_units_from_json_invalid() {
        let json: Value = Value::from_str("{}").unwrap();
        let expected: Units = Units {
            pressure: PressureUnit::Hpa,
            altitude: AltitudeUnit::Ft,
            wind_speed: SpeedUnit::Kt,
            temperature: TemperatureUnit::C,
        };
        let actual = Units::from_json(&json);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_units_from_json_us_units() {
        let json: Value = Value::from_str("{\"units\":{\"altimeter\": \"inHg\",\"altitude\":\"ft\",\"temperature\":\"F\",\"wind_speed\": \"mph\"}}").unwrap();
        let expected: Units = Units {
            pressure: PressureUnit::Inhg,
            altitude: AltitudeUnit::Ft,
            wind_speed: SpeedUnit::Mph,
            temperature: TemperatureUnit::F,
        };
        let actual = Units::from_json(&json);
        assert_eq!(actual, expected);
    }
}
