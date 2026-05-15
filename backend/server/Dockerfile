# syntax=docker/dockerfile:1
# Сборка из каталога backend/: docker build -f server/Dockerfile .
FROM rust:1-bookworm AS build
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY server ./server
WORKDIR /app/server
RUN cargo build --release --bin board_server

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates libpq5 \
  && rm -rf /var/lib/apt/lists/*
COPY --from=build /app/target/release/board_server /usr/local/bin/board_server
ENV HOST=0.0.0.0
EXPOSE 3000
ENTRYPOINT ["/usr/local/bin/board_server"]
