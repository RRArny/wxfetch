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
// WxFetch - config.rs

use std::{fs::File, io::Read};

use chrono::TimeDelta;
use toml::{Table, Value};

use crate::{
    Args, Secrets,
    api::check_icao_code,
    position::{LatLong, Position},
};

#[derive(PartialEq, Debug)]
#[allow(clippy::struct_excessive_bools)]
pub struct Config {
    pub position: Position,
    pub cloud_minimum: i64,
    pub cloud_marginal: i64,
    pub temp_minimum: i64,
    pub spread_minimum: i64,
    pub wind_var_maximum: i64,
    pub wind_maximum: i64,
    pub gust_maximum: i64,
    pub age_maximum: TimeDelta,
    pub age_marginal: TimeDelta,
    pub visibility_minimum: i64,
    pub visibility_marginal: i64,
    /// Print Terminal Aerodrome Forecast (TAF) instead of METAR
    pub print_taf: bool,
    /// TAF-specific: Maximum age for TAF forecasts (hours)
    pub taf_age_maximum: TimeDelta,
    /// TAF-specific: Marginal age for TAF forecasts (hours)
    pub taf_age_marginal: TimeDelta,
    /// TAF-specific: Highlight probability groups
    pub taf_highlight_probability: bool,
/// TAF-specific: Show change group time windows
    pub taf_show_change_times: bool,
    /// Raw output: disable ANSI color codes
    pub raw: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            position: Position::GeoIP,
            cloud_minimum: 6,
            cloud_marginal: 15,
            temp_minimum: 0,
            spread_minimum: 3,
            wind_var_maximum: 45,
            wind_maximum: 15,
            gust_maximum: 10,
            age_maximum: TimeDelta::hours(6),
            age_marginal: TimeDelta::hours(1),
            visibility_minimum: 1500,
            visibility_marginal: 5000,
            print_taf: false,
            taf_age_maximum: TimeDelta::hours(24), // TAFs valid longer
            taf_age_marginal: TimeDelta::hours(6), // TAFs age slower
            taf_highlight_probability: true,
            taf_show_change_times: true,
            raw: false,
        }
    }
}

impl Config {
    pub async fn get_config(secrets: &Secrets, args: &Args) -> Config {
        let mut config: Config = read_config_file(args.config_file.clone());

        if args.taf {
            config.print_taf = true;
        }

        if args.raw {
            config.raw = true;
        }

        if let Some(icao) = args.airfield.clone() {
            config.position = Position::Airfield(icao.clone());
        } else if let Some(lat) = args.latitude {
            if let Some(long) = args.longitude {
                config.position = Position::LatLong(LatLong(lat, long));
            }
            println!("Please provide both Latitude and Longitude. Defaulting to geoip...");
        }

        if let Position::Airfield(ref icao) = config.position
            && !check_icao_code(icao, secrets).await
        {
            println!("Invalid airfield {icao}. Defaulting to geoip...");
            config.position = Position::GeoIP;
        }
        config
    }
}

