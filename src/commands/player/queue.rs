use poise::{serenity_prelude::CreateEmbed, CreateReply};
use songbird::tracks::PlayMode;
use uuid::Uuid;

use crate::{data::player_data::GuildChannelID, Context, Error};

/// List all tracks in the queue
#[poise::command(prefix_command, slash_command, guild_only)]
pub async fn queue(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx
        .guild()
        .ok_or("commands::play: guild not found from where the command was invoked")?
        .id;
    let guild_channel_id = {
        let channel_id = ctx.channel_id();
        GuildChannelID::from((guild_id, channel_id))
    };

    let playlist = {
        let player_data = ctx.data().player_data.clone();
        let playlists = player_data.playlist.lock().await;
        playlists
            .get(&guild_channel_id)
            .filter(|playlist| !playlist.is_empty())
            .cloned()
    };

    if let Some(playlist) = playlist {
        // get the playing track info
        let playing_track_id: Option<Uuid> = 'scoped: {
            let songbird_manager = songbird::get(ctx.serenity_context())
                .await
                .ok_or("commands::player::pause: songbird not loaded")?;
            let call = match songbird_manager.get(guild_id) {
                Some(call) => call,
                None => break 'scoped None,
            };
            let track_handle = call.lock().await.queue().current();

            // make sure it's actually playing
            if let Some(track_handle) = track_handle {
                let is_playing = match track_handle.get_info().await {
                    Ok(info) => info.playing == PlayMode::Play,
                    Err(_) => break 'scoped None,
                };
                if is_playing {
                    break 'scoped Some(track_handle.uuid());
                }
            }
            None
        };

        // for adding to the embed later if is_some()
        let mut thumbnail = None;

        // create the embed
        let mut embed = CreateEmbed::default().title("Queue").fields(
            playlist
                .iter()
                .map(|track_info| {
                    let title = match track_info.id == playing_track_id.unwrap_or_default() {
                        true => {
                            thumbnail.clone_from(&track_info.thumbnail);
                            format!("▶️  {}", track_info.get_title())
                        }
                        false => track_info.get_title(),
                    };

                    let description = {
                        let author = track_info
                            .uploader
                            .clone()
                            .or_else(|| track_info.artist.clone())
                            .unwrap_or_else(|| "Unknown".to_string());

                        let duration = {
                            let duration_in_sec = track_info.duration_in_sec;
                            let hours = duration_in_sec / 3600;
                            let minutes = (duration_in_sec % 3600) / 60;
                            let seconds = duration_in_sec % 60;
                            format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
                        };

                        format!("{} | {} | [Source]({})", author, duration, track_info.url)
                    };

                    (title, description, false)
                })
                .collect::<Vec<_>>(),
        );

        if let Some(thumbnail) = thumbnail {
            embed = embed.thumbnail(thumbnail);
        }

        ctx.send(CreateReply::default().embed(embed))
            .await
            .map_err(|e| format!("commands::queue: failed to send the message [2]: {}", e))?;

        return Ok(());
    }

    ctx.send(CreateReply::default().content("It's empty").ephemeral(true))
        .await
        .map_err(|e| format!("commands::queue: failed to send the message [3]: {}", e))?;

    Ok(())
}
