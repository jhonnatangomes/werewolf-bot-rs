FROM rust:1.69 as builder
WORKDIR /usr/src/werewolf-bot
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim
RUN apt-get update && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/werewolf-bot/target/release/werewolf-bot-rs /usr/local/bin/werewolf-bot
CMD ["werewolf-bot"]
EXPOSE 8080
