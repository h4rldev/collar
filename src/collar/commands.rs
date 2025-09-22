use std::io::Write;

use super::{CollarAppContext, CollarContext, CollarError};
use crate::collar::{
    NotifChannel,
    http::{self, ErrorResponse, make_request},
    notifs::{DeleteUserResponse, send_submit_notif},
};
use chrono::DateTime;
use poise::{CreateReply, Modal, command, serenity_prelude as serenity};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serenity::{
    Color, CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter, FormattedTimestamp,
    FormattedTimestampStyle, Timestamp,
};
use tracing::error;
#[allow(unused_imports)]
use tracing::info;

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct User {
    pub username: String,
    pub discord_id: i64,
    pub url: String,
    pub verified: bool,
    pub created_at: String,
    pub edited_at: String,
    pub verified_at: String,
}

#[command(
    slash_command,
    description_localized(
        locale = "en-US",
        description = "See your petring account information, as a verified user"
    ),
    description_localized(
        locale = "sv-SE",
        description = "Se din petrings kontoinformation, som en verifierad användare"
    ),
    name_localized(locale = "en-US", name = "me"),
    name_localized(locale = "sv-SE", name = "mig"),
    category = "petring"
)]
pub async fn me(ctx: CollarContext<'_>) -> Result<(), CollarError> {
    let data = ctx.data();
    let user_id = ctx.author().id;
    let base_url = data.api_base_url.clone();

    let bot_id = ctx.data().bot_id;
    let bot_pfp = ctx.cache().user(bot_id).unwrap().avatar_url().unwrap(); // if this fails to unwrap, i'll buy myself a beer

    let response = make_request(
        data.clone(),
        None::<String>,
        &format!("/api/get/user/by-discord/{}", user_id),
        Method::GET,
    )
    .await?;
    match response {
        http::ResponseTypes::Success(user) => {
            let user: User = user;

            let avatar_url = match ctx.http().get_user(user_id).await {
                Ok(user) => user.avatar_url().unwrap(),
                Err(_) => {
                    return Err("User not found".into());
                }
            };

            let created_at_timestamp =
                Timestamp::from(DateTime::parse_from_rfc3339(&user.created_at)?);

            let formatted_created_at_timestamp = FormattedTimestamp::new(
                created_at_timestamp,
                Some(FormattedTimestampStyle::LongDateTime),
            )
            .to_string();

            let mut formatted_edited_at_timestamp = "Never".to_string();
            let mut formatted_verified_at_timestamp = "Never".to_string();

            if !user.verified_at.is_empty() {
                let verified_at_timestamp =
                    Timestamp::from(DateTime::parse_from_rfc3339(&user.verified_at).unwrap());
                formatted_verified_at_timestamp = FormattedTimestamp::new(
                    verified_at_timestamp,
                    Some(FormattedTimestampStyle::LongDateTime),
                )
                .to_string();
            }

            if !user.edited_at.is_empty() {
                let edited_at_timestamp =
                    Timestamp::from(DateTime::parse_from_rfc3339(&user.edited_at).unwrap());
                formatted_edited_at_timestamp = FormattedTimestamp::new(
                    edited_at_timestamp,
                    Some(FormattedTimestampStyle::RelativeTime),
                )
                .to_string();
            }

            let user_url = format!("{base_url}/user/{}", user.username);

            let embed = serenity::CreateEmbed::default()
                .title("Your information :3")
                .author(
                    CreateEmbedAuthor::new(format!("{} (press here to visit)", user.username))
                        .url(user_url),
                )
                .thumbnail(avatar_url)
                .field("User Website", user.url, false)
                .field("Created at", formatted_created_at_timestamp, false)
                .field("Edited at", formatted_edited_at_timestamp, false)
                .field("Verified at", formatted_verified_at_timestamp, false)
                .footer(
                    CreateEmbedFooter::new(
                        "Collar :3, a Discord bot helper for petring and petads :3",
                    )
                    .icon_url(bot_pfp),
                )
                .color(serenity::Color::from_rgb(0, 0, 255));

            let reply = CreateReply::default().embed(embed).reply(true);
            ctx.send(reply).await?;
        }
        http::ResponseTypes::Error(error) => {
            let error: ErrorResponse = error;
            let embed = serenity::CreateEmbed::default()
                .title(format!("Error {}", error.status))
                .description(error.message)
                .footer(
                    CreateEmbedFooter::new(
                        "Collar :3, a Discord bot helper for petring and petads :3",
                    )
                    .icon_url(bot_pfp),
                )
                .color(Color::from_rgb(255, 0, 0));
            let reply = CreateReply::default().embed(embed).reply(true);
            ctx.send(reply).await?;
        }
    }

    Ok(())
}

