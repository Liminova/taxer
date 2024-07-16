use crate::{
    data::player_data::{GuildChannelID, TrackInfo},
    Context, Error,
};

use std::{collections::VecDeque, io::BufRead, process::Command};

use poise::{
    serenity_prelude::{CreateEmbed, CreateMessage},
    CreateReply,
};
use reqwest::header::{HeaderMap, HeaderValue};
use songbird::{input::HttpRequest, tracks::Track};
use tracing::error;
use uuid::Uuid;

/// Play something
#[poise::command(prefix_command, slash_command, guild_only)]
pub async fn play(
    ctx: Context<'_>,
    #[description = "Anything `yt-dlp` supports"] url: String,
) -> Result<(), Error> {
    let url = match url.trim() {
        "" => return Err("play: URL is empty".into()),
        _ => url,
    };

    // =========================
    // 1. Getting necessary data
    // =========================

    let player_data = ctx.data().player_data.clone();
    let guild_id = ctx
        .guild()
        .ok_or("play: this command must be invoke in a guild")?
        .id;
    let voice_channel_id = ctx
        .guild()
        .ok_or("play: this command must be invoke in a guild")?
        .voice_states
        .get(&ctx.author().id)
        .and_then(|voice_state| voice_state.channel_id)
        .ok_or("play: you are not in a voice channel")?;
    let guild_channel_id = GuildChannelID::from((guild_id, ctx.channel_id()));

    let call = {
        let songbird_manager = songbird::get(ctx.serenity_context())
            .await
            .ok_or("play: songbird not loaded")?;
        match songbird_manager.get(guild_id) {
            Some(call) => call,
            None => songbird_manager
                .join(guild_id, voice_channel_id)
                .await
                .map_err(|e| format!("play: {}", e))?,
        }
    };

    // deafen the bot, in closure avoid long-locking
    {
        let mut call = call.lock().await;
        if !call.is_deaf() {
            call.deafen(true)
                .await
                .map_err(|e| format!("play: failed to deafen the bot: {}", e))?;
        }
    }

    // send initial message
    let messenger = ctx
        .send(CreateReply::default().content("Initializing `yt-dlp`..."))
        .await
        .map_err(|e| format!("play: failed to send the first message: {}", e))?;

    // add global event handlers once per guild
    if !player_data
        .call_global_event_handler_added
        .lock()
        .await
        .contains(&guild_channel_id)
    {
        let mut call = call.lock().await;
        call.add_global_event(
            songbird::Event::Track(songbird::TrackEvent::Play),
            super::track_event_handler::PlayEventHandler {
                player_data: ctx.data().player_data.clone(),
                http: ctx.serenity_context().http.clone(),
                cache: ctx.serenity_context().cache.clone(),
            },
        );
        call.add_global_event(
            songbird::Event::Track(songbird::TrackEvent::End),
            super::track_event_handler::EndEventHandler {
                player_data: ctx.data().player_data.clone(),
            },
        );
        player_data
            .call_global_event_handler_added
            .lock()
            .await
            .insert(guild_channel_id.clone());
    };

    // create channels for sending track info between threads
    // - Some(track_info): got a track info
    // - None: everything completed
    let (track_info_tx, mut track_info_rx) = tokio::sync::mpsc::channel::<Option<TrackInfo>>(1);
    // signal from ytdlp thread to break tokio::select! loop
    let (stop_tx, mut stop_rx) = tokio::sync::oneshot::channel::<String>();

    // spawn yt-dlp thread, push data through channel
    let yt_dlp_path = ctx.data().config.yt_dlp_path.clone();
    let yt_dlp_thread_handle = tokio::spawn(async move {
        // create yt-dlp process
        let mut yt_dlp_process = match Command::new(yt_dlp_path)
            .arg("-x")
            .arg("--skip-download")
            .arg("--print-json")
            .arg(&url)
            .stdout(std::process::Stdio::piped())
            .spawn()
        {
            Ok(yt_dlp_process) => yt_dlp_process,
            Err(e) => {
                stop_tx.send(format!("failed to run yt-dlp: {}", e)).ok();
                return;
            }
        };

        // read yt-dlp output
        if let Some(stdout) = yt_dlp_process.stdout.take() {
            let reader: std::io::BufReader<_> = std::io::BufReader::new(stdout);
            for line in reader.lines() {
                let line = match line {
                    Ok(line) => line,
                    Err(e) => {
                        error!("failed to read yt-dlp output: {}", e);
                        continue;
                    }
                };

                // parse & assign ID
                let mut track_info: TrackInfo = match serde_json::from_str(line.as_str()) {
                    Ok(track_info) => track_info,
                    Err(e) => {
                        error!("failed to parse yt-dlp output: {}", e);
                        continue;
                    }
                };
                track_info.id = Uuid::new_v4();

                // send new track info
                if let Err(e) = track_info_tx.send(Some(track_info)).await {
                    error!("failed to send [new track info] signal: {}", e);
                    continue;
                }
            }
        };

        // wait for yt-dlp to finish
        if let Err(e) = yt_dlp_process.wait() {
            error!("failed to wait for yt-dlp to finish: {}", e);
            stop_tx
                .send(format!("failed to wait for yt-dlp to finish: {}", e))
                .ok();
            return;
        }

        drop(yt_dlp_process);

        // if everything went well
        if let Err(e) = track_info_tx.send(None).await {
            error!("failed to send [yt-dlp exited success] signal: {}", e);
        }
    });

    // ========================================================================
    // 5. Collect incoming track info from channel, download and send to player
    // ========================================================================

    let mut track_count: usize = 0;
    let mut nuke_signal = ctx.data().player_data.nuke_signal.subscribe();
    loop {
        tokio::select! {
            Some(incoming) = track_info_rx.recv() => {
                if let Some(track_info) = incoming {
                    // try to get a playable URL first
                    let songbird_track = if let Some(direct_url) = track_info.get_playable_url() {
                        let client = ctx.data().player_data.http_client.clone();
                        let mut headers = HeaderMap::new();
                        headers.insert("User-Agent", HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/107.0.0.0 Safari/537.36"));
                        let input = HttpRequest::new_with_headers(client, direct_url, headers);
                        Some(Track::new_with_uuid(input.into(), track_info.id))
                    } else { // else download the track w/ ffmpeg to convert to aac
                        let proc = std::process::Command::new(ctx.data().config.yt_dlp_path.clone())
                            .arg("--ffmpeg-location")
                            .arg(ctx.data().config.ffmpeg_path.clone())
                            .arg("--audio-format")
                            .arg("aac")
                            .arg("--audio-quality")
                            .arg("0")
                            .arg("-x")
                            .arg("-o")
                            .arg(track_info.get_download_path())
                            .arg(&track_info.url)
                            .spawn();
                        match proc {
                            Ok(mut proc) => if let Err(e) = proc.wait() {
                                error!("failed to wait for yt-dlp to finish: {}", e);
                                continue;
                            },
                            Err(e) => {
                                error!("failed to spawn yt-dlp process to download track: {}", e);
                                continue;
                            }
                        };

                        match track_info.get_input_path() {
                            Ok(direct_url) => {
                                let input = songbird::input::File::new(direct_url);
                                Some(Track::new_with_uuid(input.into(), track_info.id))
                            }
                            Err(e) => {
                                error!("failed to get input path: {}", e);
                                None
                            },
                        }
                    };

                    let songbird_track = match songbird_track {
                        Some(songbird_track) => songbird_track,
                        None => continue,
                    };

                    // update message
                    track_count += 1;
                    let _ = messenger
                        .edit(
                            ctx,
                            CreateReply::default().content(format!(
                                "Adding `{}` track{} to the queue...",
                                track_count, if track_count > 1 { "s" } else { "" }
                            )),
                        )
                        .await
                        .map_err(|e| {
                            format!("play: failed to edit the message: {}", e)
                        });

                    { // push to global playlist, in closure avoid long-locking
                        let mut playlist = player_data.playlist.lock().await;
                        if let Some(playlist) = playlist.get_mut(&guild_channel_id) {
                            playlist.push_back(track_info.clone());
                        } else {
                            playlist.insert(guild_channel_id.clone(), VecDeque::from([track_info.clone()]));
                        };
                    }

                    // push to <track -> GuildChannelID> map
                    player_data.track_2_channel
                        .lock()
                        .await
                        .insert(track_info.id, guild_channel_id.clone());

                    { // add track to the queue, in closure avoid long-locking
                        let mut call = call.lock().await;
                        let handle = call.enqueue(songbird_track).await;
                        let _ = handle.make_playable();
                    }

                    continue;
                }

                // send final update message
                let content = match track_count {
                    0 => "No track added to the queue!".to_string(),
                    1 => "Added `1` track to the queue!".to_string(),
                    count => format!("Added `{}` tracks to the queue!", count),
                };
                messenger
                    .edit(
                        ctx,
                        CreateReply::default().content(content),
                    )
                    .await
                    .map_err(|e| {
                        yt_dlp_thread_handle.abort();
                        format!("play: failed to edit the message: {}", e)
                    })?;

                break;
            },
            Ok(err) = &mut stop_rx => {
                error!("yt-dlp thread stopped: {}", err);
                ctx.channel_id().send_message(
                    ctx.serenity_context().http.clone(),
                    CreateMessage::default().embed(
                        CreateEmbed::default()
                            .title("Error")
                            .description(err),
                    ),
                )
                .await
                .map_err(|e| {
                    yt_dlp_thread_handle.abort();
                    format!("play: failed to send the message: {}", e)
                })?;
                break;
            },
            target_guild_channel_id = nuke_signal.recv() => {
                if let Ok(target_guild_channel_id) = target_guild_channel_id {
                    if target_guild_channel_id == guild_channel_id {
                        yt_dlp_thread_handle.abort();
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}