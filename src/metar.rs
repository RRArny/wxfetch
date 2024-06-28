use colored::{Color, ColoredString, Colorize};

use crate::{Config, Position};
use serde_json::Value;

/// Represents a METAR report.
pub struct Metar {
    /// ICAO code of the issuing station.
    icao_code: String,
    /// Contents of the report.
    fields: Vec<MetarField>,
    /// True, if this METAR was issued by the exact station provided to WXfetch, false otherwise.
    exact_match: bool,
}

#[derive(PartialEq, Eq)]
enum MetarField {
    TimeStamp,
    Wind {
        direction: i64,
        strength: i64,
        gusts: i64,
    },
    WindVariability {
        low_dir: i64,
        hi_dir: i64,
    },
    Visibility(u64),
    Temperature {
        temp: i64,
        dewpoint: i64,
    },
    Qnh(i64),
    WxCodes,
    Remarks,
}

impl MetarField {
    pub fn colourise(&self) -> ColoredString {
        match self {
            Self::Visibility(vis) => colourise_visibility(vis),
            MetarField::TimeStamp => colourize_timestamp(),
            MetarField::Wind {
                direction,
                strength,
                gusts,
            } => colourise_wind(direction, strength, gusts),
            MetarField::WindVariability { low_dir, hi_dir } => colourise_wind_var(low_dir, hi_dir),
            MetarField::Temperature { temp, dewpoint } => colourise_temperature(temp, dewpoint),
            MetarField::Qnh(qnh) => colourise_qnh(qnh),
            MetarField::WxCodes => todo!(),
            MetarField::Remarks => todo!(),
        }
    }
}

fn colourise_qnh(qnh: &i64) -> ColoredString {
    qnh.to_string().color(if *qnh >= 1013 {
        Color::Green
    } else {
        Color::Yellow
    })
}

fn colourise_temperature(temp: &i64, dewpoint: &i64) -> ColoredString {
    let temp_str = temp.to_string().color(if *temp > 0 {
        Color::BrightGreen
    } else {
        Color::BrightRed
    });
    let dew_str = dewpoint.to_string().color(if *temp - *dewpoint > 3 {
        Color::Green
    } else {
        Color::Red
    });
    format!("{}/{}", temp_str, dew_str).into()
}

fn colourise_wind_var(_low_dir: &i64, _hi_dirr: &i64) -> ColoredString {
    todo!()
}

fn colourise_wind(_direction: &i64, _strength: &i64, _gusts: &i64) -> ColoredString {
    todo!()
}

fn colourize_timestamp() -> ColoredString {
    // TODO compare timestamp now, if older than 6h red else if older than 1h yellow else green
    todo!()
}

fn colourise_visibility(vis: &u64) -> ColoredString {
    if *vis >= 6000 {
        vis.to_string().green()
    } else if *vis > 1500 {
        vis.to_string().yellow()
    } else {
        vis.to_string().red()
    }
}

impl Metar {
    pub fn from_json(json: Value, config: &Config) -> Option<Self> {
        let mut station = String::new();
        if let Some(icao) = json.get("station") {
            station = icao.as_str()?.to_string();
        }

        let mut fields: Vec<MetarField> = Vec::new();

        if let Some(time) = get_timestamp(&json) {
            fields.push(time);
        }

        if let Some(wind) = get_winds(&json) {
            fields.push(wind);
        }

        if let Some(wind_var) = get_wind_var(&json) {
            fields.push(wind_var);
        }

        if let Some(vis) = get_visibility(&json) {
            fields.push(MetarField::Visibility(vis));
        }

        if let Some(temp) = get_temp(&json) {
            fields.push(temp);
        }

        if let Some(qnh) = get_qnh(&json) {
            fields.push(qnh);
        }

        let exact_match = is_exact_match(&json, config);

        Some(Metar {
            icao_code: station,
            fields,
            exact_match,
        })
    }

    pub fn colorise(self) -> ColoredString {
        let mut coloured_string: ColoredString = if self.exact_match {
            self.icao_code.bright_white().on_blue()
        } else {
            self.icao_code.black().on_yellow()
        };

        for field in self.fields {
            coloured_string = format!("{} {}", coloured_string, field.colourise()).into();
        }

        coloured_string
    }
}

fn get_timestamp(_json: &Value) -> Option<MetarField> {
    // TODO parse timestamp from json["time"]["dt"]
    todo!()
}

fn get_qnh(json: &Value) -> Option<MetarField> {
    let qnh = json.get("altimeter")?.get("value")?.as_i64()?;
    Some(MetarField::Qnh(qnh))
}

fn get_temp(json: &Value) -> Option<MetarField> {
    let temp = json.get("temperature")?.get("value")?.as_i64()?;
    let dewpoint = json.get("dewpoint")?.get("value")?.as_i64()?;
    Some(MetarField::Temperature { temp, dewpoint })
}

fn get_wind_var(json: &Value) -> Option<MetarField> {
    let wind_dirs = json.get("wind_variable_direction")?.as_array()?;
    let mut dirs: Vec<i64> = Vec::new();
    for dir in wind_dirs {
        dirs.push(dir.get("value")?.as_i64()?);
    }
    dirs.sort();
    let low_dir = dirs.first()?;
    let hi_dir = dirs.last()?;
    Some(MetarField::WindVariability {
        low_dir: *low_dir,
        hi_dir: *hi_dir,
    })
}

fn get_winds(json: &Value) -> Option<MetarField> {
    let direction = json.get("wind_direction")?.get("value")?.as_i64()?;
    let strength = json.get("wind_speed")?.get("value")?.as_i64()?;
    let gusts = json.get("wind_gust")?.get("value")?.as_i64()?;

    Some(MetarField::Wind {
        direction,
        strength,
        gusts,
    })
}

fn get_visibility(json: &Value) -> Option<u64> {
    json.get("visibility")?.get("value")?.as_u64()
}

fn is_exact_match(json: &Value, config: &Config) -> bool {
    match &config.position {
        Position::Airfield(icao) => {
            if let Some(station_string) = json.get("station") {
                if let Some(station) = station_string.as_str() {
                    station == icao
                } else {
                    false
                }
            } else {
                false
            }
        }
        Position::GeoIP => todo!(),
        Position::LatLong(_) => todo!(),
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_metar_from_json_icao() {
        let json: Value = Value::from_str("").unwrap();
        let config = Config {
            position: Position::Airfield("EDRK".to_string()),
        };
        if let Some(metar) = Metar::from_json(json, &config) {
            assert_eq!(metar.icao_code, "EDRK");
        } else {
            panic!("Invalid Station code.");
        }
    }

    #[test]
    fn test_metar_from_json_time() {
        let json: Value = Value::from_str("").unwrap();
        let config = Config {
            position: Position::Airfield("EDRK".to_string()),
        };
        if let Some(metar) = Metar::from_json(json, &config) {
            assert!(metar.fields.contains(&MetarField::TimeStamp));
        } else {
            panic!("Invalid time stamp.");
        }
    }
}
