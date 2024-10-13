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
use clap::Parser;
use colored::ColoredString;
use std::{fs::File, io::Read};
use toml::Table;

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
}

struct Config {
    position: Position,
}

struct Secrets {
    avwx_api_key: String,
}

fn get_secrets() -> Secrets {
    let msg = "Could not load secret keys.";
    let secrets_filepath = std::env::var("HOME").expect(msg) + "/.config/wxfetch/secrets.toml";
    let mut secrets_file = File::open(secrets_filepath).expect(msg);
    let mut contents = String::new();
    secrets_file.read_to_string(&mut contents).expect(msg);
    let secrets = contents.parse::<Table>().expect(msg);

    let avwx_api_key = secrets["avwx-key"]["avwx-api-key"]
        .as_str()
        .expect(msg)
        .to_string();

    Secrets { avwx_api_key }
}

async fn get_config(secrets: &Secrets) -> Config {
    let args = Args::parse();

    if let Some(icao) = args.airfield {
        if check_icao_code(&icao, secrets).await {
            return Config {
                position: Position::Airfield(icao),
            };
        }
        println!("Invalid airfield {icao}. Defaulting to geoip...");
    } else if let Some(lat) = args.latitude {
        if let Some(long) = args.longitude {
            return Config {
                position: Position::LatLong(LatLong(lat, long)),
            };
        }
        println!("Please provide both Latitude and Longitude. Defaulting to geoip...");
    }

    Config {
        position: Position::GeoIP,
    }
}

async fn get_weather(config: &Config, secrets: &Secrets) -> ColoredString {
    let json = request_wx(config, secrets)
        .await
        .expect("Weather request failed.");
    let metar = Metar::from_json(&json, config).expect("Invalid weather data received...");
    metar.colorise()
}

#[tokio::main]
async fn main() {
    let secrets = get_secrets();
    let config = get_config(&secrets).await;
    let wx_string = get_weather(&config, &secrets).await;

    println!("{wx_string}");
}
