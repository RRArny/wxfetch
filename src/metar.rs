use std::ops::Mul;

use colored::{Color, ColoredString, Colorize};

use crate::{Config, Position};
use serde_json::Value;

mod units;
mod wxcodes;
use crate::metar::units::*;
use crate::metar::wxcodes::*;

/// Represents a METAR report.
pub struct Metar {
    /// ICAO code of the issuing station.
    icao_code: String,
    /// Contents of the report.
    fields: Vec<MetarField>,
    /// True, if this METAR was issued by the exact station that was requested, false otherwise.
    exact_match: bool,
    /// Units.
    units: Units,
}

#[derive(PartialEq, Eq, Debug)]
/// Elements of a METAR report.
enum MetarField {
    /// Issue time.
    TimeStamp,
    /// Prevailing winds.
    Wind {
        direction: i64,
        strength: i64,
        gusts: i64,
    },
    /// Provided if wind direction changing.
    WindVariability { low_dir: i64, hi_dir: i64 },
    /// Visibility.
    Visibility(i64),
    /// Temperature and dewpoint.
    Temperature { temp: i64, dewpoint: i64 },
    /// Altimeter setting.
    Qnh(i64, PressureUnit),
    /// Observed cloud layers.
    Clouds(Clouds, i64),
    /// Prevailing weather conditions.
    WxCode(WxCode, WxCodeIntensity, WxCodeProximity, WxCodeDescription),
    /// Various remarks.
    Remarks(String),
}

#[derive(PartialEq, Eq, Debug)]
/// Describes a cloud layer.
enum Clouds {
    /// Sky clear.
    Skc,
    /// Few. Up to 2 octas coverage.
    Few,
    // Scattered. 3 - 4 octas coverage.
    Sct,
    /// Broken. 5 - 7 octas coverage.
    Brk,
    /// Overcast. 8 octas coverage.
    Ovc,
}

impl MetarField {
    pub fn colourise(&self) -> ColoredString {
        match self {
            MetarField::Visibility(vis) => colourise_visibility(vis),
            MetarField::TimeStamp => colourize_timestamp(),
            MetarField::Wind {
                direction,
                strength,
                gusts,
            } => colourise_wind(direction, strength, gusts),
            MetarField::WindVariability { low_dir, hi_dir } => colourise_wind_var(low_dir, hi_dir),
            MetarField::Temperature { temp, dewpoint } => colourise_temperature(temp, dewpoint),
            MetarField::Qnh(qnh, unit) => colourise_qnh(qnh, unit),
            MetarField::WxCode(code, intensity, proximity, descriptor) => {
                colourise_wx_code(code, intensity, proximity, descriptor)
            }
            MetarField::Remarks(str) => str.black().on_white(),
            MetarField::Clouds(_, _) => todo!(),
        }
    }
}

fn colourise_wx_code(
    _code: &WxCode,
    _intensity: &WxCodeIntensity,
    _proximity: &WxCodeProximity,
    _descriptor: &WxCodeDescription,
) -> ColoredString {
    // todo!()
    format!("{}{}{}{}", _intensity, _descriptor, _code, _proximity).magenta()
}

