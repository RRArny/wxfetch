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
// WxFetch - taf.rs

use chrono::{DateTime, FixedOffset, Utc};
use colored::{Color, ColoredString, Colorize};
use serde_json::Value;

use crate::config::Config;
use crate::metar::{
    clouds::get_clouds_from_json,
    units::{Units},
    wxcodes::get_wxcodes_from_json,
    WxField,
};
use crate::metar::{get_visibility, get_winds, is_exact_match};

pub struct Taf {
    /// ICAO code of the issuing station
    icao_code: String,
    /// Issue time of the forecast
    issue_time: DateTime<FixedOffset>,
    /// Validity period start and end
    validity_start: DateTime<FixedOffset>,
    validity_end: DateTime<FixedOffset>,
    /// Forecast periods and change groups
    forecast_periods: Vec<ForecastPeriod>,
    /// True if this TAF was issued by the exact station requested
    exact_match: bool,
    /// Units parsed from the report
    units: Units,
}

/// Represents a forecast period or change group
pub struct ForecastPeriod {
    /// Type of period (FM, BECMG, TEMPO, or initial forecast)
    period_type: PeriodType,
    /// Start time of this period
    start_time: Option<DateTime<FixedOffset>>,
    /// End time of this period (if applicable)
    end_time: Option<DateTime<FixedOffset>>,
    /// Weather fields for this period
    fields: Vec<WxField>,
    /// Probability for PROB groups (30 or 40)
    probability: Option<u8>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum PeriodType {
    Initial,     // Initial forecast period
    From,        // FM - From (permanent change)
    Becoming,    // BECMG - Becoming (gradual change)
    Temporary,   // TEMPO - Temporary fluctuation
    Probability, // PROBxx - Probability
}

impl Taf {
    pub fn from_json(json: &Value, config: &Config) -> Option<Self> {
        let station = json.get("station")?.as_str()?.to_string();

        // Parse issue time
        let issue_time = parse_issue_time(json)?;

        // Parse validity period
        let (validity_start, validity_end) = parse_validity_period(json)?;

        // Parse units
        let units = Units::from_json(json);

        // Parse forecast periods
        let forecast_periods = parse_forecast_periods(json, units)?;

        let exact_match = is_exact_match(&station, config);

        Some(Taf {
            icao_code: station,
            issue_time,
            validity_start,
            validity_end,
            forecast_periods,
            exact_match,
            units,
        })
    }

    pub fn colourise(&self, config: &Config) -> ColoredString {
        // Start with "TAF" header
        let mut output = "TAF ".bright_white();

        // Add station identifier with coloring based on exact match
        let station_str = if self.exact_match {
            self.icao_code.bright_white().on_blue()
        } else {
            self.icao_code.black().on_yellow()
        };
        output = format!("{output}{station_str}").into();

        // Add issue time
        let issue_str = format!(" {}", self.issue_time.format("%d%H%MZ"))
            .color(get_time_color(&self.issue_time, config));
        output = format!("{output}{issue_str}").into();

        // Add validity period (DDHH/DDHH format)
        let validity_str = format!(
            " {}/{}",
            self.validity_start.format("%d%H"),
            self.validity_end.format("%d%H")
        )
        .bright_cyan();
        output = format!("{output}{validity_str}").into();

        // Add initial forecast period (first period without change indicator)
        if let Some(initial_period) = self.forecast_periods.first()
            && initial_period.period_type == PeriodType::Initial
        {
            let period_str =
                colourise_forecast_period(initial_period, config, false, self.units);
            output = format!("{output} {period_str}").into();
        }

        // Add change groups with proper indentation
        for period in self.forecast_periods.iter().skip(1) {
            let period_str =
                colourise_forecast_period(period, config, true, self.units);
            output = format!("{output}\n     {period_str}").into();
        }

        output
    }
}

fn parse_issue_time(json: &Value) -> Option<DateTime<FixedOffset>> {
    let time_str = json.get("time")?.get("dt")?.as_str()?;
    DateTime::parse_from_rfc3339(time_str).ok()
}

fn parse_validity_period(json: &Value) -> Option<(DateTime<FixedOffset>, DateTime<FixedOffset>)> {
    let start_str = json.get("start_time")?.get("dt")?.as_str()?;
    let end_str = json.get("end_time")?.get("dt")?.as_str()?;
    let start_time = DateTime::parse_from_rfc3339(start_str).ok()?;
    let end_time = DateTime::parse_from_rfc3339(end_str).ok()?;
    Some((start_time, end_time))
}

fn parse_forecast_periods(
    json: &Value,
    units: Units,
) -> Option<Vec<ForecastPeriod>> {
    let mut periods = Vec::new();

    if let Some(forecast_array) = json.get("forecast")?.as_array() {
        for (i, change_group) in forecast_array.iter().enumerate() {
            if let Some(period) = parse_change_group(change_group, i == 0, units) {
                periods.push(period);
            }
        }
    }

    Some(periods)
}

fn parse_change_group(
    json: &Value,
    is_initial: bool,
    units: Units,
) -> Option<ForecastPeriod> {
    let period_type = if is_initial {
        PeriodType::Initial
    } else {
        match json.get("type")?.as_str()? {
            "FM" => PeriodType::From,
            "BECMG" => PeriodType::Becoming,
            "TEMPO" => PeriodType::Temporary,
            "PROB" => PeriodType::Probability,
            _ => return None,
        }
    };

    let start_time = json
        .get("start_time")
        .and_then(|t| t.get("dt"))
        .and_then(|t| t.as_str())
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok());

