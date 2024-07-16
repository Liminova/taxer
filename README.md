# taxer

## pre-requisites
- `yt-dlp`
- `ffmpeg`
- `discord_bot_token`

## usage
- clone this repo
- `cp docker-compose.example.yml docker-compose.yml`
- edit `docker-compose.yml` with your discord bot token & `yt-dlp`, `ffmpeg` path
- `docker compose up -d`

## update
```bash
docker compose down && git pull && docker compose up -d --build
```

## license

licensed under either of

*   Apache License, Version 2.0
    ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
*   MIT license
	([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.

## contribution

unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
