/// Contains environment variables and other configurations.
#[derive(Debug)]
pub struct Config {
    pub yt_dlp_path: String,
    pub ffmpeg_path: String,

    pub discord_token: String,
    pub bot_maintainer_uid: String,
}

impl Config {
    fn get_env(key: &str) -> String {
        std::env::var(key).unwrap_or_else(|_| panic!("{} must be set.", key))
    }

    pub fn init() -> Self {
        Self {
            yt_dlp_path: {
                let path = Self::get_env("YT_DLP_PATH");
                if path.is_empty() {
                    tracing::error!("YT_DLP_PATH is empty");
                    std::process::exit(1);
                }
                if !std::path::Path::new(&path).exists() {
                    tracing::error!("YT_DLP_PATH points to a non-existing path");
                    std::process::exit(1);
                }
                path
            },
            ffmpeg_path: {
                let path = Self::get_env("FFMPEG_PATH");
                if path.is_empty() {
                    tracing::error!("FFMPEG_PATH is empty");
                    std::process::exit(1);
                }
                if !std::path::Path::new(&path).exists() {
                    tracing::error!("FFMPEG_PATH points to a non-existing path");
                    std::process::exit(1);
                }
                path
            },
            discord_token: {
                let path = Self::get_env("DISCORD_TOKEN");
                if path.is_empty() {
                    tracing::error!("DISCORD_TOKEN is empty");
                    std::process::exit(1);
                }
                path
            },
            bot_maintainer_uid: {
                let path = Self::get_env("BOT_MAINTAINER_UID");
                if path.is_empty() {
                    tracing::error!("BOT_MAINTAINER_UID is empty");
                    std::process::exit(1);
                }
                path
            },
        }
    }
}
