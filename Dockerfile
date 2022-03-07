# We use the latest Rust stable realease as a baseImage
FROM rust:1.58.0

# Let's switch our working directory to `app` 
# The app folder will be created for us by Docker in case it does not exist already
WORKDIR /app

# COPY all the files from our working environment to our Docker image
COPY . . 

# Run sqlx offline
ENV SQLX_OFFLINE true

# Let's build our bin!
# We will use the release profile to make it faast
RUN cargo build --release
# When `docker run` is executed , launch the binary
ENTRYPOINT ["./target/release/zero2prod"]


# build a docker image tagged as "zero2prod" according to the recipe specified in Docker file
# `docker build --tage zero2prod --file Dockerfile .`