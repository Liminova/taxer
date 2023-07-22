use crate::{Context, Error};
use youtube_dl::YoutubeDl;

#[poise::command(prefix_command, slash_command)]
pub async fn play(
    ctx: Context<'_>,
    #[description = "playback anything that yt-dlp supports"] url: String,
) -> Result<(), Error> {
    let output = YoutubeDl::new(url).socket_timeout("15").run_async().await?;
    let title = output.into_single_video().unwrap().title;

    ctx.send(|b| b.embed(|e| e.title("Now Playing").description(format!("{}", title))))
        .await?;

    Ok(())
}
