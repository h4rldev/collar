use super::{CollarContext, CollarError, EmbedWrapper, NotifChannelType};
use poise::{CreateReply, command, serenity_prelude as serenity};
use serenity::Color;
use std::io::Write;
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
    channel_type: NotifChannelType,
) -> Result<(), CollarError> {
    let data = ctx.data();

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

    let mut notif_channel_id = data.notif_channel_ids.lock().await;
    match channel_type {
        NotifChannelType::Submit => {
            info!("Setting notif_channel_id.submit_id to {}", channel.id());
            notif_channel_id.submit_id = Some(channel.id().into());
        }
        NotifChannelType::Verify => {
            info!("Setting notif_channel_id.verify_id to {}", channel.id());
            notif_channel_id.verify_id = Some(channel.id().into());
        }
        NotifChannelType::General => {
            info!("Setting notif_channel_id.general_id to {}", channel.id());
            notif_channel_id.general_id = Some(channel.id().into());
        }
        NotifChannelType::Fallback => {
            info!("Setting notif_channel_id.fallback_id to {}", channel.id());
            notif_channel_id.fallback_id = Some(channel.id().into());
        }
    }

    let mut file_to_write = match std::fs::File::create(".notif_channel_id.json") {
        Ok(file) => file,
        Err(err) => {
            error!("Failed to cache notif_channel_id: {err}");
            return Err(CollarError::from(format!(
                "Failed to create .notif_channel_id.json: {err}"
            )));
        }
    };

    let notif_channel_id_str = match serde_json::to_string(&*notif_channel_id) {
        Ok(notif_channel_id) => notif_channel_id,
        Err(err) => {
            error!("Failed to serialize notif_channel_id: {err}");
            return Err(CollarError::from(format!(
                "Failed to serialize .notif_channel_id.json: {err}"
            )));
        }
    };

    match file_to_write.write_all(notif_channel_id_str.as_bytes()) {
        Ok(_) => {
            info!("Wrote notif_channel_id to .notif_channel_id.json");
        }
        Err(err) => {
            error!("Failed to write notif_channel_id: {err}");
            return Err(CollarError::from(format!(
                "Failed to write .notif_channel_id.json: {err}"
            )));
        }
    }

    let channel_type_str = match channel_type {
        NotifChannelType::Submit => "Submit",
        NotifChannelType::Verify => "Verify",
        NotifChannelType::General => "General",
        NotifChannelType::Fallback => "Fallback",
    };

    let channel_type_desc = match channel_type {
        NotifChannelType::Submit => "when someone submits a website or ad",
        NotifChannelType::Verify => "when someone verifies a website or ad",
        NotifChannelType::General => "when someone deletes, edits a website or ad",
        NotifChannelType::Fallback => "when the bot fails to dm a user",
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
    channel_type: NotifChannelType,
) -> Result<(), CollarError> {
    let data = ctx.data();

    let channel_id = data.notif_channel_ids.lock().await;
    let channel_id = match channel_type {
        NotifChannelType::Submit => channel_id.submit_id,
        NotifChannelType::Verify => channel_id.verify_id,
        NotifChannelType::General => channel_id.general_id,
        NotifChannelType::Fallback => channel_id.fallback_id,
    };

    let channel_id = match channel_id {
        Some(channel_id) => channel_id,
        None => {
            error!("No channel id for {:?}", channel_type);
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
        NotifChannelType::Submit => "Submit",
        NotifChannelType::Verify => "Verify",
        NotifChannelType::General => "General",
        NotifChannelType::Fallback => "Fallback",
    };

    let channel_type_desc = match channel_type {
        NotifChannelType::Submit => "when someone submits a website or ad",
        NotifChannelType::Verify => "when someone verifies a website or ad",
        NotifChannelType::General => "when someone deletes, edits a website or ad",
        NotifChannelType::Fallback => "when the bot fails to dm a user",
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

    let all_notif_channel_ids = data.notif_channel_ids.lock().await;
    let all_notif_channel_ids = all_notif_channel_ids.clone();

    let (is_submit, is_verify, is_general, is_fallback) = (
        all_notif_channel_ids.submit_id.is_some(),
        all_notif_channel_ids.verify_id.is_some(),
        all_notif_channel_ids.general_id.is_some(),
        all_notif_channel_ids.fallback_id.is_some(),
    );

    let submit = if is_submit {
        match ctx
            .http()
            .get_channel(all_notif_channel_ids.submit_id.unwrap().into())
            .await
        {
            Ok(channel) => channel.to_string(),
            Err(_) => String::from("Submit: Invalid channel id, did a channel get deleted?"),
        }
    } else {
        String::from("Unset! Set it using `/set_notification_channel`")
    };

    let verify = if is_verify {
        match ctx
            .http()
            .get_channel(all_notif_channel_ids.verify_id.unwrap().into())
            .await
        {
            Ok(channel) => channel.to_string(),
            Err(_) => String::from("Invalid channel id, did a channel get deleted?"),
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

    let fallback = if is_fallback {
        match ctx
            .http()
            .get_channel(all_notif_channel_ids.fallback_id.unwrap().into())
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
        .field("Submit", submit, true)
        .field("Verify", verify, true)
        .field("General", general, true)
        .field("DM Fallback", fallback, true);

    let reply = CreateReply::default()
        .embed(embed)
        .reply(true)
        .ephemeral(true);

    ctx.send(reply).await?;
    Ok(())
}
