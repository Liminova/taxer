use poise::{
    serenity_prelude::{CreateEmbed, CreateEmbedFooter},
    CreateReply,
};

use crate::{Context, Error};

/// A ping command.
#[poise::command(prefix_command, slash_command)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let start = std::time::Instant::now();
    let msg = ctx.say("Calculating latency...").await?;

    let end = start.elapsed();
    let latency = end.as_millis();

    msg.edit(
        ctx,
        CreateReply::default()
            .embed(
                CreateEmbed::new()
                    .title("Pong!")
                    .description(format!("Latency: {}ms", latency))
                    .color(0x00ff00)
                    .footer(CreateEmbedFooter::new(format!(
                        "Rustc version: {}",
                        rustc_version_runtime::version(),
                    ))),
            )
            .ephemeral(false),
    )
    .await?;

    Ok(())
}
