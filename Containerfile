FROM docker.io/bufbuild/buf:latest AS buf-provider

FROM docker.io/rust:latest AS builder

COPY --from=buf-provider /usr/local/bin/buf /usr/local/bin/buf

RUN apt-get update && apt-get install -y pkg-config libssl-dev libpq-dev protobuf-compiler

WORKDIR /app
COPY . .

RUN cargo build --release

FROM docker.io/debian:stable-slim

RUN apt-get update && apt-get install -y libssl-dev libpq-dev ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/imbibe /usr/local/bin/imbibe
COPY config/ config/

ENTRYPOINT ["imbibe"]
