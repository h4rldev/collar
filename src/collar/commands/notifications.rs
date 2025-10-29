use super::{CollarContext, CollarError, EmbedWrapper, NotifChannelType, NotifType};
use poise::{CreateReply, command, serenity_prelude as serenity};
use serenity::Color;
use tracing::{error, info};

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
  category = "Notifications",
  required_permissions = "MANAGE_CHANNELS | BAN_MEMBERS | KICK_MEMBERS | MUTE_MEMBERS"
)]
pub async fn set_notif_channel(
  ctx: CollarContext<'_>,
  channel: serenity::Channel,
  channel_type: NotifType,
) -> Result<(), CollarError> {
  let data = ctx.data();
  let mut cache = data.cache.lock().await;

  match channel.clone().guild() {
    Some(_) => {}
    None => {
      let embed = EmbedWrapper::new_normal(&ctx)
        .title("You can't set a notification channel in a DM :3")
        .description("You need to be in a server to set a notification channel :3")
        .color(Color::from_rgb(255, 0, 0));
      let reply = CreateReply::default().embed(embed).reply(true);
      ctx.send(reply).await?;
      return Ok(());
    }
  }

  let channel_type_to_set = match channel_type {
    NotifType::UserSubmit => NotifChannelType::UserSubmit,
    NotifType::AdSubmit => NotifChannelType::AdSubmit,
    NotifType::UserVerify => NotifChannelType::UserVerify,
    NotifType::AdVerify => NotifChannelType::AdVerify,
    NotifType::General => NotifChannelType::General,
    NotifType::DmFallback => NotifChannelType::DmFallback,
  };

  info!(
    "Setting notif_channel_id.{} to {}",
    channel_type_to_set,
    channel.id()
  );
  cache.set_notif_channel(channel.id().into(), channel_type_to_set);

  let channel_type_str = match channel_type {
    NotifType::UserSubmit => "User Submit",
    NotifType::AdSubmit => "Ad Submit",
    NotifType::UserVerify => "User Verify",
    NotifType::AdVerify => "Ad Verify",
    NotifType::General => "General",
    NotifType::DmFallback => "DM Fallback",
  };

  let channel_type_desc = match channel_type {
    NotifType::UserSubmit => "when someone submits a website",
    NotifType::AdSubmit => "when someone submits an ad",
    NotifType::UserVerify => "when someone's website gets verified",
    NotifType::AdVerify => "when someone's ad gets verified",
    NotifType::General => "when someone deletes, edits a website or ad",
    NotifType::DmFallback => "when the I fail to dm a user",
  };

  let embed = EmbedWrapper::new_normal(&ctx)
    .title(format!("{} Notification channel set!", channel_type_str))
    .description(format!(
      "Expect a notification {} in {}",
      channel_type_desc, channel
    ))
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
    description = "Get a channel that notifications are sent to"
  ),
  description_localized(locale = "sv-SE", description = "Hämta notifieringskanal"),
  name_localized(locale = "en-US", name = "get_notification_channel"),
  name_localized(locale = "sv-SE", name = "hämta_notifieringskanal"),
  required_permissions = "MANAGE_CHANNELS | BAN_MEMBERS | KICK_MEMBERS | MUTE_MEMBERS",
  category = "Notifications"
)]
pub async fn get_notif_channel(
  ctx: CollarContext<'_>,
  channel_type: NotifType,
) -> Result<(), CollarError> {
  let data = ctx.data();
  let cache = data.cache.lock().await;
  let cache_channel_type = match channel_type {
    NotifType::UserSubmit => NotifChannelType::UserSubmit,
    NotifType::AdSubmit => NotifChannelType::AdSubmit,
    NotifType::UserVerify => NotifChannelType::UserVerify,
    NotifType::AdVerify => NotifChannelType::AdVerify,
    NotifType::General => NotifChannelType::General,
    NotifType::DmFallback => NotifChannelType::DmFallback,
  };

  let channel_id = cache.get_notif_channel(cache_channel_type);

  let channel_id = match channel_id {
    Some(channel_id) => channel_id,
    None => {
      error!("No channel id for {}", cache_channel_type);
      return Ok(());
    }
  };

  let channel = match ctx.http().get_channel(channel_id.into()).await {
    Ok(channel) => channel,
    Err(_) => {
      return Ok(());
    }
  };

  let channel_type_str = match channel_type {
    NotifType::UserSubmit => "User Submit",
    NotifType::AdSubmit => "Ad Submit",
    NotifType::UserVerify => "User Verify",
    NotifType::AdVerify => "Ad Verify",
    NotifType::General => "General",
    NotifType::DmFallback => "DM Fallback",
  };

  let channel_type_desc = match channel_type {
    NotifType::UserSubmit => "when someone submits a website",
    NotifType::AdSubmit => "when someone submits an ad",
    NotifType::UserVerify => "when someone's website gets verified",
    NotifType::AdVerify => "when someone's ad gets verified",
    NotifType::General => "when someone deletes, edits a website or ad",
    NotifType::DmFallback => "when the I fail to dm a user",
  };

  let embed = EmbedWrapper::new_normal(&ctx)
    .title(format!(
      "Here's the {channel_type_str} Notification channel"
    ))
    .description(format!("The channel for {channel_type_desc} is {channel}"))
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
  description_localized(
    locale = "en-US",
    description = "Get all channels that notifications are sent to"
  ),
  description_localized(locale = "sv-SE", description = "Hämta alla notifieringskanaler"),
  name_localized(locale = "en-US", name = "get_all_notification_channels"),
  name_localized(locale = "sv-SE", name = "hämta_alla_notifieringskanaler"),
  required_permissions = "MANAGE_CHANNELS | BAN_MEMBERS | KICK_MEMBERS | MUTE_MEMBERS",
  category = "Notifications"
)]
pub async fn get_all_notif_channels(ctx: CollarContext<'_>) -> Result<(), CollarError> {
  let data = ctx.data();
  let cache = data.cache.lock().await;

  let all_notif_channel_ids = cache.get_all_notif_channels();

  let (is_user_submit, is_ad_submit, is_user_verify, is_ad_verify, is_general, is_dm_fallback) = (
    all_notif_channel_ids.user_submit_id.is_some(),
    all_notif_channel_ids.ad_submit_id.is_some(),
    all_notif_channel_ids.user_verify_id.is_some(),
    all_notif_channel_ids.ad_verify_id.is_some(),
    all_notif_channel_ids.general_id.is_some(),
    all_notif_channel_ids.dm_fallback_id.is_some(),
  );

  let user_submit = if is_user_submit {
    match ctx
      .http()
      .get_channel(all_notif_channel_ids.user_submit_id.unwrap().into())
      .await
    {
      Ok(channel) => channel.to_string(),
      Err(_) => String::from("User Submit: Invalid channel id, did a channel get deleted?"),
    }
  } else {
    String::from("Unset! Set it using `/set_notification_channel`")
  };

  let ad_submit = if is_ad_submit {
    match ctx
      .http()
      .get_channel(all_notif_channel_ids.ad_submit_id.unwrap().into())
      .await
    {
      Ok(channel) => channel.to_string(),
      Err(_) => String::from("Ad Submit: Invalid channel id, did a channel get deleted?"),
    }
  } else {
    String::from("Unset! Set it using `/set_notification_channel`")
  };

  let user_verify = if is_user_verify {
    match ctx
      .http()
      .get_channel(all_notif_channel_ids.user_verify_id.unwrap().into())
      .await
    {
      Ok(channel) => channel.to_string(),
      Err(_) => String::from("User Verify: Invalid channel id, did a channel get deleted?"),
    }
  } else {
    String::from("Unset! Set it using `/set_notification_channel`")
  };

  let ad_verify = if is_ad_verify {
    match ctx
      .http()
      .get_channel(all_notif_channel_ids.ad_verify_id.unwrap().into())
      .await
    {
      Ok(channel) => channel.to_string(),
      Err(_) => String::from("Ad Verify: Invalid channel id, did a channel get deleted?"),
    }
  } else {
    String::from("Unset! Set it using `/set_notification_channel`")
  };

  let general = if is_general {
    match ctx
      .http()
      .get_channel(all_notif_channel_ids.general_id.unwrap().into())
      .await
    {
      Ok(channel) => channel.to_string(),
      Err(_) => String::from("Invalid channel id, did a channel get deleted?"),
    }
  } else {
    String::from("Unset! Set it using `/set_notification_channel`")
  };

  let dm_fallback = if is_dm_fallback {
    match ctx
      .http()
      .get_channel(all_notif_channel_ids.dm_fallback_id.unwrap().into())
      .await
    {
      Ok(channel) => channel.to_string(),
      Err(_) => String::from("Invalid channel id, did a channel get deleted?"),
    }
  } else {
    String::from("Unset! Set it using `/set_notification_channel`")
  };

  let embed = EmbedWrapper::new_normal(&ctx)
    .title("Notification channels")
    .field("User Submit", user_submit, true)
    .field("Ad Submit", ad_submit, true)
    .field("User Verify", user_verify, true)
    .field("Ad Verify", ad_verify, true)
    .field("General", general, true)
    .field("DM Fallback", dm_fallback, true);

  let reply = CreateReply::default()
    .embed(embed)
    .reply(true)
    .ephemeral(true);

  ctx.send(reply).await?;
  Ok(())
}
