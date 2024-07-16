use poise::serenity_prelude::{ChannelId, GuildChannel, GuildId};

/// This exists because GuildChannel is not hashable.
#[derive(Debug, Clone, Hash, PartialEq, Eq, Default)]
pub struct GuildChannelID {
    pub guild_id: GuildId,
    pub channel_id: ChannelId,
}

impl From<(GuildId, ChannelId)> for GuildChannelID {
    fn from(tuple: (GuildId, ChannelId)) -> Self {
        Self {
            guild_id: tuple.0,
            channel_id: tuple.1,
        }
    }
}

impl From<GuildChannel> for GuildChannelID {
    fn from(guild_channel: GuildChannel) -> Self {
        Self {
            guild_id: guild_channel.guild_id,
            channel_id: guild_channel.id,
        }
    }
}

impl From<GuildChannelID> for (GuildId, ChannelId) {
    fn from(guild_channel_id: GuildChannelID) -> Self {
        (guild_channel_id.guild_id, guild_channel_id.channel_id)
    }
}

impl<'a> From<&'a GuildChannelID> for (&'a GuildId, &'a ChannelId) {
    fn from(guild_channel_id: &'a GuildChannelID) -> Self {
        (&guild_channel_id.guild_id, &guild_channel_id.channel_id)
    }
}

impl<'a> From<&'a GuildChannelID> for &'a GuildId {
    fn from(guild_channel_id: &'a GuildChannelID) -> Self {
        &guild_channel_id.guild_id
    }
}

impl<'a> From<&'a GuildChannelID> for &'a ChannelId {
    fn from(guild_channel_id: &'a GuildChannelID) -> Self {
        &guild_channel_id.channel_id
    }
}