#[command(
    slash_command,
    description_localized(
        locale = "en-US",
        description = "Get account information for a verified petring user"
    ),
    description_localized(
        locale = "sv-SE",
        description = "Hämta kontoinformation för en verifierad petring användare"
    ),
    name_localized(locale = "en-US", name = "get"),
    name_localized(locale = "sv-SE", name = "hämta"),
    category = "petring"
)]
pub async fn get(ctx: CollarContext<'_>, user: serenity::User) -> Result<(), CollarError> {
    let data = ctx.data();
    let user_id = user.id;
    let base_url = data.api_base_url.clone();

    let bot_id = ctx.data().bot_id;
    let bot_pfp = ctx.cache().user(bot_id).unwrap().avatar_url().unwrap(); // if this fails to unwrap, i'll buy myself a beer

    let response = make_request(
        data.clone(),
        None::<String>,
        &format!("/api/get/user/by-discord/{}", user_id),
        Method::GET,
    )
    .await?;
    match response {
        http::ResponseTypes::Success(user) => {
            let user: User = user;

            let avatar_url = match ctx.http().get_user(user_id).await {
                Ok(user) => user.avatar_url().unwrap(),
                Err(_) => {
                    return Err("User not found".into());
                }
            };

            let created_at_timestamp =
                Timestamp::from(DateTime::parse_from_rfc3339(&user.created_at)?);

            let formatted_created_at_timestamp = FormattedTimestamp::new(
                created_at_timestamp,
                Some(FormattedTimestampStyle::LongDateTime),
            )
            .to_string();

            let mut formatted_edited_at_timestamp = "Never".to_string();
            let mut formatted_verified_at_timestamp = "Never".to_string();

            if !user.verified_at.is_empty() {
                let verified_at_timestamp =
                    Timestamp::from(DateTime::parse_from_rfc3339(&user.verified_at)?);
                formatted_verified_at_timestamp = FormattedTimestamp::new(
                    verified_at_timestamp,
                    Some(FormattedTimestampStyle::LongDateTime),
                )
                .to_string();
            }

            if !user.edited_at.is_empty() {
                let edited_at_timestamp =
                    Timestamp::from(DateTime::parse_from_rfc3339(&user.edited_at)?);
                formatted_edited_at_timestamp = FormattedTimestamp::new(
                    edited_at_timestamp,
                    Some(FormattedTimestampStyle::RelativeTime),
                )
                .to_string();
            }

            let user_url = format!("{base_url}/user/{}", user.username);

            let embed = serenity::CreateEmbed::default()
                .title(format!("Info for {} :3c", user.username))
                .author(
                    CreateEmbedAuthor::new(format!("{} (press here to visit)", user.username))
                        .url(user_url),
                )
                .thumbnail(avatar_url)
                .field("User Website", user.url, false)
                .field("Created at", formatted_created_at_timestamp, false)
                .field("Edited at", formatted_edited_at_timestamp, false)
                .field("Verified at", formatted_verified_at_timestamp, false)
                .footer(
                    CreateEmbedFooter::new(
                        "Collar :3, a Discord bot helper for petring and petads :3",
                    )
                    .icon_url(bot_pfp),
                )
                .color(serenity::Color::from_rgb(0, 0, 255));

            let reply = CreateReply::default().embed(embed).reply(true);
            ctx.send(reply).await?;
        }
        http::ResponseTypes::Error(error) => {
            let error: ErrorResponse = error;
            let embed = CreateEmbed::default()
                .title(format!("Error {}", error.status))
                .description(error.message)
                .footer(
                    CreateEmbedFooter::new(
                        "Collar :3, a Discord bot helper for petring and petads :3",
                    )
                    .icon_url(bot_pfp),
                )
                .color(Color::from_rgb(255, 0, 0));
            let reply = CreateReply::default().embed(embed).reply(true);
            ctx.send(reply).await?;
        }
    }

    Ok(())
}

