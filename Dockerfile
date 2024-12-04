FROM rust:1.83-bullseye AS builder

WORKDIR /app

COPY . .

RUN cargo build --release


FROM alpine:3.20

RUN apk add --no-cache \
    libgcc \
    openssl \
    ca-certificates

# FROM debian:bullseye-slim
#
# RUN apt-get update && \
#     apt-get install -y openssl ca-certificates && \
#     rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/gpt-rs /app/gpt-rs

EXPOSE 8080

CMD ["/app/gpt-rs"]
