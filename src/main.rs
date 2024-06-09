use std::env;

use dotenvy::dotenv;
use poise::{
    serenity_prelude::{ClientBuilder, Error as SerenityError, GatewayIntents},
    FrameworkError, FrameworkOptions,
};
use tracing::error;

mod commands;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;
pub struct Data;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().expect("failed to load .env file.");
    tracing_subscriber::fmt::init();

    let framework = poise::Framework::builder()
        .options(FrameworkOptions {
            commands: vec![
                commands::ping::ping(),
                commands::help::help(),
                commands::qt::qt(),
            ],
            on_error: |error| {
                Box::pin(async move {
                    match error {
                        poise::FrameworkError::ArgumentParse { error, .. } => {
                            if let Some(e) = error.downcast_ref::<serenity::RoleParseError>() {
                                error!("found a RoleParseError: {:#?}", e);
                            } else {
                                error!("not a RoleParseError: {:#?}", error);
                            }
                        }
                        other => {
                            if let Err(e) = poise::builtins::on_error(other).await {
                                error!("fatal error: {}", e);
                            }
                        }
                    }
                })
            },
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("t!".into()),
                case_insensitive_commands: true,
                ..Default::default()
            },
            ..Default::default()
        })
        .token(env::var("DISCORD_TOKEN").unwrap())
        .intents(
            serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT,
        )
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data)
            })
        });

    framework.run().await.unwrap();

    Ok(())
}