#[derive(Debug, Modal)]
#[name = "Add your website to petring :3"] // Struct name by default
struct AddWebsite {
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
    reason: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct UserSubmission {
    pub username: String,
    pub url: String,
    pub discord_id: i64,
}

#[command(
    slash_command,
    description_localized(locale = "en-US", description = "Submit a website to petring"),
    description_localized(locale = "sv-SE", description = "Skicka en webbplats till petring"),
    name_localized(locale = "en-US", name = "submit"),
    name_localized(locale = "sv-SE", name = "skicka"),
    category = "petring"
)]
pub async fn submit(ctx: CollarAppContext<'_>) -> Result<(), CollarError> {
    let bot_id = ctx.data().bot_id;
    let bot_pfp = ctx.cache().user(bot_id).unwrap().avatar_url().unwrap(); // if this fails to unwrap, i'll buy myself a beer

    let user_id = ctx.author().id;

    let modal_data = AddWebsite::execute(ctx).await?;
    let modal_data = match modal_data {
        Some(modal_data) => modal_data,
        None => {
            let embed = CreateEmbed::default()
                .title("You didn't submit anything")
                .description("No data was submitted 3:")
                .color(Color::from_rgb(255, 0, 0))
                .footer(
                    CreateEmbedFooter::new(
                        "Collar :3, a Discord bot helper for petring and petads :3",
                    )
                    .icon_url(bot_pfp),
                );

            let reply = CreateReply::default().embed(embed).reply(true);
            ctx.send(reply).await?;
            return Ok(());
        }
    };

    let username = modal_data.username;
    let user_url = modal_data.url;
    let discord_id = ctx.author().id;

    let response = make_request(
        ctx.data().clone(),
        Some(UserSubmission {
            username,
            url: user_url,
            discord_id: discord_id.into(),
        }),
        "/api/post/user/submit",
        Method::POST,
    )
    .await?;
    match response {
        http::ResponseTypes::Success(user) => {
            let user: User = user;

            let avatar_url = match ctx.http().get_user(user_id).await {
                Ok(user) => user.avatar_url().unwrap(),
                Err(_) => {
                    return Err("User not found".into());
                }
            };

            let created_at_timestamp =
                Timestamp::from(DateTime::parse_from_rfc3339(&user.created_at).unwrap());

            let formatted_created_at_timestamp = FormattedTimestamp::new(
                created_at_timestamp,
                Some(FormattedTimestampStyle::LongDateTime),
            )
            .to_string();

            let embed = CreateEmbed::default()
                .title("Your submission was successful! :3")
                .author(CreateEmbedAuthor::new(user.username))
                .thumbnail(avatar_url)
                .field("User Website", user.url, false)
                .field(
                    "Verification",
                    "You're not verified yet, but we'll let you know when you are :3",
                    false,
                )
                .field("Created at", formatted_created_at_timestamp, false)
                .footer(
                    CreateEmbedFooter::new(
                        "Collar :3, a Discord bot helper for petring and petads :3",
                    )
                    .icon_url(bot_pfp),
                )
                .color(Color::from_rgb(0, 0, 255));

            let reply = CreateReply::default().embed(embed).reply(true);
            ctx.send(reply).await?;
        }
        http::ResponseTypes::Error(error) => {
            let error: ErrorResponse = error;
            let embed = CreateEmbed::default()
                .title(format!("Error {}", error.status))
                .description(error.message)
                .footer(
                    CreateEmbedFooter::new(
                        "Collar :3, a Discord bot helper for petring and petads :3",
                    )
                    .icon_url(bot_pfp),
                )
                .color(Color::from_rgb(255, 0, 0));
            let reply = CreateReply::default().embed(embed).reply(true);
            ctx.send(reply).await?;
            return Ok(());
        }
    }

    send_submit_notif(ctx, discord_id.get()).await?;
    Ok(())
}

