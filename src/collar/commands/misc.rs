use super::{
    COLLAR_FOOTER, CollarAppContext, CollarContext, CollarError, EmbedWrapper, FeedbackSubmission,
    FeedbackTopicType, WebhookEmbed, WebhookEmbedAuthor, WebhookEmbedFooter, WebhookEmbedThumbnail,
    WebhookPost,
};
use poise::{
    CreateReply, Modal, command, samples::HelpConfiguration, serenity_prelude as serenity,
};
use reqwest::Method;
use serenity::Color;
use std::io::Write;
use tokio::time::Instant;

async fn measure_api_latency(ctx: CollarContext<'_>) -> Result<(u128, u128), reqwest::Error> {
    let total_start = Instant::now();
    let client = ctx.data().client.clone();
    let url = format!("{}/api/", ctx.data().api_base_url);

    let res = client.get(url).send().await?;

    let latency = total_start.elapsed().as_millis(); // Time to first byte

    let _body = res.bytes().await?;
    let total_time = total_start.elapsed().as_millis(); // Total response time

    Ok((latency, total_time))
}

#[command(
    slash_command,
    description_localized(locale = "en-US", description = "Get discord gateway ping"),
    description_localized(locale = "sv-SE", description = "Hämta discord gateway ping"),
    name_localized(locale = "en-US", name = "ping"),
    name_localized(locale = "sv-SE", name = "ping"),
    category = "Miscellaneous"
)]
pub async fn ping(ctx: CollarContext<'_>) -> Result<(), CollarError> {
    let discord_ping = ctx.ping().await.as_millis();
    let petring_ping = measure_api_latency(ctx).await?;

    let embed = EmbedWrapper::new_normal(&ctx)
        .title("Pong!")
        .field(
            "Gateway Heartbeat Latency",
            format!("{}ms", discord_ping),
            true,
        )
        .field(
            "Petring",
            format!("Latency: {}ms, Total: {}ms", petring_ping.0, petring_ping.1),
            true,
        );

    let reply = CreateReply::default().reply(true).embed(embed);

    ctx.send(reply).await?;
    Ok(())
}

#[poise::command(slash_command, category = "Miscellaneous")]
pub async fn help(
    ctx: CollarContext<'_>,
    #[description = "Command to get help for"]
    #[rest]
    command: Option<String>,
) -> Result<(), CollarError> {
    let extra_text_at_bottom = "\
Type `/help command` for more info on a command.
Made with <3";

    let config = HelpConfiguration {
        show_subcommands: false,
        show_context_menu_commands: false,
        ephemeral: true,
        extra_text_at_bottom,

        ..Default::default()
    };
    poise::builtins::help(ctx, command.as_deref(), config).await?;
    Ok(())
}

#[command(slash_command, category = "Miscellaneous")]
pub async fn set_feedback_webhook(
    ctx: CollarAppContext<'_>,
    #[description = "Webhook to send feedback to"] webhook: String,
) -> Result<(), CollarError> {
    let mut feedback_webhook = ctx.data().feedback_webhook.lock().await;
    *feedback_webhook = Some(webhook.clone());

    let mut file_to_write = match std::fs::File::create(".feedback_webhook") {
        Ok(file) => file,
        Err(err) => {
            return Err(CollarError::from(format!(
                "Failed to create .feedback_webhook: {err}"
            )));
        }
    };

    if file_to_write.write_all(webhook.as_bytes()).is_err() {
        return Err(CollarError::from("Failed to cache to .feedback_webhook"));
    }

    let embed = EmbedWrapper::new_application(&ctx)
        .title("Feedback webhook set!")
        .description(format!("Feedback will now be sent to {}", webhook))
        .color(Color::from_rgb(0, 255, 0));

    let reply = CreateReply::default()
        .reply(true)
        .ephemeral(true)
        .embed(embed);

    ctx.send(reply).await?;
    Ok(())
}

#[command(slash_command, category = "Miscellaneous")]
pub async fn feedback(
    ctx: CollarAppContext<'_>,
    #[description = "Topic to send feedback about"] topic: FeedbackTopicType,
) -> Result<(), CollarError> {
    let data = ctx.data();
    let feedback_webhook = data.feedback_webhook.lock().await;
    let webhook = match feedback_webhook.clone() {
        Some(webhook) => webhook,
        None => {
            let no_webhook_embed = EmbedWrapper::new_application(&ctx)
                .title("No webhook set 3':")
                .description("No webhook was set :C")
                .color(Color::from_rgb(255, 0, 0));

            let reply = CreateReply::default()
                .reply(true)
                .ephemeral(true)
                .embed(no_webhook_embed);

            ctx.send(reply).await?;
            return Ok(());
        }
    };

    let modal_data = match FeedbackSubmission::execute(ctx).await? {
        Some(modal_data) => modal_data,
        None => {
            let no_data_embed = EmbedWrapper::new_application(&ctx)
                .title("No data submitted 3':")
                .description("No data was submitted :C")
                .color(Color::from_rgb(255, 0, 0));

            let reply = CreateReply::default()
                .reply(true)
                .ephemeral(true)
                .embed(no_data_embed);

            ctx.send(reply).await?;
            return Ok(());
        }
    };

    let author = match topic {
        FeedbackTopicType::PetRing => WebhookEmbedAuthor {
            name: String::from("Regarding: PetRing"),
        },
        FeedbackTopicType::PetAds => WebhookEmbedAuthor {
            name: String::from("Regarding: PetAds"),
        },
        FeedbackTopicType::Collar => WebhookEmbedAuthor {
            name: String::from("Regarding: Collar"),
        },
    };

    let user_name = ctx.author().clone().name;

    let user_pfp = ctx.author().avatar_url().unwrap(); // if this
    // fails to unwrap, i'll buy myself a beer

    let embed = WebhookEmbed {
        author,
        color: 16711680,
        title: modal_data.title,
        description: modal_data.description,
        footer: WebhookEmbedFooter {
            text: COLLAR_FOOTER.to_string(),
        },
        thumbnail: WebhookEmbedThumbnail {
            url: user_pfp.clone(),
        },
    };

    let post_body = WebhookPost {
        avatar_url: user_pfp,
        username: user_name,
        embeds: vec![embed],
        tts: false,
    };

    let client = ctx.data().client.clone();

    let request = client.request(Method::POST, webhook).json(&post_body);

    let response = request.send().await?;

    if response.status().is_success() {
        let embed = EmbedWrapper::new_application(&ctx)
            .title("Feedback sent!")
            .description("Your feedback was sent successfully :3")
            .color(Color::from_rgb(0, 255, 0));

        let reply = CreateReply::default()
            .reply(true)
            .embed(embed)
            .ephemeral(true);

        ctx.send(reply).await?;
    } else {
        let error = response.text().await?;

        let embed = EmbedWrapper::new_application(&ctx)
            .title("Failed to send feedback 3:")
            .description(format!("Error message: {error}"))
            .color(Color::from_rgb(255, 0, 0));

        let reply = CreateReply::default()
            .reply(true)
            .embed(embed)
            .ephemeral(true);

        ctx.send(reply).await?;
    }

    Ok(())
}
