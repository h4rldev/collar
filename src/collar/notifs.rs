use crate::collar::{EmbedWrapper, NotifChannelType};

use super::{
  CollarAppContext, CollarError,
  commands::{Ad, User},
  http::{ResponseTypes, make_request},
};
use poise::{
  CreateReply,
  serenity_prelude::{
    self as serenity, ChannelId, CreateActionRow, CreateInputText, CreateQuickModal,
  },
};
use reqwest::Method;
use serenity::{
  ButtonStyle, Color, ComponentInteractionCollector, CreateButton, CreateEmbed, CreateEmbedAuthor,
  CreateInteractionResponse, CreateInteractionResponseMessage, CreateMessage, Mentionable,
};
use tracing::{error, info};

pub enum SubmitType {
  Ad,
  User,
}

pub type VerifyType = SubmitType;

#[derive(Clone)]
pub struct Notif {
  embed: CreateEmbed,
}

async fn process_mci(
  user_id: u64,
  ctx: &CollarAppContext<'_>,
  shard: &serenity::Context,
  submit_type: SubmitType,
) -> Result<(), CollarError> {
  let user = ctx.http().get_user(user_id.into()).await?;
  let user_mention = user.mention();
  let user_pfp = user.face();

  let user_verification_done_embed = EmbedWrapper::new_application(ctx)
    .title("A User has been verified :3")
    .description(format!("Verified user: {}", user_mention))
    .color(Color::from_rgb(0, 255, 0));

  let ad_verification_done_embed = EmbedWrapper::new_application(ctx)
    .title("An Ad has been verified :3")
    .description(format!("Verified ad for: {}", user_mention))
    .color(Color::from_rgb(0, 255, 0));

  let success_ad_embed = EmbedWrapper::new_application(ctx)
    .title("Verified :3")
    .description(format!("Verified ad for: {}", user_mention))
    .thumbnail(&user_pfp)
    .color(Color::from_rgb(0, 255, 0));

  let success_user_embed = EmbedWrapper::new_application(ctx)
    .title("Verified :3")
    .description(format!("Verified user: {}", user_mention))
    .thumbnail(&user_pfp)
    .color(Color::from_rgb(0, 255, 0));

  let error_ad_embed = EmbedWrapper::new_application(ctx)
    .title("Failed to verify 3:")
    .description(format!("Failed to verify ad for: {}", user_mention))
    .thumbnail(&user_pfp)
    .color(Color::from_rgb(255, 0, 0));

  let error_user_embed = EmbedWrapper::new_application(ctx)
    .title("Failed to verify 3:")
    .description(format!("Failed to verify user: {}", user_mention))
    .thumbnail(&user_pfp)
    .color(Color::from_rgb(255, 0, 0));

  let dm_user_verify_embed = EmbedWrapper::new_application(ctx)
    .title("You've been verified!!")
    .description(format!(
      "Hi there, {user_mention}, you've been verified, welcome to PetRing !! :3"
    ))
    .author(
      CreateEmbedAuthor::new(format!("Verified by: {}", ctx.author().name))
        .icon_url(ctx.author().face()),
    )
    .color(Color::from_rgb(0, 255, 0));

  let dm_ad_verify_embed = EmbedWrapper::new_application(ctx)
    .title("Your ad was verified!!")
    .description(format!(
      "Hi, there, {user_mention}, your ad has been verified :3"
    ))
    .color(Color::from_rgb(0, 255, 0));

  let mut dm_reject_user_embed = EmbedWrapper::new_application(ctx)
    .title("You were rejected 3:")
    .color(Color::from_rgb(255, 0, 0));

  let mut dm_reject_ad_embed = EmbedWrapper::new_application(ctx)
    .title("Your ad was rejected 3:")
    .color(Color::from_rgb(255, 0, 0));

  let reject_ad_embed = EmbedWrapper::new_application(ctx)
    .title("Rejected ad :3")
    .description(format!("Rejected ad for: {user_mention}"))
    .thumbnail(&user_pfp)
    .color(Color::from_rgb(255, 0, 0));

  let reject_user_embed = EmbedWrapper::new_application(ctx)
    .title("Rejected user :3")
    .description(format!("Rejected user: {user_mention}"))
    .thumbnail(&user_pfp)
    .color(Color::from_rgb(255, 0, 0));

  let error_reject_user_embed = EmbedWrapper::new_application(ctx)
    .title("Failed to reject user 3:")
    .description(format!("Failed to reject user: {user_mention}"))
    .thumbnail(&user_pfp)
    .color(Color::from_rgb(255, 0, 0));

  let error_reject_ad_embed = EmbedWrapper::new_application(ctx)
    .title("Failed to reject ad 3:")
    .description(format!("Failed to reject ad for: {user_mention}"))
    .thumbnail(&user_pfp)
    .color(Color::from_rgb(255, 0, 0));

  let mut notif = Notif::new(ctx);

  while let Some(mci) = ComponentInteractionCollector::new(shard).await {
    let channel_id = mci.channel_id;
    let id_str = mci.data.custom_id.as_str();

    match id_str {
      "verify-submission" => {
        info!("Verifying submission");
        match submit_type {
          SubmitType::Ad => match verify_ad(ctx, user_id).await {
            Ok(ad) => {
              info!("Sending ephermeral embed for successful ad verification");
              let success_ad_embed = success_ad_embed.thumbnail(&ad.image_url);

              mci
                .create_response(
                  &ctx.http(),
                  CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                      .embed(success_ad_embed.clone())
                      .ephemeral(true),
                  ),
                )
                .await?;

              channel_id
                .delete_message(&ctx.http(), mci.message.id)
                .await?;

              let dm_ad_verify_embed = dm_ad_verify_embed.thumbnail(&ad.image_url).author(
                CreateEmbedAuthor::new(format!("Verified by: {}", mci.user.name))
                  .icon_url(mci.user.face()),
              );
              notif
                .set_embed(dm_ad_verify_embed)
                .dm_notif(ctx, user_id)
                .await?;

              let ad_verification_done_embed =
                ad_verification_done_embed.thumbnail(&ad.image_url).author(
                  CreateEmbedAuthor::new(format!("Verified by: {}", mci.user.name))
                    .icon_url(mci.user.face()),
                );

              Notif::new(ctx)
                .set_embed(ad_verification_done_embed)
                .verification(ctx, VerifyType::Ad)
                .await?;
              break;
            }
            Err(err) => {
              error!("Failed to verify ad: {err}");
              mci
                .create_response(
                  &ctx.http(),
                  CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                      .embed(error_ad_embed.clone())
                      .ephemeral(true),
                  ),
                )
                .await?;

              channel_id
                .delete_message(&ctx.http(), mci.message.id)
                .await?;
              break;
            }
          },
          SubmitType::User => match verify_user(ctx, user_id).await {
            Ok(_) => {
              info!("Sending ephermeral embed for successful user verification");
              mci
                .create_response(
                  &ctx.http(),
                  CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                      .embed(success_user_embed.clone())
                      .ephemeral(true),
                  ),
                )
                .await?;

              channel_id
                .delete_message(&ctx.http(), mci.message.id)
                .await?;
              notif
                .set_embed(dm_user_verify_embed)
                .dm_notif(ctx, user_id)
                .await?;

              let user_verification_done_embed = user_verification_done_embed.author(
                CreateEmbedAuthor::new(format!("Verified by: {}", mci.user.name))
                  .icon_url(mci.user.face()),
              );
              Notif::new(ctx)
                .set_embed(user_verification_done_embed)
                .verification(ctx, VerifyType::User)
                .await?;
              break;
            }
            Err(err) => {
              error!("Failed to verify user: {err}");
              mci
                .create_response(
                  &ctx.http(),
                  CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                      .embed(error_user_embed.clone())
                      .ephemeral(true),
                  ),
                )
                .await?;

              channel_id
                .delete_message(&ctx.http(), mci.message.id)
                .await?;
              break;
            }
          },
        }
      }
      "reject-submission" => {
        info!("Rejecting submission");

        match submit_type {
          SubmitType::Ad => {
            let reject_modal = CreateQuickModal::new("Reject Ad submission")
              .timeout(std::time::Duration::from_secs(600))
              .field(
                CreateInputText::new(
                  serenity::InputTextStyle::Short,
                  "Reason",
                  "rejection-reason",
                )
                .placeholder("Enter rejection reason here!")
                .required(true)
                .min_length(10),
              );

            let response = mci
              .quick_modal(ctx.serenity_context(), reject_modal)
              .await?;

            match response {
              Some(modal) => {
                let reason = &modal.inputs[0];
                dm_reject_ad_embed = dm_reject_ad_embed
                  .description(format!("Reason: {reason}"))
                  .author(
                    CreateEmbedAuthor::new(format!("Rejected by: {}", mci.user.name))
                      .icon_url(mci.user.face()),
                  );
              }
              None => {
                let embed = EmbedWrapper::new_application(ctx)
                  .title("You didn't specify reason")
                  .description("No data was submitted 3:")
                  .color(Color::from_rgb(255, 0, 0));
                mci
                  .create_response(
                    &ctx.http(),
                    CreateInteractionResponse::Message(
                      CreateInteractionResponseMessage::new()
                        .embed(embed)
                        .ephemeral(true),
                    ),
                  )
                  .await?;
                continue;
              }
            }

            match reject_ad(ctx, user_id).await {
              Ok(ad) => {
                info!("Sending ephermeral embed for successful ad rejection");
                let reject_ad_embed = reject_ad_embed.thumbnail(&ad.image_url);

                mci
                  .create_response(
                    &ctx.http(),
                    CreateInteractionResponse::Message(
                      CreateInteractionResponseMessage::new()
                        .embed(reject_ad_embed)
                        .ephemeral(true),
                    ),
                  )
                  .await?;

                let dm_reject_ad_embed = dm_reject_ad_embed.thumbnail(&ad.image_url);
                channel_id
                  .delete_message(&ctx.http(), mci.message.id)
                  .await?;
                notif
                  .set_embed(dm_reject_ad_embed)
                  .dm_notif(ctx, user_id)
                  .await?;
                break;
              }
              Err(err) => {
                error!("Failed to reject ad: {err}");
                mci
                  .create_response(
                    &ctx.http(),
                    CreateInteractionResponse::Message(
                      CreateInteractionResponseMessage::new()
                        .embed(error_reject_ad_embed)
                        .ephemeral(true),
                    ),
                  )
                  .await?;

                channel_id
                  .delete_message(&ctx.http(), mci.message.id)
                  .await?;
                break;
              }
            }
          }
          SubmitType::User => {
            let reject_modal = CreateQuickModal::new("Reject User submission")
              .timeout(std::time::Duration::from_secs(600))
              .field(
                CreateInputText::new(
                  serenity::InputTextStyle::Short,
                  "Reason",
                  "rejection-reason",
                )
                .placeholder("Enter rejection reason here!")
                .required(true)
                .min_length(10),
              );

            let response = mci
              .quick_modal(ctx.serenity_context(), reject_modal)
              .await?;

            match response {
              Some(modal) => {
                let reason = &modal.inputs[0];
                dm_reject_user_embed = dm_reject_user_embed
                  .description(format!("Reason: {reason}"))
                  .author(
                    CreateEmbedAuthor::new(format!("Rejected by: {}", mci.user.name))
                      .icon_url(mci.user.face()),
                  );
              }
              None => {
                let embed = EmbedWrapper::new_application(ctx)
                  .title("You didn't specify reason")
                  .description("No data was submitted 3:")
                  .color(Color::from_rgb(255, 0, 0));
                mci
                  .create_response(
                    &ctx.http(),
                    CreateInteractionResponse::Message(
                      CreateInteractionResponseMessage::new()
                        .embed(embed)
                        .ephemeral(true),
                    ),
                  )
                  .await?;
                continue;
              }
            }

            match reject_user(ctx, user_id).await {
              Ok(_) => {
                info!("Sending ephermeral embed for successful user rejection");
                mci
                  .create_response(
                    &ctx.http(),
                    CreateInteractionResponse::Message(
                      CreateInteractionResponseMessage::new()
                        .embed(reject_user_embed)
                        .ephemeral(true),
                    ),
                  )
                  .await?;

                channel_id
                  .delete_message(&ctx.http(), mci.message.id)
                  .await?;

                notif
                  .set_embed(dm_reject_user_embed)
                  .dm_notif(ctx, user_id)
                  .await?;
                break;
              }
              Err(err) => {
                error!("Failed to reject user: {err}");
                mci
                  .create_response(
                    &ctx.http(),
                    CreateInteractionResponse::Message(
                      CreateInteractionResponseMessage::new()
                        .embed(error_reject_user_embed)
                        .ephemeral(true),
                    ),
                  )
                  .await?;

                channel_id
                  .delete_message(&ctx.http(), mci.message.id)
                  .await?;
                break;
              }
            }
          }
        }
      }
      _ => {
        continue;
      }
    }
  }
  Ok(())
}