#[derive(Debug, Modal)]
#[name = "Edit your petring account information"] // Struct name by default
struct EditSubmission {
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
    pub discord_id: i64,
    pub username: Option<String>,
    pub url: Option<String>,
}

#[command(
    slash_command,
    description_localized(
        locale = "en-US",
        description = "Edit your petring account information, as a verified user"
    ),
    description_localized(
        locale = "sv-SE",
        description = "Redigera ditt petrings kontoinformation, som en verifierad användare"
    ),
    name_localized(locale = "en-US", name = "edit"),
    name_localized(locale = "sv-SE", name = "redigera"),
    category = "petring"
)]
pub async fn edit(ctx: CollarAppContext<'_>) -> Result<(), CollarError> {
    let bot_id = ctx.data().bot_id;
    let bot_pfp = ctx.cache().user(bot_id).unwrap().avatar_url().unwrap(); // if this fails to unwrap, i'll buy myself a beer

    let modal_data = EditSubmission::execute(ctx).await?;
    let modal_data = match modal_data {
        Some(modal_data) => modal_data,
        None => {
            let embed = CreateEmbed::default()
                .title("You didn't edit anything")
                .description("No data was submitted 3:")
                .color(Color::from_rgb(255, 0, 0))
                .footer(
                    CreateEmbedFooter::new(
                        "Collar :3, a Discord bot helper for petring and petads :3",
                    )
                    .icon_url(bot_pfp),
                );

            let reply = CreateReply::default().embed(embed).reply(true);
            ctx.send(reply).await?;
            return Ok(());
        }
    };

    let username = modal_data.username;
    let user_url = modal_data.url;

    let data = ctx.data();
    let user_id = ctx.author().id;
    let base_url = data.api_base_url.clone();

    let response = make_request(
        data.clone(),
        Some(UserEditSubmission {
            username,
            url: user_url,
            discord_id: user_id.into(),
        }),
        "/api/put/user/edit/",
        Method::PUT,
    )
    .await?;
    match response {
        http::ResponseTypes::Success(user) => {
            let user: User = user;

            let avatar_url = match ctx.http().get_user(user_id).await {
                Ok(user) => user.avatar_url().unwrap(),
                Err(_) => {
                    return Err("User not found".into());
                }
            };

            let created_at_timestamp =
                Timestamp::from(DateTime::parse_from_rfc3339(&user.created_at)?);

            let formatted_created_at_timestamp = FormattedTimestamp::new(
                created_at_timestamp,
                Some(FormattedTimestampStyle::LongDateTime),
            )
            .to_string();

            let mut formatted_edited_at_timestamp = "Never".to_string();
            let mut formatted_verified_at_timestamp = "Never".to_string();

            if !user.verified_at.is_empty() {
                let verified_at_timestamp =
                    Timestamp::from(DateTime::parse_from_rfc3339(&user.verified_at)?);
                formatted_verified_at_timestamp = FormattedTimestamp::new(
                    verified_at_timestamp,
                    Some(FormattedTimestampStyle::LongDateTime),
                )
                .to_string();
            }

            if !user.edited_at.is_empty() {
                let edited_at_timestamp =
                    Timestamp::from(DateTime::parse_from_rfc3339(&user.edited_at)?);
                formatted_edited_at_timestamp = FormattedTimestamp::new(
                    edited_at_timestamp,
                    Some(FormattedTimestampStyle::RelativeTime),
                )
                .to_string();
            }

            let user_url = format!("{base_url}/user/{}", user.username);

            let embed = serenity::CreateEmbed::default()
                .title("Your edit was successful! :3")
                .author(CreateEmbedAuthor::new(user.username).url(user_url))
                .thumbnail(avatar_url)
                .field("User Website", user.url, false)
                .field("Created at", formatted_created_at_timestamp, false)
                .field("Edited at", formatted_edited_at_timestamp, false)
                .field("Verified at", formatted_verified_at_timestamp, false)
                .footer(
                    CreateEmbedFooter::new(
                        "Collar :3, a Discord bot helper for petring and petads :3",
                    )
                    .icon_url(bot_pfp),
                )
                .color(Color::from_rgb(0, 255, 0));

            let reply = CreateReply::default().embed(embed).reply(true);
            ctx.send(reply).await?;
        }
        http::ResponseTypes::Error(error) => {
            let error: ErrorResponse = error;
            let embed = serenity::CreateEmbed::default()
                .title(format!("Error {}", error.status))
                .description(error.message)
                .footer(
                    CreateEmbedFooter::new(
                        "Collar :3, a Discord bot helper for petring and petads :3",
                    )
                    .icon_url(bot_pfp),
                )
                .color(Color::from_rgb(255, 0, 0));
            let reply = CreateReply::default().embed(embed).reply(true);
            ctx.send(reply).await?;
        }
    }

    Ok(())
}

