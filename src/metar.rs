use colored::{ColoredString, Colorize};

use crate::{Config, Position};
use serde_json::Value;

pub struct Metar {
    icao_code: String,
    fields: Vec<MetarField>,
    exact_match: bool,
}

#[derive(PartialEq, Eq)]
enum MetarField {
    TimeStamp,
    Wind,
    WindVariability,
    Visibility(u64),
    Temperature,
    DewPoint,
    Qnh,
    WxCodes,
    Remarks,
}

impl MetarField {
    pub fn colourise(&self) -> ColoredString {
        match self {
            Self::Visibility(vis) => colourise_visibility(vis),
            MetarField::TimeStamp => todo!(),
            MetarField::Wind => todo!(),
            MetarField::WindVariability => todo!(),
            MetarField::Temperature => todo!(),
            MetarField::DewPoint => todo!(),
            MetarField::Qnh => todo!(),
            MetarField::WxCodes => todo!(),
            MetarField::Remarks => todo!(),
        }
    }
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
    pub fn from_json(json: Value, config: &Config) -> Self {
        let mut fields: Vec<MetarField> = Vec::new();

        if let Some(vis) = get_visibility(&json) {
            fields.push(MetarField::Visibility(vis));
        }
        let exact_match = is_exact_match(&json, config);

        Metar {
            icao_code: "EDRK".to_string(),
            fields,
            exact_match,
        }
    }

    pub fn colorise(self) -> ColoredString {
        let mut coloured_string: ColoredString = if self.exact_match {
            self.icao_code.bright_white().on_blue()
        } else {
            self.icao_code.bright_white().on_yellow()
        };

        for field in self.fields {
            coloured_string = format!("{} {}", coloured_string, field.colourise()).into();
        }

        coloured_string
    }
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
