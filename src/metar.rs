use std::fmt::Display;

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
    Qnh(i64),
    /// Observed cloud layers.
    Clouds(Clouds, i64),
    /// Prevailing weather conditions.
    WxCode(WxCode, WxCodeIntensity, WxCodeProximity, WxCodeDescription),
    /// Various remarks.
    Remarks(String),
}

#[derive(PartialEq, Eq)]
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

#[derive(PartialEq, Eq)]
/// Standardised codes for weather phenomena.
enum WxCode {
    /// Rain.
    Ra,
    /// Drizzle.
    Dz,
    /// Hail (Diameter >= 5mm).
    Gr,
    /// Small Hail (Diameter < 5mm).
    Gs,
    /// Ice crystals.
    Ic,
    /// Ice pellets.
    Pl,
    /// Snow grains.
    Sg,
    /// Snow.
    Sn,
    /// Unknown precipitation (automated reports only).
    Up,
    /// Mist (visibility >= 1000m).
    Br,
    /// Widespread dust.
    Du,
    /// Fog (visibility >= 1000m).
    Fg,
    /// Smoke.
    Fu,
    /// Haze.
    Hz,
    /// Spray.
    Py,
    /// Sand.
    Sa,
    /// Volcanic ash.
    Va,
    /// Dust storm.
    Ds,
    /// Funnel clouds.
    Fc,
    /// Well-developed sand or dust whirls.
    Po,
    /// Squalls.
    Sq,
    /// Sandstorm.
    Ss,
}

#[derive(PartialEq, Eq)]
/// Used to specify a weather phenomenon's intensity.
enum WxCodeIntensity {
    Moderate,
    Light,
    Heavy,
}

#[derive(PartialEq, Eq)]
/// Used to specify a weather phenomenon's distance from reporting staion.
enum WxCodeProximity {
    /// On station.
    OnStation,
    /// In vicinity of the station (distance 5 - 10 miles).
    Vicinity,
    /// More than 10 miles from station.
    Distant,
}

#[derive(PartialEq, Eq)]
/// Used to further specify a weather phenomenon.
enum WxCodeDescription {
    /// No description.
    None,
    /// Thunderstorm.
    Ts,
    /// Patches.
    Bc,
    /// Blowing.
    Bl,
    /// Low drifting.
    Dr,
    /// Freezing.
    Fz,
    /// Shallow.
    Mi,
    /// Partial.
    Pr,
    /// Shower(s).
    Sh,
}

impl Display for WxCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str_repr: &str = match self {
            WxCode::Ra => "RA",
            WxCode::Dz => "DZ",
            WxCode::Gr => "GR",
            WxCode::Gs => "GS",
            WxCode::Ic => "IC",
            WxCode::Pl => "PL",
            WxCode::Sg => "SG",
            WxCode::Sn => "SN",
            WxCode::Up => "UP",
            WxCode::Br => "BR",
            WxCode::Du => "DU",
            WxCode::Fg => "FG",
            WxCode::Fu => "FU",
            WxCode::Hz => "HZ",
            WxCode::Py => "PY",
            WxCode::Sa => "SA",
            WxCode::Va => "VA",
            WxCode::Ds => "DS",
            WxCode::Fc => "FC",
            WxCode::Po => "PO",
            WxCode::Sq => "SQ",
            WxCode::Ss => "SS",
        };
        write!(f, "{}", str_repr)
    }
}

impl Display for WxCodeIntensity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                WxCodeIntensity::Moderate => "",
                WxCodeIntensity::Light => "-",
                WxCodeIntensity::Heavy => "+",
            }
        )
    }
}

impl Display for WxCodeProximity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str_repr: &str = match self {
            WxCodeProximity::OnStation => "",
            WxCodeProximity::Vicinity => "VC",
            WxCodeProximity::Distant => "DSNT",
        };
        write!(f, "{}", str_repr)
    }
}

impl Display for WxCodeDescription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str_repr: &str = match self {
            WxCodeDescription::None => "",
            WxCodeDescription::Ts => "TS",
            WxCodeDescription::Bc => "BC",
            WxCodeDescription::Bl => "BL",
            WxCodeDescription::Dr => "DR",
            WxCodeDescription::Fz => "FZ",
            WxCodeDescription::Mi => "MI",
            WxCodeDescription::Pr => "PR",
            WxCodeDescription::Sh => "SH",
        };
        write!(f, "{}", str_repr)
    }
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
            MetarField::Qnh(qnh) => colourise_qnh(qnh),
            MetarField::WxCode(code, intensity, proximity, descriptor) => {
                colourise_wx_code(code, intensity, proximity, descriptor)
            }
            MetarField::Remarks(str) => str.black().on_white(),
            MetarField::Clouds(_, _) => todo!(),
        }
    }

    fn wxcode_from_str(_repr: &str) -> MetarField {
        // TODO
        MetarField::WxCode(
            WxCode::Ra,
            WxCodeIntensity::Light,
            WxCodeProximity::OnStation,
            WxCodeDescription::None,
        )
    }
}

fn colourise_wx_code(
    _code: &WxCode,
    _intensity: &WxCodeIntensity,
    _proximity: &WxCodeProximity,
    _descriptor: &WxCodeDescription,
) -> ColoredString {
    todo!()
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

        let exact_match = is_exact_match(&station, config);

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

fn get_visibility(json: &Value) -> Option<i64> {
    json.get("visibility")?.get("value")?.as_i64()
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
        assert!(metar.is_some());
        assert_eq!(metar.unwrap().icao_code, "EDRK");
    }

    #[test]
    fn test_metar_from_json_time() {
        let json: Value = Value::from_str("").unwrap();
        let config = Config {
            position: Position::Airfield("EDRK".to_string()),
        };

        let metar = Metar::from_json(json, &config);
        assert!(metar.is_some());
        assert!(metar.unwrap().fields.contains(&MetarField::TimeStamp));
    }

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
}
