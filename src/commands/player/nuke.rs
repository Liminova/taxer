use crate::{data::player_data::GuildChannelID, AppError, Context};

use anyhow::anyhow;

/// Stop everything, clear the queue and leave the voice channel
#[poise::command(prefix_command, slash_command, guild_only)]
pub async fn nuke(ctx: Context<'_>) -> Result<(), AppError> {
    if let Err(e) = ctx.defer().await {
        return Err(AppError::from(anyhow!("can't send defer msg: {}", e)));
    }

    // cloning and creating necessary identifiers
    let player_data = ctx.data().player_data.clone();
    let guild_id = match ctx.guild().map(|guild| guild.id) {
        Some(guild_id) => guild_id,
        None => {
            if let Err(e) = ctx.say("This command must be invoke in a guild!").await {
                tracing::warn!("can't send message 'guild command only': {}", e);
            }
            return Ok(());
        }
    };
    let guild_channel_id = GuildChannelID::from((guild_id, ctx.channel_id()));

    // send nuke signal to /play commands
    let _ = ctx
        .data()
        .player_data
        .nuke_signal
        .send(guild_channel_id.clone());

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

    // stop the call and clear the queue
    call.lock().await.stop();
    if let Err(e) = songbird_manager.remove(guild_id).await {
        tracing::warn!("can't disconnect from voice channel: {}", e);
    }

    // clear global event handlers
    {
        let mut call_global_event_handler_added =
            player_data.call_global_event_handler_added.lock().await;
        if call_global_event_handler_added.contains(&guild_channel_id) {
            call.lock().await.remove_all_global_events();
            call_global_event_handler_added.remove(&guild_channel_id);
        }
    }

    {
        let mut playlists = player_data.playlist.lock().await;
        if playlists.contains_key(&guild_channel_id) {
            playlists.remove(&guild_channel_id);
        }
    }
    // clear playlist

    // clear track -> GuildChannelID map, in closure avoid long-locking
    {
        let mut track_2_channel = player_data.track_2_channel.lock().await;
        let track_id = track_2_channel
            .iter()
            .find_map(|(track_id, guild_channel_id_in_map)| {
                match guild_channel_id_in_map == &guild_channel_id {
                    true => Some(*track_id),
                    false => None,
                }
            });
        if let Some(track_id) = track_id {
            track_2_channel.remove(&track_id);
        }
    }

    if let Err(e) = ctx.say("💥 Nuked!").await {
        tracing::warn!("can't send message 'nuked': {}", e);
    }

    Ok(())
}