fn colourise_qnh(qnh: &i64, unit: &PressureUnit) -> ColoredString {
    match unit {
        PressureUnit::Hpa => format!("Q{}", qnh).color(if *qnh >= 1013 {
            Color::Green
        } else {
            Color::Yellow
        }),
        PressureUnit::Inhg => format!("A{}", qnh / 100).color(if *qnh >= 2992 {
            Color::Green
        } else {
            Color::Yellow
        }),
    }
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

fn colourise_visibility(vis: &i64) -> ColoredString {
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

        let units: Units = Units::from_json(&json);

        let mut fields: Vec<MetarField> = Vec::new();

        if let Some(time) = get_timestamp(&json) {
            fields.push(time);
        }

        if let Some(wind) = get_winds(&json, &units) {
            fields.push(wind);
        }

        if let Some(wind_var) = get_wind_var(&json) {
            fields.push(wind_var);
        }

        if let Some(vis) = get_visibility(&json, &units) {
            fields.push(vis);
        }

        if let Some(temp) = get_temp(&json, &units) {
            fields.push(temp);
        }

        if let Some(qnh) = get_qnh(&json, &units) {
            fields.push(qnh);
        }

        fields.append(&mut get_wxcodes(&json));

        if let Some(rmks) = get_remarks(&json) {
            fields.push(rmks);
        }

        let exact_match = is_exact_match(&station, config);

        Some(Metar {
            icao_code: station,
            fields,
            exact_match,
            units,
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

fn get_remarks(json: &Value) -> Option<MetarField> {
    let rmks = json.get("remarks")?.as_str()?.to_string();
    Some(MetarField::Remarks(rmks))
}

fn get_timestamp(_json: &Value) -> Option<MetarField> {
    // TODO parse timestamp from json["time"]["dt"]
    // todo!()
    Some(MetarField::TimeStamp)
}

fn get_qnh(json: &Value, units: &Units) -> Option<MetarField> {
    let qnh_val: &Value = json.get("altimeter")?.get("value")?;
    let qnh: i64 = if qnh_val.is_f64() {
        qnh_val.as_f64()?.mul(100.).round() as i64
    } else {
        qnh_val.as_i64()?
    };

    Some(MetarField::Qnh(qnh, units.pressure.clone()))
}

fn get_temp(json: &Value, units: &Units) -> Option<MetarField> {
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

fn get_winds(json: &Value, units: &Units) -> Option<MetarField> {
    let direction = json.get("wind_direction")?.get("value")?.as_i64()?;
    let strength = json.get("wind_speed")?.get("value")?.as_i64()?;
    let gusts = json
        .get("wind_gust")
        .and_then(|g| g.get("value"))
        .and_then(|g| g.as_i64())
        .unwrap_or(0);

    Some(MetarField::Wind {
        direction,
        strength,
        gusts,
    })
}

fn get_visibility(json: &Value, units: &Units) -> Option<MetarField> {
    let vis = json.get("visibility")?.get("value")?.as_i64()?;
    Some(MetarField::Visibility(vis))
}

fn is_exact_match(station: &str, config: &Config) -> bool {
    match &config.position {
        Position::Airfield(icao) => station.eq_ignore_ascii_case(icao),
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_metar_from_json_icao() {
        let json: Value = Value::from_str("{\"station\":\"EDRK\"}").unwrap();
        let config = Config {
            position: Position::Airfield("EDRK".to_string()),
        };
        let metar = Metar::from_json(json, &config);
        assert!(metar.is_some_and(|m| m.icao_code == "EDRK"));
    }

    // #[test]
    // fn test_metar_from_json_time() {
    //     let json: Value = Value::from_str("").unwrap();
    //     let config = Config {
    //         position: Position::Airfield("EDRK".to_string()),
    //     };

    //     let metar = Metar::from_json(json, &config);
    //     assert!(metar.is_some());
    //     assert!(metar.unwrap().fields.contains(&MetarField::TimeStamp));
    // }

    #[test]
    fn test_is_exact_match_positive() {
        let config = Config {
            position: Position::Airfield("EDDK".to_string()),
        };
        assert!(is_exact_match("EDDK", &config));
    }

    #[test]
    fn test_is_exact_match_negative() {
        let config = Config {
            position: Position::Airfield("EDDK".to_string()),
        };
        assert!(!is_exact_match("EDRK", &config));
    }

    #[test]
    fn test_is_exact_match_geoip() {
        let config = Config {
            position: Position::GeoIP,
        };
        assert!(is_exact_match("EDRK", &config));
    }

    #[test]
    fn test_is_exact_match_latlong() {
        let config = Config {
            position: Position::LatLong(crate::LatLong(10.0, 10.0)),
        };
        assert!(is_exact_match("EDRK", &config));
    }

    #[test]
    fn test_colourise_visibility_good() {
        let vis_str: ColoredString = colourise_visibility(&9999);
        assert_eq!(vis_str.fgcolor(), Some(Color::Green));
    }

    #[test]
    fn test_colourise_visibility_medium() {
        let vis_str: ColoredString = colourise_visibility(&2000);
        assert_eq!(vis_str.fgcolor(), Some(Color::Yellow));
    }

    #[test]
    fn test_colourise_visibility_bad() {
        let vis_str: ColoredString = colourise_visibility(&1000);
        assert_eq!(vis_str.fgcolor(), Some(Color::Red));
    }

    #[test]
    fn test_get_winds() {
        let json: Value = Value::from_str("{\"wind_direction\": {\"value\":100}, \"wind_speed\":{\"value\":10}, \"wind_gust\":{\"value\":15}}").unwrap();
        let expected = MetarField::Wind {
            direction: 100,
            strength: 10,
            gusts: 15,
        };
        let actual = get_winds(&json, &Units::default());
        assert!(actual.is_some_and(|w| w == expected));
    }

    #[test]
    fn test_get_winds_no_gust() {
        let json: Value =
            Value::from_str("{\"wind_direction\": {\"value\":100}, \"wind_speed\":{\"value\":10}}")
                .unwrap();
        let expected = MetarField::Wind {
            direction: 100,
            strength: 10,
            gusts: 0,
        };
        let actual = get_winds(&json, &Units::default());
        assert!(actual.is_some_and(|w| w == expected));
    }

    #[test]
    fn test_get_qnh() {
        let json: Value = Value::from_str("{\"altimeter\":{\"value\": 1013}}").unwrap();
        let expected = MetarField::Qnh(1013, PressureUnit::Hpa);
        let actual = get_qnh(&json, &Units::default());
        assert!(actual.is_some_and(|q| q == expected));
    }

    #[test]
    fn test_get_qnh_inhg() {
        let json: Value = Value::from_str("{\"altimeter\":{\"value\": 29.92}}").unwrap();
        let expected = MetarField::Qnh(2992, PressureUnit::Inhg);
        let units = Units {
            pressure: PressureUnit::Inhg,
            altitude: AltitudeUnit::Ft,
            wind_speed: SpeedUnit::Kt,
            temperature: TemperatureUnit::C,
        };
        let actual = get_qnh(&json, &units);
        println!("{:?}", actual);
        assert!(actual.is_some_and(|q| q == expected));
    }

    #[test]
    fn test_get_remarks() {
        let json: Value = Value::from_str("{\"remarks\":\"RWY UNAVAILABLE\"}").unwrap();
        let expected = "RWY UNAVAILABLE".to_string();
        let actual = get_remarks(&json);
        assert!(actual.is_some_and(|r| r == MetarField::Remarks(expected)));
    }

    #[test]
    fn test_get_temp() {
        let json: Value =
            Value::from_str("{\"temperature\":{\"value\": 10}, \"dewpoint\":{\"value\": 9}}")
                .unwrap();
        let expected: MetarField = MetarField::Temperature {
            temp: 10,
            dewpoint: 9,
        };
        let actual = get_temp(&json, &Units::default());
        assert!(actual.is_some_and(|t| t == expected));
    }

    #[test]
    fn test_get_wind_var() {
        let json: Value =
            Value::from_str("{\"wind_variable_direction\":[{\"value\" : 80},{\"value\" : 150}]}")
                .unwrap();
        let expected: MetarField = MetarField::WindVariability {
            low_dir: 80,
            hi_dir: 150,
        };
        let actual = get_wind_var(&json);
        assert!(actual.is_some_and(|v| v == expected));
    }

    #[test]
    fn test_get_visibility() {
        let json: Value = Value::from_str("{\"visibility\":{\"value\":9999}}").unwrap();
        let expected: MetarField = MetarField::Visibility(9999);
        let actual = get_visibility(&json, &Units::default());
        assert!(actual.is_some_and(|v| v == expected));
    }
}
