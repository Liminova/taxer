use std::path::PathBuf;

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
            r"(?gim) (\(|\[|\| )( ?(official|mv|audio|video|music|lyrics|lyric) ?)+(\)|\]|)?",
        );

        if let Ok(re) = re {
            return re.replace_all(&self.title, "").to_string();
        }
        self.title.clone()
    }

    /// Get playable direct URL of the track from Vec<Format>.
    /// If the track is not available in any of the formats, return `None`.
    pub fn get_playable_url(&self) -> Option<String> {
        if let Some(formats) = &self.formats {
            let mut best_url = None;
            let mut best_bitrate = 0.0;

            let mut mp3_url = None;
            let mut mp3_bitrate = 0.0;

            for format in formats.iter() {
                if let (Some(codec), Some(bitrate)) = (format.codec.as_deref(), format.bitrate) {
                    match codec {
                        // lossless codecs, return immediately
                        "alac" | "flac" | "pcm" => {
                            return Some(format.url.clone());
                        }
                        // lossy codecs, can't go wrong when
                        // choose one with the highest bitrate
                        "opus" | "aac" | "vorbis" => {
                            if bitrate > best_bitrate {
                                best_url = Some(format.url.clone());
                                best_bitrate = bitrate;
                            }
                        }
                        // final resort if can't find a better one
                        "mp3" => {
                            if bitrate > mp3_bitrate {
                                mp3_url = Some(format.url.clone());
                                mp3_bitrate = bitrate;
                            }
                        }
                        _ => (),
                    }
                }
            }

            return best_url.or(mp3_url);
        }

        None
    }

    /// Get the output path for `yt-dlp` to download the track.
    pub fn get_download_path(&self) -> String {
        format!("/tmp/taxer/{}", self.id)
    }

    /// Get the input path for songbird to play the track.
    pub fn get_input_path(&self) -> Result<PathBuf, String> {
        // list all files in /tmp/taxer
        let files = std::fs::read_dir("/tmp/taxer")
            .map_err(|_| "TrackInfo::to_songbird_track: failed to read /tmp/taxer")?
            .filter_map(|entry| entry.ok())
            .collect::<Vec<_>>();

        // find the file with the same ID
        let file = files.into_iter().find(|file| {
            file.file_name()
                .to_string_lossy()
                .contains(&self.id.to_string())
        });

        Ok(file
            .map(|file| file.path())
            .ok_or("TrackInfo::to_songbird_track: the track file doesn't exist")?)
    }
}