impl Notif {
  pub fn new(ctx: &CollarAppContext<'_>) -> Self {
    let embed = EmbedWrapper::new_application(ctx);

    Self { embed }
  }

  /*pub fn get_embed(self) -> CreateEmbed {
      self.embed
  }*/

  pub fn set_embed(&mut self, embed: CreateEmbed) -> &mut Self {
    self.embed = embed;

    self
  }

  pub async fn general(&self, ctx: &CollarAppContext<'_>) -> Result<(), CollarError> {
    let data = ctx.data();
    let cache = data.cache.lock().await;
    let channel_id = cache.get_notif_channel(NotifChannelType::General);
    let general_channel_id = match channel_id {
      Some(general_id) => general_id,
      None => {
        let save_warning_embed = EmbedWrapper::new_application(ctx).title("No channel")
                    .description("No channel was found for general notifications, please set one up using `/set_notification_channel`")
                    .color(Color::from_rgb(255, 0, 0));

        let reply = CreateReply::default()
          .ephemeral(true)
          .reply(true)
          .embed(save_warning_embed);
        ctx.send(reply).await?;
        return Ok(());
      }
    };

    let channel_id: ChannelId = general_channel_id.into();
    let message = CreateMessage::new().embed(self.embed.clone());

    channel_id.send_message(&ctx.http(), message).await?;

    Ok(())
  }

