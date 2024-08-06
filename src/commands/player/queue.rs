use crate::{data::player_data::GuildChannelID, AppError, Context};

use anyhow::anyhow;
use poise::{serenity_prelude::CreateEmbed, CreateReply};
use songbird::tracks::PlayMode;
use uuid::Uuid;

/// List all tracks in the queue
#[poise::command(prefix_command, slash_command, guild_only)]
pub async fn queue(ctx: Context<'_>) -> Result<(), AppError> {
    let guild_id = match ctx.guild_id() {
        Some(guild_id) => guild_id,
        _ => {
            if let Err(e) = ctx.say("This command must be invoke in a guild!").await {
                tracing::warn!("can't send message 'guild command only': {}", e);
            }
            return Ok(());
        }
    };

    // get playlist
    let playlist = match ctx
        .data()
        .player_data
        .playlist
        .lock()
        .await
        .get(&GuildChannelID::from((guild_id, ctx.channel_id())))
        .filter(|playlist| !playlist.is_empty())
        .cloned()
    {
        Some(playlist) => playlist,
        None => {
            ctx.send(CreateReply::default().content("It's empty").ephemeral(true))
                .await
                .map_err(|e| {
                    AppError::from(anyhow!(
                        "commands::player::queue: can't send message: {}",
                        e
                    ))
                })?;
            return Ok(());
        }
    };

    // get playing track id
    let playing_track_id: Option<Uuid> = 'scoped: {
        let songbird_manager = match songbird::get(ctx.serenity_context()).await {
            Some(songbird_manager) => songbird_manager.clone(),
            _ => {
                return Err(AppError::from(anyhow!(
                    "commands::player::queue: songbird not loaded"
                )))
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

    // create the embed
    let mut thumbnail = None;
    let mut embed = CreateEmbed::default().title("Queue").fields(
        playlist
            .iter()
            .map(|track_info| {
                (
                    format!(
                        "{}{}",
                        match track_info.id == playing_track_id.unwrap_or_default() {
                            true => {
                                thumbnail.clone_from(&track_info.thumbnail);
                                "▶️  "
                            }
                            false => "",
                        },
                        track_info.get_title()
                    ),
                    format!(
                        "{} | [Source]({})",
                        track_info.get_pretty_description(),
                        track_info.url
                    ),
                    false,
                )
            })
            .collect::<Vec<_>>(),
    );
    if let Some(thumbnail) = thumbnail {
        embed = embed.thumbnail(thumbnail);
    }

    // let _ = ctx.send(CreateReply::default().embed(embed)).await;
    ctx.send(CreateReply::default().embed(embed))
        .await
        .map_err(|e| {
            AppError::from(anyhow!(
                "commands::player::queue: can't send message: {}",
                e
            ))
        })?;

    Ok(())
}
