use clap::Parser;
use colored::ColoredString;
use reqwest::{Client, Error, Response};
use serde_json::Value;
use std::{fs::File, io::Read};
use toml::Table;

mod metar;
use metar::Metar;

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
/// Console utility for accessing aviation weather information on the commmand line.
///
/// For more information see README.md or https://github.com/RRArny/wxfetch.
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
    let msg = "Couldn't secret keys.";
    let mut secrets_file = File::open("secrets.toml").expect(msg);
    let mut contents = String::new();
    secrets_file
        .read_to_string(&mut contents)
        .expect("Couldn't load secret keys.");
    let secrets = contents.parse::<Table>().unwrap();

    let key = secrets["avwx-key"]["avwx-api-key"].as_str().unwrap();

    Secrets {
        avwx_api_key: key.to_string(),
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
            println!("Invalid airfield {}. Defaulting to geoip...", icao);
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
    println!("{:?}", json);
    let metar = Metar::from_json(json, config);
    metar.colorise()
}

async fn request_wx(config: &Config, secrets: &Secrets) -> Result<Value, Error> {
    let position = config.position.get_location().await;
    println!("{:?}", config.position);
    let uri = format!(
        "https://avwx.rest/api/metar/{}?onfail=nearest&options=info",
        position
    );

    let resp: Response = Client::new()
        .get(uri)
        .header("Authorization", format!("BEARER {}", secrets.avwx_api_key))
        .send()
        .await?;
    println!("{:?}", resp);
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

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_metar_from_json_icao() {
        let json: Value = Value::from_str("").unwrap();
        let config = Config {
            position: Position::Airfield("EDRK".to_string()),
        };
        let metar = Metar::from_json(json, &config);
        assert_eq!(metar.icao_code, "EDRK");
    }

    #[test]
    fn test_metar_from_json_time() {
        let json: Value = Value::from_str("").unwrap();
        let config = Config {
            position: Position::Airfield("EDRK".to_string()),
        };
        let metar = Metar::from_json(json, &config);
        assert!(metar.fields.contains(&MetarField::TimeStamp));
    }
}
