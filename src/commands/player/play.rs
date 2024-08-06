use crate::{
    data::player_data::{GuildChannelID, TrackInfo},
    AppError, Context,
};

use std::{collections::VecDeque, io::BufRead, process::Command};

use anyhow::anyhow;
use poise::{
    serenity_prelude::{CreateEmbed, CreateMessage},
    CreateReply, ReplyHandle,
};
use songbird::{input::HttpRequest, tracks::Track};
use tracing::error;
use uuid::Uuid;

/// Play something
#[poise::command(prefix_command, slash_command, guild_only)]
pub async fn play(
    ctx: Context<'_>,
    #[description = "URLs supported by `yt-dlp` or YT search query"] query: String,
) -> Result<(), AppError> {
    if query.trim().is_empty() {
        return Err(AppError::from(anyhow!(
            "commands::player::play: query is empty, probably due to Discord's side"
        )));
    }
    let player_data = ctx.data().player_data.clone();

    // get guild/voice/text channel IDs
    let guild_id = match ctx.guild_id() {
        Some(guild_id) => guild_id,
        None => {
            if let Err(e) = ctx.say("This command must be invoke in a guild!").await {
                tracing::warn!("can't send message 'guild command only': {}", e);
            }
            return Ok(());
        }
    };

    let guild_channel_id = GuildChannelID::from((guild_id, ctx.channel_id()));

    let call = {
        let voice_channel_id = match ctx.guild().and_then(|guild| {
            guild
                .voice_states
                .get(&ctx.author().id)
                .and_then(|voice_state| voice_state.channel_id)
        }) {
            Some(voice_channel_id) => voice_channel_id,
            None => {
                let _ = ctx.say("You're not in a voice channel!").await;
                return Ok(());
            }
        };

        let songbird_manager = match songbird::get(ctx.serenity_context()).await {
            Some(songbird_manager) => songbird_manager,
            None => {
                let _ = ctx.say("Can't get Songbird manager!").await;
                return Ok(());
            }
        };
        match songbird_manager.get(guild_id) {
            Some(call) => call,
            None => match songbird_manager.join(guild_id, voice_channel_id).await {
                Ok(call) => call,
                Err(e) => {
                    let _ = ctx.say(format!("Can't join voice channel: {}", e)).await;
                    return Ok(());
                }
            },
        }
    };

    // deafen the bot
    {
        let mut call = call.lock().await;
        if !call.is_deaf() {
            if let Err(e) = call.deafen(true).await {
                tracing::warn!("can't deafen the bot: {}", e);
            }
        }
    }

    // send initial message
    if let Err(e) = ctx.defer().await {
        return Err(AppError::from(anyhow!("can't send defer msg: {}", e)));
    }
    let mut reply_handle: Option<ReplyHandle> = None;

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
            .arg("--default-search")
            .arg("ytsearch")
            .arg("--skip-download")
            .arg("--print-json")
            .arg(&query)
            .stdout(std::process::Stdio::piped())
            .spawn()
        {
            Ok(yt_dlp_process) => yt_dlp_process,
            Err(e) => {
                stop_tx.send(format!("can't run yt-dlp: {}", e)).ok();
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
                        error!("can't read yt-dlp output: {}", e);
                        continue;
                    }
                };

                // parse & assign ID
                let mut track_info: TrackInfo = match serde_json::from_str(line.as_str()) {
                    Ok(track_info) => track_info,
                    Err(e) => {
                        error!("can't parse yt-dlp output: {}", e);
                        continue;
                    }
                };
                track_info.id = Uuid::new_v4();

                // send new track info
                if let Err(e) = track_info_tx.send(Some(track_info)).await {
                    error!("can't send new track to channel: {}", e);
                    continue;
                }
            }
        };

        // wait for yt-dlp to finish
        if let Err(e) = yt_dlp_process.wait() {
            error!("can't wait for yt-dlp to finish: {}", e);
            stop_tx
                .send(format!("can't wait for yt-dlp to finish: {}", e))
                .ok();
            return;
        }

        drop(yt_dlp_process);

        // if everything went well
        if let Err(e) = track_info_tx.send(None).await {
            error!("can't send None to channel: {}", e);
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
                        Some(Track::new_with_uuid(HttpRequest::new(client, direct_url).into(), track_info.id))
                    } else { // else download the track w/ ffmpeg to convert to aac
                        if let Err(e) = ctx.say(format!("Can't get a playable URL, downloading the track...\n\
                            <@{}> You might want to update `yt-dlp` to the latest version.",
                            ctx.data().config.bot_maintainer_uid)).await {
                                tracing::warn!("can't send message 'you might want to update yt-dlp': {}", e);
                            };

                        let proc = std::process::Command::new(ctx.data().config.yt_dlp_path.clone())
                            .arg("--ffmpeg-location")
                            .arg(ctx.data().config.ffmpeg_path.clone())
                            .arg("--audio-format")
                            .arg("aac")
                            .arg("--audio-quality")
                            .arg("0")
                            .arg("-x")
                            .arg("-o")
                            .arg(track_info.get_download_path(&guild_id))
                            .arg(&track_info.url)
                            .spawn();
                        match proc {
                            Ok(mut proc) => if let Err(e) = proc.wait() {
                                error!("can't wait for yt-dlp to finish: {}", e);
                                continue;
                            },
                            Err(e) => {
                                error!("can't spawn yt-dlp process to download track: {}", e);
                                continue;
                            }
                        };

                        match track_info.get_input_path(&guild_id) {
                            Ok(direct_url) => {
                                let input = songbird::input::File::new(direct_url);
                                Some(Track::new_with_uuid(input.into(), track_info.id))
                            }
                            Err(e) => {
                                error!("can't get input path: {}", e);
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
                    let content = CreateReply::default().content(format!(
                        "Adding `{}` track{} to the queue...",
                        track_count, if track_count > 1 { "s" } else { "" }
                    ));
                    if let Some(reply_handle) = &reply_handle {
                        if let Err(e) = reply_handle.edit(ctx, content).await {
                            tracing::warn!("can't edit reply: {}", e);
                        }
                    } else {
                        match ctx.send(content).await {
                            Ok(reply_handle_) => reply_handle = Some(reply_handle_),
                            Err(e) => tracing::warn!("can't send message: {}", e),
                        }
                    }

                    { // push to global playlist
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

                    { // add track to the queue
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
                if let Some(reply_handle) = &reply_handle {
                    if let Err(e) = reply_handle.edit(ctx, CreateReply::default().content(content)).await {
                        tracing::warn!("can't edit reply: {}", e);
                    }
                }

                break;
            },
            Ok(err) = &mut stop_rx => {
                if let Err(e) = ctx.channel_id().send_message(
                    ctx.serenity_context().http.clone(),
                    CreateMessage::default().embed(
                        CreateEmbed::default()
                            .title("Error")
                            .description(err),
                    ),
                )
                .await {
                    tracing::warn!("can't send message: {}", e);
                }
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
