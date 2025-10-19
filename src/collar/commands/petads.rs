use crate::collar::EmbedWrapper;

use super::{
    Ad, AdEditSubmission, AdSubmission, COLLAR_FOOTER, CollarAppContext, CollarError,
    ImageSubmission,
    http::{ErrorResponse, ResponseTypes, make_request},
    notifs::{Notif, SubmitType},
};
use chrono::DateTime;
use poise::{CreateReply, Modal, command, serenity_prelude as serenity};
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
        description = "Submit an Ad for a verified petring user"
    ),
    description_localized(
        locale = "sv-SE",
        description = "Skicka en annons för en verifierad petring användare"
    ),
    name_localized(locale = "en-US", name = "submit_ad"),
    name_localized(locale = "sv-SE", name = "skicka_annons"),
    category = "PetAds"
)]
pub async fn submit_ad(ctx: CollarAppContext<'_>) -> Result<(), CollarError> {
    let bot_id = ctx.data().bot_id;
    let bot_pfp = ctx.cache().user(bot_id).unwrap().avatar_url().unwrap(); // if this fails to unwrap, i'll buy myself a beer

    let modal_data = AdSubmission::execute(ctx).await?;
    let modal_data = match modal_data {
        Some(modal_data) => modal_data,
        None => {
            let embed = CreateEmbed::default()
                .title("You didn't submit anything")
                .description("No data was submitted 3:")
                .color(Color::from_rgb(255, 0, 0))
                .footer(CreateEmbedFooter::new(COLLAR_FOOTER).icon_url(bot_pfp));

            let reply = CreateReply::default().embed(embed).reply(true);
            ctx.send(reply).await?;
            return Ok(());
        }
    };

    let image_url = modal_data.image_url;

    let response = make_request(
        ctx.data().clone(),
        Some(ImageSubmission {
            image_url,
            discord_id: ctx.author().id.into(),
        }),
        "/api/post/ad/submit",
        Method::POST,
    )
    .await?;
    match response {
        ResponseTypes::Success(ad) => {
            let ad: Ad = ad;

            let user_id_u64: u64 = ctx.author().id.into();

            if user_id_u64 != ad.discord_id {
                return Err("User not found".into());
            }

            let avatar_url = match ctx.http().get_user(ctx.author().id).await {
                Ok(user) => user.avatar_url().unwrap(),
                Err(_) => {
                    return Err("User not found".into());
                }
            };

            let base_url = ctx.data().api_base_url.clone();

            let formatted_created_at_timestamp = FormattedTimestamp::new(
                Timestamp::from(DateTime::parse_from_rfc3339(&ad.created_at)?),
                Some(FormattedTimestampStyle::LongDateTime),
            )
            .to_string();

            let embed = CreateEmbed::default()
                .title("Your ad submission was successful! :3")
                .author(
                    CreateEmbedAuthor::new(ad.username.clone())
                        .url(format!("{base_url}/user/{}", ad.username)),
                )
                .thumbnail(avatar_url)
                .image(ad.image_url)
                .field("Ad url", ad.ad_url.clone(), false)
                .field("Created at", formatted_created_at_timestamp, false)
                .footer(CreateEmbedFooter::new(COLLAR_FOOTER).icon_url(bot_pfp))
                .color(Color::from_rgb(0, 255, 0));

            let reply = CreateReply::default()
                .embed(embed)
                .reply(true)
                .ephemeral(true);
            ctx.send(reply).await?;

            let submission_embed = EmbedWrapper::new_application(&ctx)
                .0
                .title("New ad submission :3")
                .author(CreateEmbedAuthor::new(format!("from: {}", ad.username)))
                .field("Website", ad.ad_url, false)
                .color(serenity::Color::from_rgb(0, 0, 255));

            let mut notif = Notif::new(&ctx);
            notif = notif.set_embed(submission_embed);
            notif.submit(&ctx, ad.discord_id, SubmitType::Ad).await?;
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
        description = "Edit your ad submission, as a verified user"
    ),
    description_localized(
        locale = "sv-SE",
        description = "Redigera din annons för en verifierad användare"
    ),
    name_localized(locale = "en-US", name = "edit_ad"),
    name_localized(locale = "sv-SE", name = "redigera_annons"),
    category = "PetAds"
)]
pub async fn edit_ad(ctx: CollarAppContext<'_>) -> Result<(), CollarError> {
    let bot_id = ctx.data().bot_id;
    let bot_pfp = ctx.cache().user(bot_id).unwrap().avatar_url().unwrap(); // if this fails to unwrap, i'll buy myself a beer

    let user = ctx.author();
    let user_mention = user.mention();
    let user_pfp = user.avatar_url().unwrap();

    let modal_data = AdEditSubmission::execute(ctx).await?;
    let modal_data = match modal_data {
        Some(modal_data) => modal_data,
        None => {
            let embed = CreateEmbed::default()
                .title("You didn't edit anything")
                .description("No data was submitted 3:")
                .color(Color::from_rgb(255, 0, 0))
                .footer(CreateEmbedFooter::new(COLLAR_FOOTER).icon_url(bot_pfp));

            let reply = CreateReply::default().embed(embed).reply(true);
            ctx.send(reply).await?;
            return Ok(());
        }
    };

    let image_url = modal_data.image_url;

    let response = make_request(
        ctx.data().clone(),
        Some(ImageSubmission {
            image_url,
            discord_id: ctx.author().id.into(),
        }),
        "/api/put/ad/edit/",
        Method::PUT,
    )
    .await?;
    match response {
        ResponseTypes::Success(ad) => {
            let ad: Ad = ad;

            let user_id_u64: u64 = ctx.author().id.into();

            if user_id_u64 != ad.discord_id {
                return Err("User not found".into());
            }

            let avatar_url = match ctx.http().get_user(ctx.author().id).await {
                Ok(user) => user.avatar_url().unwrap(),
                Err(_) => {
                    return Err("User not found".into());
                }
            };

            let base_url = ctx.data().api_base_url.clone();

            let formatted_created_at_timestamp = FormattedTimestamp::new(
                Timestamp::from(DateTime::parse_from_rfc3339(&ad.created_at)?),
                Some(FormattedTimestampStyle::LongDateTime),
            )
            .to_string();

            let formatted_edited_at_timestamp = FormattedTimestamp::new(
                Timestamp::from(DateTime::parse_from_rfc3339(&ad.edited_at)?),
                Some(FormattedTimestampStyle::RelativeTime),
            )
            .to_string();

            let formatted_verified_at_timestamp = FormattedTimestamp::new(
                Timestamp::from(DateTime::parse_from_rfc3339(&ad.verified_at)?),
                Some(FormattedTimestampStyle::LongDateTime),
            )
            .to_string();

            let embed = CreateEmbed::default()
                .title("Your edit was successful! :3")
                .author(
                    CreateEmbedAuthor::new(ad.username.clone())
                        .url(format!("{base_url}/user/{}", ad.username)),
                )
                .thumbnail(avatar_url)
                .image(ad.image_url.clone())
                .field("Ad url", ad.ad_url.clone(), false)
                .field("Created", &formatted_created_at_timestamp, false)
                .field("Edited", &formatted_edited_at_timestamp, false)
                .field("Verified", &formatted_verified_at_timestamp, false)
                .footer(CreateEmbedFooter::new(COLLAR_FOOTER).icon_url(bot_pfp))
                .color(Color::from_rgb(0, 255, 0));

            let reply = CreateReply::default()
                .embed(embed)
                .reply(true)
                .ephemeral(true);
            ctx.send(reply).await?;

            let edit_notif_embed = EmbedWrapper::new_application(&ctx)
                .0
                .title("Ad edited :3")
                .description(format!("{user_mention} has edited their ad in PetAds :3"))
                .field("Created", &formatted_created_at_timestamp, false)
                .field("Verified", &formatted_verified_at_timestamp, false)
                .field("Edited", &formatted_edited_at_timestamp, false)
                .thumbnail(user_pfp)
                .image(ad.image_url)
                .color(Color::from_rgb(0, 255, 0));

            let mut notif = Notif::new(&ctx);
            notif = notif.set_embed(edit_notif_embed);
            notif.general(&ctx).await?;
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
    description_localized(locale = "en-US", description = "Verify a submitted petring ad"),
    description_localized(locale = "sv-SE", description = "Verifiera en skickad petring annons"),
    name_localized(locale = "en-US", name = "verify_ad"),
    name_localized(locale = "sv-SE", name = "verifiera_annons"),
    category = "PetAds",
    required_permissions = "MANAGE_CHANNELS | BAN_MEMBERS | KICK_MEMBERS | MUTE_MEMBERS"
)]
pub async fn verify_ad(ctx: CollarAppContext<'_>, user: serenity::User) -> Result<(), CollarError> {
    let user_id = user.id;
    let url = format!("/api/put/ad/verify/{}", user_id);

    let user_mention = ctx.http().get_user(user_id).await?.mention();

    let bot_id = ctx.data().bot_id;
    let bot_pfp = ctx.cache().user(bot_id).unwrap().avatar_url().unwrap(); // if this fails to unwrap, i'll buy myself a beer

    let response = make_request(ctx.data().clone(), None::<String>, &url, Method::PUT).await?;
    match response {
        ResponseTypes::Success(ad) => {
            let ad: Ad = ad;

            if !ad.verified {
                return Err("User failed to verify".into());
            }

            let user_id_u64: u64 = user_id.into();
            if user_id_u64 != ad.discord_id {
                return Err("User not found".into());
            }

            let avatar_url = match ctx.http().get_user(user_id).await {
                Ok(user) => user.avatar_url().unwrap(),
                Err(_) => {
                    return Err("User not found".into());
                }
            };

            let created_at_timestamp = FormattedTimestamp::from(Timestamp::from(
                DateTime::parse_from_rfc3339(&ad.created_at).unwrap(),
            ))
            .to_string();

            let verified_at_timestamp = FormattedTimestamp::from(Timestamp::from(
                DateTime::parse_from_rfc3339(&ad.verified_at).unwrap(),
            ))
            .to_string();

            let embed = serenity::CreateEmbed::default()
                .title("Your verification was successful")
                .author(CreateEmbedAuthor::new(format!("for: {}", ad.username)))
                .url(ad.ad_url.clone())
                .thumbnail(avatar_url)
                .field("Created", created_at_timestamp, false)
                .field("Verified", verified_at_timestamp, false)
                .footer(CreateEmbedFooter::new(COLLAR_FOOTER).icon_url(bot_pfp))
                .color(Color::from_rgb(0, 255, 0));

            let dm_ad_verify_embed = EmbedWrapper::new_application(&ctx)
                .0
                .title("Your ad was verified!!")
                .description(format!(
                    "Hi, there, {user_mention}, your ad has been verified :3"
                ))
                .author(CreateEmbedAuthor::new(user.name.clone()))
                .color(Color::from_rgb(0, 255, 0));

            let reply = CreateReply::default()
                .embed(embed)
                .reply(true)
                .ephemeral(true);
            ctx.send(reply).await?;
            info!("Sending verify ad notif dm");

            let mut notif = Notif::new(&ctx);
            notif = notif.set_embed(dm_ad_verify_embed);
            notif.dm_notif(&ctx, user_id.get()).await?;
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
        description = "Delete a petring ad (doesn't matter if they're verified or not)"
    ),
    description_localized(
        locale = "sv-SE",
        description = "Radera en petring annons (spelar ingen roll om de är verifierade eller inte)"
    ),
    name_localized(locale = "en-US", name = "remove_ad"),
    name_localized(locale = "sv-SE", name = "radera_annons"),
    category = "PetAds",
    required_permissions = "MANAGE_CHANNELS | BAN_MEMBERS | KICK_MEMBERS | MUTE_MEMBERS"
)]
pub async fn remove_ad(ctx: CollarAppContext<'_>, user: serenity::User) -> Result<(), CollarError> {
    let user_id = user.id;
    let url = format!("/api/delete/ad/by-discord/{}", user_id);

    let user_name = user.clone().name;
    let user_mention = user.mention();
    let user_pfp = user.avatar_url().unwrap();

    let bot_id = ctx.data().bot_id;
    let bot_pfp = ctx.cache().user(bot_id).unwrap().avatar_url().unwrap(); // if this fails to unwrap, i'll buy myself a beer

    let response = make_request(ctx.data().clone(), None::<String>, &url, Method::DELETE).await?;
    match response {
        ResponseTypes::Success(response) => {
            let deleted_ad: Ad = response;

            let user_id_u64: u64 = user_id.into();
            if deleted_ad.discord_id != user_id_u64 {
                return Err("Ad not found".into());
            }

            let embed = serenity::CreateEmbed::default()
                .title("Successfully removed ad :3")
                .description(format!(
                    "{user_mention}'s Ad for {}: removed :3",
                    deleted_ad.ad_url
                ))
                .thumbnail(user_pfp)
                .image(deleted_ad.image_url.clone())
                .footer(CreateEmbedFooter::new(COLLAR_FOOTER).icon_url(bot_pfp))
                .color(serenity::Color::from_rgb(255, 0, 0));

            let formatted_created_at_timestamp = FormattedTimestamp::new(
                Timestamp::from(DateTime::parse_from_rfc3339(&deleted_ad.created_at)?),
                Some(FormattedTimestampStyle::LongDateTime),
            )
            .to_string();

            let delete_ad_notif_embed = EmbedWrapper::new_application(&ctx)
                .0
                .title("Ad deleted 3:")
                .description(format!(
                    "{user_mention}, also known as {} got their ad deleted in PetAds 3':",
                    deleted_ad.username
                ))
                .author(CreateEmbedAuthor::new(user_name))
                .field("Website", deleted_ad.ad_url.clone(), false)
                .field("Verified", deleted_ad.verified.to_string(), false)
                .field("Created", formatted_created_at_timestamp, false)
                .color(serenity::Color::from_rgb(255, 0, 0));

            let reply = CreateReply::default()
                .embed(embed)
                .reply(true)
                .ephemeral(true);
            ctx.send(reply).await?;

            let mut notif = Notif::new(&ctx);
            notif = notif.set_embed(delete_ad_notif_embed);
            notif.general(&ctx).await?;
        }
        ResponseTypes::Error(error) => {
            let error: ErrorResponse = error;
            let embed = CreateEmbed::default()
                .title(format!("Error {}", error.status))
                .description(error.message)
                .footer(CreateEmbedFooter::new(COLLAR_FOOTER).icon_url(bot_pfp))
                .color(Color::from_rgb(255, 0, 0));
            let reply = CreateReply::default().embed(embed).reply(true);
            ctx.send(reply).await?;
        }
    }

    Ok(())
}
