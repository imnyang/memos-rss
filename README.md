# memos-rss-rs

Discord RSS monitor written in Rust.

## Features
- Periodically check RSS feeds.
- Post new items to Discord Forum channels.
- Category filtering support.
- Embedded storage using `sled`.
- Slash command support.

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
docker build -t memos-rss-rs .
docker run --env-file .env memos-rss-rs
```
