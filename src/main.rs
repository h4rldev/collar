use collar::{
    Collar,
    commands::{misc, notifications, petads, petring},
};
use dotenvy::dotenv;
use poise::{Framework, serenity_prelude as serenity};
use serenity::{ClientBuilder, Context, Ready};
use tracing_subscriber::{
    field::MakeExt,
    fmt::{Subscriber, format::debug_fn},
};

mod collar;

async fn setup<U, E>(
    ctx: &Context,
    _ready: &Ready,
    framework: &Framework<U, E>,
) -> Result<Collar, E>
where
    U: Send + Sync,
    E: Send + Sync + From<poise::serenity_prelude::Error>,
{
    dotenv().ok();
    let web_base_url = std::env::var("WEB_BASE_URL").expect("missing WEB_BASE_URL");
    let api_base_url = std::env::var("API_BASE_URL").expect("missing API_BASE_URL");

    poise::builtins::register_globally(ctx, &framework.options().commands).await?;

    Ok(Collar::new(api_base_url, web_base_url).await)
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let formatter =
        debug_fn(|writer, field, value| write!(writer, "{field}: {value:?}")).delimited(",");

    Subscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .fmt_fields(formatter)
        .with_ansi(true)
        .init();

    let token = std::env::var("DISCORD_BOT_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                misc::ping(),
                misc::help(),
                misc::set_feedback_webhook(),
                misc::feedback(),
                petring::me(),
                petring::get_user(),
                petring::submit_user(),
                petring::verify_user(),
                petring::edit_user(),
                petring::remove_user(),
                notifications::set_notif_channel(),
                notifications::get_notif_channel(),
                notifications::get_all_notif_channels(),
                petads::submit_ad(),
                petads::verify_ad(),
                petads::remove_ad(),
                petads::edit_ad(),
            ],
            ..Default::default()
        })
        .setup(|ctx, ready, framework| Box::pin(async move { setup(ctx, ready, framework).await }))
        .build();

    let client = ClientBuilder::new(token, intents)
        .framework(framework)
        .await;

    match client {
        Ok(mut client) => {
            if let Err(why) = client.start().await {
                println!("Client error: {:?}", why);
            }
        }
        Err(why) => println!("Authentication error: {:?}", why),
    }
}
