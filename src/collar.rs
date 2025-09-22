use std::sync::Arc;

use dotenvy::dotenv;
use poise::serenity_prelude::UserId;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

pub(crate) mod commands;
pub(crate) mod http;
pub(crate) mod notifs;

pub(crate) type CollarError = Box<dyn std::error::Error + Send + Sync>;
pub(crate) type CollarContext<'a> = poise::Context<'a, Collar, CollarError>;
pub(crate) type CollarAppContext<'a> = poise::ApplicationContext<'a, Collar, CollarError>;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct Secrets {
    pub access_token: String,
    pub refresh_token: String,
    pub access_token_expires_at: i64,
    pub refresh_token_expires_at: i64,
}

#[derive(Clone)]
pub(crate) struct Collar {
    secrets: Arc<Mutex<Secrets>>,
    notif_channel_id: Arc<Mutex<Option<u64>>>,
    client: Client,
    api_base_url: String,
    bot_id: UserId,
}

#[derive(Serialize, Deserialize, Clone)]
struct NotifChannel {
    id: u64,
}

impl Collar {
    pub async fn new(base_url: Option<String>) -> Self {
        dotenv().ok();

        let client = match http::make_reqwest_client().await {
            Ok(client) => client,
            Err(e) => panic!("Failed to create reqwest client: {:?}", e),
        };

        let base_url = match base_url {
            Some(url) => url,
            None => "http://localhost:8080".to_string(),
        };

        let secrets = match http::get_secrets(client.clone(), base_url.clone()).await {
            Ok(secrets) => secrets,
            Err(e) => panic!("Failed to get secrets: {:?}", e),
        };

        let bot_id = std::env::var("BOT_ID").expect("missing BOT_ID");

        let notif_channel_id_buf =
            std::fs::read_to_string(".notif_channel_id.json").unwrap_or_default();

        let notif_channel_id: NotifChannel = if notif_channel_id_buf.is_empty() {
            NotifChannel { id: 0 }
        } else {
            serde_json::from_str(&notif_channel_id_buf).unwrap()
        };

        let actual_notif_channel_id = match notif_channel_id.id {
            0 => None,
            id => Some(id),
        };

        Self {
            secrets: Arc::new(Mutex::new(secrets)),
            notif_channel_id: Arc::new(Mutex::new(actual_notif_channel_id)),
            client,
            api_base_url: base_url,
            bot_id: bot_id.parse::<UserId>().unwrap(),
        }
    }
}
