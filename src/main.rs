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

use api::{check_icao_code, request_wx};
use chrono::TimeDelta;
use clap::Parser;
use colored::ColoredString;
use std::{fs::File, io::Read};
use toml::{Table, Value};

mod metar;
use metar::Metar;

mod position;
use position::{LatLong, Position};

mod api;

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
}

struct Config {
    position: Position,
    cloud_minimum: i64,
    cloud_marginal: i64,
    temp_minimum: i64,
    spread_minimum: i64,
    wind_var_maximum: i64,
    wind_maximum: i64,
    gust_maximum: i64,
    age_maximum: TimeDelta,
    age_marginal: TimeDelta,
    visibility_minimum: i64,
    visibility_marginal: i64,
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
        }
    }
}

struct Secrets {
    avwx_api_key: String,
}

fn get_secrets() -> Secrets {
    let avwx_api_key = std::env::var("AVWX_API_KEY").expect("Could not load secret keys.");
    Secrets { avwx_api_key }
}

async fn get_config(secrets: &Secrets) -> Config {
    let args = Args::parse();
    let mut config: Config = read_config_file(args.config_file);

    if let Some(icao) = args.airfield {
        config.position = Position::Airfield(icao.clone());
    } else if let Some(lat) = args.latitude {
        if let Some(long) = args.longitude {
            config.position = Position::LatLong(LatLong(lat, long));
        }
        println!("Please provide both Latitude and Longitude. Defaulting to geoip...");
    }

    if let Position::Airfield(ref icao) = config.position {
        if !check_icao_code(icao, secrets).await {
            println!("Invalid airfield {icao}. Defaulting to geoip...");
            config.position = Position::GeoIP;
        }
    }
    config
}

fn read_config_file(config_filepath: Option<String>) -> Config {
    let msg = "Failed to load config.";
    let mut config = Config::default();
    let config_filepath = config_filepath
        .unwrap_or(std::env::var("HOME").expect(msg) + "/.config/wxfetch/config.toml");
    let mut config_file = File::open(config_filepath).expect(msg);
    let mut contents = String::new();
    config_file.read_to_string(&mut contents).expect(msg);
    let contents = contents.parse::<Table>().expect(msg);

    if contents.contains_key("position") {
        if let Some(airfield) = contents["position"].get("airfield").and_then(Value::as_str) {
            config.position = Position::Airfield(airfield.to_string());
        }
        if let Some(lat) = contents["position"].get("lat").and_then(Value::as_float) {
            if let Some(lon) = contents["position"].get("lon").and_then(Value::as_float) {
                config.position = Position::LatLong(LatLong(lat, lon));
            }
        }
    }

    if contents.contains_key("clouds") {
        if let Some(minimum) = contents["clouds"]
            .get("cloud_minimum")
            .and_then(Value::as_integer)
        {
            config.cloud_minimum = minimum;
        }
        if let Some(marginal) = contents["clouds"]
            .get("cloud_marginal")
            .and_then(Value::as_integer)
        {
            config.cloud_marginal = marginal;
        }
    }

    if contents.contains_key("temperature") {
        if let Some(minimum) = contents["temperature"]
            .get("temp_minimum")
            .and_then(Value::as_integer)
        {
            config.temp_minimum = minimum;
        }
        if let Some(spread_minimum) = contents["temperature"]
            .get("spread_minimum")
            .and_then(Value::as_integer)
        {
            config.spread_minimum = spread_minimum;
        }
    }

    if contents.contains_key("wind") {
        if let Some(var_maximum) = contents["wind"]
            .get("wind_var_maximum")
            .and_then(Value::as_integer)
        {
            config.wind_var_maximum = var_maximum;
        }
        if let Some(maximum) = contents["wind"]
            .get("wind_maximum")
            .and_then(Value::as_integer)
        {
            config.wind_maximum = maximum;
        }
        if let Some(gust_maximum) = contents["wind"]
            .get("gust_maximum")
            .and_then(Value::as_integer)
        {
            config.gust_maximum = gust_maximum;
        }
    }

    if contents.contains_key("age") {
        if let Some(maximum) = contents["age"]
            .get("age_maximum")
            .and_then(Value::as_integer)
        {
            config.age_maximum = TimeDelta::seconds(maximum);
        }
        if let Some(marginal) = contents["age"]
            .get("age_marginal")
            .and_then(Value::as_integer)
        {
            config.age_marginal = TimeDelta::seconds(marginal);
        }
    }

    if contents.contains_key("visibility") {
        if let Some(minimum) = contents["visibility"]
            .get("visibility_minimum")
            .and_then(Value::as_integer)
        {
            config.visibility_minimum = minimum;
        }
        if let Some(marginal) = contents["visibility"]
            .get("visibility_marginal")
            .and_then(Value::as_integer)
        {
            config.visibility_marginal = marginal;
        }
    }

    config
}

async fn get_weather(config: &Config, secrets: &Secrets) -> ColoredString {
    let json = request_wx(config, secrets)
        .await
        .expect("Weather request failed.");
    let metar = Metar::from_json(&json, config).expect("Invalid weather data received...");
    metar.colorise(config)
}

#[tokio::main]
async fn main() {
    let secrets = get_secrets();
    let config = get_config(&secrets).await;
    let wx_string = get_weather(&config, &secrets).await;

    println!("{wx_string}");
}