  pub async fn submit(
    &mut self,
    ctx: &CollarAppContext<'_>,
    user_id: u64,
    submit_type: SubmitType,
  ) -> Result<(), CollarError> {
    let data = ctx.data();
    let cache = data.cache.lock().await;

    let what_channel_id = match submit_type {
      SubmitType::User => cache.get_notif_channel(NotifChannelType::UserSubmit),
      SubmitType::Ad => cache.get_notif_channel(NotifChannelType::AdSubmit),
    };

    let submit_channel_id = match what_channel_id {
      Some(channel_id) => channel_id,
      None => {
        let mut save_warning_embed = EmbedWrapper::new_application(ctx)
          .title("No channel")
          .color(Color::from_rgb(255, 0, 0));

        match submit_type {
          SubmitType::User => {
            save_warning_embed = save_warning_embed.description("No channel was found for user submit notifications, please set one up using `/set_notification_channel`");
          }
          SubmitType::Ad => {
            save_warning_embed = save_warning_embed.description("No channel was found for ad submit notifications, please set one up using `/set_notification_channel`");
          }
        }

        let reply = CreateReply::default()
          .ephemeral(true)
          .reply(true)
          .embed(save_warning_embed);
        ctx.send(reply).await?;
        return Ok(());
      }
    };

    let channel_id: ChannelId = submit_channel_id.into();
    let action_row = CreateActionRow::Buttons(vec![
      CreateButton::new("verify-submission")
        .label("Verify submission")
        .style(ButtonStyle::Success),
      CreateButton::new("reject-submission")
        .label("Reject submission")
        .style(ButtonStyle::Danger),
    ]);

    let message = CreateMessage::new()
      .embed(self.embed.clone())
      .components(vec![action_row]);

    channel_id.send_message(ctx.http(), message).await?;
    process_mci(user_id, ctx, ctx.serenity_context(), submit_type).await?;

    Ok(())
  }

