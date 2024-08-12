mod track_info;

pub use track_info::TrackInfo;

use std::collections::{HashMap, HashSet, VecDeque};

use poise::serenity_prelude::GuildId;
use tokio::sync::{broadcast, Mutex};
use uuid::Uuid;

#[derive(Debug)]
pub struct PlayerData {
    /// A "flag" to indicate if the track end global event handler has been
    /// added for any given guild-channel, so that users can execute the play
    /// command multiple times without creating duplicate event handlers.
    pub call_global_event_handler_added: Mutex<HashSet<GuildId>>,

    /// Mapping track ID to channel ID since it's the only thing
    /// songbird's context provides inside the EventHandler trait.
    pub track_2_guild: Mutex<HashMap<Uuid, GuildId>>,

    /// Just for displaying the playlist in the message since songbird's
    /// queue is out of order.
    pub guild_2_tracks: Mutex<HashMap<GuildId, VecDeque<TrackInfo>>>,

    /// The reqwest client used for downloading the track
    /// when yt-dlp being able to use playable direct url.
    pub http_client: reqwest::Client,

    /// Send a signal when users execute /nuke, this is used to
    /// stop the play commands that are currently fetching new tracks.
    pub nuke_signal: broadcast::Sender<GuildId>,
}

impl Default for PlayerData {
    fn default() -> Self {
        Self {
            call_global_event_handler_added: Mutex::new(HashSet::new()),
            track_2_guild: Mutex::new(HashMap::new()),
            guild_2_tracks: Mutex::new(HashMap::new()),
            http_client: reqwest::Client::new(),
            nuke_signal: broadcast::channel::<GuildId>(1).0,
        }
    }
}
