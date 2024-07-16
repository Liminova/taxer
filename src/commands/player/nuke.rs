use poise::CreateReply;
use tracing::error;

use crate::{data::player_data::GuildChannelID, Context, Error};

/// Stop everything, clear the queue and leave the voice channel
#[poise::command(prefix_command, slash_command, guild_only)]
pub async fn nuke(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;

    // cloning and creating necessary identifiers
    let player_data = ctx.data().player_data.clone();
    let guild_id = ctx
        .guild()
        .ok_or("stop: guild not found from where the command was invoked")?
        .id;
    let guild_channel_id = GuildChannelID::from((guild_id, ctx.channel_id()));

    // send nuke signal to /play commands
    let _ = ctx
        .data()
        .player_data
        .nuke_signal
        .send(guild_channel_id.clone());

    let songbird_manager = songbird::get(ctx.serenity_context())
        .await
        .ok_or("stop: songbird not loaded")?;

    let call = match songbird_manager.get(guild_id) {
        Some(call) => call,
        None => {
            if let Err(e) = ctx
                .send(CreateReply::default().content("Not in a voice channel"))
                .await
            {
                error!("failed to send the message: {}", e);
            };
            return Ok(());
        }
    };

    // stop the call and clear the queue
    call.lock().await.stop();
    let _ = songbird_manager.remove(guild_id).await;

    // clear global event handlers, in closure avoid long-locking
    {
        let mut call_global_event_handler_added =
            player_data.call_global_event_handler_added.lock().await;
        if call_global_event_handler_added.contains(&guild_channel_id) {
            call.lock().await.remove_all_global_events();
            call_global_event_handler_added.remove(&guild_channel_id);
        }
    }

    // clear playlist, in closure avoid long-locking
    {
        let mut playlists = player_data.playlist.lock().await;
        if playlists.contains_key(&guild_channel_id) {
            playlists.remove(&guild_channel_id);
        }
    }

    // clear track -> GuildChannelID map, in closure avoid long-locking
    {
        let mut track_2_channel = player_data.track_2_channel.lock().await;
        let track_id = track_2_channel
            .iter()
            .find_map(|(track_id, guild_channel_id_in_map)| {
                if guild_channel_id_in_map == &guild_channel_id {
                    Some(*track_id)
                } else {
                    None
                }
            });
        if let Some(track_id) = track_id {
            track_2_channel.remove(&track_id);
        }
    }

    ctx.say("💥 Nuked!").await?;

    Ok(())
}
