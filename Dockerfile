FROM rust:1.81.0 AS builder
WORKDIR /usr/src/gfp

# Overview
# Copy Cargo.toml and Cargo.lock
# Create an empty main.rs file to allow cargo to compile and build dependencies
# Remove the empty main.rs and copy over the actual source code
# This way the dependencies should remain cached

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/gfp/target/release/gfp /usr/local/bin/gfp
CMD ["gfp"]
