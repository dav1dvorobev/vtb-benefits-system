FROM rust:1.89-bookworm AS builder

WORKDIR /app

COPY . .

RUN cargo build --release -p benefits-backend

FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/benefits-backend /app/benefits-backend
COPY stamp.png /app/stamp.png

EXPOSE 3001

CMD ["/app/benefits-backend"]
