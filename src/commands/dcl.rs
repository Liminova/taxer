use poise::CreateReply;

use crate::{AppError, Context};

/// địt cả lò
#[poise::command(prefix_command, slash_command)]
pub async fn dcl(ctx: Context<'_>) -> Result<(), AppError> {
    let _ = ctx
        .send(CreateReply::default().content("cứ từ từ").ephemeral(true))
        .await?;

    ctx.channel_id()
        .say(ctx.serenity_context(), "địt cả lò")
        .await?;

    Ok(())
}
