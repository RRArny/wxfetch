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
// WxFetch - metar.rs

use crate::{position::Position, Config};
use chrono::DateTime;
use chrono::FixedOffset;
use chrono::Utc;
use colored::{Color, ColoredString, Colorize};
use serde_json::Value;
use std::ops::Mul;
use std::ops::Sub;

mod clouds;
mod units;
mod wxcodes;
use crate::metar::clouds::{get_clouds_from_json, Clouds};
use crate::metar::units::{PressureUnit, SpeedUnit, TemperatureUnit, Units};
use crate::metar::wxcodes::{
    get_wxcodes_from_json, WxCode, WxCodeDescription, WxCodeIntensity, WxCodeProximity,
};

/// Represents a METAR report.
pub struct Metar {
    /// ICAO code of the issuing station.
    icao_code: String,
    /// Contents of the report.
    fields: Vec<WxField>,
    /// True, if this METAR was issued by the exact station that was requested, false otherwise.
    exact_match: bool,
    // / Units.
    // units: Units,
}

#[derive(PartialEq, Eq, Debug)]
/// Elements of a METAR report.
pub enum WxField {
    /// Issue time.
    TimeStamp(DateTime<FixedOffset>),
    /// Prevailing winds.
    Wind {
        direction: i64,
        strength: i64,
        gusts: i64,
        unit: SpeedUnit,
    },
    /// Provided if wind direction changing.
    WindVariability { low_dir: i64, hi_dir: i64 },
    /// Visibility.
    Visibility(i64),
    /// Temperature and dewpoint.
    Temperature {
        temp: i64,
        dewpoint: i64,
        unit: TemperatureUnit,
    },
    /// Altimeter setting.
    Qnh(i64, PressureUnit),
    /// Observed cloud layers. Altitude in FL (flight level).
    Clouds(Clouds, i64),
    /// Prevailing weather conditions.
    WxCode(WxCode, WxCodeIntensity, WxCodeProximity, WxCodeDescription),
    /// Various remarks.
    Remarks(String),
}

impl WxField {
    pub fn colourise(&self, config: &Config) -> ColoredString {
        match self {
            WxField::Visibility(vis) => colourise_visibility(*vis, config),
            WxField::TimeStamp(datetime) => colourize_timestamp(datetime, config),
            WxField::Wind {
                direction,
                strength,
                gusts,
                unit,
            } => colourise_wind(*direction, *strength, *gusts, *unit, config),
            WxField::WindVariability { low_dir, hi_dir } => {
                colourise_wind_var(*low_dir, *hi_dir, config)
            }
            WxField::Temperature {
                temp,
                dewpoint,
                unit,
            } => colourise_temperature(*temp, *dewpoint, *unit, config),
            WxField::Qnh(qnh, unit) => colourise_qnh(*qnh, *unit, config),
            WxField::WxCode(code, intensity, proximity, descriptor) => {
                colourise_wx_code(code, intensity, proximity, descriptor, config)
            }
            WxField::Remarks(str) => str.black().on_white(),
            WxField::Clouds(cloud, alt) => colourise_clouds(cloud, *alt, config),
        }
    }
}

fn colourise_clouds(cloud: &Clouds, alt: i64, config: &Config) -> ColoredString {
    let res: ColoredString = format!("{cloud}").color(match cloud {
        Clouds::Ovc => Color::Red,
        Clouds::Brk => Color::Yellow,
        _ => Color::Green,
    });
    let altstr: ColoredString = format!("{alt}").color(if alt <= config.cloud_minimum {
        Color::Red
    } else if alt <= config.cloud_marginal {
        Color::Yellow
    } else {
        Color::Green
    });
    format!("{res}{altstr}").into()
}

