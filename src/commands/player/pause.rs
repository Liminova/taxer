use poise::CreateReply;
use songbird::tracks::PlayMode;

use crate::{Context, Error};

/// Pause/resume the current track
#[poise::command(prefix_command, slash_command, guild_only)]
pub async fn pause(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;

    let guild_id = ctx
        .guild()
        .ok_or("commands::player::pause: guild not found from where the command was invoked")?
        .id;

    let songbird_manager = songbird::get(ctx.serenity_context())
        .await
        .ok_or("commands::player::pause: songbird not loaded")?;

    let call = match songbird_manager.get(guild_id) {
        Some(call) => call,
        None => {
            ctx.send(CreateReply::default().content("Not in a voice channel"))
                .await?;
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
                ctx.send(CreateReply::default().content("Nothing was playing"))
                    .await
                    .map_err(|e| {
                        format!(
                            "commands::player::pause: can't send ephemeral message: {}",
                            e,
                        )
                    })?;
                return Ok(());
            }
        }
        .map_err(|e| format!("commands::player::pause: can't change playing state: {}", e))?;

        ctx.send(CreateReply::default().content(match was_playing {
            true => "⏸️ Paused",
            false => "▶️ Resumed",
        }))
        .await
        .map_err(|e| {
            format!(
                "commands::player::pause: can't send ephemeral message: {}",
                e
            )
        })?;

        return Ok(());
    }

    ctx.send(CreateReply::default().content("Nothing is playing"))
        .await
        .map_err(|e| {
            format!(
                "commands::player::pause: can't send ephemeral message: {}",
                e,
            )
        })?;

    Ok(())
}
