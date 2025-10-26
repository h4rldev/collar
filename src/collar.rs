use std::{io::Read, sync::Arc};

use dotenvy::dotenv;
use poise::serenity_prelude::{CreateEmbed, CreateEmbedFooter, UserId};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::{
    sync::Mutex,
    time::{Duration, Instant, interval_at},
};
use tracing::info;

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
    notif_channel_ids: Arc<Mutex<NotifChannels>>,
    feedback_webhook: Arc<Mutex<Option<String>>>,
    client: Client,
    api_base_url: String,
    web_base_url: String,
    bot_id: UserId,
}

#[derive(Serialize, Deserialize, Clone)]
struct NotifChannels {
    submit_id: Option<u64>,
    verify_id: Option<u64>,
    fallback_id: Option<u64>,
    general_id: Option<u64>,
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

pub(crate) fn fetch_cached_secrets() -> Result<Secrets, CollarError> {
    let mut cached_secrets_buf = String::new();
    let mut cached_secrets_file = match std::fs::File::open(".secrets.json") {
        Ok(file) => file,
        Err(err) => {
            return Err(CollarError::from(format!(
                "There was an error opening the cached secrets file: {err}"
            )));
        }
    };

    match cached_secrets_file.read_to_string(&mut cached_secrets_buf) {
        Ok(_) => (),
        Err(err) => {
            return Err(CollarError::from(format!(
                "There was an error reading the cached secrets file: {err}"
            )));
        }
    };

    match serde_json::from_str::<Secrets>(&cached_secrets_buf) {
        Ok(secrets) => Ok(secrets),
        Err(err) => Err(CollarError::from(format!(
            "Failed to deserialize cached secrets: {err}"
        ))),
    }
}

impl Collar {
    pub async fn new(api_base_url: String, web_base_url: String) -> Self {
        dotenv().ok();

        let client = match http::make_reqwest_client().await {
            Ok(client) => client,
            Err(e) => panic!("Failed to create reqwest client: {:?}", e),
        };

        let bot_id = std::env::var("BOT_ID").expect("missing BOT_ID");

        let notif_channel_id_buf =
            std::fs::read_to_string(".notif_channel_id.json").unwrap_or_default();

        let notif_channel_ids: NotifChannels = if notif_channel_id_buf.is_empty() {
            NotifChannels {
                submit_id: None,
                verify_id: None,
                fallback_id: None,
                general_id: None,
            }
        } else {
            serde_json::from_str(&notif_channel_id_buf).unwrap()
        };

        let feedback_webhook_buf =
            std::fs::read_to_string(".feedback_webhook.json").unwrap_or_default();

        let feedback_webhook = if feedback_webhook_buf.is_empty() {
            None
        } else {
            Some(feedback_webhook_buf)
        };

        let client_clone = client.clone();
        let api_base_url_clone = api_base_url.clone();

        let mut interval = interval_at(
            Instant::now() + Duration::from_secs(30 * 60),
            Duration::from_secs(30 * 60),
        );

        let secrets = match http::get_secrets(client.clone(), api_base_url.clone()).await {
            Ok(secrets) => secrets,
            Err(e) => panic!("Failed to get secrets: {:?}", e),
        };

        tokio::spawn(async move {
            let client = match http::make_reqwest_client().await {
                Ok(client) => client,
                Err(e) => panic!("Failed to create reqwest client: {:?}", e),
            };

            let secrets = match http::get_secrets(client.clone(), api_base_url.clone()).await {
                Ok(secrets) => secrets,
                Err(e) => panic!("Failed to get secrets: {:?}", e),
            };

            loop {
                info!("Starting background token refresh");
                match http::refresh_secrets(
                    api_base_url.clone(),
                    client.clone(),
                    secrets.refresh_token.clone(),
                    secrets.access_token.clone(),
                )
                .await
                {
                    Ok(_) => {}
                    Err(e) => panic!("Failed to refresh secrets: {:?}", e),
                };
                interval.tick().await;
            }
        });

        Self {
            secrets: Arc::new(Mutex::new(secrets)),
            notif_channel_ids: Arc::new(Mutex::new(notif_channel_ids)),
            feedback_webhook: Arc::new(Mutex::new(feedback_webhook)),
            client: client_clone,
            api_base_url: api_base_url_clone,
            web_base_url,
            bot_id: bot_id.parse::<UserId>().unwrap(),
        }
    }
}
