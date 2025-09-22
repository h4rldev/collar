use std::{
    fmt::Debug,
    io::{Read, Write},
    time::Duration,
};

use super::{Collar, CollarError, Secrets};
use dotenvy::dotenv;
use reqwest::{Client, Method, StatusCode};
use serde::{Deserialize, Serialize};
use tokio::time::sleep;
#[allow(unused_imports)]
use tracing::{debug, error, info};

pub async fn make_reqwest_client() -> Result<Client, CollarError> {
    Ok(Client::new())
}

#[derive(Serialize, Deserialize)]
struct GetSecretsRequest {
    bot_token: String,
}

pub(crate) async fn get_secrets(client: Client, base_url: String) -> Result<Secrets, CollarError> {
    dotenv().ok();
    let token = std::env::var("DISCORD_BOT_TOKEN").expect("missing DISCORD_BOT_TOKEN");

    let body = GetSecretsRequest { bot_token: token };
    let mut status: StatusCode = StatusCode::IM_A_TEAPOT;
    let mut response: String = String::new();

    while status == StatusCode::IM_A_TEAPOT {
        let url = format!("{base_url}/bot/setup");
        let resp = client
            .post(url)
            .json(&body)
            .timeout(Duration::from_secs(60));
        let got_response = match resp.send().await {
            Ok(response) => {
                status = response.status();
                response.text().await?
            }
            Err(_) => {
                sleep(Duration::from_secs(1)).await;
                error!("Failed to get secrets: {status:?}, retrying");
                continue;
            }
        };

        response = got_response;
        break;
    }

    if !status.is_success() {
        let mut file_to_read = match std::fs::File::open(".secrets.json") {
            Ok(file) => file,
            Err(_) => {
                error!("Failed to get secrets: {status:?}");
                return Err(CollarError::from("Failed to get secrets"));
            }
        };
        let mut secrets_str = String::new();
        match file_to_read.read_to_string(&mut secrets_str) {
            Ok(_) => (),
            Err(err) => {
                error!("Failed to read secrets file: {err}");
                return Err(CollarError::from("Failed to get secrets from file"));
            }
        };

        response = secrets_str;
    }

    let secrets = serde_json::from_str(&response).unwrap();

    /**/
    Ok(secrets)
}

#[derive(Serialize, Deserialize)]
struct RefreshTokenRequest {
    access_token: String,
    refresh_token: String,
}

pub(crate) async fn refresh_secrets(
    base_url: String,
    client: Client,
    refresh_token: String,
    access_token: String,
) -> Result<Secrets, CollarError> {
    let body = RefreshTokenRequest {
        access_token,
        refresh_token,
    };

    let mut response = String::new();
    let mut status: StatusCode = StatusCode::IM_A_TEAPOT;

    while response.is_empty() && status == StatusCode::IM_A_TEAPOT {
        let url = format!("{base_url}/bot/refresh");
        let resp = client.post(url).json(&body);
        match resp.send().await {
            Ok(resp) => {
                status = resp.status();
                response = resp.text().await?;
                break;
            }
            Err(_) => {
                sleep(Duration::from_secs(1)).await;
                continue;
            }
        }
    }

    if !status.is_success() {
        let error: ErrorResponse = serde_json::from_str(&response).unwrap();
        error!("Failed to refresh secrets: {error:?}");
        return Err(CollarError::from(format!(
            "Failed to refresh secrets: {} - {}",
            error.status, error.message
        )));
    }

    info!("Got response");

    let secrets: Secrets = serde_json::from_str(&response).unwrap();

    info!("Refreshed secrets");

    let mut file_to_write = std::fs::File::create(".secrets.json").unwrap();
    let secrets_str = serde_json::to_string(&secrets).unwrap();
    file_to_write.write_all(secrets_str.as_bytes()).unwrap();

    Ok(secrets)
}

#[derive(Deserialize, Debug)]
pub struct ErrorResponse {
    pub status: u16,
    pub message: String,
}

#[derive(Deserialize, Debug)]
pub enum ResponseTypes<R> {
    Success(R),
    Error(ErrorResponse),
}

pub async fn make_request<T, R>(
    collar: Collar,
    body: Option<T>,
    route: &str,
    method: Method,
) -> Result<ResponseTypes<R>, CollarError>
where
    T: Serialize + Clone,
    R: for<'de> Deserialize<'de> + Debug,
{
    let url = format!("{}{}", collar.api_base_url, route);
    let client = collar.client;
    let mut secrets = collar.secrets.lock().await;

    info!("Making request to {url}");

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::AUTHORIZATION,
        reqwest::header::HeaderValue::from_str(&format!("Bearer {}", &secrets.access_token))?,
    );

    let mut req = client
        .request(method.clone(), url.clone())
        .headers(headers.clone());

    if let Some(body) = body {
        req = req.json(&body);
    }

    let resp = req.send().await?;
    if resp.status().is_success() {
        match resp.json::<R>().await {
            Ok(return_type) => return Ok(ResponseTypes::Success(return_type)),
            Err(err) => {
                error!("Failed to convert response to json: {err}");
                return Ok(ResponseTypes::Error(ErrorResponse {
                    status: 500,
                    message: "Failed to convert response to json".to_string(),
                }));
            }
        }
    } else if resp.status() != 401 {
        match resp.json::<ErrorResponse>().await {
            Ok(error) => return Ok(ResponseTypes::Error(error)),
            Err(err) => {
                error!("Failed to convert response to json: {err}");
                return Ok(ResponseTypes::Error(ErrorResponse {
                    status: 500,
                    message: "Failed to convert response to json".to_string(),
                }));
            }
        };
    }

    if resp.status() == 401 {
        info!("Invalid token, refreshing secrets");
        let new_secrets = refresh_secrets(
            collar.api_base_url.clone(),
            client.clone(),
            secrets.refresh_token.clone(),
            secrets.access_token.clone(),
        )
        .await?;
        *secrets = new_secrets;

        headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::AUTHORIZATION,
            reqwest::header::HeaderValue::from_str(&format!("Bearer {}", &secrets.access_token))?,
        );
    }

    let new_req = client.request(method, url).headers(headers);
    let resp = new_req.send().await?;

    if !resp.status().is_success() {
        match resp.json::<ErrorResponse>().await {
            Ok(error) => return Ok(ResponseTypes::Error(error)),
            Err(err) => {
                error!("Failed to convert response to json: {err}");
                return Ok(ResponseTypes::Error(ErrorResponse {
                    status: 500,
                    message: "Failed to convert response to json".to_string(),
                }));
            }
        };
    }

    match resp.json::<R>().await {
        Ok(response) => Ok(ResponseTypes::Success(response)),
        Err(err) => {
            error!("Failed to convert response to json: {err}");
            Ok(ResponseTypes::Error(ErrorResponse {
                status: 500,
                message: "Failed to convert response to json".to_string(),
            }))
        }
    }
}