    let end_time = json
        .get("end_time")
        .and_then(|t| t.get("dt"))
        .and_then(|t| t.as_str())
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok());

    let probability = json
        .get("probability")
        .and_then(|p| p.get("value"))
        .and_then(serde_json::Value::as_u64)
        .and_then(|p| u8::try_from(p).ok());

    let mut fields: Vec<WxField> = Vec::new();

    // Parse standard weather fields (wind, visibility, etc.)
    if let Some(wind) = get_winds(json) {
        fields.push(wind);
    }

    if let Some(vis) = get_visibility(json) {
        fields.push(vis);
    }

    // Parse weather codes and clouds
    fields.append(&mut get_wxcodes_from_json(json));
    fields.append(&mut get_clouds_from_json(json));

    // Parse wind shear
    if let Some(wind_shear) = parse_wind_shear(json, units) {
        fields.push(wind_shear);
    }

    // Parse temperature extremes
    if let Some(max_temp) = parse_max_temp(json, units) {
        fields.push(max_temp);
    }

    if let Some(min_temp) = parse_min_temp(json, units) {
        fields.push(min_temp);
    }

    Some(ForecastPeriod {
        period_type,
        start_time,
        end_time,
        fields,
        probability,
    })
}

fn parse_wind_shear(json: &Value, units: Units) -> Option<WxField> {
    let shear = json.get("wind_shear")?;
    let altitude = shear.get("value")?.as_i64()?;
    let direction = shear.get("wind_direction")?.get("value")?.as_i64()?;
    let strength = shear.get("wind_speed")?.get("value")?.as_i64()?;
    Some(WxField::WindShear {
        altitude,
        direction,
        strength,
        unit: units.wind_speed,
    })
}

fn parse_max_temp(json: &Value, units: Units) -> Option<WxField> {
    let max_t = json.get("max_temperature")?;
    let temp = max_t.get("value")?.as_i64()?;
    let time_str = max_t.get("time")?.get("dt")?.as_str()?;
    let time = DateTime::parse_from_rfc3339(time_str).ok()?;
    Some(WxField::MaxTemperature {
        temp,
        time,
        unit: units.temperature,
    })
}

fn parse_min_temp(json: &Value, units: Units) -> Option<WxField> {
    let min_t = json.get("min_temperature")?;
    let temp = min_t.get("value")?.as_i64()?;
    let time_str = min_t.get("time")?.get("dt")?.as_str()?;
    let time = DateTime::parse_from_rfc3339(time_str).ok()?;
    Some(WxField::MinTemperature {
        temp,
        time,
        unit: units.temperature,
    })
}

fn colourise_forecast_period(
    period: &ForecastPeriod,
    config: &Config,
    show_change_indicator: bool,
    units: Units,
) -> ColoredString {
    let mut period_output: ColoredString = "".normal();

    // Add change indicator only if requested and for change periods
    if show_change_indicator && config.taf_show_change_times {
        match period.period_type {
            PeriodType::From => {
                if let Some(start_time) = period.start_time {
                    period_output = format!("FM{}", start_time.format("%d%H%M"))
                        .bright_yellow();
                }
            }
            PeriodType::Becoming => {
                if let (Some(start_time), Some(end_time)) = (period.start_time, period.end_time)
                {
                    period_output = format!(
                        "BECMG {}/{}",
                        start_time.format("%d%H"),
                        end_time.format("%d%H")
                    )
                    .bright_magenta();
                } else {
                    period_output = "BECMG".bright_magenta();
                }
            }
            PeriodType::Temporary => {
                if let (Some(start_time), Some(end_time)) = (period.start_time, period.end_time)
                {
                    period_output = format!(
                        "TEMPO {}/{}",
                        start_time.format("%d%H"),
                        end_time.format("%d%H")
                    )
                    .bright_blue();
                } else {
                    period_output = "TEMPO".bright_blue();
                }
            }
            PeriodType::Probability => {
                if let Some(prob) = period.probability {
                    if let (Some(start_time), Some(end_time)) =
                        (period.start_time, period.end_time)
                    {
                        period_output = format!(
                            "PROB{} {}/{}",
                            prob,
                            start_time.format("%d%H"),
                            end_time.format("%d%H")
                        )
                        .bright_red();
                    } else {
                        period_output = format!("PROB{prob}").bright_red();
                    }
                }
            }
            PeriodType::Initial => {} // No indicator for initial period
        }
    }

    // Add weather fields
    for (i, field) in period.fields.iter().enumerate() {
        let field_str = field.colourise(config, units);
        if i > 0 || !period_output.is_empty() {
            period_output = format!("{period_output} {field_str}").into();
        } else {
            period_output = field_str;
        }
    }

    period_output
}

