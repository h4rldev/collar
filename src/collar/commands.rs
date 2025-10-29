use super::{
  COLLAR_FOOTER, CollarAppContext, CollarContext, CollarError, EmbedWrapper, NotifChannelType,
  http, notifs,
};
use poise::{ChoiceParameter, CreateReply, Modal, serenity_prelude::Color};
use serde::{Deserialize, Serialize};

pub mod misc;
pub mod notifications;
pub mod petads;
pub mod petring;

#[derive(Deserialize, Debug, Clone)]
pub struct User {
  pub username: String,
  pub discord_id: u64,
  pub url: String,
  pub verified: bool,
  pub created_at: String,
  pub edited_at: String,
  pub verified_at: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct EditedUser {
  pub old: User,
  pub new: User,
}

#[derive(Debug, Modal)]
#[name = "Add your website to petring :3"] // Struct name by default
pub struct AddWebsite {
  #[name = "Pick username :3, webring.pet/user/{username}"] // Field name by default
  #[placeholder = "username_with_no_spaces_or_periods :3"] // No placeholder by default
  #[min_length = 1] // No length restriction by default (so, 1-4000 chars)
  #[max_length = 64]
  username: String,
  #[name = "Enter your website url, be sure it's valid :3"] // Field name by default
  #[placeholder = "https://example.com"]
  #[min_length = 10]
  #[max_length = 2000]
  url: String,
  #[name = "Yap abt why u want ur website in petring :3"]
  #[placeholder = "Bla bla bla, i like it, and i like fopses :3"]
  #[paragraph]
  #[max_length = 500]
  reason: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct UserSubmission {
  pub username: String,
  pub url: String,
  pub discord_id: u64,
}

#[derive(Debug, Modal)]
#[name = "Edit your petring account information"] // Struct name by default
pub struct EditSubmission {
  #[name = "Change your username maybe? :3"]
  #[placeholder = "username_with_no_spaces_or_periods :3"]
  #[min_length = 1]
  #[max_length = 64]
  username: Option<String>,
  #[name = "Change website url, be sure it's valid :3"]
  #[placeholder = "https://example.com"]
  #[min_length = 10]
  #[max_length = 2000]
  url: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct UserEditSubmission {
  pub discord_id: u64,
  pub username: Option<String>,
  pub url: Option<String>,
}

#[derive(Debug, Modal)]
#[name = "Submit an ad :3"]
pub struct AdSubmission {
  #[name = "Enter an image url, gif or image :3"]
  #[placeholder = "a permanent url, no discord attachments."]
  pub image_url: String,
}

#[derive(Serialize, Clone)]
pub struct ImageSubmission {
  pub image_url: String,
  pub discord_id: u64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Ad {
  pub username: String,
  pub discord_id: u64,
  pub image_url: String,
  pub ad_url: String,
  pub verified: bool,
  pub created_at: String,
  pub edited_at: String,
  pub verified_at: String,
}

#[derive(Deserialize, Debug, Clone, Modal)]
pub struct AdEditSubmission {
  #[name = "Change your image url, be sure it's valid :3"]
  #[placeholder = "a permanent url, no discord attachments."]
  pub image_url: String,
}

#[derive(Deserialize, Debug, Clone, ChoiceParameter)]
pub enum NotifType {
  #[name = "User Submit"]
  #[name = "Notification channel for when someone submits a website"]
  UserSubmit,

  #[name = "Submit Ad"]
  #[name = "Notification channel for when someone submits an ad"]
  AdSubmit,

  #[name = "Verify User"]
  #[name = "Notification channel for when someone's website gets verified"]
  UserVerify,

  #[name = "Verify Ad"]
  #[name = "Notification channel for when someone's ad gets verified"]
  AdVerify,

  #[name = "General"]
  #[name = "Notification channel for when someone deletes, edits a website or ad"]
  General,

  #[name = "DM Fallback"]
  #[name = "Incase User DM fails, send the message to this channel instead"]
  DmFallback,
}

#[derive(ChoiceParameter)]
pub enum FeedbackTopicType {
  #[name = "PetRing"]
  #[name = "Send Feedback or an issue regarding PetRing"]
  PetRing,
  #[name = "PetAds"]
  #[name = "Send Feedback or an issue regarding PetAds"]
  PetAds,
  #[name = "Collar"]
  #[name = "Send Feedback or an issue regarding Collar"]
  Collar,
}

#[derive(Deserialize, Debug, Clone, Modal)]
#[name = "Submit feedback or an issue :3"]
pub struct FeedbackSubmission {
  #[name = "Title"]
  #[placeholder = "Title of your feedback/issue"]
  #[min_length = 5]
  pub title: String,

  #[name = "Description"]
  #[placeholder = "Description of your feedback/issue"]
  #[min_length = 50]
  #[paragraph]
  pub description: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WebhookEmbedAuthor {
  name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WebhookEmbedThumbnail {
  url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WebhookEmbedFooter {
  text: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WebhookEmbed {
  title: String,
  description: String,
  color: i64,
  footer: WebhookEmbedFooter,
  thumbnail: WebhookEmbedThumbnail,
  author: WebhookEmbedAuthor,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WebhookPost {
  username: String,
  avatar_url: String,
  tts: bool,
  embeds: Vec<WebhookEmbed>,
}

pub async fn send_generic_error_application(
  ctx: CollarAppContext<'_>,
  body: &str,
) -> Result<(), CollarError> {
  let embed = EmbedWrapper::new_application(&ctx)
    .title("Error")
    .description(body)
    .color(Color::from_rgb(255, 0, 0));

  let reply = CreateReply::default()
    .embed(embed)
    .reply(true)
    .ephemeral(true);

  ctx.send(reply).await?;

  Ok(())
}

pub async fn send_generic_error_normal(
  ctx: CollarContext<'_>,
  body: &str,
) -> Result<(), CollarError> {
  let embed = EmbedWrapper::new_normal(&ctx)
    .title("Error")
    .description(body)
    .color(Color::from_rgb(255, 0, 0));

  let reply = CreateReply::default()
    .embed(embed)
    .reply(true)
    .ephemeral(true);

  ctx.send(reply).await?;

  Ok(())
}
