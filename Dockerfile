FROM rust:1.94-bookworm AS builder
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY templates ./templates
COPY migrations ./migrations

RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/book_router /usr/local/bin/book_router

ENV BIND_ADDR=0.0.0.0:3000
ENV EBOOKS_ROOT=/ebooks
ENV AUDIOBOOKS_ROOT=/audiobooks
ENV DATABASE_PATH=/data/book-router/book-router.sqlite

EXPOSE 3000

CMD ["book_router"]