fn read_config_file(config_filepath: Option<String>) -> Config {
    let msg = "Failed to load config.";
    let mut config = Config::default();
    let config_filepath = config_filepath
        .unwrap_or(std::env::var("HOME").expect(msg) + "/.config/wxfetch/config.toml");
    let config_file = File::open(config_filepath.clone());
    if config_file.is_err() {
        return config;
    }
    let mut config_file = config_file.unwrap();
    let mut contents = String::new();
    config_file.read_to_string(&mut contents).expect(msg);
    let contents = contents.parse::<Table>().expect(msg);

    if contents.contains_key("position") {
        let position = &contents["position"];
        if let Some(airfield) = position.get("airfield").and_then(Value::as_str) {
            config.position = Position::Airfield(airfield.to_string());
        }
        if let Some(lat) = position.get("lat").and_then(Value::as_float)
            && let Some(lon) = contents["position"].get("lon").and_then(Value::as_float)
        {
            config.position = Position::LatLong(LatLong(lat, lon));
        }
    }

    if contents.contains_key("clouds") {
        let clouds = &contents["clouds"];
        if let Some(minimum) = clouds.get("cloud_minimum").and_then(Value::as_integer) {
            config.cloud_minimum = minimum;
        }
        if let Some(marginal) = clouds.get("cloud_marginal").and_then(Value::as_integer) {
            config.cloud_marginal = marginal;
        }
    }

    if contents.contains_key("temperature") {
        let temperature = &contents["temperature"];
        if let Some(minimum) = temperature.get("temp_minimum").and_then(Value::as_integer) {
            config.temp_minimum = minimum;
        }
        if let Some(spread_minimum) = temperature
            .get("spread_minimum")
            .and_then(Value::as_integer)
        {
            config.spread_minimum = spread_minimum;
        }
    }

    if contents.contains_key("wind") {
        let wind = &contents["wind"];
        if let Some(var_maximum) = wind.get("wind_var_maximum").and_then(Value::as_integer) {
            config.wind_var_maximum = var_maximum;
        }
        if let Some(maximum) = wind.get("wind_maximum").and_then(Value::as_integer) {
            config.wind_maximum = maximum;
        }
        if let Some(gust_maximum) = wind.get("gust_maximum").and_then(Value::as_integer) {
            config.gust_maximum = gust_maximum;
        }
    }

    if contents.contains_key("age") {
        let age = &contents["age"];
        if let Some(maximum) = age.get("age_maximum").and_then(Value::as_integer) {
            config.age_maximum = TimeDelta::seconds(maximum);
        }
        if let Some(marginal) = age.get("age_marginal").and_then(Value::as_integer) {
            config.age_marginal = TimeDelta::seconds(marginal);
        }
    }

    if contents.contains_key("visibility") {
        let visibility = &contents["visibility"];
        if let Some(minimum) = visibility
            .get("visibility_minimum")
            .and_then(Value::as_integer)
        {
            config.visibility_minimum = minimum;
        }
        if let Some(marginal) = visibility
            .get("visibility_marginal")
            .and_then(Value::as_integer)
        {
            config.visibility_marginal = marginal;
        }
    }

    if contents.contains_key("taf") {
        let taf = &contents["taf"];
        if let Some(maximum) = taf.get("taf_age_maximum").and_then(Value::as_integer) {
            config.taf_age_maximum = TimeDelta::seconds(maximum);
        }
        if let Some(marginal) = taf.get("taf_age_marginal").and_then(Value::as_integer) {
            config.taf_age_marginal = TimeDelta::seconds(marginal);
        }
        if let Some(highlight) = taf
            .get("taf_highlight_probability")
            .and_then(Value::as_bool)
        {
            config.taf_highlight_probability = highlight;
        }
        if let Some(show_times) = taf.get("taf_show_change_times").and_then(Value::as_bool) {
            config.taf_show_change_times = show_times;
        }
    }

    config
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs;

    #[tokio::test]
    async fn test_read_config_file() {
        let expected = Config::default();
        let actual = read_config_file(Some("./config.toml".to_string()));
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_read_config_file_nonexistent() {
        // Should return default config when file doesn't exist
        let config = read_config_file(Some("nonexistent_config.toml".to_string()));
        let expected = Config::default();
        assert_eq!(config, expected);
    }

    #[test]
    fn test_read_config_file_empty() {
        // Create a temporary empty TOML file
        let temp_path = "temp_empty_config.toml";
        fs::write(temp_path, "").expect("Failed to create temp file");

        let config = read_config_file(Some(temp_path.to_string()));
        let expected = Config::default();
        assert_eq!(config, expected);

        // Clean up
        let _ = fs::remove_file(temp_path);
    }

    #[test]
    #[should_panic(expected = "Failed to load config")]
    fn test_read_config_file_invalid_toml() {
        // Create a temporary invalid TOML file
        let temp_path = "temp_invalid_config.toml";
        fs::write(temp_path, "invalid toml [ content").expect("Failed to create temp file");

        // This should panic when trying to parse invalid TOML
        let _config = read_config_file(Some(temp_path.to_string()));

        // Clean up (this won't run due to panic, but good practice)
        let _ = fs::remove_file(temp_path);
    }

    #[test]
    fn test_read_config_file_partial_sections() {
        // Test with a config file that has only some sections
        let temp_path = "temp_partial_config.toml";
        let toml_content = r#"
[position]
airfield = "KJFK"

[clouds]
cloud_minimum = 1500
cloud_marginal = 3000
"#;
        fs::write(temp_path, toml_content).expect("Failed to create temp file");

        let config = read_config_file(Some(temp_path.to_string()));

        // Should have the values from the file
        assert_eq!(config.position, Position::Airfield("KJFK".to_string()));
        assert_eq!(config.cloud_minimum, 1500);
        assert_eq!(config.cloud_marginal, 3000);

        // Should have default values for missing sections
        let default = Config::default();
        assert_eq!(config.temp_minimum, default.temp_minimum);
        assert_eq!(config.wind_maximum, default.wind_maximum);

        // Clean up
        let _ = fs::remove_file(temp_path);
    }

    #[test]
    fn test_read_config_file_lat_long_position() {
        // Test with lat/long position configuration
        let temp_path = "temp_latlong_config.toml";
        let toml_content = r"
[position]
lat = 40.7128
lon = -74.0060
";
        fs::write(temp_path, toml_content).expect("Failed to create temp file");

        let config = read_config_file(Some(temp_path.to_string()));

        assert_eq!(
            config.position,
            Position::LatLong(LatLong(40.7128, -74.0060))
        );

        // Clean up
        let _ = fs::remove_file(temp_path);
    }

    #[test]
    fn test_read_config_file_all_sections() {
        // Test with all config sections populated
        let temp_path = "temp_full_config.toml";
        let toml_content = r#"
[position]
airfield = "EGLL"

[clouds]
cloud_minimum = 2000
cloud_marginal = 4000

[temperature]
temp_minimum = -10
spread_minimum = 3

[wind]
wind_var_maximum = 60
wind_maximum = 25
gust_maximum = 35

[age]
age_maximum = 7200
age_marginal = 3600

[visibility]
visibility_minimum = 5000
visibility_marginal = 8000

[taf]
taf_age_maximum = 28800
taf_age_marginal = 14400
taf_highlight_probability = true
taf_show_change_times = false
"#;
        fs::write(temp_path, toml_content).expect("Failed to create temp file");

        let config = read_config_file(Some(temp_path.to_string()));

        // Verify all values are parsed correctly
        assert_eq!(config.position, Position::Airfield("EGLL".to_string()));
        assert_eq!(config.cloud_minimum, 2000);
        assert_eq!(config.cloud_marginal, 4000);
        assert_eq!(config.temp_minimum, -10);
        assert_eq!(config.spread_minimum, 3);
        assert_eq!(config.wind_var_maximum, 60);
        assert_eq!(config.wind_maximum, 25);
        assert_eq!(config.gust_maximum, 35);
        assert_eq!(config.age_maximum, TimeDelta::seconds(7200));
        assert_eq!(config.age_marginal, TimeDelta::seconds(3600));
        assert_eq!(config.visibility_minimum, 5000);
        assert_eq!(config.visibility_marginal, 8000);
        assert_eq!(config.taf_age_maximum, TimeDelta::seconds(28800));
        assert_eq!(config.taf_age_marginal, TimeDelta::seconds(14400));
        assert!(config.taf_highlight_probability);
        assert!(!config.taf_show_change_times);

        // Clean up
        let _ = fs::remove_file(temp_path);
    }

    #[test]
    fn test_read_config_file_invalid_field_types() {
        // Test graceful handling of wrong data types (should use defaults)
        let temp_path = "temp_invalid_types_config.toml";
        let toml_content = r#"
[clouds]
cloud_minimum = "not_a_number"
cloud_marginal = 3000

[wind]
wind_maximum = true
"#;
        fs::write(temp_path, toml_content).expect("Failed to create temp file");

        let config = read_config_file(Some(temp_path.to_string()));

        // Should use default values for invalid types, but keep valid ones
        let default = Config::default();
        assert_eq!(config.cloud_minimum, default.cloud_minimum); // Should be default (invalid type)
        assert_eq!(config.cloud_marginal, 3000); // Should be parsed (valid type)
        assert_eq!(config.wind_maximum, default.wind_maximum); // Should be default (invalid type)

        // Clean up
        let _ = fs::remove_file(temp_path);
    }
}