fn colourise_wx_code(
    code: &WxCode,
    intensity: &WxCodeIntensity,
    proximity: &WxCodeProximity,
    descriptor: &WxCodeDescription,
    _config: &Config,
) -> ColoredString {
    let codestr = format!("{code}").color(match code {
        WxCode::Ra => Color::BrightYellow,
        WxCode::Gr | WxCode::Sn | WxCode::Up => Color::Red,
        WxCode::Gs => Color::Yellow,
        WxCode::Po => Color::BrightRed,
        _ => Color::White,
    });

    let intensitystr = format!("{intensity}").color(match intensity {
        WxCodeIntensity::Light => Color::BrightGreen,
        WxCodeIntensity::Heavy => Color::BrightRed,
        WxCodeIntensity::Moderate => Color::White,
    });

    let descrstr = format!("{descriptor}").color(match descriptor {
        WxCodeDescription::Ts => Color::Red,
        WxCodeDescription::Fz => Color::BrightBlue,
        WxCodeDescription::Sh => Color::Yellow,
        _ => Color::White,
    });

    let proxstr = format!("{proximity}").white();

    format!("{intensitystr}{descrstr}{codestr}{proxstr}").into()
}

fn colourise_qnh(qnh: i64, unit: PressureUnit, _config: &Config) -> ColoredString {
    match unit {
        PressureUnit::Hpa => format!("Q{qnh}").color(if qnh >= 1013 {
            Color::Green
        } else {
            Color::Yellow
        }),
        PressureUnit::Inhg => format!("A{}", qnh / 100).color(if qnh >= 2992 {
            Color::Green
        } else {
            Color::Yellow
        }),
    }
}

fn colourise_temperature(
    temp: i64,
    dewpoint: i64,
    _unit: TemperatureUnit,
    config: &Config,
) -> ColoredString {
    let temp_str = temp.to_string().color(if temp > config.temp_minimum {
        Color::BrightGreen
    } else {
        Color::BrightRed
    });
    let dew_str = dewpoint
        .to_string()
        .color(if temp - dewpoint > config.spread_minimum {
            Color::Green
        } else {
            Color::Red
        });
    format!("{temp_str}/{dew_str}").into()
}

fn colourise_wind_var(low_dir: i64, hi_dir: i64, config: &Config) -> ColoredString {
    format!("{low_dir}V{hi_dir}").color(if hi_dir - low_dir < config.wind_var_maximum {
        Color::Green
    } else {
        Color::Yellow
    })
}

fn colourise_wind(
    direction: i64,
    strength: i64,
    gusts: i64,
    _unit: SpeedUnit,
    config: &Config,
) -> ColoredString {
    let dir_str = format!("{direction:03}").to_string();
    let strength_str =
        format!("{strength:02}")
            .to_string()
            .color(if strength > config.wind_maximum {
                Color::Red
            } else {
                Color::Green
            });
    let mut output: ColoredString = format!("{dir_str}{strength_str}").into();
    if gusts > 0 {
        let gust_str =
            format!("{gusts:02}")
                .to_string()
                .color(if gusts - strength > config.gust_maximum {
                    Color::BrightRed
                } else {
                    Color::Green
                });
        output = format!("{output}G{gust_str}").into();
    }
    output = format!("{output}KT").into();
    output
}

fn colourize_timestamp(datetime: &DateTime<FixedOffset>, config: &Config) -> ColoredString {
    let now: DateTime<Utc> = Utc::now();
    let utctime = datetime.to_utc();
    let dt = now.sub(utctime);
    let str_rep: String = utctime.format("%d%H%MZ").to_string();
    str_rep.color(if dt.lt(&config.age_marginal) {
        Color::Green
    } else if dt.lt(&config.age_maximum) {
        Color::Yellow
    } else {
        Color::Red
    })
}

fn colourise_visibility(vis: i64, config: &Config) -> ColoredString {
    format!("{vis:04}").color(if vis >= config.visibility_marginal {
        Color::Green
    } else if vis > config.visibility_minimum {
        Color::Yellow
    } else {
        Color::Red
    })
}

