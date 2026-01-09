FROM rust:slim as builder

WORKDIR /usr/src/app
COPY . .

RUN cargo build --release

FROM debian:13-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/app
COPY --from=builder /usr/src/app/target/release/memos-rss-rs .
COPY --from=builder /usr/src/app/rss.toml .

CMD ["./memos-rss-rs"]
