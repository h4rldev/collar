use super::{
    commands::{Ad, EditedUser, User},
    http::{make_request, ResponseTypes},
    CollarAppContext, CollarError, COLLAR_FOOTER,
};
use chrono::DateTime;
use poise::serenity_prelude as serenity;
use reqwest::Method;
use serenity::{
    ButtonStyle, Color, ComponentInteractionCollector, CreateButton, CreateEmbed,
    CreateEmbedAuthor, CreateEmbedFooter, CreateInteractionResponse,
    CreateInteractionResponseMessage, CreateMessage, FormattedTimestamp, FormattedTimestampStyle,
    Mentionable, Timestamp,
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

async fn verify_ad(ctx: CollarAppContext<'_>, discord_id: u64) -> Result<(), CollarError> {
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

async fn reject_ad(ctx: CollarAppContext<'_>, discord_id: u64) -> Result<Ad, CollarError> {
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

async fn reject_user(ctx: CollarAppContext<'_>, discord_id: u64) -> Result<User, CollarError> {
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

pub async fn send_submit_user_notif(
    ctx: CollarAppContext<'_>,
    user: User,
    reason: Option<String>,
) -> Result<(), CollarError> {
    let data = ctx.data();
    let bot_id = ctx.data().bot_id;
    let bot_pfp = ctx.cache().user(bot_id).unwrap().avatar_url().unwrap(); // if this fails to unwrap, i'll buy myself a beer

    let discord_id = user.discord_id;

    let channel_id = data.notif_channel_ids.lock().await;
    let channel_id = match channel_id.submit_id {
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
        .field("Website", user.url.clone(), false)
        .field("Created at", formatted_created_at_timestamp, false)
        .field("User joined at", formatted_joined_at_timestamp, false)
        .field(
            "User Created at",
            formatted_user_created_at_timestamp,
            false,
        )
        .footer(CreateEmbedFooter::new(COLLAR_FOOTER).icon_url(bot_pfp))
        .color(Color::from_rgb(0, 0, 255));

    if !user_pfp.is_empty() {
        embed = embed.thumbnail(user_pfp);
    }

    if let Some(reason) = reason {
        embed = embed.description(reason);
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

                    info!("Making ephemeral embed");
                    let embed = CreateEmbed::default()
                        .title("Verified :3")
                        .description(format!("Verified user: {}", discord_user.mention()))
                        .footer(CreateEmbedFooter::new(COLLAR_FOOTER).icon_url(bot_pfp.clone()))
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

                    send_verify_user_notif_dm(ctx, user).await?;

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
                        .footer(CreateEmbedFooter::new(COLLAR_FOOTER).icon_url(bot_pfp))
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
                Ok(response) => {
                    info!(
                        "Deleted user {} ({})",
                        response.username, response.discord_id
                    );
                    let bot_pfp = ctx.cache().user(bot_id).unwrap().avatar_url().unwrap(); // if this fails to unwrap, i'll buy myself a beer
                    let embed = CreateEmbed::default()
                        .title("Rejected :3")
                        .description(format!("Rejected user: {}", discord_user.mention()))
                        .footer(CreateEmbedFooter::new(COLLAR_FOOTER).icon_url(bot_pfp))
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
                        .footer(CreateEmbedFooter::new(COLLAR_FOOTER).icon_url(bot_pfp))
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

pub async fn send_submit_ad_notif(ctx: CollarAppContext<'_>, ad: Ad) -> Result<(), CollarError> {
    let data = ctx.data();
    let bot_id = ctx.data().bot_id;
    let bot_pfp = ctx.cache().user(bot_id).unwrap().avatar_url().unwrap(); // if this fails to unwrap, i'll buy myself a beer
    let base_url = data.api_base_url.clone();

    let discord_id = ad.discord_id;

    let channel_id = data.notif_channel_ids.lock().await;
    let channel_id = match channel_id.submit_id {
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

    let user_pfp = match ctx.http().get_user(discord_id.into()).await {
        Ok(user) => user.avatar_url().unwrap(),
        Err(_) => {
            error!("Failed to get user pfp");
            "".to_string()
        }
    };

    let user_url = format!("{base_url}/user/{}", ad.username);

    let mut embed = CreateEmbed::default()
        .title("New ad submission :3")
        .author(CreateEmbedAuthor::new(format!("from: {}", ad.username)))
        .field("Website", user_url, false)
        .footer(CreateEmbedFooter::new(COLLAR_FOOTER).icon_url(bot_pfp))
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
            "verify-submission" => match verify_ad(ctx, discord_id).await {
                Ok(_) => {
                    info!("Verified ad {} ({})", ad.username, discord_id);

                    let user_mention = ctx.http().get_user(discord_id.into()).await?.mention();
                    let bot_pfp = ctx.cache().user(bot_id).unwrap().avatar_url().unwrap(); // if this fails to unwrap, i'll buy myself a beer
                    let embed = CreateEmbed::default()
                        .title("Verified :3")
                        .description(format!("Verified ad for: {}", user_mention))
                        .footer(CreateEmbedFooter::new(COLLAR_FOOTER).icon_url(bot_pfp))
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
                        .footer(CreateEmbedFooter::new(COLLAR_FOOTER).icon_url(bot_pfp))
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
            "reject-submission" => match reject_ad(ctx, discord_id).await {
                Ok(_) => {
                    info!("Deleted ad {} ({})", ad.username, discord_id);
                    let user_mention = ctx.http().get_user(discord_id.into()).await?.mention();
                    let bot_pfp = ctx.cache().user(bot_id).unwrap().avatar_url().unwrap(); // if this fails to unwrap, i'll buy myself a beer
                    let embed = CreateEmbed::default()
                        .title("Rejected :3")
                        .description(format!("Rejected ad for: {}", user_mention))
                        .footer(CreateEmbedFooter::new(COLLAR_FOOTER).icon_url(bot_pfp))
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
                        .footer(CreateEmbedFooter::new(COLLAR_FOOTER).icon_url(bot_pfp))
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

pub async fn send_verify_user_notif_dm(
    ctx: CollarAppContext<'_>,
    user: User,
) -> Result<(), CollarError> {
    let data = ctx.data();
    let bot_id = ctx.data().bot_id;
    let bot_pfp = ctx.cache().user(bot_id).unwrap().avatar_url().unwrap(); // if this fails to unwrap, i'll buy myself a beer

    let discord_id = user.discord_id;

    let channel_id = data.notif_channel_ids.lock().await;
    let channel_id = match channel_id.verify_id {
        Some(channel_id) => channel_id,
        None => {
            return Ok(());
        }
    };

    let verify_channel = match ctx.http().get_channel(channel_id.into()).await {
        Ok(channel) => channel,
        Err(_) => {
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

    let formatted_created_at_timestamp = FormattedTimestamp::new(
        Timestamp::from(chrono::DateTime::parse_from_rfc3339(&user.created_at).unwrap()),
        Some(FormattedTimestampStyle::LongDateTime),
    )
    .to_string();

    let formatted_verified_at_timestamp = FormattedTimestamp::new(
        Timestamp::from(chrono::DateTime::parse_from_rfc3339(&user.verified_at)?),
        Some(FormattedTimestampStyle::RelativeTime),
    )
    .to_string();

    let user_mention = ctx.http().get_user(discord_id.into()).await?.mention();
    let mut embed = CreateEmbed::default()
        .title("You've been verified :3")
        .description(format!("Hi there, {user_mention}, you've been verified :3"))
        .author(CreateEmbedAuthor::new(user.username))
        .field("Website", user.url.clone(), false)
        .field("Created", formatted_created_at_timestamp, false)
        .field("Verified", formatted_verified_at_timestamp, false)
        .footer(CreateEmbedFooter::new(COLLAR_FOOTER).icon_url(bot_pfp))
        .color(Color::from_rgb(0, 255, 0));

    if !user_pfp.is_empty() {
        embed = embed.thumbnail(user_pfp);
    }

    let message = CreateMessage::new().embed(embed);

    info!("Getting user dm channel");
    let discord_user = match ctx.http().get_user(discord_id.into()).await {
        Ok(user) => user,
        Err(_) => {
            panic!("Failed to get user");
        }
    };

    match discord_user
        .direct_message(&ctx.http(), message.clone())
        .await
    {
        Ok(_) => {
            info!("Sent dm");
            return Ok(());
        }
        Err(err) => {
            error!("Failed to send dm: {err}");
        }
    }

    verify_channel
        .id()
        .send_message(&ctx.http(), message)
        .await?;

    Ok(())
}

pub async fn send_verify_ad_notif_dm(ctx: CollarAppContext<'_>, ad: Ad) -> Result<(), CollarError> {
    let data = ctx.data();
    let bot_id = ctx.data().bot_id;
    let bot_pfp = ctx.cache().user(bot_id).unwrap().avatar_url().unwrap(); // if this fails to unwrap, i'll buy myself a beer

    let discord_id = ad.discord_id;

    let channel_id = data.notif_channel_ids.lock().await;
    let channel_id = match channel_id.verify_id {
        Some(channel_id) => channel_id,
        None => {
            return Ok(());
        }
    };

    let verify_channel = match ctx.http().get_channel(channel_id.into()).await {
        Ok(channel) => channel,
        Err(_) => {
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

    let formatted_created_at_timestamp = FormattedTimestamp::new(
        Timestamp::from(chrono::DateTime::parse_from_rfc3339(&ad.created_at).unwrap()),
        Some(FormattedTimestampStyle::LongDateTime),
    )
    .to_string();

    let formatted_verified_at_timestamp = FormattedTimestamp::new(
        Timestamp::from(chrono::DateTime::parse_from_rfc3339(&ad.verified_at).unwrap()),
        Some(FormattedTimestampStyle::RelativeTime),
    )
    .to_string();

    let user_mention = ctx.http().get_user(discord_id.into()).await?.mention();
    let embed = serenity::CreateEmbed::default()
        .title("Hii, your ad was verified :3")
        .description(format!("Hi there, {user_mention}, your ad was verified :3"))
        .author(CreateEmbedAuthor::new(ad.username))
        .url(ad.ad_url.clone())
        .thumbnail(user_pfp)
        .field("Created", formatted_created_at_timestamp, false)
        .field("Verified", formatted_verified_at_timestamp, false)
        .footer(CreateEmbedFooter::new(COLLAR_FOOTER).icon_url(bot_pfp))
        .color(Color::from_rgb(0, 255, 0));

    let message = CreateMessage::new().embed(embed);

    let discord_user = match ctx.http().get_user(discord_id.into()).await {
        Ok(user) => user,
        Err(_) => {
            panic!("Failed to get user");
        }
    };

    match discord_user
        .direct_message(&ctx.http(), message.clone())
        .await
    {
        Ok(_) => {
            info!("Sent dm");
            return Ok(());
        }
        Err(err) => {
            error!("Failed to send dm: {err}");
        }
    }

    verify_channel
        .id()
        .send_message(&ctx.http(), message)
        .await?;

    Ok(())
}

pub async fn send_delete_user_notif(
    ctx: CollarAppContext<'_>,
    user: User,
) -> Result<(), CollarError> {
    let bot_id = ctx.data().bot_id;
    let bot_pfp = ctx.cache().user(bot_id).unwrap().avatar_url().unwrap(); // if this fails to unwrap, i'll buy myself a beer
    let channel_id = ctx.data().notif_channel_ids.lock().await;
    let channel_id = match channel_id.general_id {
        Some(channel_id) => channel_id,
        None => {
            return Ok(());
        }
    };

    let formatted_created_at_timestamp = FormattedTimestamp::new(
        Timestamp::from(chrono::DateTime::parse_from_rfc3339(&user.created_at).unwrap()),
        Some(FormattedTimestampStyle::LongDateTime),
    )
    .to_string();

    let user_mention = ctx.http().get_user(user.discord_id.into()).await?.mention();
    let mut embed = CreateEmbed::default()
        .title("User deleted 3:")
        .description(format!(
            "{user_mention}, also known as {} got their spot deleted in the petring 3':",
            user.username
        ))
        .author(CreateEmbedAuthor::new(user.username))
        .field("Website", user.url.clone(), false)
        .field("Created", formatted_created_at_timestamp, false)
        .footer(CreateEmbedFooter::new(COLLAR_FOOTER).icon_url(bot_pfp))
        .color(Color::from_rgb(255, 0, 0));

    if user.verified {
        let formatted_verified_at_timestamp = FormattedTimestamp::new(
            Timestamp::from(chrono::DateTime::parse_from_rfc3339(&user.verified_at).unwrap()),
            Some(FormattedTimestampStyle::RelativeTime),
        )
        .to_string();
        embed = embed.field("Verified", formatted_verified_at_timestamp, false);
    }

    if !user.edited_at.is_empty() {
        let formatted_edited_at_timestamp = FormattedTimestamp::new(
            Timestamp::from(chrono::DateTime::parse_from_rfc3339(&user.edited_at).unwrap()),
            Some(FormattedTimestampStyle::RelativeTime),
        )
        .to_string();
        embed = embed.field("Edited", formatted_edited_at_timestamp, false);
    }

    let message = CreateMessage::new().embed(embed);

    let channel = match ctx.http().get_channel(channel_id.into()).await {
        Ok(channel) => channel,
        Err(_) => {
            return Ok(());
        }
    };

    channel.id().send_message(&ctx.http(), message).await?;

    Ok(())
}

pub async fn send_edit_user_notif(
    ctx: CollarAppContext<'_>,
    user: EditedUser,
) -> Result<(), CollarError> {
    let data = ctx.data();
    let bot_id = ctx.data().bot_id;
    let bot_pfp = ctx.cache().user(bot_id).unwrap().avatar_url().unwrap(); // if this fails to unwrap, i'll buy myself a beer

    let discord_id = user.new.discord_id;

    let channel_id = data.notif_channel_ids.lock().await;
    let channel_id = match channel_id.general_id {
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

    let user_pfp = match ctx.http().get_user(discord_id.into()).await {
        Ok(user) => user.avatar_url().unwrap(),
        Err(_) => {
            error!("Failed to get user pfp");
            "".to_string()
        }
    };

    let formatted_created_at_timestamp = FormattedTimestamp::new(
        Timestamp::from(chrono::DateTime::parse_from_rfc3339(&user.new.created_at).unwrap()),
        Some(FormattedTimestampStyle::LongDateTime),
    )
    .to_string();

    let formatted_verified_at_timestamp = FormattedTimestamp::new(
        Timestamp::from(chrono::DateTime::parse_from_rfc3339(&user.new.verified_at).unwrap()),
        Some(FormattedTimestampStyle::RelativeTime),
    )
    .to_string();

    let formatted_edited_at_timestamp = FormattedTimestamp::new(
        Timestamp::from(chrono::DateTime::parse_from_rfc3339(&user.new.edited_at).unwrap()),
        Some(FormattedTimestampStyle::RelativeTime),
    )
    .to_string();

    let user_mention = ctx.http().get_user(discord_id.into()).await?.mention();
    let mut embed = CreateEmbed::default()
        .title("User edited :3")
        .description(format!(
            "{user_mention} has edited their spot in the petring :3"
        ))
        .thumbnail(user_pfp)
        .field("Created", formatted_created_at_timestamp, false)
        .field("Verified", formatted_verified_at_timestamp, false)
        .field("Edited", formatted_edited_at_timestamp, false)
        .footer(CreateEmbedFooter::new(COLLAR_FOOTER).icon_url(bot_pfp))
        .color(Color::from_rgb(0, 255, 0));
    if user.new.username != user.old.username {
        embed = embed.author(CreateEmbedAuthor::new(format!(
            "{} → {}",
            user.old.username, user.new.username
        )));
    }

    if user.new.url != user.old.url {
        embed = embed.field(
            "Website",
            format!("{} → {}", user.old.url, user.new.url),
            false,
        );
    }

    let message = CreateMessage::new().embed(embed);

    channel.id().send_message(&ctx.http(), message).await?;

    Ok(())
}

pub async fn send_edit_ad_notif(ctx: CollarAppContext<'_>, ad: Ad) -> Result<(), CollarError> {
    let data = ctx.data();
    let bot_id = ctx.data().bot_id;
    let bot_pfp = ctx.cache().user(bot_id).unwrap().avatar_url().unwrap(); // if this fails to unwrap, i'll buy myself a beer

    let discord_id = ad.discord_id;

    let channel_id = data.notif_channel_ids.lock().await;
    let channel_id = match channel_id.general_id {
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

    let user_pfp = match ctx.http().get_user(discord_id.into()).await {
        Ok(user) => user.avatar_url().unwrap(),
        Err(_) => {
            error!("Failed to get user pfp");
            "".to_string()
        }
    };

    let formatted_created_at_timestamp = FormattedTimestamp::new(
        Timestamp::from(chrono::DateTime::parse_from_rfc3339(&ad.created_at).unwrap()),
        Some(FormattedTimestampStyle::LongDateTime),
    )
    .to_string();

    let formatted_edited_at_timestamp = FormattedTimestamp::new(
        Timestamp::from(chrono::DateTime::parse_from_rfc3339(&ad.edited_at).unwrap()),
        Some(FormattedTimestampStyle::RelativeTime),
    )
    .to_string();

    let formatted_verified_at_timestamp = FormattedTimestamp::new(
        Timestamp::from(chrono::DateTime::parse_from_rfc3339(&ad.verified_at).unwrap()),
        Some(FormattedTimestampStyle::LongDateTime),
    )
    .to_string();

    let ad_mention = ctx.http().get_user(discord_id.into()).await?.mention();
    let embed = CreateEmbed::default()
        .title("Ad edited :3")
        .description(format!("{ad_mention} has edited their ad in PetAds :3"))
        .field("Created", formatted_created_at_timestamp, false)
        .field("Verified", formatted_verified_at_timestamp, false)
        .field("Edited", formatted_edited_at_timestamp, false)
        .thumbnail(user_pfp)
        .image(ad.image_url)
        .footer(CreateEmbedFooter::new(COLLAR_FOOTER).icon_url(bot_pfp))
        .color(Color::from_rgb(0, 255, 0));

    let message = CreateMessage::new().embed(embed);

    channel.id().send_message(&ctx.http(), message).await?;

    Ok(())
}

pub async fn send_delete_ad_notif(ctx: CollarAppContext<'_>, ad: Ad) -> Result<(), CollarError> {
    let bot_id = ctx.data().bot_id;
    let bot_pfp = ctx.cache().user(bot_id).unwrap().avatar_url().unwrap(); // if this fails to unwrap, i'll buy myself a beer
    let channel_id = ctx.data().notif_channel_ids.lock().await;
    let channel_id = match channel_id.general_id {
        Some(channel_id) => channel_id,
        None => {
            return Ok(());
        }
    };

    let formatted_created_at_timestamp = FormattedTimestamp::new(
        Timestamp::from(chrono::DateTime::parse_from_rfc3339(&ad.created_at).unwrap()),
        Some(FormattedTimestampStyle::LongDateTime),
    )
    .to_string();

    let ad_mention = ctx.http().get_user(ad.discord_id.into()).await?.mention();
    let embed = CreateEmbed::default()
        .title("Ad deleted 3:")
        .description(format!(
            "{ad_mention}, also known as {} got their ad deleted in PetAds 3':",
            ad.username
        ))
        .author(CreateEmbedAuthor::new(ad.username))
        .field("Website", ad.ad_url.clone(), false)
        .field("Verified", ad.verified.to_string(), false)
        .field("Created", formatted_created_at_timestamp, false)
        .footer(CreateEmbedFooter::new(COLLAR_FOOTER).icon_url(bot_pfp))
        .color(Color::from_rgb(255, 0, 0));

    let message = CreateMessage::new().embed(embed);

    let channel = match ctx.http().get_channel(channel_id.into()).await {
        Ok(channel) => channel,
        Err(_) => {
            return Ok(());
        }
    };

    channel.id().send_message(&ctx.http(), message).await?;

    Ok(())
}
