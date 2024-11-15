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

use api::request_wx;
use clap::Parser;
use colored::ColoredString;

mod metar;
use metar::Metar;

mod position;

mod api;

mod config;
use config::get_config;
use config::Config;

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
}

struct Secrets {
    avwx_api_key: String,
}

fn get_secrets() -> Secrets {
    let avwx_api_key = std::env::var("AVWX_API_KEY").expect("Could not load secret keys.");
    Secrets { avwx_api_key }
}

async fn get_weather(config: &Config, secrets: &Secrets) -> ColoredString {
    let json = request_wx(config, secrets)
        .await
        .expect("Weather request failed.");
    let metar = Metar::from_json(&json, config).expect("Invalid weather data received...");
    metar.colorise(config)
}

fn get_weather_from_file(filename: String, config: &Config) -> ColoredString {
    let file = File::open(filename).expect("No such file: {filename}");
    let reader = BufReader::new(file);
    let json = serde_json::from_reader(reader).expect("Failed to read data from file {filename}");
    let metar = Metar::from_json(&json, config).expect("Invalid weather data received...");

    metar.colorise(config)
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let secrets = get_secrets();
    let config = get_config(&secrets, &args).await;
    let wx_string = match args.file {
        Some(filename) => get_weather_from_file(filename, &config),
        None => get_weather(&config, &secrets).await,
    };

    println!("{wx_string}");
}
