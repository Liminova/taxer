pub mod config;
pub mod player_data;

use config::Config;
use player_data::PlayerData;

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use poise::serenity_prelude::ShardManager;
pub struct Data {
    pub config: Config,
    pub player_data: Arc<PlayerData>,
    pub shard_manager: Arc<ShardManager>,
    pub start_time: u64,
}

impl Data {
    /// Create a new [`Data`] instance.
    pub fn new(config: Config, shard_manager: Arc<ShardManager>) -> Self {
        Self {
            config,
            player_data: Arc::new(PlayerData::default()),
            shard_manager,
            start_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs(),
        }
    }
}
