use songbird::tracks::PlayMode;
use crate::{AppError, Context};

use anyhow::anyhow;

/// Pause/resume the current track
#[poise::command(prefix_command, slash_command, guild_only)]
pub async fn pause(ctx: Context<'_>) -> Result<(), AppError> {
    let guild_id = match ctx.guild().map(|guild| guild.id) {
        Some(guild_id) => guild_id,
        None => {
            if let Err(e) = ctx.say("This command must be invoke in a guild!").await {
                tracing::warn!("can't send message 'guild command only': {}", e);
            }
            return Ok(());
        }
    };

    let songbird_manager = match songbird::get(ctx.serenity_context()).await {
        Some(songbird_manager) => songbird_manager,
        None => {
            if let Err(e) = ctx.say("Can't get Songbird manager!").await {
                tracing::warn!("can't send message 'can't get songbird manager': {}", e);
            }
            return Ok(());
        }
    };

    let call = match songbird_manager.get(guild_id) {
        Some(call) => call,
        None => {
            if let Err(e) = ctx.say("Not in a voice channel.").await {
                tracing::warn!("can't send message 'not in a voice channel': {}", e);
            }
            return Ok(());
        }
    };

    let track_handle = call.lock().await.queue().current();
    if let Some(track_handle) = track_handle {
        let mut was_playing = false;
        match track_handle.get_info().await?.playing {
            PlayMode::Play => {
                was_playing = true;
                track_handle.pause()
            }
            PlayMode::Pause => track_handle.play(),
            _ => {
                if let Err(e) = ctx.say("Nothing was playing.").await {
                    tracing::warn!("can't send message 'nothing was playing': {}", e);
                }
                return Ok(());
            }
        }
        .map_err(|e| {
            AppError::from(anyhow!(
                "commands::player::pause: can't change playing state: {}",
                e
            ))
        })?;

        if let Err(e) = ctx
            .say(match was_playing {
                true => "⏸️ Paused",
                false => "▶️ Resumed",
            })
            .await
        {
            tracing::warn!("can't state send message: {}", e);
        }

        return Ok(());
    }

    if let Err(e) = ctx.say("Nothing was playing.").await {
        tracing::warn!("can't send message 'nothing was playing': {}", e);
    }

    Ok(())
}
