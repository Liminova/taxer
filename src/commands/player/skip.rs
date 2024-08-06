use crate::{data::player_data::GuildChannelID, AppError, Context};

use anyhow::anyhow;
use poise::{
    serenity_prelude::{CreateEmbed, CreateEmbedAuthor},
    CreateReply,
};
use tracing::warn;
use uuid::Uuid;

/// Skip the current track
#[poise::command(prefix_command, slash_command, guild_only)]
pub async fn skip(ctx: Context<'_>) -> Result<(), AppError> {
    if let Err(e) = ctx.defer().await {
        warn!("can't send defer msg: {}", e);
    }

    let guild_id = match ctx.guild_id() {
        Some(guild_id) => guild_id,
        None => {
            if let Err(e) = ctx.say("This command must be invoke in a guild!").await {
                tracing::warn!("can't send message 'guild command only': {}", e);
            }
            return Ok(());
        }
    };

    let call = {
        let songbird_manager = match songbird::get(ctx.serenity_context()).await {
            Some(songbird_manager) => songbird_manager,
            None => {
                let _ = ctx.say("Can't get Songbird manager!").await;
                return Ok(());
            }
        };
        match songbird_manager.get(guild_id) {
            Some(call) => call,
            None => match songbird_manager.join(guild_id, ctx.channel_id()).await {
                Ok(call) => call,
                Err(e) => {
                    let _ = ctx.say(format!("Can't join voice channel: {}", e)).await;
                    return Ok(());
                }
            },
        }
    };

    // check songbird's queue
    {
        let call = call.lock().await;
        let queue = call.queue();
        if queue.is_empty() {
            if let Err(e) = ctx.say("There's no track in the queue!").await {
                tracing::warn!("can't send message: {}", e);
            }
            return Ok(());
        }
        queue.skip().map_err(|e| {
            AppError::from(anyhow!("commands::player::skip: can't skip track: {}", e))
        })?;
    }

    // check our own playlist queue
    let guild_channel_id = GuildChannelID::from((guild_id, ctx.channel_id()));
    let mut next_track_id: Option<Uuid> = None;
    let just_skipped_track = ctx
        .data()
        .player_data
        .clone()
        .guild_2_tracks
        .lock()
        .await
        .get_mut(&guild_channel_id)
        .and_then(|tracks| {
            if tracks.len() >= 2 {
                next_track_id = Some(tracks[1].id);
            }
            tracks.pop_front()
        });
    let just_skipped_track = match just_skipped_track {
        Some(track_info) => track_info,
        None => {
            if let Err(e) = ctx.say("There's no track in the queue!").await {
                tracing::warn!("can't send message: {}", e);
            }
            return Ok(());
        }
    };

    // replace track in track_2_guild mapping
    let player_data = ctx.data().player_data.clone();
    let mut track_2_guild = player_data.track_2_guild.lock().await;
    track_2_guild.remove(&just_skipped_track.id);
    if let Some(next_track_id) = next_track_id {
        track_2_guild.insert(next_track_id, guild_channel_id);
    }

    let mut embed = CreateEmbed::default()
        .author(CreateEmbedAuthor::new("Skipped track"))
        .title(just_skipped_track.get_title())
        .description(just_skipped_track.get_pretty_description())
        .url(&just_skipped_track.url);
    if let Some(thumbnail) = just_skipped_track.thumbnail {
        embed = embed.thumbnail(thumbnail);
    }
    ctx.send(CreateReply::default().embed(embed)).await?;

    Ok(())
}
