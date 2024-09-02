FROM rust:1.80-alpine AS builder

WORKDIR /usr/src/app

RUN apk add --no-cache musl-dev

COPY Cargo.toml Cargo.lock ./

COPY . .

RUN cargo build --release

FROM alpine:3.20

RUN apk add --no-cache libgcc

COPY --from=builder /usr/src/app/target/release/actix-otel-rs .

EXPOSE 8080

ENV OTEL_SERVICE_NAME=actix-otel-rs
ENV OTEL_EXPORTER_OTLP_ENDPOINT=http://otel-collector:4317

CMD ["./actix-otel-rs"]