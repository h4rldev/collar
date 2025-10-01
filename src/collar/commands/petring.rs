use super::{
    http::{make_request, ErrorResponse, ResponseTypes},
    notifs::{
        send_delete_user_notif, send_edit_user_notif, send_submit_user_notif,
        send_verify_user_notif_dm,
    },
    AddWebsite, CollarAppContext, CollarContext, CollarError, EditSubmission, EditedUser, User,
    UserEditSubmission, UserSubmission, COLLAR_FOOTER,
};
use chrono::DateTime;
use poise::{command, serenity_prelude as serenity, CreateReply, Modal};
use reqwest::Method;
use serenity::{
    Color, CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter, FormattedTimestamp,
    FormattedTimestampStyle, Mentionable, Timestamp,
};
use tracing::info;

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
    category = "PetRing"
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
        ResponseTypes::Success(user) => {
            let user: User = user;

            let user_id_u64: u64 = user_id.into();

            if user.discord_id != user_id_u64 {
                return Err("User not found".into());
            }

            if !user.verified {
                return Err("User not verified".into());
            }

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

            let formatted_verified_at_timestamp = FormattedTimestamp::new(
                Timestamp::from(DateTime::parse_from_rfc3339(&user.verified_at).unwrap()),
                Some(FormattedTimestampStyle::LongDateTime),
            )
            .to_string();

            if !user.edited_at.is_empty() {
                formatted_edited_at_timestamp = FormattedTimestamp::new(
                    Timestamp::from(DateTime::parse_from_rfc3339(&user.edited_at).unwrap()),
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
                .footer(CreateEmbedFooter::new(COLLAR_FOOTER).icon_url(bot_pfp))
                .color(serenity::Color::from_rgb(0, 0, 255));

            let reply = CreateReply::default().embed(embed).reply(true);
            ctx.send(reply).await?;
        }
        ResponseTypes::Error(error) => {
            let error: ErrorResponse = error;
            let embed = serenity::CreateEmbed::default()
                .title(format!("Error {}", error.status))
                .description(error.message)
                .footer(CreateEmbedFooter::new(COLLAR_FOOTER).icon_url(bot_pfp))
                .color(Color::from_rgb(255, 0, 0));
            let reply = CreateReply::default()
                .embed(embed)
                .reply(true)
                .ephemeral(true);
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
    name_localized(locale = "en-US", name = "get_user"),
    name_localized(locale = "sv-SE", name = "hämta_användare"),
    category = "PetRing"
)]
pub async fn get_user(ctx: CollarContext<'_>, user: serenity::User) -> Result<(), CollarError> {
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
        ResponseTypes::Success(user) => {
            let user: User = user;
            let user_id_u64: u64 = user_id.into();

            if user.discord_id != user_id_u64 {
                return Err("User not found".into());
            }

            if !user.verified {
                return Err("User not verified".into());
            }

            let avatar_url = match ctx.http().get_user(user_id).await {
                Ok(user) => user.avatar_url().unwrap(),
                Err(_) => {
                    return Err("User not found".into());
                }
            };

            let formatted_created_at_timestamp = FormattedTimestamp::new(
                Timestamp::from(DateTime::parse_from_rfc3339(&user.created_at)?),
                Some(FormattedTimestampStyle::LongDateTime),
            )
            .to_string();

            let mut formatted_edited_at_timestamp = "Never".to_string();

            let formatted_verified_at_timestamp = FormattedTimestamp::new(
                Timestamp::from(DateTime::parse_from_rfc3339(&user.verified_at)?),
                Some(FormattedTimestampStyle::LongDateTime),
            )
            .to_string();

            if !user.edited_at.is_empty() {
                formatted_edited_at_timestamp = FormattedTimestamp::new(
                    Timestamp::from(DateTime::parse_from_rfc3339(&user.edited_at)?),
                    Some(FormattedTimestampStyle::RelativeTime),
                )
                .to_string();
            }

            let embed = CreateEmbed::default()
                .title(format!("Info for {} :3c", user.username))
                .author(
                    CreateEmbedAuthor::new(format!("{} (press here to visit)", user.username))
                        .url(format!("{base_url}/user/{}", user.username)),
                )
                .thumbnail(avatar_url)
                .field("User Website", user.url, false)
                .field("Created", formatted_created_at_timestamp, false)
                .field("Edited", formatted_edited_at_timestamp, false)
                .field("Verified", formatted_verified_at_timestamp, false)
                .footer(CreateEmbedFooter::new(COLLAR_FOOTER).icon_url(bot_pfp))
                .color(Color::from_rgb(0, 0, 255));

            let reply = CreateReply::default().embed(embed).reply(true);
            ctx.send(reply).await?;
        }
        ResponseTypes::Error(error) => {
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
            let reply = CreateReply::default()
                .embed(embed)
                .reply(true)
                .ephemeral(true);
            ctx.send(reply).await?;
        }
    }

    Ok(())
}

