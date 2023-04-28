FROM rust:1.69 as builder
WORKDIR /usr/src/werewolf-bot
COPY . .
RUN cargo install --path .

FROM debian:bullseye-slim
RUN apt-get update && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/werewolf-bot-rs /usr/local/bin/werewolf-bot
CMD ["werewolf-bot"]
EXPOSE 8080
