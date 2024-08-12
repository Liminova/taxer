use std::sync::Arc;

use poise::serenity_prelude::{
    async_trait, ChannelId, CreateEmbed, CreateEmbedAuthor, CreateMessage, Http,
};
use songbird::tracks::PlayMode;
use tracing::warn;

use crate::data::player_data::PlayerData;

#[derive(Debug)]
pub struct PlayEventHandler {
    pub player_data: Arc<PlayerData>,
    pub http: Arc<Http>,
}

#[async_trait]
impl songbird::EventHandler for PlayEventHandler {
    async fn act(&self, ctx: &songbird::EventContext<'_>) -> Option<songbird::Event> {
        // get the just started track's id
        let track_id = {
            let (track_state, track_handle) = match ctx {
                songbird::EventContext::Track(track) => track,
                _ => return None,
            }[0];
            if track_state.playing != PlayMode::Play {
                return None;
            }
            track_handle.uuid()
        };

        // track ID -> guild ID
        let guild_id = match self
            .player_data
            .track_2_guild
            .lock()
            .await
            .get(&track_id)
            .cloned()
        {
            Some(guild_id) => guild_id,
            None => {
                warn!("track_2_guild doesn't contain track_id");
                return None;
            }
        };

        // guild ID -> tracks[track ID] -> track info
        let track_info = match self
            .player_data
            .guild_2_tracks
            .lock()
            .await
            .get(&guild_id)
            .and_then(|tracks| {
                tracks
                    .iter()
                    .find(|track_info| track_info.id == track_id)
                    .cloned()
            }) {
            Some(track_info) => track_info,
            None => {
                warn!("guild_2_tracks doesn't contain guild_id to get track_info");
                return None;
            }
        };

        let channel_id: ChannelId = match track_info.text_channel_id {
            Some(channel_id) => channel_id,
            None => {
                warn!("track_info.text_channel_id is None");
                return None;
            }
        };

        // send msg to channel
        if let Err(e) = channel_id
            .send_message(
                self.http.clone(),
                CreateMessage::default().embed({
                    let mut embed = CreateEmbed::default()
                        .author(CreateEmbedAuthor::new("Now playing"))
                        .title(track_info.get_title())
                        .description(track_info.get_pretty_description())
                        .url(&track_info.url);

                    if let Some(thumbnail) = track_info.thumbnail.clone() {
                        embed = embed.thumbnail(thumbnail);
                    }
                    embed
                }),
            )
            .await
        {
            tracing::warn!("can't send message 'now playing': {}", e);
        };

        None
    }
}

#[derive(Debug)]
pub struct EndEventHandler {
    pub player_data: Arc<PlayerData>,
}

#[async_trait]
impl songbird::EventHandler for EndEventHandler {
    async fn act(&self, ctx: &songbird::EventContext<'_>) -> Option<songbird::Event> {
        // get the just ended track's id
        let track_id = {
            let track = match ctx {
                songbird::EventContext::Track(track) => track,
                _ => return None,
            };
            let (track_state, track_handle) = track[0];
            match track_state.playing {
                PlayMode::Pause | PlayMode::Play => return None,
                _ => (),
            };
            track_handle.uuid()
        };

        // get where the track ended from
        let guild_id = *self.player_data.track_2_guild.lock().await.get(&track_id)?;

        // cleanup
        let mut guild_2_tracks = self.player_data.guild_2_tracks.lock().await;
        if let Some(tracks) = guild_2_tracks.get_mut(&guild_id) {
            match tracks.len() {
                0 | 1 => {
                    guild_2_tracks.remove(&guild_id);
                }
                _ => tracks.retain(|track_info| track_info.id != track_id),
            };
        };
        self.player_data
            .track_2_guild
            .lock()
            .await
            .remove(&track_id);

        None
    }
}
