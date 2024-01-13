FROM rust:1.75.0-slim-buster AS builder
WORKDIR /app
COPY . .
RUN cargo install --path .

FROM debian:bullseye-slim
COPY --from=builder /usr/local/cargo/bin/todo /usr/local/bin/todo
CMD ["todo"]
