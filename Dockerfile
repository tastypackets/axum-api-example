FROM debian:bookworm-slim AS certificates
RUN apt-get update \
    && apt-get install -y --no-install-recommends openssl util-linux \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY scripts/dev-cert.sh ./scripts/dev-cert.sh
COPY scripts/docker-dev-cert.sh ./scripts/docker-dev-cert.sh
ENTRYPOINT ["bash", "scripts/docker-dev-cert.sh"]

FROM rust:1.99-bookworm AS development
RUN cargo install watchexec-cli --locked --version 2.8.0
WORKDIR /app
EXPOSE 3000
CMD ["bash", "-c", "bash scripts/dev-cert.sh generate && exec bash scripts/dev-watch.sh"]

FROM rust:1.99-bookworm AS build
WORKDIR /app
COPY Cargo.toml Cargo.lock build.rs ./
COPY src ./src
COPY migrations ./migrations
RUN cargo build --release --locked

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --system app \
    && useradd --system --gid app app
COPY --from=build /app/target/release/api-example /usr/local/bin/api-example
USER app
EXPOSE 3000
ENTRYPOINT ["api-example"]