impl Metar {
    pub fn from_json(json: &Value, config: &Config) -> Option<Self> {
        let mut station = String::new();
        if let Some(icao) = json.get("station") {
            station = icao.as_str()?.to_string();
        }

        let units: Units = Units::from_json(json);

        let mut fields: Vec<WxField> = Vec::new();

        if let Some(time) = get_timestamp(json) {
            fields.push(time);
        }

        if let Some(wind) = get_winds(json, units) {
            fields.push(wind);
        }

        if let Some(wind_var) = get_wind_var(json) {
            fields.push(wind_var);
        }

        if let Some(vis) = get_visibility(json, units) {
            fields.push(vis);
        }

        if let Some(temp) = get_temp(json, units) {
            fields.push(temp);
        }

        if let Some(qnh) = get_qnh(json, units) {
            fields.push(qnh);
        }

        fields.append(&mut get_wxcodes_from_json(json));

        fields.append(&mut get_clouds_from_json(json));

        if let Some(rmks) = get_remarks(json) {
            fields.push(rmks);
        }

        let exact_match = is_exact_match(&station, config);

        Some(Metar {
            icao_code: station,
            fields,
            exact_match,
        })
    }

    pub fn colorise(self, config: &Config) -> ColoredString {
        let mut coloured_string: ColoredString = if self.exact_match {
            self.icao_code.bright_white().on_blue()
        } else {
            self.icao_code.black().on_yellow()
        };

        for field in self.fields {
            coloured_string = format!("{} {}", coloured_string, field.colourise(config)).into();
        }

        coloured_string
    }
}

fn get_remarks(json: &Value) -> Option<WxField> {
    let rmks = json.get("remarks")?.as_str()?.to_string();
    Some(WxField::Remarks(rmks))
}

fn get_timestamp(json: &Value) -> Option<WxField> {
    let datetime_str = json.get("time")?.get("dt")?.as_str()?;
    let datetime = DateTime::parse_from_rfc3339(datetime_str).ok()?;
    Some(WxField::TimeStamp(datetime))
}

#[allow(clippy::cast_possible_truncation)]
fn get_qnh(json: &Value, units: Units) -> Option<WxField> {
    let qnh_val: &Value = json.get("altimeter")?.get("value")?;
    let qnh: i64 = if qnh_val.is_f64() {
        qnh_val.as_f64()?.mul(100.).round() as i64
    } else {
        qnh_val.as_i64()?
    };

    Some(WxField::Qnh(qnh, units.pressure))
}

fn get_temp(json: &Value, units: Units) -> Option<WxField> {
    let temp = json.get("temperature")?.get("value")?.as_i64()?;
    let dewpoint = json.get("dewpoint")?.get("value")?.as_i64()?;
    Some(WxField::Temperature {
        temp,
        dewpoint,
        unit: units.temperature,
    })
}

fn get_wind_var(json: &Value) -> Option<WxField> {
    let wind_dirs = json.get("wind_variable_direction")?.as_array()?;
    let mut dirs: Vec<i64> = Vec::new();
    for dir in wind_dirs {
        dirs.push(dir.get("value")?.as_i64()?);
    }
    dirs.sort_unstable();
    let low_dir = dirs.first()?;
    let hi_dir = dirs.last()?;
    Some(WxField::WindVariability {
        low_dir: *low_dir,
        hi_dir: *hi_dir,
    })
}

fn get_winds(json: &Value, units: Units) -> Option<WxField> {
    let direction = json.get("wind_direction")?.get("value")?.as_i64()?;
    let strength = json.get("wind_speed")?.get("value")?.as_i64()?;
    let gusts = json
        .get("wind_gust")
        .and_then(|g| g.get("value"))
        .and_then(serde_json::Value::as_i64)
        .unwrap_or(0);

    Some(WxField::Wind {
        direction,
        strength,
        gusts,
        unit: units.wind_speed,
    })
}

