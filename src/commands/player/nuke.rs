use crate::{AppError, Context};

use anyhow::anyhow;

/// Stop everything, clear the queue and leave the voice channel
#[poise::command(prefix_command, slash_command, guild_only)]
pub async fn nuke(ctx: Context<'_>) -> Result<(), AppError> {
    if let Err(e) = ctx.defer().await {
        return Err(AppError::from(anyhow!("can't send defer msg: {}", e)));
    }

    // cloning and creating necessary identifiers
    // let player_data = ctx.data().player_data.clone();
    let guild_id = match ctx.guild().map(|guild| guild.id) {
        Some(guild_id) => guild_id,
        None => {
            if let Err(e) = ctx.say("This command must be invoke in a guild!").await {
                tracing::warn!("can't send message 'guild command only': {}", e);
            }
            return Ok(());
        }
    };

    // send nuke signal to any running /play command
    let _ = ctx.data().player_data.nuke_signal.send(guild_id);

    let songbird_manager = match songbird::get(ctx.serenity_context()).await {
        Some(songbird_manager) => songbird_manager,
        None => {
            return Err(AppError::from(anyhow!(
                "commands::player::nuke: songbird not loaded"
            )));
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

    // stop the call and clear songbird's queue
    call.lock().await.stop();
    if let Err(e) = songbird_manager.remove(guild_id).await {
        tracing::warn!("can't disconnect from voice channel: {}", e);
    }

    // clear global event handlers
    {
        let mut call_global_event_handler_added = ctx
            .data()
            .player_data
            .call_global_event_handler_added
            .lock()
            .await;
        if call_global_event_handler_added.contains(&guild_id) {
            call.lock().await.remove_all_global_events();
            call_global_event_handler_added.remove(&guild_id);
        }
    }

    // clear guild_2_tracks
    ctx.data()
        .player_data
        .guild_2_tracks
        .lock()
        .await
        .remove(&guild_id);

    // clear track_2_guild
    ctx.data()
        .player_data
        .track_2_guild
        .lock()
        .await
        .retain(|_, guild_id_in_map| guild_id_in_map != &guild_id);

    // clear temp dir
    if let Err(e) = std::fs::remove_dir_all(format!("/tmp/taxer/{}", guild_id)) {
        tracing::warn!("can't remove temp dir: {}", e);
    }

    if let Err(e) = ctx.say("💥 Nuked!").await {
        tracing::warn!("can't send message 'nuked': {}", e);
    }

    Ok(())
}
