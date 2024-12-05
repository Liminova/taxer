mod commands;
mod data;

use std::fmt::Display;

use data::{config::Config, Data};

use dotenvy::dotenv;
use poise::{
    serenity_prelude::{ClientBuilder, GatewayIntents},
    FrameworkError, FrameworkOptions,
};
use songbird::SerenityInit;
use tracing::{error, info};

#[derive(Debug)]
pub struct AppError(anyhow::Error);
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

type Context<'a> = poise::Context<'a, Data, AppError>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if dotenv().is_err() {
        info!(".env file not exists");
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
                commands::dcl::dcl(),
                commands::player::play(),
                commands::player::pause(),
                commands::player::queue(),
                commands::player::restart(),
                commands::player::skip(),
                commands::player::nuke(),
            ],
            on_error: |error: FrameworkError<Data, AppError>| {
                Box::pin(async move {
                    match error {
                        // args parse error
                        poise::FrameworkError::ArgumentParse { error, .. } => {
                            if let Some(error) =
                                error.downcast_ref::<poise::serenity_prelude::RoleParseError>()
                            {
                                println!("Found a RoleParseError: {:?}", error);
                                return;
                            }
                            error!("ArgumentParse error: {}", error);
                        }

                        // error in commands
                        poise::FrameworkError::Command { error, ctx, .. } => {
                            error!("Command error: {}", error);

                            if let Some(guild_channel) = ctx.guild_channel().await {
                                let _ = guild_channel
                                    .say(
                                        ctx.serenity_context(),
                                        format!("Command Error\n```\n{}\n```", error),
                                    )
                                    .await;
                            };
                        }

                        // other errors
                        other => {
                            error!("other error: {}", other);
                            // try to send error message to Discord
                            if let Some(ctx) = other.ctx() {
                                if let Some(guild_channel) = ctx.guild_channel().await {
                                    let _ = guild_channel
                                        .say(
                                            ctx.serenity_context(),
                                            format!("Other Error\n```\n{}\n```", other),
                                        )
                                        .await;
                                };
                            }
                        }
                    }
                })
            },
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data::new(config, framework.shard_manager().clone()))
            })
        })
        .build();

    ClientBuilder::new(
        discord_token,
        GatewayIntents::non_privileged()
            | GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::DIRECT_MESSAGES
            | GatewayIntents::MESSAGE_CONTENT
            | GatewayIntents::GUILD_VOICE_STATES,
    )
    .register_songbird()
    .framework(framework)
    .await
    .expect("Failed to create Discord client")
    .start()
    .await
    .expect("Failed to start Discord client");

    Ok(())
}