fn get_visibility(json: &Value, _units: Units) -> Option<WxField> {
    let vis = json.get("visibility")?.get("value")?.as_i64()?;
    Some(WxField::Visibility(vis))
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

    use units::{AltitudeUnit, DistanceUnit};

    use crate::position::LatLong;

    use super::*;

    #[test]
    fn test_metar_from_json_icao() {
        let json: Value = Value::from_str("{\"station\":\"EDRK\"}").unwrap();
        let config = Config {
            position: Position::Airfield("EDRK".to_string()),
            ..Default::default()
        };
        let metar = Metar::from_json(&json, &config);
        assert!(metar.is_some_and(|m| m.icao_code == "EDRK"));
    }

    #[test]
    fn test_metar_from_json_time() {
        let json: Value = Value::from_str("{\"time\":{\"dt\":\"2024-06-21T05:50:00Z\"}}").unwrap();
        let config = Config {
            position: Position::Airfield("EDRK".to_string()),
            ..Default::default()
        };
        let expected = DateTime::parse_from_rfc3339("2024-06-21T05:50:00Z").unwrap();
        let metar = Metar::from_json(&json, &config);
        assert!(metar.is_some_and(|m| m.fields.contains(&WxField::TimeStamp(expected))));
    }

    #[test]
    fn test_is_exact_match_positive() {
        let config = Config {
            position: Position::Airfield("EDDK".to_string()),
            ..Default::default()
        };
        assert!(is_exact_match("EDDK", &config));
    }

    #[test]
    fn test_is_exact_match_negative() {
        let config = Config {
            position: Position::Airfield("EDDK".to_string()),
            ..Default::default()
        };
        assert!(!is_exact_match("EDRK", &config));
    }

    #[test]
    fn test_is_exact_match_geoip() {
        let config = Config {
            position: Position::GeoIP,
            ..Default::default()
        };
        assert!(is_exact_match("EDRK", &config));
    }

    #[test]
    fn test_is_exact_match_latlong() {
        let config = Config {
            position: Position::LatLong(LatLong(10.0, 10.0)),
            ..Default::default()
        };
        assert!(is_exact_match("EDRK", &config));
    }

    #[test]
    fn test_colourise_visibility_good() {
        let config = Config::default();
        let vis_str: ColoredString = colourise_visibility(9999, &config);
        assert_eq!(vis_str.fgcolor(), Some(Color::Green));
    }

    #[test]
    fn test_colourise_visibility_medium() {
        let config = Config::default();
        let vis_str: ColoredString = colourise_visibility(2000, &config);
        assert_eq!(vis_str.fgcolor(), Some(Color::Yellow));
    }

    #[test]
    fn test_colourise_visibility_bad() {
        let config = Config::default();
        let vis_str: ColoredString = colourise_visibility(1000, &config);
        assert_eq!(vis_str.fgcolor(), Some(Color::Red));
    }

    #[test]
    fn test_get_winds() {
        let json: Value = Value::from_str("{\"wind_direction\": {\"value\":100}, \"wind_speed\":{\"value\":10}, \"wind_gust\":{\"value\":15}}").unwrap();
        let expected = WxField::Wind {
            direction: 100,
            strength: 10,
            gusts: 15,
            unit: SpeedUnit::Kt,
        };
        let actual = get_winds(&json, Units::default());
        assert!(actual.is_some_and(|w| w == expected));
    }

    #[test]
    fn test_get_winds_no_gust() {
        let json: Value =
            Value::from_str("{\"wind_direction\": {\"value\":100}, \"wind_speed\":{\"value\":10}}")
                .unwrap();
        let expected = WxField::Wind {
            direction: 100,
            strength: 10,
            gusts: 0,
            unit: SpeedUnit::Kt,
        };
        let actual = get_winds(&json, Units::default());
        assert!(actual.is_some_and(|w| w == expected));
    }

    #[test]
    fn test_get_qnh() {
        let json: Value = Value::from_str("{\"altimeter\":{\"value\": 1013}}").unwrap();
        let expected = WxField::Qnh(1013, PressureUnit::Hpa);
        let actual = get_qnh(&json, Units::default());
        assert!(actual.is_some_and(|q| q == expected));
    }

    #[test]
    fn test_get_qnh_inhg() {
        let json: Value = Value::from_str("{\"altimeter\":{\"value\": 29.92}}").unwrap();
        let expected = WxField::Qnh(2992, PressureUnit::Inhg);
        let units = Units {
            pressure: PressureUnit::Inhg,
            altitude: AltitudeUnit::Ft,
            wind_speed: SpeedUnit::Kt,
            temperature: TemperatureUnit::C,
            distance: DistanceUnit::M,
        };
        let actual = get_qnh(&json, units);
        println!("{:?}", actual);
        assert!(actual.is_some_and(|q| q == expected));
    }

    #[test]
    fn test_get_remarks() {
        let json: Value = Value::from_str("{\"remarks\":\"RWY UNAVAILABLE\"}").unwrap();
        let expected = "RWY UNAVAILABLE".to_string();
        let actual = get_remarks(&json);
        assert!(actual.is_some_and(|r| r == WxField::Remarks(expected)));
    }

    #[test]
    fn test_get_temp() {
        let json: Value =
            Value::from_str("{\"temperature\":{\"value\": 10}, \"dewpoint\":{\"value\": 9}}")
                .unwrap();
        let expected: WxField = WxField::Temperature {
            temp: 10,
            dewpoint: 9,
            unit: TemperatureUnit::C,
        };
        let actual = get_temp(&json, Units::default());
        assert!(actual.is_some_and(|t| t == expected));
    }

    #[test]
    fn test_get_wind_var() {
        let json: Value =
            Value::from_str("{\"wind_variable_direction\":[{\"value\" : 80},{\"value\" : 150}]}")
                .unwrap();
        let expected: WxField = WxField::WindVariability {
            low_dir: 80,
            hi_dir: 150,
        };
        let actual = get_wind_var(&json);
        assert!(actual.is_some_and(|v| v == expected));
    }

    #[test]
    fn test_get_visibility() {
        let json: Value = Value::from_str("{\"visibility\":{\"value\":9999}}").unwrap();
        let expected: WxField = WxField::Visibility(9999);
        let actual = get_visibility(&json, Units::default());
        assert!(actual.is_some_and(|v| v == expected));
    }

    #[test]
    fn test_colourise_vis() {
        let config = Config::default();
        let vis = WxField::Visibility(9999);
        let expected = colourise_visibility(9999, &config);
        let actual = vis.colourise(&config);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_colourise_wind() {
        let config = Config::default();
        let wind = WxField::Wind {
            direction: 0,
            strength: 0,
            gusts: 0,
            unit: SpeedUnit::Kt,
        };
        let expected = colourise_wind(0, 0, 0, SpeedUnit::Kt, &config);
        let actual = wind.colourise(&config);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_colourise_wind_var() {
        let config = Config::default();
        let wind = WxField::WindVariability {
            low_dir: 0,
            hi_dir: 10,
        };
        let expected = colourise_wind_var(0, 10, &config);
        let actual = wind.colourise(&config);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_colourise_temp() {
        let config = Config::default();
        let temp = WxField::Temperature {
            temp: 20,
            dewpoint: 10,
            unit: TemperatureUnit::C,
        };
        let expected = colourise_temperature(20, 10, TemperatureUnit::C, &config);
        let actual = temp.colourise(&config);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_colourise_qnh() {
        let config = Config::default();
        let qnh = WxField::Qnh(1013, PressureUnit::Hpa);
        let expected = colourise_qnh(1013, PressureUnit::Hpa, &config);
        let actual = qnh.colourise(&config);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_colourise_wxcode() {
        let config = Config::default();
        let wxcode = WxField::WxCode(
            WxCode::Ra,
            WxCodeIntensity::Moderate,
            WxCodeProximity::OnStation,
            WxCodeDescription::None,
        );
        let expected = colourise_wx_code(
            &WxCode::Ra,
            &WxCodeIntensity::Moderate,
            &WxCodeProximity::OnStation,
            &WxCodeDescription::None,
            &config,
        );
        let actual = wxcode.colourise(&config);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_colourise_rmk() {
        let config = Config::default();
        let rmk = WxField::Remarks("NONE".to_string());
        let expected = "NONE".black().on_white();
        let actual = rmk.colourise(&config);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_colourise_clouds() {
        let config = Config::default();
        let clouds = WxField::Clouds(Clouds::Sct, 50);
        let expected = colourise_clouds(&Clouds::Sct, 50, &config);
        let actual = clouds.colourise(&config);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_colourise_timestamp() {
        let config = Config::default();
        let fixed_offset = Utc::now().fixed_offset();
        let timestamp = WxField::TimeStamp(fixed_offset);
        let expected = colourize_timestamp(&fixed_offset, &config);
        let actual = timestamp.colourise(&config);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_colourise_clouds_marginal() {
        let config = Config::default();
        let clouds = WxField::Clouds(Clouds::Ovc, 8);
        let expected = colourise_clouds(&Clouds::Ovc, 8, &config);
        let actual = clouds.colourise(&config);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_colourise_clouds_bad() {
        let config = Config::default();
        let clouds = WxField::Clouds(Clouds::Brk, 5);
        let expected = colourise_clouds(&Clouds::Brk, 5, &config);
        let actual = clouds.colourise(&config);
        assert_eq!(actual, expected);
    }

    // #[test]
    // fn test_colourise_wxcode_sn() {
    //     let config = Config::default();
    //     let wxcode = WxField::WxCode(
    //         WxCode::Sn,
    //         WxCodeIntensity::Moderate,
    //         WxCodeProximity::OnStation,
    //         WxCodeDescription::None,
    //     );
    //     let expected_colour = Color::Red;
    //     let actual = wxcode.colourise(&config);
    //     assert_eq!(actual.fgcolor().unwrap(), expected_colour);
    // }

    // #[test]
    // fn test_colourise_wxcode_gs() {
    //     let config = Config::default();
    //     let wxcode = WxField::WxCode(
    //         WxCode::Gs,
    //         WxCodeIntensity::Moderate,
    //         WxCodeProximity::OnStation,
    //         WxCodeDescription::None,
    //     );
    //     let expected_colour = Color::Yellow;
    //     let actual = wxcode.colourise(&config);
    //     assert_eq!(actual.fgcolor().unwrap(), expected_colour);
    // }

    // #[test]
    // fn test_colourise_wxcode_po() {
    //     let config = Config::default();
    //     let wxcode = WxField::WxCode(
    //         WxCode::Po,
    //         WxCodeIntensity::Moderate,
    //         WxCodeProximity::OnStation,
    //         WxCodeDescription::None,
    //     );
    //     let expected_colour = Color::BrightRed;
    //     let actual = wxcode.colourise(&config);
    //     assert_eq!(actual.fgcolor().unwrap(), expected_colour);
    // }

    // #[test]
    // fn test_colourise_wxcode_ic() {
    //     let config = Config::default();
    //     let wxcode = WxField::WxCode(
    //         WxCode::Ic,
    //         WxCodeIntensity::Moderate,
    //         WxCodeProximity::OnStation,
    //         WxCodeDescription::None,
    //     );
    //     let expected_colour = Color::White;
    //     let actual = wxcode.colourise(&config);
    //     assert_eq!(actual.fgcolor().unwrap(), expected_colour);
    // }
    // // WxCode::Gr | WxCode::Sn | WxCode::Up => Color::Red,
    // WxCode::Gs => Color::Yellow,
    // WxCode::Po => Color::BrightRed,
    // _ => Color::White,
}
