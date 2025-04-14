FROM docker.io/rust:latest AS builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev libpq-dev

WORKDIR /app
COPY . .

RUN cargo build --release

FROM docker.io/debian:stable-slim

RUN apt-get update && apt-get install -y libssl-dev libpq-dev ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/imbibe /usr/local/bin/imbibe
COPY config/ config/

ENTRYPOINT ["imbibe"]
