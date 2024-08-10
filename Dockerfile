# Base layer
FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app
RUN apt update && apt install lld clang -y

# Compute a lock file
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build project dependencies -- not the app!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
ENV SQLX_OFFLINE true
RUN cargo build --release --bin zero2prod

FROM debian:bookworm-slim AS runtime
WORKDIR /app
RUN set -eux; \
    apt-get update; \
    apt-get install -y --no-install-recommends \
    ca-certificates

COPY --from=builder /app/target/release/zero2prod /usr/local/bin/zero2prod
COPY configuration configuration
ENV APP_ENVIRONMENT production
ENTRYPOINT [ "/usr/local/bin/zero2prod" ]
