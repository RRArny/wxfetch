use core::panic;

use clap::Parser;
use colored::{ColoredString, Colorize};
use reqwest::{Client, Error, Response};
use serde_json::Value;

#[derive(Debug, Clone)]
enum Position {
    Airfield(String),
    GeoIP,
    LatLong(LatLong),
}

impl Position {
    async fn get_location(&self) -> String {
        match self {
            Self::Airfield(icao_code) => icao_code.to_string(),
            Self::LatLong(LatLong(lat, long)) => format!("{},{}", lat, long),
            Self::GeoIP => {
                let LatLong(lat, long) = get_geoip()
                    .await
                    .expect("Could not get location based on IP. Try supplying position instead.");
                format!("{},{}", lat, long)
            }
        }
    }
}

#[derive(Debug, Clone)]
struct LatLong(f64, f64);

async fn get_geoip() -> Option<LatLong> {
    let response = Client::new()
        .get("http://ip-api.com/json/")
        .send()
        .await
        .ok()?;
    let json: Value = response.json().await.ok()?;

    let success = json.get("status")?;
    if *success != *"success" {
        return None;
    }

    let lat = json.get("lat")?.as_f64()?;
    let long = json.get("lon")?.as_f64()?;

    Some(LatLong(lat, long))
}

#[derive(Parser, Debug)]
#[command(version, about)]
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

struct Metar {
    icao_code: String,
    fields: Vec<MetarField>,
    exact_match: bool,
}

enum MetarField {
    Visibility(i32),
}

impl MetarField {
    fn colourise(&self) -> ColoredString {
        match self {
            Self::Visibility(vis) => {
                if *vis >= 6000 {
                    vis.to_string().green()
                } else if *vis >= 3000 {
                    vis.to_string().yellow()
                } else {
                    vis.to_string().red()
                }
            }
        }
    }
}

impl Metar {
    fn from_json(json: Value) -> Self {
        println!("{}", json);

        let vis: i32 = serde_json::from_value(
            json.get("visibility")
                .unwrap()
                .get("value")
                .unwrap()
                .clone(),
        )
        .unwrap();

        Metar {
            icao_code: "EDRK".to_string(),
            fields: vec![MetarField::Visibility(vis)],
            exact_match: false,
        }
    }

    fn colorise(self) -> ColoredString {
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

fn get_secrets() -> Secrets {
    // let secrets = toml::Deserializer

    Secrets {
        avwx_api_key: "XXMWfeyKXH7emqYFW97jCNL17XU_iZFPg7aEEwP9ttc".to_string(),
    }
}

async fn get_config(secrets: &Secrets) -> Config {
    let args = Args::parse();

    if let Some(icao) = args.airfield {
        if check_icao_code(&icao, secrets).await {
            return Config {
                position: Position::Airfield(icao),
            };
        } else {
            panic!("No such Airfield: {}.", icao);
        }
    } else if let Some(lat) = args.latitude {
        if let Some(long) = args.longitude {
            return Config {
                position: Position::LatLong(LatLong(lat, long)),
            };
        } else {
            println!("Please provide both Latitude and Longitude. Defaulting to geoip...");
        }
    }
    Config {
        position: Position::GeoIP,
    }
}

async fn check_icao_code(icao: &String, secrets: &Secrets) -> bool {
    let uri = format!("https://avwx.rest/api/station/{}", icao);

    let resp = Client::new()
        .get(uri)
        .header("Authorization", format!("BEARER {}", secrets.avwx_api_key))
        .send()
        .await;

    match resp {
        Ok(resp) => match resp.json::<Value>().await {
            Ok(json) => json.get("error").is_none(),
            Err(_) => false,
        },
        Err(_) => false,
    }
}

async fn get_weather(config: &Config, secrets: &Secrets) -> ColoredString {
    let json = request_wx(config, secrets).await.expect("Request failed.");
    let metar = Metar::from_json(json);
    metar.colorise()
}

async fn request_wx(config: &Config, secrets: &Secrets) -> Result<Value, Error> {
    let position = config.position.get_location().await;

    let uri = format!(
        "https://avwx.rest/api/metar/{}?onfail=nearest&options=info",
        position
    );

    let resp: Response = Client::new()
        .get(uri)
        .header("Authorization", format!("BEARER {}", secrets.avwx_api_key))
        .send()
        .await?;

    let json: Value = resp.json().await?;

    Ok(json)
}

#[tokio::main]
async fn main() {
    let secrets = get_secrets();
    let config = get_config(&secrets).await;
    let wx_string = get_weather(&config, &secrets).await;

    println!("{}", wx_string);
}