#[command(
    slash_command,
    description_localized(
        locale = "en-US",
        description = "Set the notification channel for when someone submits a website"
    ),
    description_localized(
        locale = "sv-SE",
        description = "Anger en notifieringskanal när någon skickar in en webbplats"
    ),
    name_localized(locale = "en-US", name = "set_notification_channel"),
    name_localized(locale = "sv-SE", name = "ställ_in_notifieringskanal"),
    category = "notifications",
    required_permissions = "MANAGE_CHANNELS | BAN_MEMBERS | KICK_MEMBERS | MUTE_MEMBERS"
)]
pub async fn set_notif_channel(
    ctx: CollarContext<'_>,
    channel: serenity::Channel,
) -> Result<(), CollarError> {
    let data = ctx.data();

    let bot_id = ctx.data().bot_id;
    let bot_pfp = ctx.cache().user(bot_id).unwrap().avatar_url().unwrap(); // if this fails to unwrap, i'll buy myself a beer

    match channel.clone().guild() {
        Some(_) => {}
        None => {
            let embed = serenity::CreateEmbed::default()
                .title("You can't set a notification channel in a DM :3")
                .description("You need to be in a server to set a notification channel :3")
                .color(Color::from_rgb(255, 0, 0));
            let reply = CreateReply::default().embed(embed).reply(true);
            ctx.send(reply).await?;
            return Ok(());
        }
    }

    let mut notif_channel_id = data.notif_channel_id.lock().await;
    *notif_channel_id = Some(channel.id().into());

    let mut file_to_write = match std::fs::File::create(".notif_channel_id.json") {
        Ok(file) => file,
        Err(err) => {
            error!("Failed to cache notif_channel_id: {err}");
            return Err(CollarError::from(format!(
                "Failed to create .notif_channel_id.json: {err}"
            )));
        }
    };

    let notif_channel_id = match serde_json::to_string(&NotifChannel {
        id: channel.id().into(),
    }) {
        Ok(notif_channel_id) => notif_channel_id,
        Err(err) => {
            error!("Failed to serialize notif_channel_id: {err}");
            return Err(CollarError::from(format!(
                "Failed to serialize .notif_channel_id.json: {err}"
            )));
        }
    };

    match file_to_write.write_all(notif_channel_id.as_bytes()) {
        Ok(_) => (),
        Err(err) => {
            error!("Failed to write notif_channel_id: {err}");
            return Err(CollarError::from(format!(
                "Failed to write .notif_channel_id.json: {err}"
            )));
        }
    }

    let embed = serenity::CreateEmbed::default()
        .title("Notification channel set!")
        .description(format!(
            "Expect a notification when someone submits a website in {}",
            channel
        ))
        .footer(
            CreateEmbedFooter::new("Collar :3, a Discord bot helper for petring and petads :3")
                .icon_url(bot_pfp),
        )
        .color(Color::from_rgb(0, 255, 0));

    let reply = CreateReply::default()
        .embed(embed)
        .reply(true)
        .ephemeral(true);

    ctx.send(reply).await?;

    Ok(())
}

