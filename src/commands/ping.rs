use std::time::Duration;

use crate::{AppError, Context};

use anyhow::anyhow;
use humantime::format_duration;
use poise::{
    serenity_prelude::{CreateEmbed, CreateEmbedFooter},
    CreateReply,
};

/// Check the App's status and latency
#[poise::command(prefix_command, slash_command)]
pub async fn ping(ctx: Context<'_>) -> Result<(), AppError> {
    let latency = match ctx
        .data()
        .shard_manager
        .clone()
        .runners
        .lock()
        .await
        .get(&ctx.serenity_context().shard_id)
        .and_then(|runner| runner.latency)
    {
        Some(latency) => format!("Latency: {}ms", latency.as_millis()),
        None => "Try again later".to_string(),
    };

    let uptime = format_duration(Duration::from_secs(
        ctx.data().start_time.elapsed().as_secs(),
    ));

    ctx.send(
        CreateReply::default().embed(
            CreateEmbed::new()
                .title("Pong!")
                .color(0x0a5c36)
                .fields(vec![
                    ("Latency", latency, true),
                    ("Uptime", uptime.to_string(), true),
                ])
                .footer(CreateEmbedFooter::new(format!(
                    "Rustc version: {}",
                    rustc_version_runtime::version()
                ))),
        ),
    )
    .await
    .map_err(|e| AppError::from(anyhow!("can't send message: {}", e)))?;

    Ok(())
}
