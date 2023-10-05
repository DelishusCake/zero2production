# Builder stage
FROM rust:1.72.1 AS builder

WORKDIR /app
RUN apt update && apt install lld clang -y
COPY . .
# Build the binary with SQLx in offline mode
# Make sure to run `cargo sqlx prepare --workspace` before building
ENV SQLX_OFFLINE true
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim AS runner

WORKDIR /app
# Install OpenSSL
RUN apt-get update -y \
  && apt-get install -y --no-install-recommends openssl ca-certificates \
  && apt-get autoremove -y \
  && rm -rf /var/lib/apt/lists/*
# Copy the binary from the builder container
COPY --from=builder /app/target/release/zero2prod zero2prod
COPY settings settings
# Set the app to prod mode
ENV APP_ENV prod
ENTRYPOINT ["./zero2prod"]
