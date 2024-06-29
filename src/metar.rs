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
    Clouds(Clouds, i64),
    WxCode(WxCode, WxCodeModifier),
    Remarks(String),
}

#[derive(PartialEq, Eq)]
enum Clouds {
    Few,
    Sct,
    Brk,
    Ovc,
}

#[derive(PartialEq, Eq)]
enum WxCode {
    Ra,
    Ts,
}

#[derive(PartialEq, Eq)]
enum WxCodeModifier {
    Moderate,
    Light,
    Heavy,
}

impl From<&str> for WxCode {
    fn from(_value: &str) -> Self {
        todo!()
    }
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
            MetarField::WxCode(_wxcode, _modifier) => todo!(),
            MetarField::Remarks(str) => str.black().on_white(),
            MetarField::Clouds(_, _) => todo!(),
        }
    }

    fn wxcode_from_str(_repr: &str) -> MetarField {
        // todo!()
        MetarField::WxCode(WxCode::Ra, WxCodeModifier::Light)
    }
}

fn colourise_qnh(qnh: &i64) -> ColoredString {
    format!("Q{}", qnh).color(if *qnh >= 1013 {
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

fn colourise_wind_var(low_dir: &i64, hi_dir: &i64) -> ColoredString {
    format!("{}V{}", low_dir, hi_dir).color(if hi_dir - low_dir < 45 {
        Color::Green
    } else {
        Color::Yellow
    })
}

fn colourise_wind(direction: &i64, strength: &i64, gusts: &i64) -> ColoredString {
    let dir_str = direction.to_string();
    let strength_str = strength.to_string().color(if *strength > 15 {
        Color::Red
    } else {
        Color::Green
    });
    let mut output: ColoredString = format!("{}{}", dir_str, strength_str).into();
    if *gusts > 0 {
        let gust_str = gusts.to_string().color(if gusts - strength > 5 {
            Color::BrightRed
        } else {
            Color::Green
        });
        output = format!("{}G{}", output, gust_str).into();
    }
    output = format!("{}KT", output).into();
    output
}

fn colourize_timestamp() -> ColoredString {
    // TODO compare timestamp now, if older than 6h red else if older than 1h yellow else green
    // todo!()
    "280930Z".green()
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
        // println!("{:?}", json);

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

        fields.append(&mut get_wxcodes(&json));

        if let Some(rmks) = get_remarks(&json) {
            fields.push(rmks);
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

fn get_wxcodes(json: &Value) -> Vec<MetarField> {
    let mut result: Vec<MetarField> = Vec::new();
    if let Some(wxcodes) = json.get("wx_codes").and_then(|x| x.as_array()) {
        for code in wxcodes {
            if let Some(repr) = code.get("repr").and_then(|x| x.as_str()) {
                result.push(MetarField::wxcode_from_str(repr));
            }
        }
    }
    result
}

fn get_remarks(json: &Value) -> Option<MetarField> {
    let rmks = json.get("remarks")?.as_str()?.to_string();
    Some(MetarField::Remarks(rmks))
}

fn get_timestamp(_json: &Value) -> Option<MetarField> {
    // TODO parse timestamp from json["time"]["dt"]
    // todo!()
    Some(MetarField::TimeStamp)
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
    let gust_value = json.get("wind_gust")?;
    let mut gusts = 0;
    if gust_value.is_object() {
        gusts = gust_value.get("value")?.as_i64()?;
    }
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
        _ => true,
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
