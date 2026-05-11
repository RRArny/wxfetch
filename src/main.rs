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
// WxFetch - main.rs

use std::fs::File;
use std::io::BufReader;

use std::sync::OnceLock;

use colored::ColoredString;
use regex::Regex;

use api::request_wx;
use clap::Parser;

mod metar;
use metar::Metar;

mod taf;
use taf::Taf;

mod position;

mod api;

mod config;
use config::Config;
use serde_json::Value;

/// Strip ANSI escape codes from a string for raw output.
fn strip_ansi_escapes(input: &str) -> String {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\x1b\[[0-9;]*m").unwrap())
        .replace_all(input, "")
        .into_owned()
}

#[derive(Parser, Debug)]
#[command(version, about)]
/// Console utility for accessing aviation weather information from the commmand line.
///
/// For more information see README.md or <https://github.com/RRArny/wxfetch>.
struct Args {
    #[arg(short, long, value_name = "ICAO code for an Airfield.")]
    airfield: Option<String>,
    #[arg(long = "lat", value_name = "Latitude")]
    latitude: Option<f64>,
    #[arg(long = "lon", value_name = "Longitude")]
    longitude: Option<f64>,
    #[arg(short, long, value_name = "Config File")]
    config_file: Option<String>,
    #[arg(short, long = "file", value_name = "JSON Source File")]
    file: Option<String>,
    #[arg(short, long, value_name = "AvWx API key")]
    key: Option<String>,
    #[arg(short = 't', long, value_name = "Print Terminal Aerodrome Forecast")]
    taf: bool,
    #[arg(short = 'R', long, value_name = "Raw output (no ANSI colors)")]
    raw: bool,
}

struct Secrets {
    avwx_api_key: String,
}

fn get_secrets(param: Option<String>) -> Secrets {
    let avwx_api_key =
        param.unwrap_or(std::env::var("AVWX_API_KEY").expect("Could not load secret key."));
    Secrets { avwx_api_key }
}

async fn get_weather(config: &Config, secrets: &Secrets) -> Value {
    request_wx(config, secrets)
        .await
        .expect("Weather request failed. Check the API key for AvWx and your internet connection. Maybe try another position.")
}

fn get_weather_from_file(filename: String) -> Value {
    let file = File::open(filename).expect("No such file.");
    let reader = BufReader::new(file);
    serde_json::from_reader(reader).expect("Failed to read data from file.")
}

fn print_output(output: &ColoredString, raw: bool) {
    if raw {
        println!("{}", strip_ansi_escapes(output));
    } else {
        println!("{output}");
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let secrets = get_secrets(args.key.clone());
    let config = Config::get_config(&secrets, &args).await;
    let json = match args.file {
        Some(filename) => get_weather_from_file(filename),
        None => get_weather(&config, &secrets).await,
    };

    if config.print_taf {
        let taf_string = Taf::from_json(&json, &config)
            .expect("Invalid TAF data received.")
            .colourise(&config);
        print_output(&taf_string, config.raw);
    } else {
        let wx_string = Metar::from_json(&json, &config)
            .expect("Invalid weather data received.")
            .colourise(&config);

        print_output(&wx_string, config.raw);
    }
}

#[cfg(test)]
mod test {
    use std::fs;

    use super::*;

    #[tokio::test]
    async fn test_get_weather_from_file() {
        for entry in fs::read_dir("tests/testdata").unwrap() {
            let path = entry.unwrap().path();
            let result = get_weather_from_file(path.into_os_string().into_string().unwrap());
            assert!(result.is_object());
        }
    }

    #[tokio::test]
    async fn test_get_weather_from_file_metar() {
        for entry in fs::read_dir("tests/testdata").unwrap() {
            let path = entry.unwrap().path();
            let json = get_weather_from_file(path.into_os_string().into_string().unwrap());
            let metar = Metar::from_json(&json, &Config::default());
            assert!(metar.is_some());
        }
    }

    #[test]
    fn test_get_secrets_with_param() {
        // Ensure environment variable exists in case other tests removed it
        unsafe {
            std::env::set_var("AVWX_API_KEY", "fallback_key");
        }

        let test_key = "test_api_key_123".to_string();
        let secrets = get_secrets(Some(test_key.clone()));
        assert_eq!(secrets.avwx_api_key, test_key);
    }

    #[test]
    #[should_panic(expected = "Could not load secret key.")]
    fn test_get_secrets_no_param_no_env() {
        // Remove any existing environment variable
        unsafe {
            std::env::remove_var("AVWX_API_KEY");
        }
        let _secrets = get_secrets(None);
    }

    #[test]
    fn test_get_secrets_from_env() {
        let test_key = "env_test_key_456";
        unsafe {
            std::env::set_var("AVWX_API_KEY", test_key);
        }
        let secrets = get_secrets(None);
        assert_eq!(secrets.avwx_api_key, test_key);
        unsafe {
            std::env::remove_var("AVWX_API_KEY"); // cleanup
        }
    }

    #[test]
    #[should_panic(expected = "No such file.")]
    fn test_get_weather_from_file_nonexistent() {
        let _result = get_weather_from_file("nonexistent_file.json".to_string());
    }

    #[test]
    #[should_panic(expected = "Failed to read data from file.")]
    fn test_get_weather_from_file_invalid_json() {
        // Create a temporary file with invalid JSON
        use std::io::Write;
        let mut temp_file = std::fs::File::create("temp_invalid.json").unwrap();
        temp_file.write_all(b"invalid json content").unwrap();
        temp_file.sync_all().unwrap();
        drop(temp_file);

        let _result = get_weather_from_file("temp_invalid.json".to_string());

        // Cleanup
        std::fs::remove_file("temp_invalid.json").ok();
    }

    #[tokio::test]
    async fn test_get_weather_from_file_taf() {
        // Test TAF-specific files
        let taf_files = ["kjfk-taf.json", "eddf-taf-prob.json"];

        for file in &taf_files {
            let json = get_weather_from_file(format!("tests/testdata/{file}"));
            let taf = Taf::from_json(&json, &Config::default());
            assert!(taf.is_some(), "Failed to parse TAF from {file}");
        }
    }

    #[test]
    fn test_args_default_values() {
        use clap::Parser;

        // Test default argument values
        let args = Args::try_parse_from(["wxfetch"]).unwrap();
        assert!(args.airfield.is_none());
        assert!(args.latitude.is_none());
        assert!(args.longitude.is_none());
        assert!(args.config_file.is_none());
        assert!(args.file.is_none());
        assert!(args.key.is_none());
        assert!(!args.taf);
        assert!(!args.raw);
    }

    #[test]
    fn test_args_with_airfield() {
        use clap::Parser;

        let args = Args::try_parse_from(["wxfetch", "-a", "KJFK"]).unwrap();
        assert_eq!(args.airfield, Some("KJFK".to_string()));
    }

    #[test]
    fn test_args_with_coordinates() {
        use clap::Parser;

        let args = Args::try_parse_from(["wxfetch", "--lat", "40.7", "--lon", "74.0"]).unwrap();
        assert_eq!(args.latitude, Some(40.7));
        assert_eq!(args.longitude, Some(74.0));
    }

    #[test]
    fn test_args_with_taf_flag() {
        use clap::Parser;

        let args = Args::try_parse_from(["wxfetch", "--taf"]).unwrap();
        assert!(args.taf);
    }

    #[test]
    fn test_args_with_raw_flag() {
        use clap::Parser;

        let args = Args::try_parse_from(["wxfetch", "-R"]).unwrap();
        assert!(args.raw);
    }

    #[test]
    fn test_args_with_file() {
        use clap::Parser;

        let args = Args::try_parse_from(["wxfetch", "-f", "test.json"]).unwrap();
        assert_eq!(args.file, Some("test.json".to_string()));
    }

    #[test]
    fn test_args_with_config_file() {
        use clap::Parser;

        let args = Args::try_parse_from(["wxfetch", "-c", "config.toml"]).unwrap();
        assert_eq!(args.config_file, Some("config.toml".to_string()));
    }

    #[test]
    fn test_args_with_api_key() {
        use clap::Parser;

        let args = Args::try_parse_from(["wxfetch", "-k", "api123"]).unwrap();
        assert_eq!(args.key, Some("api123".to_string()));
    }

    #[test]
    fn test_args_combined() {
        use clap::Parser;

        let args = Args::try_parse_from([
            "wxfetch",
            "-a",
            "KJFK",
            "--taf",
            "-k",
            "testkey",
            "-c",
            "config.toml",
        ])
        .unwrap();

        assert_eq!(args.airfield, Some("KJFK".to_string()));
        assert!(args.taf);
        assert_eq!(args.key, Some("testkey".to_string()));
        assert_eq!(args.config_file, Some("config.toml".to_string()));
    }

    #[test]
    fn test_args_raw_flag_combined() {
        use clap::Parser;

        let args = Args::try_parse_from(["wxfetch", "--taf", "-R"]).unwrap();
        assert!(args.taf);
        assert!(args.raw);
    }

    #[test]
    fn test_strip_ansi_escapes() {
        let input = "\x1b[1mHello\x1b[0m World";
        let output = strip_ansi_escapes(input);
        assert_eq!(output, "Hello World");
    }

    #[test]
    fn test_strip_ansi_escapes_empty() {
        let input = "No colors here";
        let output = strip_ansi_escapes(input);
        assert_eq!(output, "No colors here");
    }

    #[test]
    fn test_strip_ansi_escapes_multiple() {
        let input = "\x1b[31mred\x1b[0m \x1b[32mgreen\x1b[0m \x1b[34mblue\x1b[0m";
        let output = strip_ansi_escapes(input);
        assert_eq!(output, "red green blue");
    }

    #[test]
    fn test_config_default_raw_false() {
        let config = Config::default();
        assert!(!config.raw);
    }
}