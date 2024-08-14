use super::MetarField;
use anyhow::{anyhow, Error};
use regex::Regex;
use serde_json::Value;
use std::{fmt::Display, str::FromStr};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(PartialEq, Eq, Debug, EnumIter)]
/// Standardised codes for weather phenomena.
pub enum WxCode {
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

impl WxCode {
    fn get_regex() -> String {
        let mut res: String = String::new();
        for val in Self::iter() {
            res.push_str(val.to_string().as_str());
            res.push('|');
        }
        if res.ends_with('|') {
            res.truncate(res.len() - 1);
        }
        res
    }
}

impl FromStr for WxCode {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ra" => Ok(Self::Ra),
            "dz" => Ok(Self::Dz),
            "gr" => Ok(Self::Gr),
            "gs" => Ok(Self::Gs),
            "ic" => Ok(Self::Ic),
            "pl" => Ok(Self::Pl),
            "sg" => Ok(Self::Sg),
            "sn" => Ok(Self::Sn),
            "up" => Ok(Self::Up),
            "br" => Ok(Self::Br),
            "du" => Ok(Self::Du),
            "fg" => Ok(Self::Fg),
            "fu" => Ok(Self::Fu),
            "hz" => Ok(Self::Hz),
            "py" => Ok(Self::Py),
            "sa" => Ok(Self::Sa),
            "va" => Ok(Self::Va),
            "ds" => Ok(Self::Ds),
            "fc" => Ok(Self::Fc),
            "po" => Ok(Self::Po),
            "sq" => Ok(Self::Sq),
            "ss" => Ok(Self::Ss),
            _ => Err(anyhow!("Invalid weather code {}.", s)),
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
/// Used to specify a weather phenomenon's intensity.
pub enum WxCodeIntensity {
    Moderate,
    Light,
    Heavy,
}

#[derive(PartialEq, Eq, Debug, EnumIter)]
/// Used to specify a weather phenomenon's distance from reporting staion.
pub enum WxCodeProximity {
    /// On station.
    OnStation,
    /// In vicinity of the station (distance 5 - 10 miles).
    Vicinity,
    /// More than 10 miles from station.
    Distant,
}

impl WxCodeProximity {
    fn get_regex() -> String {
        let mut res: String = String::new();
        for val in Self::iter() {
            res.push_str(val.to_string().as_str());
            res.push('|');
        }
        if res.ends_with('|') {
            res.truncate(res.len() - 1);
        }
        res
    }
}

impl FromStr for WxCodeProximity {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "" => Ok(Self::OnStation),
            "vc" => Ok(Self::Vicinity),
            "dsnt" => Ok(Self::Distant),
            _ => Err(anyhow!("Invalid weather proximity code {}.", s)),
        }
    }
}

#[derive(PartialEq, Eq, Debug, EnumIter)]
/// Used to further specify a weather phenomenon.
pub enum WxCodeDescription {
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

impl WxCodeDescription {
    fn get_regex() -> String {
        let mut res: String = String::new();
        for val in Self::iter() {
            res.push_str(val.to_string().as_str());

            if !val.to_string().is_empty() {
                res.push('|');
            }
        }
        if res.ends_with('|') {
            res.truncate(res.len() - 1);
        }
        res
    }
}

impl FromStr for WxCodeDescription {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "" => Ok(Self::None),
            "ts" => Ok(Self::Ts),
            "bc" => Ok(Self::Bc),
            "bl" => Ok(Self::Bl),
            "dr" => Ok(Self::Dr),
            "fz" => Ok(Self::Fz),
            "mi" => Ok(Self::Mi),
            "pr" => Ok(Self::Pr),
            "sh" => Ok(Self::Sh),
            _ => Err(anyhow!("Invalid weather code descriptor {}.", s)),
        }
    }
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
        write!(f, "{str_repr}")
    }
}

impl WxCodeIntensity {
    fn get_regex() -> String {
        String::from(r"[+]|-")
    }
}