  pub async fn verification(
    &self,
    ctx: &CollarAppContext<'_>,
    verify_type: VerifyType,
  ) -> Result<(), CollarError> {
    let data = ctx.data();
    let cache = data.cache.lock().await;

    let what_channel_id = match verify_type {
      VerifyType::User => cache.get_notif_channel(NotifChannelType::UserVerify),
      VerifyType::Ad => cache.get_notif_channel(NotifChannelType::AdVerify),
    };

    let verification_channel_id = match what_channel_id {
      Some(channel_id) => channel_id,
      None => {
        let mut save_warning_embed = EmbedWrapper::new_application(ctx)
          .title("No channel")
          .color(Color::from_rgb(255, 0, 0));

        match verify_type {
          VerifyType::User => {
            save_warning_embed = save_warning_embed.description("No channel was found for user verify notifications, please set one up using `/set_notification_channel`");
          }
          VerifyType::Ad => {
            save_warning_embed = save_warning_embed.description("No channel was found for ad verify notifications, please set one up using `/set_notification_channel`");
          }
        }

        let reply = CreateReply::default()
          .ephemeral(true)
          .reply(true)
          .embed(save_warning_embed);
        ctx.send(reply).await?;
        return Ok(());
      }
    };

    let channel_id: ChannelId = verification_channel_id.into();
    let message = CreateMessage::new().embed(self.embed.clone());

    channel_id.send_message(&ctx.http(), message).await?;
    Ok(())
  }

