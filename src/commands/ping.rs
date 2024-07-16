use poise::{
    serenity_prelude::{CreateEmbed, CreateEmbedFooter},
    CreateReply,
};

use crate::{Context, Error};

/// Check the App's status and latency
#[poise::command(prefix_command, slash_command)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let latency = ctx
        .data()
        .shard_manager
        .clone()
        .runners
        .lock()
        .await
        .get(&ctx.serenity_context().shard_id)
        .and_then(|runner| runner.latency);

    if let Some(latency) = latency {
        let _ = ctx
            .send(
                CreateReply::default().embed(
                    CreateEmbed::new()
                        .title("Pong!")
                        .description(format!("Latency: {}ms", latency.as_millis()))
                        .color(0x00ff00)
                        .footer(CreateEmbedFooter::new(format!(
                            "Rustc version: {}",
                            rustc_version_runtime::version()
                        ))),
                ),
            )
            .await;

        return Ok(());
    }

    let _ = ctx
        .send(
            CreateReply::default().embed(
                CreateEmbed::new()
                    .title("Pong!")
                    .description("Try again later")
                    .color(0xff3300)
                    .footer(CreateEmbedFooter::new(format!(
                        "Rustc version: {}",
                        rustc_version_runtime::version()
                    ))),
            ),
        )
        .await;

    Ok(())
}