#[command(
    slash_command,
    description_localized(
        locale = "en-US",
        description = "Get the channel that notifications are sent to"
    ),
    description_localized(locale = "sv-SE", description = "Hämta notifieringskanalen"),
    name_localized(locale = "en-US", name = "get_notification_channel"),
    name_localized(locale = "sv-SE", name = "hämta_notifieringskanal"),
    required_permissions = "MANAGE_CHANNELS | BAN_MEMBERS | KICK_MEMBERS | MUTE_MEMBERS",
    category = "notifications"
)]
pub async fn get_notif_channel(ctx: CollarContext<'_>) -> Result<(), CollarError> {
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

    let embed = serenity::CreateEmbed::default()
        .title("Notification channel")
        .description(format!("The notification channel is {}", channel))
        .footer(
            CreateEmbedFooter::new("Collar :3, a Discord bot helper for petring and petads :3")
                .icon_url(bot_pfp),
        )
        .color(serenity::Color::from_rgb(0, 0, 255));

    let reply = CreateReply::default()
        .embed(embed)
        .reply(true)
        .ephemeral(true);

    ctx.send(reply).await?;

    Ok(())
}
#[command(
    slash_command,
    description_localized(locale = "en-US", description = "Verify a submitted petring website"),
    description_localized(
        locale = "sv-SE",
        description = "Verifiera en skickad petring webbplats"
    ),
    name_localized(locale = "en-US", name = "verify"),
    name_localized(locale = "sv-SE", name = "verifiera"),
    category = "petring",
    required_permissions = "MANAGE_CHANNELS | BAN_MEMBERS | KICK_MEMBERS | MUTE_MEMBERS"
)]
pub async fn verify(ctx: CollarContext<'_>, user: serenity::User) -> Result<(), CollarError> {
    let user_id = user.id;
    let url = format!("/api/put/user/verify/{}", user_id);

    let bot_id = ctx.data().bot_id;
    let bot_pfp = ctx.cache().user(bot_id).unwrap().avatar_url().unwrap(); // if this fails to unwrap, i'll buy myself a beer

    let response = make_request(ctx.data().clone(), None::<String>, &url, Method::PUT).await?;
    match response {
        http::ResponseTypes::Success(user) => {
            let user: User = user;

            let avatar_url = match ctx.http().get_user(user_id).await {
                Ok(user) => user.avatar_url().unwrap(),
                Err(_) => {
                    return Err("User not found".into());
                }
            };

            let created_at_timestamp = FormattedTimestamp::from(Timestamp::from(
                DateTime::parse_from_rfc3339(&user.created_at).unwrap(),
            ))
            .to_string();

            let mut edited_at_timestamp = "Never".to_string();
            let mut verified_at_timestamp = "Never".to_string();

            if !user.verified_at.is_empty() {
                verified_at_timestamp = FormattedTimestamp::from(Timestamp::from(
                    DateTime::parse_from_rfc3339(&user.verified_at).unwrap(),
                ))
                .to_string();
            }

            if !user.edited_at.is_empty() {
                edited_at_timestamp = FormattedTimestamp::from(Timestamp::from(
                    DateTime::parse_from_rfc3339(&user.edited_at).unwrap(),
                ))
                .to_string();
            }

            let embed = serenity::CreateEmbed::default()
                .title("Your verification was successful")
                .author(CreateEmbedAuthor::new(user.username))
                .url(user.url)
                .thumbnail(avatar_url)
                .field("Created at", created_at_timestamp, false)
                .field("Edited at", edited_at_timestamp, false)
                .field("Verified at", verified_at_timestamp, false)
                .footer(
                    CreateEmbedFooter::new(
                        "Collar :3, a Discord bot helper for petring and petads :3",
                    )
                    .icon_url(bot_pfp),
                )
                .color(Color::from_rgb(0, 255, 0));

            let reply = CreateReply::default().embed(embed).reply(true);
            ctx.send(reply).await?;
        }
        http::ResponseTypes::Error(error) => {
            let error: ErrorResponse = error;
            let embed = serenity::CreateEmbed::default()
                .title(format!("Error {}", error.status))
                .description(error.message)
                .footer(
                    CreateEmbedFooter::new(
                        "Collar :3, a Discord bot helper for petring and petads :3",
                    )
                    .icon_url(bot_pfp),
                )
                .color(Color::from_rgb(255, 0, 0));
            let reply = CreateReply::default().embed(embed).reply(true);
            ctx.send(reply).await?;
        }
    }

    Ok(())
}

