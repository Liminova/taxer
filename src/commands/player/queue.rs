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

            }
        };

        }

    }

    if let Err(e) = ctx
        .send(CreateReply::default().content("It's empty").ephemeral(true))
        .await
    {
        tracing::warn!("can't send message 'queue is empty': {}", e);
    }

    Ok(())
}
