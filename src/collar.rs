use chrono::Utc;
use dotenvy::dotenv;
use poise::serenity_prelude::{CreateEmbed, CreateEmbedFooter, UserId};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{
  io::{Read, Write},
  path::Path,
  sync::Arc,
};
use tokio::{
  sync::Mutex,
  time::{Duration, Instant, interval_at},
};
use tracing::{error, info, warn};

pub(crate) mod commands;
pub(crate) mod http;
pub(crate) mod notifs;

pub(crate) type CollarError = Box<dyn std::error::Error + Send + Sync>;
pub(crate) type CollarContext<'a> = poise::Context<'a, Collar, CollarError>;
pub(crate) type CollarAppContext<'a> = poise::ApplicationContext<'a, Collar, CollarError>;

#[derive(Debug, Clone, Copy)]
pub(crate) enum NotifChannelType {
  UserSubmit,
  AdSubmit,
  UserVerify,
  AdVerify,
  General,
  DmFallback,
}

impl std::fmt::Display for NotifChannelType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      NotifChannelType::UserSubmit => write!(f, "notif_channel_ids.user_submit_id"),
      NotifChannelType::AdSubmit => write!(f, "notif_channel_ids.ad_submit_id"),
      NotifChannelType::UserVerify => write!(f, "notif_channel_ids.user_verify_id"),
      NotifChannelType::AdVerify => write!(f, "notif_channel_ids.ad_verify_id"),
      NotifChannelType::General => write!(f, "notif_channel_ids.general_id"),
      NotifChannelType::DmFallback => write!(f, "notif_channel_ids.dm_fallback_id"),
    }
  }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct Secrets {
  pub access_token: String,
  pub refresh_token: String,
  pub access_token_expires_at: i64,
  pub refresh_token_expires_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct NotifChannels {
  user_submit_id: Option<u64>,
  ad_submit_id: Option<u64>,
  user_verify_id: Option<u64>,
  ad_verify_id: Option<u64>,
  dm_fallback_id: Option<u64>,
  general_id: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Urls {
  api_base_url: String,
  web_base_url: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Cache {
  secrets: Secrets,
  notif_channel_ids: NotifChannels,
  feedback_webhook: Option<String>,
  urls: Urls,
}

#[derive(Clone)]
pub(crate) struct Collar {
  http_client: Client,
  cache: Arc<Mutex<Cache>>,
  bot_id: UserId,
}

impl Cache {
  pub fn new() -> Self {
    Self {
      secrets: Secrets {
        access_token: String::new(),
        refresh_token: String::new(),
        access_token_expires_at: 0,
        refresh_token_expires_at: 0,
      },
      notif_channel_ids: NotifChannels {
        user_submit_id: None,
        ad_submit_id: None,
        user_verify_id: None,
        ad_verify_id: None,
        dm_fallback_id: None,
        general_id: None,
      },
      feedback_webhook: None,
      urls: Urls {
        api_base_url: String::new(),
        web_base_url: String::new(),
      },
    }
  }

  pub fn default() -> Self {
    Self {
      secrets: Secrets {
        access_token: String::new(),
        refresh_token: String::new(),
        access_token_expires_at: 0,
        refresh_token_expires_at: 0,
      },
      notif_channel_ids: NotifChannels {
        user_submit_id: None,
        ad_submit_id: None,
        user_verify_id: None,
        ad_verify_id: None,
        dm_fallback_id: None,
        general_id: None,
      },
      feedback_webhook: None,
      urls: Urls {
        api_base_url: String::from("https://api.webring.pet"),
        web_base_url: String::from("https://webring.pet"),
      },
    }
  }

  pub fn exists_on_disk() -> bool {
    let path = Path::new("./.cache.json");

    path.exists()
  }

  pub fn read_from_disk(&self) -> Result<Self, CollarError> {
    dotenv().ok();
    let path = std::env::var("CACHE_PATH").unwrap_or(".cache.json".to_string());

    let mut cache_buf = String::new();
    let mut cache_file = match std::fs::File::open(path) {
      Ok(file) => file,
      Err(err) => {
        return Err(CollarError::from(format!(
          "There was an error opening the cache file: {err}"
        )));
      }
    };

    match cache_file.read_to_string(&mut cache_buf) {
      Ok(_) => (),
      Err(err) => {
        return Err(CollarError::from(format!(
          "There was an error reading the cached secrets file: {err}"
        )));
      }
    };

    match serde_json::from_str::<Cache>(&cache_buf) {
      Ok(cache) => Ok(cache),
      Err(err) => Err(CollarError::from(format!(
        "Failed to deserialize cache: {err}"
      ))),
    }
  }

  pub fn write_to_disk(&self) -> Result<(), CollarError> {
    dotenv().ok();
    let path = std::env::var("CACHE_PATH").unwrap_or(".cache.json".to_string());
    let mut file_to_write = std::fs::File::create(path)?;
    let secrets_str = serde_json::to_string(&self)?;

    match file_to_write.write_all(secrets_str.as_bytes()) {
      Ok(_) => Ok(()),
      Err(err) => Err(CollarError::from(format!(
        "Could not write to cache file: {err}"
      ))),
    }
  }

  pub async fn refresh_secrets(&mut self, http_client: &Client) -> Result<&mut Self, CollarError> {
    let secrets = self
      .secrets
      .clone()
      .refresh_secrets(http_client.clone(), self.get_api_base_url())
      .await?;

    self.secrets = secrets;
    Ok(self)
  }

  pub async fn fetch_secrets(
    &mut self,
    http_client: &reqwest::Client,
  ) -> Result<&mut Self, CollarError> {
    let secrets = self.secrets.clone();
    let secrets = secrets
      .get_secrets(http_client.clone(), self.get_api_base_url())
      .await?;

    self.set_secrets(secrets);
    Ok(self)
  }

  pub fn get_notif_channel(&self, notify_type: NotifChannelType) -> Option<u64> {
    match notify_type {
      NotifChannelType::UserSubmit => self.notif_channel_ids.user_submit_id,
      NotifChannelType::AdSubmit => self.notif_channel_ids.ad_submit_id,
      NotifChannelType::UserVerify => self.notif_channel_ids.user_verify_id,
      NotifChannelType::AdVerify => self.notif_channel_ids.ad_verify_id,
      NotifChannelType::General => self.notif_channel_ids.general_id,
      NotifChannelType::DmFallback => self.notif_channel_ids.dm_fallback_id,
    }
  }

  pub fn get_all_notif_channels(&self) -> NotifChannels {
    self.notif_channel_ids.clone()
  }

  pub fn get_feedback_webhook(&self) -> Option<String> {
    self.feedback_webhook.clone()
  }

  #[allow(dead_code)]
  pub fn get_urls(&self) -> Urls {
    self.urls.clone()
  }

  pub fn get_web_base_url(&self) -> String {
    self.urls.web_base_url.clone()
  }

  pub fn get_api_base_url(&self) -> String {
    self.urls.api_base_url.clone()
  }

  pub fn get_secrets(&self) -> Secrets {
    self.secrets.clone()
  }
  pub fn set_notif_channel(&mut self, channel_id: u64, notify_type: NotifChannelType) -> &mut Self {
    match notify_type {
      NotifChannelType::UserSubmit => self.notif_channel_ids.user_submit_id = Some(channel_id),
      NotifChannelType::AdSubmit => self.notif_channel_ids.ad_submit_id = Some(channel_id),
      NotifChannelType::UserVerify => self.notif_channel_ids.user_verify_id = Some(channel_id),
      NotifChannelType::AdVerify => self.notif_channel_ids.ad_verify_id = Some(channel_id),
      NotifChannelType::General => self.notif_channel_ids.general_id = Some(channel_id),
      NotifChannelType::DmFallback => self.notif_channel_ids.dm_fallback_id = Some(channel_id),
    }
    self
  }
  pub fn set_feedback_webhook(&mut self, webhook: String) -> &mut Self {
    self.feedback_webhook = Some(webhook);
    self
  }

  pub fn set_urls(&mut self, urls: Urls) -> &mut Self {
    self.urls = urls;
    self
  }

  pub fn set_secrets(&mut self, secrets: Secrets) -> &mut Self {
    self.secrets = secrets;
    self
  }
}

struct EmbedWrapper;
impl EmbedWrapper {
  fn new_normal(ctx: &CollarContext<'_>) -> CreateEmbed {
    let bot_pfp = ctx
      .cache()
      .user(ctx.data().bot_id)
      .unwrap()
      .avatar_url()
      .unwrap(); // if
    // this fails to unwrap, i'll buy myself a beer

    CreateEmbed::default().footer(CreateEmbedFooter::new(COLLAR_FOOTER).icon_url(bot_pfp))
  }

  fn new_application(ctx: &CollarAppContext<'_>) -> CreateEmbed {
    let bot_pfp = ctx
      .cache()
      .user(ctx.data().bot_id)
      .unwrap()
      .avatar_url()
      .unwrap(); // if
    // this fails to unwrap, i'll buy myself a beer

    CreateEmbed::default().footer(CreateEmbedFooter::new(COLLAR_FOOTER).icon_url(bot_pfp))
  }
}

pub const COLLAR_FOOTER: &str = "Collar :3, a Discord bot helper for PetRing and PetAds :3";

impl Collar {
  pub async fn new() -> Self {
    dotenv().ok();

    let client = match http::make_reqwest_client().await {
      Ok(client) => client,
      Err(e) => panic!("Failed to create reqwest client: {:?}", e),
    };

    let bot_id = std::env::var("BOT_ID").expect("missing BOT_ID");
    let web_base_url = std::env::var("WEB_BASE_URL").expect("missing WEB_BASE_URL");
    let api_base_url = std::env::var("API_BASE_URL").expect("missing API_BASE_URL");
    let client_clone = client.clone();

    let mut cache = if Cache::exists_on_disk() {
      match Cache::new().read_from_disk() {
        Ok(cache) => cache,
        Err(err) => {
          warn!("Failed to read cache from disk: {:?}, using defaults", err);
          Cache::default()
        }
      }
    } else {
      Cache::default()
    };

    let mut interval = interval_at(
      Instant::now() + Duration::from_secs(30 * 60),
      Duration::from_secs(30 * 60),
    );

    cache.set_urls(Urls {
      api_base_url,
      web_base_url,
    });

    let refresh_expiry = cache.get_secrets().refresh_token_expires_at;
    if refresh_expiry == 0 || refresh_expiry <= Utc::now().timestamp() {
      info!("No valid refresh token, testing fetching");
      match cache.fetch_secrets(&client).await {
        Ok(cache) => {
          info!("Successfully fetched secrets: {cache:?}");
        }
        Err(err) => {
          panic!("Failed to get secrets overall: {err}");
        }
      }
    }

    let access_expiry = cache.get_secrets().access_token_expires_at;
    if access_expiry == 0 || access_expiry <= Utc::now().timestamp() {
      match cache.refresh_secrets(&client).await {
        Ok(_) => {
          info!("Successfully refreshed secrets, caching");
          cache.write_to_disk().expect("Couldnt write to disk");
        }
        Err(err) => {
          error!("Failed to refresh secrets: {err}");
        }
      }
    }

    if let Err(err) = cache.write_to_disk() {
      panic!("{err}");
    }

    tokio::spawn(async move {
      let mut new_cache = match Cache::new().read_from_disk() {
        Ok(cache) => cache,
        Err(err) => {
          panic!("Failed to read cache from disk: {err}");
        }
      };

      let client = match http::make_reqwest_client().await {
        Ok(client) => client,
        Err(e) => panic!("Failed to create reqwest client: {:?}", e),
      };

      loop {
        info!("Starting background token refresh");
        match new_cache.refresh_secrets(&client).await {
          Ok(_) => {
            new_cache.write_to_disk().expect("Couldnt write to disk");
          }
          Err(e) => panic!("Failed to refresh secrets: {:?}", e),
        };
        interval.tick().await;
      }
    });

    Self {
      cache: Arc::new(Mutex::new(cache)),
      http_client: client_clone,
      bot_id: bot_id.parse::<UserId>().unwrap(),
    }
  }
}
