mod guild_channel_id;
mod track_info;

pub use guild_channel_id::GuildChannelID;
pub use track_info::TrackInfo;

use std::collections::{HashMap, HashSet, VecDeque};

use tokio::sync::{broadcast, Mutex};
use uuid::Uuid;

#[derive(Debug)]
pub struct PlayerData {
    /// A "flag" to indicate if the track end global event handler has been
    /// added for any given guild-channel, so that users can execute the play
    /// command multiple times without creating duplicate event handlers.
    pub call_global_event_handler_added: Mutex<HashSet<GuildChannelID>>,

    /// Mapping track's id to GuildChannelID since it's the only thing
    /// songbird's context provides inside the EventHandler trait.
    pub track_2_guild: Mutex<HashMap<Uuid, GuildChannelID>>,

    /// The playlist for each Guild, this is just for displaying the
    /// playlist in the message. We'll be using the songbird's queue
    /// for actually queueing the tracks.
    pub guild_2_tracks: Mutex<HashMap<GuildChannelID, VecDeque<TrackInfo>>>,

    /// The reqwest client used for downloading the track
    /// when yt-dlp being able to use playable direct url.
    pub http_client: reqwest::Client,

    /// Send a signal when users execute /nuke, this is used to
    /// stop the play commands that are currently fetching new tracks.
    pub nuke_signal: broadcast::Sender<GuildChannelID>,
}

impl Default for PlayerData {
    fn default() -> Self {
        Self {
            call_global_event_handler_added: Mutex::new(HashSet::new()),
            track_2_guild: Mutex::new(HashMap::new()),
            guild_2_tracks: Mutex::new(HashMap::new()),
            http_client: reqwest::Client::new(),
            nuke_signal: broadcast::channel::<GuildChannelID>(1).0,
        }
    }
}