#[command(
    slash_command,
    description_localized(locale = "en-US", description = "Register a user to petring"),
    description_localized(locale = "sv-SE", description = "Registrera en användare till petring"),
    name_localized(locale = "en-US", name = "submit_user"),
    name_localized(locale = "sv-SE", name = "skicka_användare"),
    category = "PetRing"
)]
pub async fn submit_user(ctx: CollarAppContext<'_>) -> Result<(), CollarError> {
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
                .footer(CreateEmbedFooter::new(COLLAR_FOOTER).icon_url(bot_pfp));

            let reply = CreateReply::default()
                .embed(embed)
                .reply(true)
                .ephemeral(true);
            ctx.send(reply).await?;
            return Ok(());
        }
    };

    let username = modal_data.username;
    let user_url = modal_data.url;
    let reason = modal_data.reason;
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
        ResponseTypes::Success(user) => {
            let user: User = user;
            let cloned_user = user.clone();
            let user_id_u64: u64 = user_id.into();

            if user_id_u64 != user.discord_id {
                return Err("User not found".into());
            }

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
                .field("Created", formatted_created_at_timestamp, false)
                .footer(CreateEmbedFooter::new(COLLAR_FOOTER).icon_url(bot_pfp))
                .color(Color::from_rgb(0, 0, 255));

            let reply = CreateReply::default()
                .embed(embed)
                .reply(true)
                .ephemeral(true);
            ctx.send(reply).await?;

            send_submit_user_notif(ctx, cloned_user, reason).await?;
        }
        ResponseTypes::Error(error) => {
            let error: ErrorResponse = error;
            let embed = CreateEmbed::default()
                .title(format!("Error {}", error.status))
                .description(error.message)
                .footer(CreateEmbedFooter::new(COLLAR_FOOTER).icon_url(bot_pfp))
                .color(Color::from_rgb(255, 0, 0));
            let reply = CreateReply::default()
                .embed(embed)
                .reply(true)
                .ephemeral(true);
            ctx.send(reply).await?;
        }
    }

    Ok(())
}

