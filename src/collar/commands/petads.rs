use crate::collar::notifs::VerifyType;

use super::{
  Ad, AdEditSubmission, AdSubmission, CollarAppContext, CollarError, EmbedWrapper, ImageSubmission,
  http::{ErrorResponse, ResponseTypes, make_request},
  notifs::{Notif, SubmitType},
  send_generic_error_application,
};
use chrono::DateTime;
use poise::{CreateReply, Modal, command, serenity_prelude as serenity};
use reqwest::Method;
use serenity::{
  Color, CreateEmbedAuthor, FormattedTimestamp, FormattedTimestampStyle, Mentionable, Timestamp,
};
use tracing::info;

#[command(
  slash_command,
  description_localized(
    locale = "en-US",
    description = "Get your Ad as a verified PetRing user"
  ),
  description_localized(
    locale = "sv-SE",
    description = "Få din annons som en verifierad PetRing användare"
  ),
  name_localized(locale = "en-US", name = "my_ad"),
  name_localized(locale = "sv-SE", name = "min_annons"),
  category = "PetAds"
)]
pub async fn my_ad(ctx: CollarAppContext<'_>) -> Result<(), CollarError> {
  let user_id = ctx.author().id;

  let response = make_request(
    ctx.data().clone(),
    None::<String>,
    &format!("/get/ad/{user_id}"),
    Method::GET,
  )
  .await?;
  match response {
    ResponseTypes::Success(_ad) => {
      let ad: Ad = _ad;
      let user_id_u64: u64 = user_id.into();

      if ad.discord_id != user_id_u64 {
        return send_generic_error_application(ctx, "Ad not found").await;
      }

      if !ad.verified {
        return send_generic_error_application(ctx, "Ad not verified").await;
      }
    }
    ResponseTypes::Error(_error) => {
      let error: ErrorResponse = _error;

      let embed = EmbedWrapper::new_application(&ctx)
        .title(format!("Error {}", error.status))
        .description(error.message)
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
    description = "Submit your Ad as a verified PetRing user"
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
  let data = ctx.data();
  let cache = data.cache.lock().await;
  let web_base_url = cache.get_web_base_url();

  let modal_data = AdSubmission::execute(ctx).await?;
  let modal_data = match modal_data {
    Some(modal_data) => modal_data,
    None => {
      let embed = EmbedWrapper::new_application(&ctx)
        .title("You didn't submit anything")
        .description("No data was submitted 3:")
        .color(Color::from_rgb(255, 0, 0));

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
    "/post/ad/submit",
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

      let user_pfp = ctx.author().face();

      let formatted_created_at_timestamp = FormattedTimestamp::new(
        Timestamp::from(DateTime::parse_from_rfc3339(&ad.created_at)?),
        Some(FormattedTimestampStyle::LongDateTime),
      )
      .to_string();

      let embed = EmbedWrapper::new_application(&ctx)
        .title("Your ad submission was successful! :3")
        .author(
          CreateEmbedAuthor::new(&ad.username)
            .url(format!("{web_base_url}/user/{}", &ad.username))
            .icon_url(&user_pfp),
        )
        .thumbnail(&ad.image_url)
        .field("Ad url", &ad.ad_url, false)
        .field("Created at", formatted_created_at_timestamp, false)
        .color(Color::from_rgb(0, 255, 0));

      let reply = CreateReply::default()
        .embed(embed)
        .reply(true)
        .ephemeral(true);
      ctx.send(reply).await?;

      let submission_embed = EmbedWrapper::new_application(&ctx)
        .title("New ad submission :3")
        .author(CreateEmbedAuthor::new(format!("from: {}", ctx.author().name)).icon_url(&user_pfp))
        .field("Petring Username", &ad.username, false)
        .field("Ad url", &ad.ad_url, false)
        .thumbnail(&ad.image_url)
        .color(Color::from_rgb(0, 0, 255));

      Notif::new(&ctx)
        .set_embed(submission_embed)
        .submit(&ctx, ad.discord_id, SubmitType::Ad)
        .await?;
    }
    ResponseTypes::Error(error) => {
      let error: ErrorResponse = error;

      let embed = EmbedWrapper::new_application(&ctx)
        .title(format!("Error {}", error.status))
        .description(error.message)
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
  let user = ctx.author();
  let user_mention = user.mention();
  let user_pfp = user.face();

  let data = ctx.data();
  let cache = data.cache.lock().await;
  let web_base_url = cache.get_web_base_url();

  let modal_data = AdEditSubmission::execute(ctx).await?;
  let modal_data = match modal_data {
    Some(modal_data) => modal_data,
    None => {
      let embed = EmbedWrapper::new_application(&ctx)
        .title("You didn't edit anything")
        .description("No data was submitted 3:")
        .color(Color::from_rgb(255, 0, 0));

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
    "/patch/ad/edit/",
    Method::PATCH,
  )
  .await?;
  match response {
    ResponseTypes::Success(ad) => {
      let ad: Ad = ad;

      let user_id_u64: u64 = ctx.author().id.into();

      if user_id_u64 != ad.discord_id {
        return Err("User not found".into());
      }

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

      let embed = EmbedWrapper::new_application(&ctx)
        .title("Your edit was successful! :3")
        .author(
          CreateEmbedAuthor::new(&ad.username)
            .url(format!("{web_base_url}/user/{}", &ad.username))
            .icon_url(&user_pfp),
        )
        .thumbnail(&ad.image_url)
        .field("Ad url", &ad.ad_url, false)
        .field("Created", &formatted_created_at_timestamp, false)
        .field("Edited", &formatted_edited_at_timestamp, false)
        .field("Verified", &formatted_verified_at_timestamp, false)
        .color(Color::from_rgb(0, 255, 0));

      let reply = CreateReply::default()
        .embed(embed)
        .reply(true)
        .ephemeral(true);
      ctx.send(reply).await?;

      let edit_notif_embed = EmbedWrapper::new_application(&ctx)
        .title("Ad edited :3")
        .description(format!("{user_mention} has edited their ad in PetAds :P"))
        .field("Created", &formatted_created_at_timestamp, false)
        .field("Verified", &formatted_verified_at_timestamp, false)
        .field("Edited", &formatted_edited_at_timestamp, false)
        .author(CreateEmbedAuthor::new(format!("Edited by {}", user.name)).icon_url(&user_pfp))
        .thumbnail(&ad.image_url)
        .color(Color::from_rgb(0, 255, 0));

      Notif::new(&ctx)
        .set_embed(edit_notif_embed)
        .general(&ctx)
        .await?;
    }
    ResponseTypes::Error(error) => {
      let error: ErrorResponse = error;

      let embed = EmbedWrapper::new_application(&ctx)
        .title(format!("Error {}", error.status))
        .description(error.message)
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
  let url = format!("/patch/ad/verify/{}", user_id);

  let user_mention = ctx.http().get_user(user_id).await?.mention();

  let response = make_request(ctx.data().clone(), None::<String>, &url, Method::PATCH).await?;
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

      let user_pfp = ctx.author().face();

      let created_at_timestamp = FormattedTimestamp::from(Timestamp::from(
        DateTime::parse_from_rfc3339(&ad.created_at).unwrap(),
      ))
      .to_string();

      let verified_at_timestamp = FormattedTimestamp::from(Timestamp::from(
        DateTime::parse_from_rfc3339(&ad.verified_at).unwrap(),
      ))
      .to_string();

      let embed = EmbedWrapper::new_application(&ctx)
        .title("Your verification was successful")
        .author(CreateEmbedAuthor::new(format!("for: {}", &ad.username)).icon_url(&user_pfp))
        .url(&ad.ad_url)
        .thumbnail(&ad.image_url)
        .field("Created", created_at_timestamp, false)
        .field("Verified", verified_at_timestamp, false)
        .color(Color::from_rgb(0, 255, 0));

      let dm_ad_verify_embed = EmbedWrapper::new_application(&ctx)
        .title("Your ad was verified!!")
        .description(format!(
          "Hi, there, {user_mention}, your ad has been verified :3"
        ))
        .author(CreateEmbedAuthor::new(format!(
          "Verified by: {}",
          ctx.author().name
        )))
        .thumbnail(&ad.image_url)
        .author(CreateEmbedAuthor::new(user.name.clone()))
        .color(Color::from_rgb(0, 255, 0));

      let ad_verification_done_embed = EmbedWrapper::new_application(&ctx)
        .title("An Ad has been verified :3")
        .description(format!("Verified ad for: {}", user_mention))
        .color(Color::from_rgb(0, 255, 0));

      let reply = CreateReply::default()
        .embed(embed)
        .reply(true)
        .ephemeral(true);
      ctx.send(reply).await?;
      info!("Sending verify ad notif dm");

      Notif::new(&ctx)
        .set_embed(dm_ad_verify_embed)
        .dm_notif(&ctx, user_id.get())
        .await?;

      Notif::new(&ctx)
        .set_embed(ad_verification_done_embed)
        .verification(&ctx, VerifyType::Ad)
        .await?;
    }
    ResponseTypes::Error(_error) => {
      let error: ErrorResponse = _error;

      let embed = EmbedWrapper::new_application(&ctx)
        .title(format!("Error {}", error.status))
        .description(error.message)
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
  let url = format!("/delete/ad/by-discord/{}", user_id);

  let user_mention = user.mention();
  let user_pfp = user.face();

  let response = make_request(ctx.data().clone(), None::<String>, &url, Method::DELETE).await?;
  match response {
    ResponseTypes::Success(response) => {
      let deleted_ad: Ad = response;

      let user_id_u64: u64 = user_id.into();
      if deleted_ad.discord_id != user_id_u64 {
        return Err("Ad not found".into());
      }

      let embed = EmbedWrapper::new_application(&ctx)
        .title("Successfully removed ad :3")
        .description(format!("{user_mention}'s ad has been removed :3",))
        .field("Petring Username", &deleted_ad.username, false)
        .field("Ad url", &deleted_ad.ad_url, false)
        .thumbnail(&deleted_ad.image_url)
        .author(CreateEmbedAuthor::new(format!("Bye {}", user.name)).icon_url(&user_pfp))
        .color(Color::from_rgb(255, 0, 0));

      let formatted_created_at_timestamp = FormattedTimestamp::new(
        Timestamp::from(DateTime::parse_from_rfc3339(&deleted_ad.created_at)?),
        Some(FormattedTimestampStyle::LongDateTime),
      )
      .to_string();

      let delete_ad_notif_embed = EmbedWrapper::new_application(&ctx)
        .title("Ad deleted 3:")
        .description(format!("{user_mention} got their ad deleted in PetAds 3':",))
        .thumbnail(&deleted_ad.image_url)
        .author(
          CreateEmbedAuthor::new(format!("Deleted by: {}", ctx.author().name)).icon_url(&user_pfp),
        )
        .field("Petring Username", &deleted_ad.username, false)
        .field("Ad url", &deleted_ad.ad_url, false)
        .field("Verified", deleted_ad.verified.to_string(), false)
        .field("Created", formatted_created_at_timestamp, false)
        .color(Color::from_rgb(255, 0, 0));

      let reply = CreateReply::default()
        .embed(embed)
        .reply(true)
        .ephemeral(true);
      ctx.send(reply).await?;

      Notif::new(&ctx)
        .set_embed(delete_ad_notif_embed)
        .general(&ctx)
        .await?;
    }
    ResponseTypes::Error(error) => {
      let error: ErrorResponse = error;
      let embed = EmbedWrapper::new_application(&ctx)
        .title(format!("Error {}", error.status))
        .description(error.message)
        .color(Color::from_rgb(255, 0, 0));
      let reply = CreateReply::default().embed(embed).reply(true);
      ctx.send(reply).await?;
    }
  }

  Ok(())
}
