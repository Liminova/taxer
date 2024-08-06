use std::sync::Arc;

use poise::serenity_prelude::{
    async_trait, Cache, CreateEmbed, CreateEmbedAuthor, CreateMessage, Http,
};
use songbird::tracks::PlayMode;

use crate::data::player_data::PlayerData;

#[derive(Debug)]
pub struct PlayEventHandler {
    pub player_data: Arc<PlayerData>,
    pub http: Arc<Http>,
    pub cache: Arc<Cache>,
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

        // get where the track playing from
        let player_data = self.player_data.clone();
        let guild_channel_id = player_data
            .track_2_channel
            .lock()
            .await
            .get(&track_id)?
            .clone();

        let guild_channel = {
            let (guild_id, channel_id) = (&guild_channel_id).into();
            guild_id
                .to_guild_cached(&self.cache.clone())
                .and_then(|guild| guild.channels.get(channel_id).cloned())?
        };

        // get current playing track info, if any
        let track_info = match player_data
            .playlist
            .lock()
            .await
            .get(&guild_channel_id)
            .and_then(|playlist| {
                playlist
                    .iter()
                    .find(|track_info| track_info.id == track_id)
                    .cloned()
            }) {
            Some(track_info) => track_info,
            None => {
                if let Err(e) = guild_channel
                    .send_message(
                        self.http.clone(),
                        CreateMessage::default().content("There's no track in the queue!"),
                    )
                    .await
                {
                    tracing::warn!("can't send message 'no track in queue': {}", e);
                }
                return None;
            }
        };

        // send msg to channel
        if let Err(e) = guild_channel
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
        let player_data = self.player_data.clone();
        let guild_channel_id = player_data
            .track_2_channel
            .lock()
            .await
            .get(&track_id)?
            .clone();

        // cleanup
        let mut playlists = player_data.playlist.lock().await;
        if let Some(playlist) = playlists.get_mut(&guild_channel_id) {
            playlist.retain(|track_info| track_info.id != track_id);
            if playlist.is_empty() {
                playlists.remove(&guild_channel_id);
            }
        };

        None
    }
}
