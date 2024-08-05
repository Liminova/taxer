use crate::{data::player_data::GuildChannelID, AppError, Context};

use anyhow::anyhow;
use poise::{serenity_prelude::CreateEmbed, CreateReply};
use songbird::tracks::PlayMode;
use uuid::Uuid;

/// List all tracks in the queue
#[poise::command(prefix_command, slash_command, guild_only)]
    // get GuildChannelID
    let guild_id = match ctx.guild().map(|guild| guild.id) {
pub async fn queue(ctx: Context<'_>) -> Result<(), AppError> {
        Some(guild_id) => guild_id,
        _ => {
            if let Err(e) = ctx.say("This command must be invoke in a guild!").await {
                tracing::warn!("can't send message 'guild command only': {}", e);
            }
            return Ok(());
        }
    };
    let guild_channel_id = {
        let channel_id = ctx.channel_id();
        GuildChannelID::from((guild_id, channel_id))
    };

    // get corresponding playlist
    let playlist = {
        let player_data = ctx.data().player_data.clone();
        let playlists = player_data.playlist.lock().await;
        playlists
            .get(&guild_channel_id)
            .filter(|playlist| !playlist.is_empty())
            .cloned()
    };

    if let Some(playlist) = playlist {
        // get playing track info
        let playing_track_id: Option<Uuid> = 'scoped: {
            let songbird_manager = match songbird::get(ctx.serenity_context()).await {
                Some(songbird_manager) => songbird_manager,
                    let _ = ctx.say("Can't get Songbird manager!").await;
                    return Err("commands::player::queue: songbird not loaded".into());
                _ => {
                }
            };
            let call = match songbird_manager.get(guild_id) {
                Some(call) => call,
                _ => break 'scoped None,
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

        let _ = ctx.send(CreateReply::default().embed(embed)).await;

        return Ok(());
    }

    if let Err(e) = ctx
        .send(CreateReply::default().content("It's empty").ephemeral(true))
        .await
    {
        tracing::warn!("can't send message 'queue is empty': {}", e);
    }

    Ok(())
}
