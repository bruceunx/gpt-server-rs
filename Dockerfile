FROM rust:1.83-alpine3.20 AS builder

RUN apk add --no-cache \
    musl-dev \
    openssl-dev \
    build-base \
    pkgconfig


WORKDIR /app

COPY . .

RUN cargo build --release

FROM alpine:3.20

RUN apk add --no-cache \
    openssl \
    ca-certificates \
    libgcc

WORKDIR /app

COPY --from=builder /app/target/release/gpt-rs /app/gpt-rs

EXPOSE 8080

CMD ["/app/gpt-rs"]
