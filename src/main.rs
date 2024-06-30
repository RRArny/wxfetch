use clap::Parser;
use colored::ColoredString;
use reqwest::{Client, Error, Response};
use serde_json::Value;
use std::{fmt::Display, fs::File, io::Read};
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
            Self::LatLong(latlong) => latlong.to_string(),
            Self::GeoIP => get_geoip()
                .await
                .expect("Could not get location based on IP. Try supplying position instead.")
                .to_string(),
        }
    }
}

#[derive(Debug, Clone)]
struct LatLong(f64, f64);

impl Display for LatLong {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{},{}", self.0, self.1)
    }
}

async fn get_geoip() -> Option<LatLong> {
    let response = reqwest::get("http://ip-api.com/json/").await.ok()?;
    let json: Value = response.json().await.ok()?;

    let success = json.get("status")?;
    if success != "success" {
        return None;
    }

    let lat = json.get("lat")?.as_f64()?;
    let long = json.get("lon")?.as_f64()?;

    Some(LatLong(lat, long))
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
}

struct Config {
    position: Position,
}

struct Secrets {
    avwx_api_key: String,
}

fn get_secrets() -> Secrets {
    let msg = "Could not load secret keys.";
    let mut secrets_file = File::open("secrets.toml").expect(msg);
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
    let json = request_wx(config, secrets)
        .await
        .expect("Weather request failed.");
    let metar = Metar::from_json(json, config).expect("Invalid weather data received...");
    metar.colorise()
}

async fn request_wx(config: &Config, secrets: &Secrets) -> Option<Value> {
    let position = config.position.get_location().await;
    let resp = send_api_call(position, secrets).await.ok()?;

    if resp.status().as_u16() == 200 {
        resp.json().await.ok()
    } else if let Some(nearest_station_code) = get_nearest_station(config, secrets).await {
        send_api_call(nearest_station_code, secrets)
            .await
            .ok()?
            .json::<Value>()
            .await
            .ok()
    } else {
        println!("No nearest station...");
        None
    }
}

async fn send_api_call(position: String, secrets: &Secrets) -> Result<Response, Error> {
    let uri = format!(
        "https://avwx.rest/api/metar/{}?onfail=nearest&options=info",
        position
    );
    let resp: Response = Client::new()
        .get(uri)
        .header("Authorization", format!("BEARER {}", secrets.avwx_api_key))
        .send()
        .await?;
    Ok(resp)
}

async fn get_nearest_station(config: &Config, secrets: &Secrets) -> Option<String> {
    let uri = format!(
        "https://avwx.rest/api/station/{}?filter=latitude,longitude",
        config.position.get_location().await
    );
    let client = Client::new();
    let resp = client
        .get(uri)
        .header("Authorization", format!("BEARER {}", secrets.avwx_api_key))
        .send()
        .await
        .ok()?;

    let body = resp.json::<Value>().await.ok()?;
    let lat = body.get("latitude")?.as_f64()?;
    let lon = body.get("longitude")?.as_f64()?;

    println!("{}/{}", lat, lon);

    let uri = format!(
        "https://avwx.rest/api/station/near/{},{}?n=1&reporting=true",
        lat, lon
    );
    let resp = client
        .get(uri)
        .header("Authorization", format!("BEARER {}", secrets.avwx_api_key))
        .send()
        .await
        .ok()?;
    println!("{:?}", resp);
    let value = &resp.json::<Value>().await.ok()?;
    println!("val: {value}");
    let station = value.get(0)?.get("station")?.get("icao")?.as_str()?;

    Some(station.to_string())
}

#[tokio::main]
async fn main() {
    let secrets = get_secrets();
    let config = get_config(&secrets).await;
    let wx_string = get_weather(&config, &secrets).await;

    println!("{}", wx_string);
}