impl FromStr for WxCodeIntensity {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "" => Ok(Self::Moderate),
            "+" => Ok(Self::Heavy),
            "-" => Ok(Self::Light),
            _ => Err(anyhow!("Invalid proximity code {s}.")),
        }
    }
}

impl Display for WxCodeIntensity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str_repr: &str = match self {
            WxCodeIntensity::Moderate => "",
            WxCodeIntensity::Light => "-",
            WxCodeIntensity::Heavy => "+",
        };
        write!(f, "{str_repr}")
    }
}

impl Display for WxCodeProximity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str_repr: &str = match self {
            WxCodeProximity::OnStation => "",
            WxCodeProximity::Vicinity => "VC",
            WxCodeProximity::Distant => "DSNT",
        };
        write!(f, "{str_repr}")
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
        write!(f, "{str_repr}")
    }
}

pub(crate) fn wxcode_from_str(repr: &str) -> Option<MetarField> {
    let regex_pattern = format!(
        r"(?<intensity>({})?)(?<descr>({})?)(?<code>{})(?<location>({})?)",
        WxCodeIntensity::get_regex(),
        WxCodeDescription::get_regex(),
        WxCode::get_regex(),
        WxCodeProximity::get_regex()
    );
    let regex = Regex::new(&regex_pattern)
        .expect("Creating RegEx pattern failed. This is likely a software bug, please report it.");
    let matches = regex.captures(repr)?;
    let code: WxCode = matches["code"].parse().ok()?;
    let intensity: WxCodeIntensity = matches["intensity"].parse().ok()?;
    let descriptor: WxCodeDescription = matches["descr"].parse().ok()?;
    let proximity: WxCodeProximity = matches["location"].parse().ok()?;

    Some(MetarField::WxCode(code, intensity, proximity, descriptor))
}

pub fn get_wxcodes_from_json(json: &Value) -> Vec<MetarField> {
    let mut result: Vec<MetarField> = Vec::new();
    if let Some(wxcodes) = json.get("wx_codes").and_then(|x| x.as_array()) {
        for code in wxcodes {
            if let Some(repr) = code.get("repr").and_then(|x| x.as_str()) {
                if let Some(field) = wxcode_from_str(repr) {
                    result.push(field);
                }
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use serde_json::Value;
    use std::str::FromStr;

    use super::{get_wxcodes_from_json, WxCode, WxCodeIntensity};
    use crate::metar::{wxcode_from_str, MetarField, WxCodeProximity};

    #[test]
    fn test_get_regex() {
        let expected: &str = "RA|DZ|GR|GS|IC|PL|SG|SN|UP|BR|DU|FG|FU|HZ|PY|SA|VA|DS|FC|PO|SQ|SS";
        let actual = WxCode::get_regex();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_get_wxcodes_empty() {
        let json: Value = Value::from_str("{\"wx_codes\":[]}").unwrap();
        let actual = get_wxcodes_from_json(&json);
        assert!(actual.is_empty());
    }

    #[test]
    fn test_get_wxcodes_no_field() {
        let json: Value = Value::from_str("{}").unwrap();
        let actual = get_wxcodes_from_json(&json);
        assert!(actual.is_empty());
    }

    #[test]
    fn test_get_wxcodes_one() {
        let expected: Vec<MetarField> = vec![MetarField::WxCode(
            WxCode::Ra,
            WxCodeIntensity::Light,
            WxCodeProximity::OnStation,
            crate::metar::WxCodeDescription::None,
        )];
        let json: Value = Value::from_str("{\"wx_codes\":[{\"repr\":\"-RA\"}]}").unwrap();
        let actual = get_wxcodes_from_json(&json);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_wxcode_from_str() {
        let expected: MetarField = MetarField::WxCode(
            WxCode::Ra,
            WxCodeIntensity::Light,
            WxCodeProximity::OnStation,
            crate::metar::WxCodeDescription::None,
        );
        let actual = wxcode_from_str("-RA");
        assert_eq!(Some(expected), actual);
    }
}
