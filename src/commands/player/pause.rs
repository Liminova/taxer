use songbird::tracks::PlayMode;
use crate::{AppError, Context};

use anyhow::anyhow;

/// Pause/resume the current track
#[poise::command(prefix_command, slash_command, guild_only)]
pub async fn pause(ctx: Context<'_>) -> Result<(), AppError> {
    let guild_id = match ctx.guild().map(|guild| guild.id) {
        Some(guild_id) => guild_id,
        None => {
            let _ = ctx.say("This command must be invoke in a guild!").await;
            return Ok(());
        }
    };

    let songbird_manager = match songbird::get(ctx.serenity_context()).await {
        Some(songbird_manager) => songbird_manager,
        None => {
            let _ = ctx.say("Can't get Songbird manager!").await;
            return Ok(());
        }
    };

    let call = match songbird_manager.get(guild_id) {
        Some(call) => call,
        None => {
            let _ = ctx.say("Not in a voice channel.").await;
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
                let _ = ctx.say("Nothing was playing.").await;
                return Ok(());
            }
        }
        .map_err(|e| format!("commands::player::pause: can't change playing state: {}", e))?;

        let _ = ctx
            .say(match was_playing {
                true => "⏸️ Paused",
                false => "▶️ Resumed",
            })
            .await;

        return Ok(());
    }

    let _ = ctx.say("Nothing was playing.").await;

    Ok(())
}
