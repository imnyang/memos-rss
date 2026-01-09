# memos-rss-rs

Discord RSS monitor written in Rust.

## Features
- Periodically check RSS feeds.
- Post new items to Discord Forum channels.
- Category filtering support.
- Embedded storage using `sled`.

## Setup
1. Create a `.env` file with:
   ```
   DISCORD_TOKEN=your_token
   INTERVAL_MINUTES=5
   ```
2. Configure `rss.toml`.
3. Run with `cargo run`.

## Docker
```bash
docker run --env-file .env ghcr.io/imnyang/memos-rss/bot:latest
```