#[command(
    slash_command,
    description_localized(
        locale = "en-US",
        description = "Edit your petring account information, as a verified user"
    ),
    description_localized(
        locale = "sv-SE",
        description = "Redigera din petrings kontoinformation, som en verifierad användare"
    ),
    name_localized(locale = "en-US", name = "edit_user"),
    name_localized(locale = "sv-SE", name = "redigera_användare"),
    category = "PetRing"
)]
pub async fn edit_user(ctx: CollarAppContext<'_>) -> Result<(), CollarError> {
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
                .footer(CreateEmbedFooter::new(COLLAR_FOOTER).icon_url(bot_pfp));

            let reply = CreateReply::default()
                .embed(embed)
                .reply(true)
                .ephemeral(true);
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
        ResponseTypes::Success(user) => {
            let user: EditedUser = user;
            let user_id_u64: u64 = user_id.into();

            if user_id_u64 != user.new.discord_id {
                return Err("User not found".into());
            }

            if !user.new.verified {
                return Err("User not verified".into());
            }

            let avatar_url = match ctx.http().get_user(user_id).await {
                Ok(user) => user.avatar_url().unwrap(),
                Err(_) => {
                    return Err("User not found".into());
                }
            };

            let created_at_timestamp =
                Timestamp::from(DateTime::parse_from_rfc3339(&user.new.created_at)?);

            let formatted_created_at_timestamp = FormattedTimestamp::new(
                created_at_timestamp,
                Some(FormattedTimestampStyle::LongDateTime),
            )
            .to_string();

            let mut formatted_edited_at_timestamp = "Never".to_string();
            let mut formatted_verified_at_timestamp = "Never".to_string();

            if !user.new.verified_at.is_empty() {
                let verified_at_timestamp =
                    Timestamp::from(DateTime::parse_from_rfc3339(&user.new.verified_at)?);
                formatted_verified_at_timestamp = FormattedTimestamp::new(
                    verified_at_timestamp,
                    Some(FormattedTimestampStyle::LongDateTime),
                )
                .to_string();
            }

            if !user.new.edited_at.is_empty() {
                let edited_at_timestamp =
                    Timestamp::from(DateTime::parse_from_rfc3339(&user.new.edited_at)?);
                formatted_edited_at_timestamp = FormattedTimestamp::new(
                    edited_at_timestamp,
                    Some(FormattedTimestampStyle::RelativeTime),
                )
                .to_string();
            }

            let user_url = format!("{base_url}/user/{}", user.new.username);

            let mut embed = serenity::CreateEmbed::default()
                .title("Your edit was successful! :3")
                .thumbnail(avatar_url)
                .field("Created", formatted_created_at_timestamp, false)
                .field("Verified", formatted_verified_at_timestamp, false)
                .field("Edited", formatted_edited_at_timestamp, false)
                .footer(CreateEmbedFooter::new(COLLAR_FOOTER).icon_url(bot_pfp))
                .color(Color::from_rgb(0, 255, 0));

            if user.new.username != user.old.username {
                embed = embed.author(
                    CreateEmbedAuthor::new(format!(
                        "{} → {}",
                        user.old.username, user.new.username
                    ))
                    .url(user_url),
                );
            }

            if user.new.url != user.old.url {
                embed = embed.field(
                    "Website",
                    format!("{} → {}", user.old.url, user.new.url),
                    false,
                );
            }

            let reply = CreateReply::default()
                .embed(embed)
                .reply(true)
                .ephemeral(true);
            ctx.send(reply).await?;
            send_edit_user_notif(ctx, user).await?;
        }
        ResponseTypes::Error(error) => {
            let error: ErrorResponse = error;
            let embed = CreateEmbed::default()
                .title(format!("Error {}", error.status))
                .description(error.message)
                .footer(CreateEmbedFooter::new(COLLAR_FOOTER).icon_url(bot_pfp))
                .color(Color::from_rgb(255, 0, 0));
            let reply = CreateReply::default()
                .embed(embed)
                .reply(true)
                .ephemeral(true);
            ctx.send(reply).await?;
        }
    }

    Ok(())
}

