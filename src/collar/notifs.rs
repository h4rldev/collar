use crate::collar::EmbedWrapper;

use super::{
    COLLAR_FOOTER, CollarAppContext, CollarError,
    commands::{Ad, User},
    http::{ResponseTypes, make_request},
};
use poise::{
    CreateReply,
    serenity_prelude::{self as serenity, ChannelId, CreateActionRow},
};
use reqwest::Method;
use serenity::{
    ButtonStyle, Color, ComponentInteractionCollector, CreateButton, CreateEmbed,
    CreateEmbedAuthor, CreateEmbedFooter, CreateInteractionResponse,
    CreateInteractionResponseMessage, CreateMessage, Mentionable,
};
use tracing::{error, info};

pub enum SubmitType {
    Ad,
    User,
}

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

    let success_ad_embed = EmbedWrapper::new_application(ctx)
        .0
        .title("Verified :3")
        .description(format!("Verified ad for: {}", user_mention))
        .color(Color::from_rgb(0, 255, 0));

    let success_user_embed = EmbedWrapper::new_application(ctx)
        .0
        .title("Verified :3")
        .description(format!("Verified user: {}", user_mention))
        .color(Color::from_rgb(0, 255, 0));

    let error_ad_embed = EmbedWrapper::new_application(ctx)
        .0
        .title("Failed to verify 3:")
        .description(format!("Failed to verify ad for: {}", user_mention))
        .color(Color::from_rgb(255, 0, 0));

    let error_user_embed = EmbedWrapper::new_application(ctx)
        .0
        .title("Failed to verify 3:")
        .description(format!("Failed to verify user: {}", user_mention))
        .color(Color::from_rgb(255, 0, 0));

    let dm_user_verify_embed = EmbedWrapper::new_application(ctx)
        .0
        .title("You've been verified!!")
        .description(format!(
            "Hi there, {user_mention}, you've been verified !!! Welcome to petring !! :3"
        ))
        .author(CreateEmbedAuthor::new(user.name.clone()))
        .color(Color::from_rgb(0, 255, 0));

    let dm_ad_verify_embed = EmbedWrapper::new_application(ctx)
        .0
        .title("Your ad was verified!!")
        .description(format!(
            "Hi, there, {user_mention}, your ad has been verified :3"
        ))
        .author(CreateEmbedAuthor::new(user.name.clone()))
        .color(Color::from_rgb(0, 255, 0));

    let dm_reject_user_embed = EmbedWrapper::new_application(ctx)
        .0
        .title("You were rejected 3:")
        .description("Please discuss with staff about your rejection")
        .author(CreateEmbedAuthor::new(user.name.clone()))
        .color(Color::from_rgb(255, 0, 0));

    let dm_reject_ad_embed = EmbedWrapper::new_application(ctx)
        .0
        .title("Your ad was rejected 3:")
        .description("Please discuss with staff about your rejection")
        .author(CreateEmbedAuthor::new(user.name))
        .color(Color::from_rgb(255, 0, 0));

    let reject_ad_embed = EmbedWrapper::new_application(ctx)
        .0
        .title("Rejected ad :3")
        .description(format!("Rejected ad for: {user_mention}"))
        .color(Color::from_rgb(255, 0, 0));

    let reject_user_embed = EmbedWrapper::new_application(ctx)
        .0
        .title("Rejected user :3")
        .description(format!("Rejected user: {user_mention}"))
        .color(Color::from_rgb(255, 0, 0));

    let error_reject_user_embed = EmbedWrapper::new_application(ctx)
        .0
        .title("Failed to reject user 3:")
        .description(format!("Failed to reject user: {user_mention}"))
        .color(Color::from_rgb(255, 0, 0));

    let error_reject_ad_embed = EmbedWrapper::new_application(ctx)
        .0
        .title("Failed to reject ad 3:")
        .description(format!("Failed to reject ad for: {user_mention}"))
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
                        Ok(_) => {
                            info!("Sending ephermeral embed for successful ad verification");
                            mci.create_response(
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
                            notif = notif.set_embed(dm_ad_verify_embed);
                            notif.dm_notif(ctx, user_id).await?;
                            break;
                        }
                        Err(err) => {
                            error!("Failed to verify ad: {err}");
                            mci.create_response(
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
                            mci.create_response(
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
                            notif = notif.set_embed(dm_user_verify_embed);
                            notif.dm_notif(ctx, user_id).await?;
                            break;
                        }
                        Err(err) => {
                            error!("Failed to verify user: {err}");
                            mci.create_response(
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
                    SubmitType::Ad => match reject_ad(ctx, user_id).await {
                        Ok(_) => {
                            info!("Sending ephermeral embed for successful ad rejection");
                            mci.create_response(
                                &ctx.http(),
                                CreateInteractionResponse::Message(
                                    CreateInteractionResponseMessage::new()
                                        .embed(reject_ad_embed)
                                        .ephemeral(true),
                                ),
                            )
                            .await?;

                            channel_id
                                .delete_message(&ctx.http(), mci.message.id)
                                .await?;
                            notif = notif.set_embed(dm_reject_ad_embed);
                            notif.dm_notif(ctx, user_id).await?;
                            break;
                        }
                        Err(err) => {
                            error!("Failed to reject ad: {err}");
                            mci.create_response(
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
                    },
                    SubmitType::User => match reject_user(ctx, user_id).await {
                        Ok(_) => {
                            info!("Sending ephermeral embed for successful user rejection");
                            mci.create_response(
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

                            notif = notif.set_embed(dm_reject_user_embed);
                            notif.dm_notif(ctx, user_id).await?;
                            break;
                        }
                        Err(err) => {
                            error!("Failed to reject user: {err}");
                            mci.create_response(
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
                    },
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
        let bot_pfp = ctx
            .cache()
            .user(ctx.data().bot_id)
            .unwrap()
            .avatar_url()
            .unwrap(); // if
        // this fails to unwrap, i'll buy myself a beer

        let embed =
            CreateEmbed::default().footer(CreateEmbedFooter::new(COLLAR_FOOTER).icon_url(bot_pfp));

        Self { embed }
    }

    pub fn get_embed(self) -> CreateEmbed {
        self.embed
    }

    pub fn set_embed(mut self, embed: CreateEmbed) -> Self {
        self.embed = embed;

        self
    }

    pub async fn general(self, ctx: &CollarAppContext<'_>) -> Result<(), CollarError> {
        let channel_ids = ctx.data().notif_channel_ids.lock().await;
        let general_channel_id = match channel_ids.general_id {
            Some(channel_id) => channel_id,
            None => {
                let save_warning_embed = EmbedWrapper::new_application(ctx).0.title("No channel")
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
        let message = CreateMessage::new().embed(self.embed);

        channel_id.send_message(&ctx.http(), message).await?;

        Ok(())
    }

    pub async fn submit(
        self,
        ctx: &CollarAppContext<'_>,
        user_id: u64,
        submit_type: SubmitType,
    ) -> Result<(), CollarError> {
        let channel_ids = ctx.data().notif_channel_ids.lock().await;
        let submit_channel_id = match channel_ids.submit_id {
            Some(channel_id) => channel_id,
            None => {
                let save_warning_embed = EmbedWrapper::new_application(ctx).0.title("No channel")
                    .description("No channel was found for submit notifications, please set one up using `/set_notification_channel`")
                    .color(Color::from_rgb(255, 0, 0));

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
            .embed(self.embed)
            .components(vec![action_row]);

        channel_id.send_message(ctx.http(), message).await?;
        process_mci(user_id, ctx, ctx.serenity_context(), submit_type).await?;

        Ok(())
    }

    async fn dm_notif_fallback(self, ctx: &CollarAppContext<'_>) -> Result<(), CollarError> {
        let channel_ids = ctx.data().notif_channel_ids.lock().await;
        let fallback_channel_id = match channel_ids.fallback_id {
            Some(channel_id) => channel_id,
            None => {
                let save_warning_embed = EmbedWrapper::new_application(ctx).0.title("No channel")
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

        let channel_id: ChannelId = fallback_channel_id.into();

        let message = CreateMessage::new().embed(self.embed);

        channel_id.send_message(&ctx.http(), message).await?;
        Ok(())
    }
    pub async fn dm_notif(
        self,
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

async fn verify_user(ctx: &CollarAppContext<'_>, discord_id: u64) -> Result<(), CollarError> {
    let response = make_request(
        ctx.data().clone(),
        None::<String>,
        &format!("/api/put/user/verify/{}", discord_id),
        Method::PUT,
    )
    .await?;

    match response {
        ResponseTypes::Success(_user) => {
            let _user: User = _user;
            Ok(())
        }
        ResponseTypes::Error(error) => {
            error!("Failed to get user: {error:?}");
            Err(CollarError::from(format!("Failed to get user: {error:?}")))
        }
    }
}

async fn verify_ad(ctx: &CollarAppContext<'_>, discord_id: u64) -> Result<(), CollarError> {
    let response = make_request(
        ctx.data().clone(),
        None::<String>,
        &format!("/api/put/ad/verify/{}", discord_id),
        Method::PUT,
    )
    .await?;

    match response {
        ResponseTypes::Success(_user) => {
            let _user: User = _user;
            Ok(())
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
        &format!("/api/delete/ad/by-discord/{discord_id}"),
        Method::DELETE,
    )
    .await?;

    match response {
        ResponseTypes::Success(_user) => {
            let _response: Ad = _user;
            Ok(_response)
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
        &format!("/api/delete/user/by-discord/{discord_id}"),
        Method::DELETE,
    )
    .await?;

    match response {
        ResponseTypes::Success(_user) => {
            let _response: User = _user;
            Ok(_response)
        }
        ResponseTypes::Error(error) => {
            error!("Failed to get user: {error:?}");
            Err(CollarError::from(format!("Failed to get user: {error:?}")))
        }
    }
}