  async fn dm_notif_fallback(&self, ctx: &CollarAppContext<'_>) -> Result<(), CollarError> {
    let data = ctx.data();
    let cache = data.cache.lock().await;

    let channel_id = cache.get_notif_channel(NotifChannelType::DmFallback);
    let dm_fallback_channel_id = match channel_id {
      Some(dm_fallback_channel_id) => dm_fallback_channel_id,
      None => {
        let save_warning_embed = EmbedWrapper::new_application(ctx)
                    .title("No channel")
                    .description("No channel was found for failed dm notifications, please set one up using `/set_notification_channel`")
                    .color(Color::from_rgb(255, 0, 0));

        let reply = CreateReply::default()
          .ephemeral(true)
          .reply(true)
          .embed(save_warning_embed);
        ctx.send(reply).await?;
        return Ok(());
      }
    };

    let channel_id: ChannelId = dm_fallback_channel_id.into();
    let message = CreateMessage::new().embed(self.embed.clone());

    channel_id.send_message(&ctx.http(), message).await?;
    Ok(())
  }
  pub async fn dm_notif(
    &self,
    ctx: &CollarAppContext<'_>,
    user_id: u64,
  ) -> Result<(), CollarError> {
    let discord_user = ctx.http().get_user(user_id.into()).await?;
    let message = CreateMessage::new().embed(self.embed.clone());
    match discord_user.direct_message(&ctx.http(), message).await {
      Ok(_) => {
        info!(
          "Successfully dmed user: {} ({user_id}) with notif",
          discord_user.name
        );
        Ok(())
      }
      Err(_) => return Self::dm_notif_fallback(self, ctx).await,
    }
  }
}

async fn verify_user(ctx: &CollarAppContext<'_>, discord_id: u64) -> Result<User, CollarError> {
  let response = make_request(
    ctx.data().clone(),
    None::<String>,
    &format!("/patch/user/verify/{}", discord_id),
    Method::PATCH,
  )
  .await?;

  match response {
    ResponseTypes::Success(_user) => {
      let user: User = _user;
      Ok(user)
    }
    ResponseTypes::Error(error) => {
      error!("Failed to get user: {error:?}");
      Err(CollarError::from(format!("Failed to get user: {error:?}")))
    }
  }
}

async fn verify_ad(ctx: &CollarAppContext<'_>, discord_id: u64) -> Result<Ad, CollarError> {
  let response = make_request(
    ctx.data().clone(),
    None::<String>,
    &format!("/patch/ad/verify/{}", discord_id),
    Method::PATCH,
  )
  .await?;

  match response {
    ResponseTypes::Success(_ad) => {
      let ad: Ad = _ad;
      Ok(ad)
    }
    ResponseTypes::Error(error) => {
      error!("Failed to get user: {error:?}");
      Err(CollarError::from(format!("Failed to get user: {error:?}")))
    }
  }
}

async fn reject_ad(ctx: &CollarAppContext<'_>, discord_id: u64) -> Result<Ad, CollarError> {
  let response = make_request(
    ctx.data().clone(),
    None::<String>,
    &format!("/delete/ad/by-discord/{discord_id}"),
    Method::DELETE,
  )
  .await?;

  match response {
    ResponseTypes::Success(_ad) => {
      let ad: Ad = _ad;
      Ok(ad)
    }
    ResponseTypes::Error(error) => {
      error!("Failed to get user: {error:?}");
      Err(CollarError::from(format!("Failed to get user: {error:?}")))
    }
  }
}

async fn reject_user(ctx: &CollarAppContext<'_>, discord_id: u64) -> Result<User, CollarError> {
  let response = make_request(
    ctx.data().clone(),
    None::<String>,
    &format!("/delete/user/by-discord/{discord_id}"),
    Method::DELETE,
  )
  .await?;

  match response {
    ResponseTypes::Success(_user) => {
      let user: User = _user;
      Ok(user)
    }
    ResponseTypes::Error(error) => {
      error!("Failed to get user: {error:?}");
      Err(CollarError::from(format!("Failed to get user: {error:?}")))
    }
  }
}
