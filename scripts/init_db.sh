#!/usr/bin/env bash
set -x
set -eo pipefail

if ! [ -x "$(command -v sqlx)" ]; then
  echo >&2 "ERROR: sqlx not installed"
  echo >&2 "Run:"
  echo >&2 "  cargo install sqlx-cli --no-default-features --features rustls,postgres"
  exit 1
fi

DB_USER="${POSTGRES_USER:=postgres}"
DB_PASS="${POSTGRES_PASSWORD:=password}"
DB_NAME="${POSTGRES_DB:=newsletter}"
DB_PORT="${POSTGRES_PORT:=5432}"
DB_HOST="${POSTGRES_HOST:=localhost}"

if [[ -z "${SKIP_DOCKER}" ]]
then
  # Create the postgres container with more connections for testing
  docker run \
    -e POSTGRES_USER=${DB_USER} \
    -e POSTGRES_PASSWORD=${DB_PASS} \
    -e POSTGRES_DB=${DB_NAME} \
    -p "${DB_PORT}":5432 \
    -d postgres \
    postgres -N 1000
fi

# Create the .env file for SQLx to use
echo "DATABASE_URL=postgres://${DB_USER}:${DB_PASS}@${DB_HOST}:${DB_PORT}/${DB_NAME}" > .env
# Have sqlx create the database
sqlx database create
sqlx migrate run
