pub mod config;
pub mod player_data;

use config::Config;
use player_data::PlayerData;

use std::sync::Arc;

use poise::serenity_prelude::ShardManager;
pub struct Data {
    pub config: Config,
    pub player_data: Arc<PlayerData>,
    pub shard_manager: Arc<ShardManager>,
}

impl Data {
    /// Create a new [`Data`] instance.
    pub fn new(config: Config, shard_manager: Arc<ShardManager>) -> Self {
        Self {
            config,
            player_data: Arc::new(PlayerData::default()),
            shard_manager,
        }
    }
}
