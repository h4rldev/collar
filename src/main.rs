use collar::{
    Collar,
    commands::{get, me, submit, verify},
};
use dotenvy::dotenv;
use poise::serenity_prelude as serenity;
use tracing_subscriber::{
    field::MakeExt,
    fmt::{Subscriber, format::debug_fn},
};

mod collar;

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
    let base_url = std::env::var("WEBRING_BASE_URL").expect("missing WEBRING_BASE_URL");
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![me(), get(), submit(), verify()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Collar::new(Some(base_url)).await)
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
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
