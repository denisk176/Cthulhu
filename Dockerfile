FROM rust:1.83-bookworm AS builder
RUN apt-get update && apt-get install -y libudev-dev && rm -rf /var/lib/apt/lists/*
WORKDIR /usr/src/cthulhu
COPY . .
RUN cargo install --path .

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libudev1 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/cthulhu /usr/local/bin/cthulhu
CMD ["cthulhu"]
