use std::fmt::Display;

use super::MetarField;
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

pub fn get_clouds(json: &Value) -> Vec<MetarField> {
    let mut result: Vec<MetarField> = Vec::new();

    // json.get("clouds").and_then(|x| x.as_array()).and_then(|x| x.into_iter().map())

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

fn clouds_from_str(_repr: &str) -> Option<MetarField> {
    let _regex = format!("(?<obscuration>{})(?<level>\\d[2,3])", Clouds::get_regex());
    // TODO
    Some(MetarField::Clouds(Clouds::Skc, 0))
}

impl Clouds {
    fn get_regex() -> String {
        let mut res: String = String::new();
        for val in Self::iter() {
            res.push_str(val.to_string().as_str());
            res.push('|');
        }
        if res.ends_with("|") {
            res.truncate(res.len() - 1);
        }
        res
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
        write!(f, "{}", str_repr)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use serde_json::Value;

    use crate::metar::MetarField;

    use super::{clouds_from_str, get_clouds, Clouds};

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
        let actual = get_clouds(&json);
        assert_eq!(expected, actual);
    }
}
