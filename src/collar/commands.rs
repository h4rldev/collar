use super::{CollarAppContext, CollarContext, CollarError};
use crate::collar::http::{self, ErrorResponse, make_request};
use chrono::{DateTime, TimeZone, Utc};
use poise::{
    CreateReply, Modal, command,
    serenity_prelude::{self as serenity, FormattedTimestamp, Timestamp},
};
use reqwest::Method;
use serde::Serialize;
use serenity::{Color, CreateEmbedAuthor, CreateEmbedFooter};
use tracing::info;

#[derive(serde::Deserialize, Debug)]
struct User {
    username: String,
    discord_id: i64,
    url: String,
    verified: bool,
    created_at: String,
    edited_at: String,
    verified_at: String,
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
    let url = format!("/api/get/user/by-discord/{}", user_id);

    let bot_id = ctx.data().bot_id;
    let bot_pfp = ctx.cache().user(bot_id).unwrap().avatar_url().unwrap(); // if this fails to unwrap, i'll buy myself a beer

    let response = make_request(data.clone(), None::<String>, &url, Method::GET).await?;
    match response {
        http::ResponseTypes::Success(user) => {
            let user: User = user;

            let username = match ctx.http().get_user(user_id).await {
                Ok(user) => user.name.clone(),
                Err(_) => {
                    return Err("User not found".into());
                }
            };

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
                .title(user.username)
                .author(CreateEmbedAuthor::new(username))
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
    let url = format!("/api/get/user/by-discord/{}", user_id);

    let bot_id = ctx.data().bot_id;
    let bot_pfp = ctx.cache().user(bot_id).unwrap().avatar_url().unwrap(); // if this fails to unwrap, i'll buy myself a beer

    let response = make_request(data.clone(), None::<String>, &url, Method::GET).await?;
    match response {
        http::ResponseTypes::Success(user) => {
            let user: User = user;

            let username = match ctx.http().get_user(user_id).await {
                Ok(user) => user.name.clone(),
                Err(_) => {
                    return Err("User not found".into());
                }
            };

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
                .title(user.username)
                .author(CreateEmbedAuthor::new(username))
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
            let embed = serenity::CreateEmbed::default()
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
                .title("Your submission was successful")
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
                .color(Color::from_rgb(0, 0, 255));

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
