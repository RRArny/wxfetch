use std::{fmt::Display, str::FromStr};

use super::MetarField;
use anyhow::anyhow;
use regex::Regex;
use serde_json::Value;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(PartialEq, Eq, Debug, EnumIter)]
/// Describes a cloud layer.
pub enum Clouds {
    /// Sky clear.
    Skc,
    /// Few. Up to 2 / 8 coverage.
    Few,
    // Scattered. 3 - 4 / 8 coverage.
    Sct,
    /// Broken. 5 - 7 / 8 coverage.
    Brk,
    /// Overcast. 8 / 8 coverage.
    Ovc,
}

/// Parses a METAR in JSON form and returns a `Vec` of `MetarField::Clouds` describing the cloud information contained.
pub fn get_clouds_from_json(json: &Value) -> Vec<MetarField> {
    let mut result: Vec<MetarField> = Vec::new();
    if let Some(wxcodes) = json.get("clouds").and_then(|x| x.as_array()) {
        for code in wxcodes {
            if let Some(repr) = code.get("repr").and_then(|x| x.as_str()) {
                if let Some(cloud) = clouds_from_str(repr) {
                    result.push(cloud);
                }
            }
        }
    }
    result
}

/// From a METAR compliant cloud code representation string (`&str`) parses a `MetarField::Cloud`.
fn clouds_from_str(repr: &str) -> Option<MetarField> {
    let regex = format!("(?<obscuration>{})(?<level>\\d*)", Clouds::get_regex());
    let regex = Regex::new(&regex)
        .expect("Creating RegEx pattern failed. This is likely a software bug, please report it.");
    let matches = regex.captures(repr)?;
    let obscuration: Clouds = matches["obscuration"].parse().ok()?;
    let level: i64 = matches["level"].parse().unwrap_or(0);
    Some(MetarField::Clouds(obscuration, level))
}

impl Clouds {
    /// Generates a regex that will match on textual representation of any of the valid obscuration levels.
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

impl FromStr for Clouds {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "skc" => Ok(Self::Skc),
            "few" => Ok(Self::Few),
            "sct" => Ok(Self::Sct),
            "brk" => Ok(Self::Brk),
            "ovc" => Ok(Self::Ovc),
            _ => Err(anyhow!("Invalid cloud obscuration {s}.")),
        }
    }
}

impl Display for Clouds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str_repr: &str = match self {
            Clouds::Skc => "SKC",
            Clouds::Few => "FEW",
            Clouds::Sct => "SCT",
            Clouds::Brk => "BRK",
            Clouds::Ovc => "OVC",
        };
        write!(f, "{str_repr}")
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use serde_json::Value;

    use crate::metar::MetarField;

    use super::{clouds_from_str, get_clouds_from_json, Clouds};

    #[test]
    fn test_get_regex() {
        let expected: &str = "SKC|FEW|SCT|BRK|OVC";
        let actual = Clouds::get_regex();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_clouds_from_str() {
        let expected = MetarField::Clouds(Clouds::Skc, 0);
        let actual = clouds_from_str("SKC");
        assert_eq!(Some(expected), actual);
    }

    #[test]
    fn test_clouds_from_str_sct() {
        let expected = MetarField::Clouds(Clouds::Sct, 50);
        let actual = clouds_from_str("SCT50");
        assert_eq!(Some(expected), actual);
    }

    #[test]
    fn test_get_clouds() {
        let json: Value = Value::from_str(
            "{\"clouds\":[{\"repr\": \"SCT050\"},{\"repr\": \"BRK100\"},{\"repr\": \"OVC200\"}]}",
        )
        .unwrap();
        let expected: Vec<MetarField> = vec![
            MetarField::Clouds(Clouds::Sct, 50),
            MetarField::Clouds(Clouds::Brk, 100),
            MetarField::Clouds(Clouds::Ovc, 200),
        ];
        let actual = get_clouds_from_json(&json);
        assert_eq!(expected, actual);
    }
}
