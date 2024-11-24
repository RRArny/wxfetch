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
// WxFetch - metar/wxcodes.rs

use super::WxField;
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
            _ => Err(anyhow!("Invalid intensity code {s}.")),
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

pub(crate) fn wxcode_from_str(repr: &str) -> Option<WxField> {
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

    Some(WxField::WxCode(code, intensity, proximity, descriptor))
}

pub fn get_wxcodes_from_json(json: &Value) -> Vec<WxField> {
    let mut result: Vec<WxField> = Vec::new();
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
    use crate::metar::{wxcodes::WxCodeDescription, WxCodeProximity, WxField};

    #[tokio::test]
    async fn test_get_regex() {
        let expected: &str = "RA|DZ|GR|GS|IC|PL|SG|SN|UP|BR|DU|FG|FU|HZ|PY|SA|VA|DS|FC|PO|SQ|SS";
        let actual = WxCode::get_regex();
        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn test_get_wxcodes_empty() {
        let json: Value = Value::from_str("{\"wx_codes\":[]}").unwrap();
        let actual = get_wxcodes_from_json(&json);
        assert!(actual.is_empty());
    }

    #[tokio::test]
    async fn test_get_wxcodes_no_field() {
        let json: Value = Value::from_str("{}").unwrap();
        let actual = get_wxcodes_from_json(&json);
        assert!(actual.is_empty());
    }

    #[tokio::test]
    async fn test_get_wxcodes_one() {
        let expected: Vec<WxField> = vec![WxField::WxCode(
            WxCode::Ra,
            WxCodeIntensity::Light,
            WxCodeProximity::OnStation,
            crate::metar::WxCodeDescription::None,
        )];
        let json: Value = Value::from_str("{\"wx_codes\":[{\"repr\":\"-RA\"}]}").unwrap();
        let actual = get_wxcodes_from_json(&json);
        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn test_wxcode_from_str_ra() {
        let expected = WxCode::Ra;
        let actual = WxCode::from_str("RA").unwrap();
        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn test_wxcode_from_str_dz() {
        let expected = WxCode::Dz;
        let actual = WxCode::from_str("DZ").unwrap();
        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn test_wxcode_from_str_gr() {
        let expected = WxCode::Gr;
        let actual = WxCode::from_str("GR").unwrap();
        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn test_wxcode_from_str_gs() {
        let expected = WxCode::Gs;
        let actual = WxCode::from_str("GS").unwrap();
        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn test_wxcode_from_str_ic() {
        let expected = WxCode::Ic;
        let actual = WxCode::from_str("IC").unwrap();
        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn test_wxcode_from_str_pl() {
        let expected = WxCode::Pl;
        let actual = WxCode::from_str("PL").unwrap();
        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn test_wxcode_from_str_sg() {
        let expected = WxCode::Sg;
        let actual = WxCode::from_str("SG").unwrap();
        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn test_wxcode_from_str_sn() {
        let expected = WxCode::Sn;
        let actual = WxCode::from_str("SN").unwrap();
        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn test_wxcode_from_str_up() {
        let expected = WxCode::Up;
        let actual = WxCode::from_str("UP").unwrap();
        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn test_wxcode_from_str_br() {
        let expected = WxCode::Br;
        let actual = WxCode::from_str("BR").unwrap();
        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn test_wxcode_from_str_du() {
        let expected = WxCode::Du;
        let actual = WxCode::from_str("DU").unwrap();
        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn test_wxcode_from_str_fg() {
        let expected = WxCode::Fg;
        let actual = WxCode::from_str("FG").unwrap();
        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn test_wxcode_from_str_fu() {
        let expected = WxCode::Fu;
        let actual = WxCode::from_str("FU").unwrap();
        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn test_wxcode_from_str_hz() {
        let expected = WxCode::Hz;
        let actual = WxCode::from_str("HZ").unwrap();
        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn test_wxcode_from_str_py() {
        let expected = WxCode::Py;
        let actual = WxCode::from_str("PY").unwrap();
        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn test_wxcode_from_str_sa() {
        let expected = WxCode::Sa;
        let actual = WxCode::from_str("SA").unwrap();
        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn test_wxcode_from_str_va() {
        let expected = WxCode::Va;
        let actual = WxCode::from_str("VA").unwrap();
        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn test_wxcode_from_str_ds() {
        let expected = WxCode::Ds;
        let actual = WxCode::from_str("DS").unwrap();
        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn test_wxcode_from_str_fc() {
        let expected = WxCode::Fc;
        let actual = WxCode::from_str("FC").unwrap();
        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn test_wxcode_from_str_po() {
        let expected = WxCode::Po;
        let actual = WxCode::from_str("PO").unwrap();
        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn test_wxcode_from_str_sq() {
        let expected = WxCode::Sq;
        let actual = WxCode::from_str("SQ").unwrap();
        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn test_wxcode_from_str_ss() {
        let expected = WxCode::Ss;
        let actual = WxCode::from_str("SS").unwrap();
        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn test_wxcode_from_str_invalid() {
        assert!(WxCode::from_str("INVALID").is_err());
    }

    #[tokio::test]
    async fn test_prox_from_str_dist() {
        let expected = WxCodeProximity::Distant;
        let actual = WxCodeProximity::from_str("DSNT").unwrap();
        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn test_prox_from_str_vic() {
        let expected = WxCodeProximity::Vicinity;
        let actual = WxCodeProximity::from_str("VC").unwrap();
        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn test_prox_from_str_invalid() {
        assert!(WxCodeProximity::from_str("SOMEWHEREELSE").is_err());
    }

    #[tokio::test]
    async fn test_desc_from_str_ts() {
        let expected = WxCodeDescription::Ts;
        let actual = WxCodeDescription::from_str("TS").unwrap();
        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn test_desc_from_str_bc() {
        let expected = WxCodeDescription::Bc;
        let actual = WxCodeDescription::from_str("BC").unwrap();
        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn test_desc_from_str_bl() {
        let expected = WxCodeDescription::Bl;
        let actual = WxCodeDescription::from_str("BL").unwrap();
        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn test_desc_from_str_dr() {
        let expected = WxCodeDescription::Dr;
        let actual = WxCodeDescription::from_str("DR").unwrap();
        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn test_desc_from_str_fz() {
        let expected = WxCodeDescription::Fz;
        let actual = WxCodeDescription::from_str("FZ").unwrap();
        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn test_desc_from_str_mi() {
        let expected = WxCodeDescription::Mi;
        let actual = WxCodeDescription::from_str("MI").unwrap();
        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn test_desc_from_str_pr() {
        let expected = WxCodeDescription::Pr;
        let actual = WxCodeDescription::from_str("PR").unwrap();
        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn test_desc_from_str_sh() {
        let expected = WxCodeDescription::Sh;
        let actual = WxCodeDescription::from_str("SH").unwrap();
        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn test_desc_from_str_invalid() {
        assert!(WxCodeDescription::from_str("SOME").is_err());
    }

    #[tokio::test]
    async fn test_intens_from_str_invalid() {
        assert!(WxCodeIntensity::from_str("#").is_err())
    }

    #[tokio::test]
    async fn test_intens_display_light() {
        let code = WxCodeIntensity::Light;
        let expected = "-";
        let actual = code.to_string();
        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn test_intens_display_moderate() {
        let code = WxCodeIntensity::Moderate;
        let expected = "";
        let actual = code.to_string();
        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn test_intens_display_heavy() {
        let code = WxCodeIntensity::Heavy;
        let expected = "+";
        let actual = code.to_string();
        assert_eq!(expected, actual);
    }
}
