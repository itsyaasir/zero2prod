set -x
set -eo pipefail

# Check if psql is installed
if ! [ -x  "$(command -v psql)" ]; then
  echo "Error : `psql` is not installed"
  exit 1
fi

# Check if sqlx is installed
if ! [ -x "$(command -v sqlx)" ]; then
  echo "Error : `sqlx` is not installed"
  echo >&2 "Use "
  echo >&2 "  cargo install sqlx-cli --no-default-features --features postgres"
  echo >&2 "to install sqlx"
  exit 1
fi

# check if a custom user has been set, otherwise default to "postgres"
DB_USER=${POSTGRES_USER:=postgres}
# check if a custom password has been set, otherwise default to "password"
DB_PASS=${POSTGRES_PASSWORD:=password}
# check if a custom database name has been set, otherwise default to "newsletter"
DB_NAME=${POSTGRES_DB:=newsletter}
# check if a custom port host has been set, otherwise default to "5432"
DB_PORT=${POSTGRES_PORT:=5432}

#  Allow to skip Docker if a dockerized database is already available

# Launch postgres using Docker
if [[-z "${SKIP_DOCKER}"]]
then 
docker run \
    -e POSTGRES_USER=${DB_USER} \
    -e POSTGRES_PASSWORD=${DB_PASS} \
    -e POSTGRES_DB=${DB_NAME} \
    -p ${DB_PORT}:5432 \
    -d postgres \
    postgres -N 1000
fi
    # Increased max number of connection for testing purposes
# Keep pinging postgres unti it is ready to accept commands
export PGPASSWORD=${DB_PASS}
until psql -h "localhost" -U "${DB_USER}" -p "${DB_PORT}" -d "postgres" -c '\q'; do
  >&2 echo "Postgres is unavailable - sleeping"
  sleep 1
done

>&2 echo "Postgres is up and running on port ${DB_PORT}"

export DATABASE_URL=postgres://${DB_USER}:${DB_PASS}@localhost:${DB_PORT}/${DB_NAME}

# Create the database
sqlx database create

# Create the tables
sqlx migrate run