fn get_time_color(datetime: &DateTime<FixedOffset>, config: &Config) -> Color {
    let now = Utc::now();
    let utctime = datetime.to_utc();
    let dt = now.signed_duration_since(utctime);

    if dt < config.taf_age_marginal {
        Color::Green
    } else if dt < config.taf_age_maximum {
        Color::Yellow
    } else {
        Color::Red
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use serde_json::Value;

    fn load_test_taf_data(file_name: &str) -> Value {
        let path = format!("tests/testdata/{file_name}");
        let data = std::fs::read_to_string(path).expect("Unable to read file");
        serde_json::from_str(&data).expect("JSON was not well-formatted")
    }

    #[tokio::test]
    async fn test_taf_from_json_basic() {
        let value = load_test_taf_data("kjfk-taf.json");
        let config = Config::default();
        let taf = Taf::from_json(&value, &config);

        assert!(taf.is_some());
        assert_eq!(taf.unwrap().icao_code, "KJFK");
    }

    #[tokio::test]
    async fn test_forecast_period_parsing() {
        let value = load_test_taf_data("kjfk-taf.json");
        let config = Config::default();
        let taf = Taf::from_json(&value, &config);

        assert!(taf.is_some());
        let taf = taf.unwrap();
        assert_eq!(taf.forecast_periods.len(), 2);
        assert_eq!(taf.forecast_periods[1].period_type, PeriodType::Becoming);
    }

    #[tokio::test]
    async fn test_taf_colorization() {
        let value = load_test_taf_data("kjfk-taf.json");
        let config = Config::default();
        let taf = Taf::from_json(&value, &config);

        assert!(taf.is_some());
        let taf = taf.unwrap();
        let colored = taf.colourise(&config);

        let output = colored.to_string();
        assert!(
            output.contains("KJFK"),
            "Output should contain KJFK, but got: {output}"
        );
        assert!(
            output.contains("TAF"),
            "Output should contain TAF header, but got: {output}"
        );
        assert!(
            output.starts_with("TAF ")
                || output
                    .chars()
                    .skip_while(|&c| {
                        c == '\x1b' || c == '[' || c.is_ascii_digit() || c == ';' || c == 'm'
                    })
                    .collect::<String>()
                    .starts_with("TAF "),
            "Output should start with 'TAF ', but got: {output}"
        );
    }

    #[tokio::test]
    async fn test_taf_format_structure() {
        let value = load_test_taf_data("kjfk-taf.json");
        let config = Config::default();
        let taf = Taf::from_json(&value, &config);

        assert!(taf.is_some());
        let taf = taf.unwrap();
        let colored = taf.colourise(&config);
        let output = colored.to_string();

        println!("TAF Output:\n{output}");

        let taf_pos = output.find("TAF").expect("TAF header not found");
        let kjfk_pos = output.find("KJFK").expect("KJFK not found");
        assert!(
            taf_pos < kjfk_pos,
            "TAF header should come before station identifier"
        );
    }

    #[tokio::test]
    async fn test_taf_prob_group() {
        let value = load_test_taf_data("eddf-taf-prob.json");
        let config = Config::default();
        let taf = Taf::from_json(&value, &config);

        assert!(taf.is_some());
        let taf = taf.unwrap();
        let colored = taf.colourise(&config);
        let output = colored.to_string();

        assert!(
            output.starts_with("TAF "),
            "TAF should start with 'TAF ' header, but got: '{output}'"
        );
        assert!(
            output.contains("EDDF"),
            "Should contain station identifier, but got: {output}"
        );
        assert!(
            output.contains("PROB30"),
            "Should contain probability indicator, but got: {output}"
        );

        println!("TAF PROB Output:\n{output}");
    }

    #[tokio::test]
    async fn test_taf_egkk_simple() {
        let value = load_test_taf_data("egkk-taf-simple.json");
        let config = Config::default();
        let taf = Taf::from_json(&value, &config);

        assert!(taf.is_some());
        assert_eq!(taf.unwrap().icao_code, "EGKK");
    }

    #[tokio::test]
    async fn test_taf_prob_highlighting_enabled() {
        let config = Config {
            taf_highlight_probability: true,
            ..Config::default()
        };
        assert!(config.taf_highlight_probability);
    }

    #[tokio::test]
    async fn test_taf_prob_highlighting_disabled() {
        let config = Config {
            taf_highlight_probability: false,
            ..Config::default()
        };
        assert!(!config.taf_highlight_probability);
    }
}
