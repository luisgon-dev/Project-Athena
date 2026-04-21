# syntax=docker/dockerfile:1.7

FROM node:22-bookworm-slim AS frontend-builder
WORKDIR /app/frontend

COPY frontend/package.json frontend/package-lock.json ./
RUN --mount=type=cache,target=/root/.npm \
    npm ci --ignore-scripts

COPY frontend ./
RUN npm run build

FROM lukemathwalker/cargo-chef:latest-rust-1.94-bookworm AS chef
WORKDIR /app

FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo chef cook --release --recipe-path recipe.json

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo build --release \
    && cp /app/target/release/book_router /app/book_router

FROM debian:bookworm-slim
WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates gosu passwd tzdata \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/book_router /usr/local/bin/book_router
COPY --from=frontend-builder /app/frontend/build /app/frontend/build
COPY docker/entrypoint.sh /usr/local/bin/docker-entrypoint.sh

RUN chmod 755 /usr/local/bin/docker-entrypoint.sh

ENV BIND_ADDR=0.0.0.0:3000 \
    EBOOKS_ROOT=/ebooks \
    AUDIOBOOKS_ROOT=/audiobooks \
    DATABASE_PATH=/config/book-router.sqlite \
    UMASK=002

EXPOSE 3000
VOLUME ["/config", "/ebooks", "/audiobooks"]

ENTRYPOINT ["docker-entrypoint.sh"]
CMD ["book_router"]
