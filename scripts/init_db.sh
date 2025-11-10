#!/usr/bin/env bash

# Prints command before it is executed (for debugging purpose)
set -x

# Immediately exit if any command has a non-zero exit status
set -e

# Set status to the failed one in pipe
set -o pipefail

# Choose tool to run container depending on whether the system is Linux/Mac
case "$(uname)" in
    Darwin) CONTAINER_TOOL=container
    ;;
    Linux) CONTAINER_TOOL=docker
    ;;
    *) exit 1
    # ;;
esac

case "$(uname)" in
    Darwin) container system status
    ;;
    Linux) docker system info
    ;;
esac

DB_IMAGE="${POSTGRES_IMAGE:=postgres:18.0-alpine3.22}"
DB_USER="${POSTGRES_USER:=postgres}"
DB_PASSWORD="${POSTGRES_PASSWORD:=mysecretpassword}"
DB_NAME="${POSTGRES_DB:=newsletter}"
DB_PORT="${POSTGRES_PORT:=5432}"
DB_HOST="${POSTGRES_HOST:=localhost}"

if [[ -z "${SKIP_DOCKER}" ]]
then
    "${CONTAINER_TOOL}" run \
        -e POSTGRES_USER="${DB_USER}" \
        -e POSTGRES_PASSWORD="${DB_PASSWORD}" \
        -e POSTGRES_DB="${DB_NAME}" \
        -p "${DB_PORT}":5432 \
        -d "${DB_IMAGE}" \
        postgres -N 1000
fi

if ! [ -x "$(command -v psql)" ]; then
    echo >&2 "Error: psql is not installed."
    exit 1
fi

if ! [ -x "$(command -v sqlx)" ]; then
    echo >&2 "Error: sqlx is not installed."
    echo >&2 "Use:"
    echo >&2 " cargo install sqlx-cli
    --no-default-features --features native-tls,postgres"
    echo >&2 "to install it."
    exit 1
fi

export PGPASSWORD="${DB_PASSWORD}"
until psql -h "${DB_HOST}" -U "${DB_USER}" -p "${DB_PORT}" -d "postgres" -c '\q'; do
    >&2 echo "Postgres is still unavailable - sleeping"
    sleep 1
done

DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}
export DATABASE_URL

sqlx database create
sqlx migrate run

>&2 echo "Postgres has been migrated, ready to go!"
