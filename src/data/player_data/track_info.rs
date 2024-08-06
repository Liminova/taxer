use std::path::PathBuf;

use poise::serenity_prelude::GuildId;
use uuid::Uuid;

/// Stores info about formats in a track.
#[derive(Debug, serde::Deserialize, Clone)]
pub struct Format {
    pub url: String,
    #[serde(rename = "acodec")]
    pub codec: Option<String>,
    #[serde(rename = "abr")]
    pub bitrate: Option<f32>,
}

/// Stores info about a track.
#[derive(Debug, serde::Deserialize, Clone)]
pub struct TrackInfo {
    // Functional fields
    #[serde(skip)]
    pub id: Uuid,
    #[serde(rename = "original_url")]
    pub url: String,
    formats: Option<Vec<Format>>,

    // Cosmetic fields
    #[serde(rename = "duration")]
    pub duration_in_sec: u64,
    title: String,
    pub thumbnail: Option<String>,
    pub artist: Option<String>,
    pub uploader: Option<String>,
}

impl Default for TrackInfo {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            url: "".to_string(),
            formats: None,

            duration_in_sec: 0,
            title: "".to_string(),
            thumbnail: None,
            artist: None,
            uploader: None,
        }
    }
}

impl TrackInfo {
    /// Get a cleaned up title.
    pub fn get_title(&self) -> String {
        // use https://regexr.com/ to build the regex, using these test cases,
        // space prefix included; remember to add them here if you find a new one
        //
        //  (official)
        //  (official mv)
        //  | official mv
        //  (official audio)
        //  [official audio]
        //  (official video)
        //  [official video]
        //  (official music video)
        //  [official music video]
        //  | official music video
        //  (official lyric video)
        //  | official lyric video
        //  [official lyric video]
        //  | lyrics music video
        //  | lyrics video
        let re = regex::Regex::new(
            r"(?im) (\(|\[|\| )( ?(official|mv|audio|video|music|lyrics|lyric) ?)+(\)|\]|)?",
        );

        if let Ok(re) = re {
            return re.replace_all(&self.title, "").to_string();
        }
        self.title.clone()
    }

    /// Get description for Discord embed.
    pub fn get_pretty_description(&self) -> String {
        let author = self
            .artist
            .clone()
            .or_else(|| self.uploader.clone())
            .unwrap_or("Unknown".to_string());

        let duration = {
            let duration_in_sec = self.duration_in_sec;
            let hours = duration_in_sec / 3600;
            let minutes = (duration_in_sec % 3600) / 60;
            let seconds = duration_in_sec % 60;
            format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
        };

        format!("{} | {}", author, duration)
    }

    /// Get playable direct URL of the track from Vec<Format>.
    /// If the track is not available in any of the formats, return `None`.
    pub fn get_playable_url(&self) -> Option<String> {
        let formats = self.formats.as_ref()?;

        let mut best_url = None;
        let mut best_bitrate = 0.0;

        let mut mp3_url = None;
        let mut mp3_bitrate = 0.0;

        formats
            .iter()
            .filter_map(|format| match (format.codec.as_deref(), format.bitrate) {
                (Some(codec), Some(bitrate)) => Some((codec, bitrate, &format.url)),
                _ => None,
            })
            .for_each(|(codec, bitrate, url)| match codec {
                "alac" | "flac" | "pcm" => {
                    best_url = Some(url.clone());
                    best_bitrate = bitrate;
                }
                "opus" | "aac" | "vorbis" => {
                    if bitrate > best_bitrate {
                        best_url = Some(url.clone());
                        best_bitrate = bitrate;
                    }
                }
                // final resort if can't find a better one
                "mp3" => {
                    if bitrate > mp3_bitrate {
                        mp3_url = Some(url.clone());
                        mp3_bitrate = bitrate;
                    }
                }
                _ => (),
            });

        best_url.or(mp3_url)
    }

    /// Get the output path for `yt-dlp` to download the track.
    pub fn get_download_path(&self, guild_id: &GuildId) -> String {
        format!("/tmp/taxer/{}/{}", guild_id, self.id)
    }

    /// Get the input path for songbird to play the track.
    pub fn get_input_path(&self, guild_id: &GuildId) -> Result<PathBuf, String> {
        std::fs::read_dir(format!("/tmp/taxer/{}", guild_id))
            .map_err(|_| {
                format!(
                    "TrackInfo::get_input_path: can't read /tmp/taxer/{}",
                    guild_id
                )
            })?
            .filter_map(|entry| entry.ok())
            .find(|file| {
                file.file_name()
                    .to_string_lossy()
                    .contains(&self.id.to_string())
            })
            .map(|file| file.path())
            .ok_or(format!(
                "TrackInfo::get_input_path: the track file doesn't exist: {}",
                self.id
            ))
    }
}
