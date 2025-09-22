use super::{
    CollarAppContext, CollarError,
    commands::User,
    http::{ResponseTypes, make_request},
};
use chrono::DateTime;
use poise::serenity_prelude::{
    self as serenity, Color, CreateInteractionResponse, CreateInteractionResponseMessage,
    FormattedTimestamp, FormattedTimestampStyle, Mentionable,
};
use reqwest::Method;
use serde::Deserialize;
use serenity::{
    ButtonStyle, ComponentInteractionCollector, CreateButton, CreateEmbed, CreateEmbedAuthor,
    CreateEmbedFooter, CreateMessage, Timestamp,
};
use tracing::{error, info};

async fn verify_user(ctx: CollarAppContext<'_>, discord_id: u64) -> Result<(), CollarError> {
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

#[derive(Deserialize, Debug)]
pub struct DeleteUserResponse {
    pub message: String,
}

async fn reject_user(ctx: CollarAppContext<'_>, discord_id: u64) -> Result<String, CollarError> {
    let response = make_request(
        ctx.data().clone(),
        None::<String>,
        &format!("/api/delete/user/by-discord/{discord_id}"),
        Method::DELETE,
    )
    .await?;

    match response {
        ResponseTypes::Success(_user) => {
            let _response: DeleteUserResponse = _user;
            Ok(_response.message)
        }
        ResponseTypes::Error(error) => {
            error!("Failed to get user: {error:?}");
            Err(CollarError::from(format!("Failed to get user: {error:?}")))
        }
    }
}

pub async fn send_submit_notif(
    ctx: CollarAppContext<'_>,
    discord_id: u64,
) -> Result<(), CollarError> {
    let data = ctx.data();
    let bot_id = ctx.data().bot_id;
    let bot_pfp = ctx.cache().user(bot_id).unwrap().avatar_url().unwrap(); // if this fails to unwrap, i'll buy myself a beer

    let channel_id = data.notif_channel_id.lock().await;
    let channel_id = match *channel_id {
        Some(channel_id) => channel_id,
        None => {
            return Ok(());
        }
    };

    let channel = match ctx.http().get_channel(channel_id.into()).await {
        Ok(channel) => channel,
        Err(_) => {
            return Ok(());
        }
    };

    let response = make_request(
        data.clone(),
        None::<String>,
        &format!("/api/get/user/by-discord/{}/unverified", discord_id),
        Method::GET,
    )
    .await?;

    let user = match response {
        ResponseTypes::Success(user) => {
            let user: User = user;
            user
        }
        ResponseTypes::Error(error) => {
            error!("Failed to get user: {error:?}");
            return Ok(());
        }
    };

    let user_pfp = match ctx.http().get_user(discord_id.into()).await {
        Ok(user) => user.avatar_url().unwrap(),
        Err(_) => {
            error!("Failed to get user pfp");
            "".to_string()
        }
    };

    let created_at_timestamp =
        Timestamp::from(chrono::DateTime::parse_from_rfc3339(&user.created_at)?);

    let discord_user = match ctx.http().get_user(discord_id.into()).await {
        Ok(user) => user,
        Err(_) => {
            panic!("Failed to get user name");
        }
    };

    let user_created_at_timestamp = Timestamp::from(DateTime::parse_from_rfc3339(
        &discord_user.created_at().to_string(),
    )?);

    let guild_member = match ctx
        .http()
        .get_member(ctx.guild_id().unwrap(), discord_id.into())
        .await
    {
        Ok(member) => member,
        Err(_) => {
            panic!("Failed to get guild member");
        }
    };

    let formatted_user_created_at_timestamp = FormattedTimestamp::new(
        user_created_at_timestamp,
        Some(FormattedTimestampStyle::LongDateTime),
    )
    .to_string();

    let joined_at_timestamp: Option<Timestamp> = match guild_member.joined_at {
        Some(joined_at) => Some(Timestamp::from(DateTime::parse_from_rfc3339(
            &joined_at.to_string(),
        )?)),
        None => None,
    };

    let formatted_created_at_timestamp = FormattedTimestamp::new(
        created_at_timestamp,
        Some(FormattedTimestampStyle::LongDateTime),
    )
    .to_string();

    let formatted_joined_at_timestamp = FormattedTimestamp::new(
        joined_at_timestamp.unwrap(),
        Some(FormattedTimestampStyle::RelativeTime),
    )
    .to_string();

    let mut embed = CreateEmbed::default()
        .title("New submission :3")
        .author(CreateEmbedAuthor::new(format!(
            "from: {}",
            discord_user.name
        )))
        .field("Website", user.url, false)
        .field("Created at", formatted_created_at_timestamp, false)
        .field("User joined at", formatted_joined_at_timestamp, false)
        .field(
            "User Created at",
            formatted_user_created_at_timestamp,
            false,
        )
        .footer(
            CreateEmbedFooter::new("Collar :3, a Discord bot helper for petring and petads :3")
                .icon_url(bot_pfp),
        )
        .color(serenity::Color::from_rgb(0, 0, 255));

    if !user_pfp.is_empty() {
        embed = embed.thumbnail(user_pfp);
    }

    let action_row = serenity::CreateActionRow::Buttons(vec![
        CreateButton::new("verify-submission")
            .label("Verify submission")
            .style(ButtonStyle::Success),
        CreateButton::new("reject-submission")
            .label("Reject submission")
            .style(ButtonStyle::Danger),
    ]);

    let message = CreateMessage::new()
        .components(vec![action_row])
        .embed(embed);

    channel.id().send_message(&ctx.http(), message).await?;

    while let Some(mci) = ComponentInteractionCollector::new(ctx.serenity_context()).await {
        match mci.data.custom_id.as_str() {
            "verify-submission" => match verify_user(ctx, discord_id).await {
                Ok(_) => {
                    info!("Verified user {} ({discord_id})", discord_user.name);
                    let bot_pfp = ctx.cache().user(bot_id).unwrap().avatar_url().unwrap(); // if this fails to unwrap, i'll buy myself a beer
                    let embed = CreateEmbed::default()
                        .title("Verified :3")
                        .description(format!("Verified user: {}", discord_user.mention()))
                        .footer(
                            CreateEmbedFooter::new(
                                "Collar :3, a Discord bot helper for petring and petads :3",
                            )
                            .icon_url(bot_pfp),
                        )
                        .color(Color::from_rgb(0, 255, 0));

                    mci.create_response(
                        &ctx.http(),
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new()
                                .embed(embed)
                                .ephemeral(true),
                        ),
                    )
                    .await?;

                    info!("Deleting message: {}", mci.message.id);

                    channel
                        .id()
                        .delete_message(&ctx.http(), mci.message.id)
                        .await?;

                    break;
                }
                Err(err) => {
                    error!("Failed to verify: {err}");
                    let bot_pfp = ctx.cache().user(bot_id).unwrap().avatar_url().unwrap(); // if this fails to unwrap, i'll buy myself a beer
                    let embed = CreateEmbed::default()
                        .title("Failed to verify 3:")
                        .description(format!("Failed to verify: {err}"))
                        .footer(
                            CreateEmbedFooter::new(
                                "Collar :3, a Discord bot helper for petring and petads :3",
                            )
                            .icon_url(bot_pfp),
                        )
                        .color(Color::from_rgb(255, 0, 0));

                    mci.create_response(
                        &ctx.http(),
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new()
                                .embed(embed)
                                .ephemeral(true),
                        ),
                    )
                    .await?;

                    info!("Deleting message: {}", mci.message.id);

                    channel
                        .id()
                        .delete_message(&ctx.http(), mci.message.id)
                        .await?;

                    break;
                }
            },
            "reject-submission" => match reject_user(ctx, discord_id).await {
                Ok(_) => {
                    info!("Deleted user {} ({discord_id})", discord_user.name);
                    let bot_pfp = ctx.cache().user(bot_id).unwrap().avatar_url().unwrap(); // if this fails to unwrap, i'll buy myself a beer
                    let embed = CreateEmbed::default()
                        .title("Rejected :3")
                        .description(format!("Rejected user: {}", discord_user.mention()))
                        .footer(
                            CreateEmbedFooter::new(
                                "Collar :3, a Discord bot helper for petring and petads :3",
                            )
                            .icon_url(bot_pfp),
                        )
                        .color(Color::from_rgb(255, 0, 0));
                    mci.create_response(
                        &ctx.http(),
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new()
                                .embed(embed)
                                .ephemeral(true),
                        ),
                    )
                    .await?;

                    info!("Deleting message: {}", mci.message.id);

                    channel
                        .id()
                        .delete_message(&ctx.http(), mci.message.id)
                        .await?;

                    break;
                }
                Err(err) => {
                    error!("Failed to delete user: {err}");
                    let bot_pfp = ctx.cache().user(bot_id).unwrap().avatar_url().unwrap(); // if this fails to unwrap, i'll buy myself a beer
                    let embed = CreateEmbed::default()
                        .title("Failed to reject 3:")
                        .description(format!("Failed to reject: {err}"))
                        .footer(
                            CreateEmbedFooter::new(
                                "Collar :3, a Discord bot helper for petring and petads :3",
                            )
                            .icon_url(bot_pfp),
                        )
                        .color(Color::from_rgb(255, 0, 0));

                    mci.create_response(
                        &ctx.http(),
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new()
                                .embed(embed)
                                .ephemeral(true),
                        ),
                    )
                    .await?;

                    info!("Deleting message: {}", mci.message.id);

                    channel
                        .id()
                        .delete_message(&ctx.http(), mci.message.id)
                        .await?;

                    break;
                }
            },
            _ => {}
        }
    }

    Ok(())
}
