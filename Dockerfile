FROM rust:1.83-bullseye AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./

RUN mkdir src && \
    echo "fn main() {}" > src/main.rs

RUN cargo build --release

RUN rm -f target/release/deps/gpt-rs*
COPY . .

RUN cargo build --release

FROM debian:bullseye-slim

RUN apt-get update && \
    apt-get install -y openssl ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/gpt-rs /app/gpt-rs

EXPOSE 8080

CMD ["/app/gpt-rs"]
