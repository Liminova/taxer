use crate::{AppError, Context};

use std::{os::unix::process::CommandExt, process::Command};

/// Restart the bot
#[poise::command(prefix_command, slash_command, guild_only)]
pub async fn restart(ctx: Context<'_>) -> Result<(), AppError> {
    if let Err(e) = ctx.say("Restarting...").await {
        tracing::warn!("can't send message 'restarting': {}", e);
    }

    Command::new("/proc/self/exe").exec();

    Ok(())
}
