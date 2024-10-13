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
// WxFetch - api.rs

use reqwest::{Client, Error, Response};
use serde_json::Value;

use crate::{Config, Secrets};

/// Given a Config and Secrets, sends a request to fetch a METAR and returns the report in JSON format wrapped in Some if successful, None otherwise.
pub async fn request_wx(config: &Config, secrets: &Secrets) -> Option<Value> {
    let position = config.position.get_location_str().await;
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

/// Given a properly formattet position string and Secrets, requests METAR from `AvWx` and wraps the Response in a Result.
async fn send_api_call(position: String, secrets: &Secrets) -> Result<Response, Error> {
    let uri = format!("https://avwx.rest/api/metar/{position}?onfail=nearest&options=info");
    let resp: Response = Client::new()
        .get(uri)
        .header("Authorization", format!("BEARER {}", secrets.avwx_api_key))
        .send()
        .await?;
    Ok(resp)
}

/// For a given Position in a Config as well as the necessary Secrets returns a String with the ICAO (or similar) code for the nearest reporting station.
async fn get_nearest_station(config: &Config, secrets: &Secrets) -> Option<String> {
    let uri = format!(
        "https://avwx.rest/api/station/{}?filter=latitude,longitude",
        config.position.get_location_str().await
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

    let uri = format!("https://avwx.rest/api/station/near/{lat},{lon}?n=1&reporting=true");
    let resp = client
        .get(uri)
        .header("Authorization", format!("BEARER {}", secrets.avwx_api_key))
        .send()
        .await
        .ok()?;
    let value = &resp.json::<Value>().await.ok()?;
    let station = value.get(0)?.get("station")?.get("icao")?.as_str()?;

    Some(station.to_string())
}

/// For a given ICAO code as String and the necessary Secrets makes request to avwx to check if the code is valid.
pub async fn check_icao_code(icao: &String, secrets: &Secrets) -> bool {
    let uri = format!("https://avwx.rest/api/station/{icao}");

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
