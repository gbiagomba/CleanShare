# syntax=docker/dockerfile:1

FROM rust:1.80-slim AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    cargo build --release && \
    strip target/release/cleanshare || true

FROM debian:bookworm-slim
RUN useradd -u 10001 -r -M appuser && \
    apt-get update && apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*
COPY --from=build /src/target/release/cleanshare /usr/local/bin/cleanshare
USER appuser
ENTRYPOINT ["/usr/local/bin/cleanshare"]
CMD ["--help"]

