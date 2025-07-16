FROM rust:1.87-slim

WORKDIR /app

RUN apt-get update && apt-get install -y \
    libssl-dev \
    pkg-config \
    build-essential \
    g++ \
    && rm -rf /var/lib/apt/lists/*

COPY . .
RUN cargo build --release

ENTRYPOINT ["./target/release/rust_kafka"]