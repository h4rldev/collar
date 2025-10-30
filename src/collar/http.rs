use super::{Cache, Collar, CollarError, Secrets};
use dotenvy::dotenv;
use reqwest::{Client, Method, StatusCode};
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, time::Duration};
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

impl Secrets {
  pub async fn get_secrets(
    &self,
    http_client: Client,
    api_base_url: String,
  ) -> Result<Self, CollarError> {
    dotenv().ok();
    let token = std::env::var("DISCORD_BOT_TOKEN").expect("missing DISCORD_BOT_TOKEN");

    let body = GetSecretsRequest { bot_token: token };
    let mut status: StatusCode = StatusCode::IM_A_TEAPOT;
    let mut response: String = String::new();

    while status == StatusCode::IM_A_TEAPOT {
      let url = format!("{api_base_url}/bot/setup");
      let resp = http_client
        .post(url)
        .json(&body)
        .timeout(Duration::from_secs(60));
      let got_response = match resp.send().await {
        Ok(response) => {
          status = response.status();
          response.text().await?
        }
        Err(err) => {
          sleep(Duration::from_secs(1)).await;
          error!("Failed to get secrets, retrying: {err}",);
          continue;
        }
      };

      response = got_response;
      break;
    }

    if status.is_success() {
      let secrets = match serde_json::from_str::<Secrets>(&response) {
        Ok(secrets) => secrets,
        Err(err) => {
          return Err(CollarError::from(format!(
            "Could not deserialize secrets: {err}"
          )));
        }
      };
      return Ok(secrets);
    }

    Err(CollarError::from(format!(
      "Could not get secrets: {response}"
    )))
  }

  pub async fn refresh_secrets(
    self,
    http_client: Client,
    api_base_url: String,
  ) -> Result<Self, CollarError> {
    let body = RefreshTokenRequest {
      access_token: self.access_token,
      refresh_token: self.refresh_token,
    };

    let mut response = String::new();
    let mut status: StatusCode = StatusCode::IM_A_TEAPOT;

    while response.is_empty() && status == StatusCode::IM_A_TEAPOT {
      let url = format!("{api_base_url}/bot/refresh");
      let req = http_client.post(url).json(&body);
      let resp = req.send().await;
      match resp {
        Ok(resp) => {
          status = resp.status();
          response = resp.text().await?;
          break;
        }
        Err(err) => {
          error!("Failed to refresh secrets: {err}");
          sleep(Duration::from_secs(1)).await;
          continue;
        }
      }
    }

    if status.is_success() {
      let secrets: Secrets = serde_json::from_str(&response).unwrap();
      return Ok(secrets);
    }

    response = String::new();
    status = StatusCode::IM_A_TEAPOT;

    let cached_secrets: Secrets = Cache::new().read_from_disk()?.secrets;
    let new_body = RefreshTokenRequest {
      access_token: cached_secrets.access_token,
      refresh_token: cached_secrets.refresh_token,
    };

    while response.is_empty() && status == StatusCode::IM_A_TEAPOT {
      let url = format!("{api_base_url}/bot/refresh");
      let resp = http_client.post(url).json(&new_body);
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

    if status.is_success() {
      let secrets = serde_json::from_str::<Secrets>(&response).unwrap();
      info!("Refreshed secrets");

      return Ok(secrets);
    }

    Err(CollarError::from(format!(
      "Failed to refresh secrets: {response}"
    )))
  }
}

#[derive(Serialize, Deserialize)]
struct RefreshTokenRequest {
  access_token: String,
  refresh_token: String,
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
  let mut cache = collar.cache.lock().await;
  let url = format!("{}{}", cache.get_api_base_url(), route);
  let http_client = collar.http_client;
  let secrets = cache.get_secrets();

  info!("Making request to {url}");

  let mut headers = reqwest::header::HeaderMap::new();

  if method != Method::GET {
    headers.insert(
      reqwest::header::CONTENT_TYPE,
      reqwest::header::HeaderValue::from_str("application/json")?,
    );
  }

  headers.insert(
    reqwest::header::ACCEPT,
    reqwest::header::HeaderValue::from_str("application/json")?,
  );
  headers.insert(
    reqwest::header::AUTHORIZATION,
    reqwest::header::HeaderValue::from_str(&format!("Bearer {}", &secrets.access_token))?,
  );

  info!("With headers: {headers:?}");

  let mut req = http_client
    .request(method.clone(), url.clone())
    .headers(headers.clone());

  if let Some(body) = body.clone() {
    req = req.json(&body);
  }

  let resp = req.send().await?;
  if resp.status().is_success() {
    let resp_text = resp.text().await?;
    debug!("Response_text: {resp_text}");

    return match serde_json::from_str::<R>(&resp_text) {
      Ok(return_type) => Ok(ResponseTypes::Success(return_type)),
      Err(err) => {
        error!("Failed to convert response to json: {err}, response: {resp_text}");
        Ok(ResponseTypes::Error(ErrorResponse {
          status: 500,
          message: format!("Failed to convert response to json: {err}, response: {resp_text}"),
        }))
      }
    };
  } else if resp.status() != 401 {
    let resp_text = resp.text().await?;
    debug!("Response_text: {resp_text}");

    return match serde_json::from_str::<ErrorResponse>(&resp_text) {
      Ok(error) => Ok(ResponseTypes::Error(error)),
      Err(err) => {
        error!("Failed to convert response to json: {err}, response: {resp_text}");
        Ok(ResponseTypes::Error(ErrorResponse {
          status: 500,
          message: format!("Failed to convert response to json: {err}, response: {resp_text}"),
        }))
      }
    };
  }

  if resp.status() == 401 {
    info!("Invalid token, refreshing secrets");
    cache.refresh_secrets(&http_client).await?;
    let secrets = cache.get_secrets();

    headers = reqwest::header::HeaderMap::new();

    if method != Method::GET {
      headers.insert(
        reqwest::header::CONTENT_TYPE,
        reqwest::header::HeaderValue::from_str("application/json")?,
      );
    }

    headers.insert(
      reqwest::header::ACCEPT,
      reqwest::header::HeaderValue::from_str("application/json")?,
    );
    headers.insert(
      reqwest::header::AUTHORIZATION,
      reqwest::header::HeaderValue::from_str(&format!("Bearer {}", &secrets.access_token))?,
    );
  }

  let new_req = http_client.request(method, url).headers(headers);
  let resp = new_req.send().await?;

  if !resp.status().is_success() {
    let resp_text = resp.text().await?;
    debug!("Response_text: {resp_text}");

    return match serde_json::from_str::<ErrorResponse>(&resp_text) {
      Ok(error) => Ok(ResponseTypes::Error(error)),
      Err(err) => {
        error!("Failed to convert response to json: {err}, response: {resp_text}");
        Ok(ResponseTypes::Error(ErrorResponse {
          status: 500,
          message: format!("Failed to convert response to json: {err}, response: {resp_text}"),
        }))
      }
    };
  }

  let resp_text = resp.text().await?;
  match serde_json::from_str::<R>(&resp_text) {
    Ok(return_type) => Ok(ResponseTypes::Success(return_type)),
    Err(err) => {
      error!("Failed to convert response to json: {err}, response: {resp_text}");
      Ok(ResponseTypes::Error(ErrorResponse {
        status: 500,
        message: format!("Failed to convert response to json: {err}, response: {resp_text}"),
      }))
    }
  }
}
