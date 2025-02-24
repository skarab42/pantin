FROM rust:1.85-slim-bookworm AS builder

WORKDIR /app

COPY . .
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y --no-install-recommends firefox-esr && \
    apt-get upgrade && \
    apt-get clean && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/pantin_server /usr/local/bin/pantin

ENTRYPOINT ["pantin"]
