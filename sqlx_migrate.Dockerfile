FROM rust:1.75.0-slim-bookworm AS builder
WORKDIR /app
RUN cargo install sqlx-cli --no-default-features --features postgres
COPY migrations migrations

CMD [ "bash", "-c", "cargo sqlx migrate run" ]