use collar::{
    Collar,
    commands::{edit, get, get_notif_channel, me, remove_user, set_notif_channel, submit, verify},
};
use dotenvy::dotenv;
use poise::{Framework, serenity_prelude as serenity};
use serenity::{Context, Ready};
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
    let base_url = std::env::var("WEBRING_BASE_URL").expect("missing WEBRING_BASE_URL");

    poise::builtins::register_globally(ctx, &framework.options().commands).await?;
    Ok(Collar::new(Some(base_url)).await)
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
                me(),
                get(),
                submit(),
                verify(),
                edit(),
                remove_user(),
                set_notif_channel(),
                get_notif_channel(),
            ],
            ..Default::default()
        })
        .setup(|ctx, ready, framework| Box::pin(async move { setup(ctx, ready, framework).await }))
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
