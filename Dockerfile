# -- Version 1
# # We use the latest Rust stable realease as a baseImage
# FROM rust:1.58.0

# # Let's switch our working directory to `app` 
# # The app folder will be created for us by Docker in case it does not exist already
# WORKDIR /app

# # COPY all the files from our working environment to our Docker image
# COPY . . 

# # Run sqlx offline
# ENV SQLX_OFFLINE true

# # Let's build our bin!
# # We will use the release profile to make it faast
# RUN cargo build --release
# # App environment
# ENV APP_ENVIRONMENT production
# # When `docker run` is executed , launch the binary
# ENTRYPOINT ["./target/release/zero2prod"]


# # build a docker image tagged as "zero2prod" according to the recipe specified in Docker file
# # `docker build --tage zero2prod --file Dockerfile .`


# # -- Version 2
# # Builder Stage
# FROM rust:1.58.0 AS builder 

# WORKDIR /app
# COPY . .
# ENV SQLX_OFFLINE true
# RUN cargo build --release

# #RUN TIME STAGE
# # verision 2 : FROM rust:1.58.0 As runtime
# # For version 3 we can use the slim image of the rust

# FROM rust:1.58.0-slim As runtime
# WORKDIR /app

# # Copy the compiled binary from the builder environment
# # to our runtime Environment
# COPY --from=builder /app/target/release/zero2prod zero2prod

# # We need config file at a runtime
# COPY configuration configuration
# ENV APP_ENVIRONMENT production
# ENTRYPOINT [ "./zero2prod" ]

# # runtime is our final image
# # the builder stage does not contribute to its size , - it is an  intermediate step and is discarded at the end of the build. 
# # the only peice of the builder stage that is found in the final artifact is what we explicitly copy over - the compiled binary


# --Version 4

# Builder Stage
# FROM rust:1.58.0 AS builder 

# WORKDIR /app
# COPY . .
# ENV SQLX_OFFLINE true
# RUN cargo build --release

# #RUN TIME STAGE
# FROM debian:bullseye-slim As runtime
# WORKDIR /app
# # Install OPENSSL - it is dynamically linked by some of our dependencies
# RUN apt-get update -y \
#     && apt-get install -y --no-install-recommends openssl \
#     # Clean up
#     && apt-get autoremove -y \
#     && apt-get clean -y \
#     && rm -rf /var/lib/apt/lists/*
# # Copy the compiled binary from the builder environment
# # to our runtime Environment
# COPY --from=builder /app/target/release/zero2prod zero2prod

# # We need config file at a runtime
# COPY configuration configuration
# ENV APP_ENVIRONMENT production
# ENTRYPOINT [ "./zero2prod" ]

# # runtime is our final image
# # the builder stage does not contribute to its size , - it is an  intermediate step and is discarded at the end of the build. 
# # the only peice of the builder stage that is found in the final artifact is what we explicitly copy over - the compiled binary



# Version 5
FROM lukemathwalker/cargo-chef:latest-rust-1.58.0 as chef
WORKDIR /app

FROM chef as planner
COPY . .
# Compute a lock-like file for our project
RUN cargo chef prepare  --recipe-path recipe.json

FROM chef as builder
COPY --from=planner /app/recipe.json recipe.json
# Build our project dependencies, not our application!
RUN cargo chef cook --release --recipe-path recipe.json
# Up to this point, if our dependency tree stays the same,
# all layers should be cached. 
COPY . .
ENV SQLX_OFFLINE true
# Build our project
RUN cargo build --release --bin zero2prod

FROM debian:bullseye-slim AS runtime
WORKDIR /app
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl \
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/zero2prod zero2prod
COPY configuration configuration
ENV APP_ENVIRONMENT production
ENTRYPOINT ["./zero2prod"]