#[command(
    slash_command,
    description_localized(locale = "en-US", description = "Verify a submitted petring user"),
    description_localized(
        locale = "sv-SE",
        description = "Verifiera en skickad petring användare"
    ),
    name_localized(locale = "en-US", name = "verify_user"),
    name_localized(locale = "sv-SE", name = "verifiera_användare"),
    category = "PetRing",
    required_permissions = "MANAGE_CHANNELS | BAN_MEMBERS | KICK_MEMBERS | MUTE_MEMBERS"
)]
pub async fn verify_user(
    ctx: CollarAppContext<'_>,
    user: serenity::User,
) -> Result<(), CollarError> {
    let user_id = user.id;
    let url = format!("/api/put/user/verify/{}", user_id);

    let bot_id = ctx.data().bot_id;
    let bot_pfp = ctx.cache().user(bot_id).unwrap().avatar_url().unwrap(); // if this fails to unwrap, i'll buy myself a beer

    let response = make_request(ctx.data().clone(), None::<String>, &url, Method::PUT).await?;
    match response {
        ResponseTypes::Success(user) => {
            let user: User = user;

            if !user.verified {
                return Err("User failed to verify".into());
            }

            let user_id_u64: u64 = user_id.into();
            if user_id_u64 != user.discord_id {
                return Err("User not found".into());
            }

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

            let verified_at_timestamp = FormattedTimestamp::from(Timestamp::from(
                DateTime::parse_from_rfc3339(&user.verified_at).unwrap(),
            ))
            .to_string();

            let embed = serenity::CreateEmbed::default()
                .title("Your verification was successful")
                .author(CreateEmbedAuthor::new(format!("for: {}", user.username)))
                .url(user.url.clone())
                .thumbnail(avatar_url)
                .field("Created", created_at_timestamp, false)
                .field("Verified", verified_at_timestamp, false)
                .footer(CreateEmbedFooter::new(COLLAR_FOOTER).icon_url(bot_pfp.clone()))
                .color(Color::from_rgb(0, 255, 0));

            let reply = CreateReply::default()
                .embed(embed)
                .reply(true)
                .ephemeral(true);
            ctx.send(reply).await?;
            info!("Sending user notif dm");
            send_verify_user_notif_dm(ctx, user).await?;
        }
        ResponseTypes::Error(error) => {
            let error: ErrorResponse = error;
            let embed = serenity::CreateEmbed::default()
                .title(format!("Error {}", error.status))
                .description(error.message)
                .footer(CreateEmbedFooter::new(COLLAR_FOOTER).icon_url(bot_pfp))
                .color(Color::from_rgb(255, 0, 0));
            let reply = CreateReply::default()
                .embed(embed)
                .reply(true)
                .ephemeral(true);
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
    category = "PetRing",
    required_permissions = "MANAGE_CHANNELS | BAN_MEMBERS | KICK_MEMBERS | MUTE_MEMBERS"
)]
pub async fn remove_user(
    ctx: CollarAppContext<'_>,
    user: serenity::User,
) -> Result<(), CollarError> {
    let user_id = user.id;
    let url = format!("/api/delete/user/by-discord/{}", user_id);

    let bot_id = ctx.data().bot_id;
    let bot_pfp = ctx.cache().user(bot_id).unwrap().avatar_url().unwrap(); // if this fails to unwrap, i'll buy myself a beer

    let response = make_request(ctx.data().clone(), None::<String>, &url, Method::DELETE).await?;
    match response {
        ResponseTypes::Success(response) => {
            let deleted_user: User = response;

            let user_mention = ctx.http().get_user(user_id).await?.mention();
            let user_pfp = ctx.http().get_user(user_id).await?.avatar_url().unwrap();

            let embed = serenity::CreateEmbed::default()
                .title("Successfully removed user :3")
                .description(format!(
                    "{user_mention} ({}): removed :3",
                    deleted_user.discord_id
                ))
                .thumbnail(user_pfp)
                .footer(CreateEmbedFooter::new(COLLAR_FOOTER).icon_url(bot_pfp))
                .color(serenity::Color::from_rgb(255, 0, 0));

            let reply = CreateReply::default()
                .embed(embed)
                .reply(true)
                .ephemeral(true);
            ctx.send(reply).await?;
            send_delete_user_notif(ctx, deleted_user).await?;
        }
        ResponseTypes::Error(error) => {
            let error: ErrorResponse = error;
            let embed = serenity::CreateEmbed::default()
                .title(format!("Error {}", error.status))
                .description(error.message)
                .footer(CreateEmbedFooter::new(COLLAR_FOOTER).icon_url(bot_pfp))
                .color(serenity::Color::from_rgb(255, 0, 0));
            let reply = CreateReply::default().embed(embed).reply(true);
            ctx.send(reply).await?;
        }
    }

    Ok(())
}