#[command(
    slash_command,
    description_localized(
        locale = "en-US",
        description = "Delete a petring user (doesn't matter if they're verified or not)"
    ),
    description_localized(
        locale = "sv-SE",
        description = "Radera en petring användare (spelar ingen roll om de är verifierade eller inte)"
    ),
    name_localized(locale = "en-US", name = "remove_user"),
    name_localized(locale = "sv-SE", name = "radera_användare"),
    category = "petring",
    required_permissions = "MANAGE_CHANNELS | BAN_MEMBERS | KICK_MEMBERS | MUTE_MEMBERS"
)]
pub async fn remove_user(ctx: CollarContext<'_>, user: serenity::User) -> Result<(), CollarError> {
    let user_id = user.id;
    let url = format!("/api/delete/user/by-discord/{}", user_id);

    let bot_id = ctx.data().bot_id;
    let bot_pfp = ctx.cache().user(bot_id).unwrap().avatar_url().unwrap(); // if this fails to unwrap, i'll buy myself a beer

    let response = make_request(ctx.data().clone(), None::<String>, &url, Method::DELETE).await?;
    match response {
        http::ResponseTypes::Success(response) => {
            let deleted_user: DeleteUserResponse = response;

            let embed = serenity::CreateEmbed::default()
                .title("Successfully removed user :3")
                .description(format!("{} :3", deleted_user.message))
                .footer(
                    CreateEmbedFooter::new(
                        "Collar :3, a Discord bot helper for petring and petads :3",
                    )
                    .icon_url(bot_pfp),
                )
                .color(serenity::Color::from_rgb(255, 0, 0));

            let reply = CreateReply::default()
                .embed(embed)
                .reply(true)
                .ephemeral(true);
            ctx.send(reply).await?;
        }
        http::ResponseTypes::Error(error) => {
            let error: ErrorResponse = error;
            let embed = serenity::CreateEmbed::default()
                .title(format!("Error {}", error.status))
                .description(error.message)
                .footer(
                    CreateEmbedFooter::new(
                        "Collar :3, a Discord bot helper for petring and petads :3",
                    )
                    .icon_url(bot_pfp),
                )
                .color(serenity::Color::from_rgb(255, 0, 0));
            let reply = CreateReply::default().embed(embed).reply(true);
            ctx.send(reply).await?;
        }
    }

    Ok(())
}
