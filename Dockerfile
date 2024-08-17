FROM --platform=linux/amd64 rust:1.80-alpine AS builder

RUN apk add --no-cache musl-dev libressl-dev

WORKDIR /usr/src/app

COPY Cargo.toml Cargo.lock ./

COPY . .

RUN cargo build --release --target x86_64-unknown-linux-musl

FROM --platform=linux/amd64 alpine:3.18

RUN apk add --no-cache ca-certificates

COPY --from=builder /usr/src/app/target/x86_64-unknown-linux-musl/release/wecom-proxy /usr/local/bin

ENV RUST_LOG=info

EXPOSE 3000

CMD ["wecom-proxy"]