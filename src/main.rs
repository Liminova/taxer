mod commands;
mod data;

use data::{config::Config, Data};

use dotenvy::dotenv;
use poise::{
    serenity_prelude::{ClientBuilder, CreateEmbed, CreateMessage, GatewayIntents},
    FrameworkError, FrameworkOptions,
};
use songbird::SerenityInit;
use tracing::{error, info};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if let Err(e) = dotenv() {
        info!("failed to load .env file: {}", e);
    }

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let config = Config::init();
    let discord_token = config.discord_token.clone();

    let framework = poise::Framework::builder()
        .options(FrameworkOptions {
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("t!".into()),
                case_insensitive_commands: true,
                ..Default::default()
            },
            commands: vec![
                commands::ping::ping(),
                commands::help::help(),
                commands::qt::qt(),
                commands::qt::qt_cm(),
                commands::kqt::kqt(),
                commands::kqt::kqt_cm(),
                commands::kqt::kqt_cm_ai_hoi_bo_truong(),
            ],
            on_error: |error: FrameworkError<Data, Error>| {
                Box::pin(async move {
                    match error {
                        poise::FrameworkError::ArgumentParse { error, .. } => {
                            match error.downcast_ref::<SerenityError>() {
                                Some(e) => error!("error: {:#?}", e),
                                None => error!("unknown error: {:#?}", error),
                            }
                        }
                        other => match poise::builtins::on_error(other).await {
                            Ok(_) => (),
                            Err(e) => error!("fatal error: {}", e),
                        },
                    }
                })
            },
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data)
            })
        })
        .build();

    let client = ClientBuilder::new(
        &env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN is not set"),
        GatewayIntents::non_privileged()
            | GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::DIRECT_MESSAGES
            | GatewayIntents::MESSAGE_CONTENT,
    )
    .framework(framework)
    .await;

    if let Err(e) = client.unwrap().start().await {
        error!("error: {:#?}", e);
        std::process::exit(1);
    }

    Ok(())
}
