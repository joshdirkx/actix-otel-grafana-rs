# Use the official Rust image as the base
FROM --platform=linux/amd64 clux/muslrust:stable AS builder

# Set the working directory inside the container
WORKDIR /usr/src/app

ENV CC=musl-gcc
ENV AR=musl-ar

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml Cargo.lock ./

# Copy the source code
COPY . .

# Build the application in release mode
RUN cargo build --release --target x86_64-unknown-linux-musl

# Use a smaller base image for the runtime
FROM debian:buster-slim

# Install necessary dependencies
RUN apt-get update && apt-get install -y \
    libssl1.1 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/app/target/x86_64-unknown-linux-musl/release/actix-otel-rs .

# Expose the port the application runs on
EXPOSE 8080

# Set the environment variables for OpenTelemetry (if needed)
ENV OTEL_SERVICE_NAME=actix-otel-rs
ENV OTEL_EXPORTER_OTLP_ENDPOINT=http://your-otel-collector-endpoint:4317

# Run the application
CMD ["./actix-otel-rs"]
