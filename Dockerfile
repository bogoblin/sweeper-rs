FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef

# DON'T put things here or you have to WAIT for things to install

# install stuff
RUN cargo install wasm-pack

WORKDIR /app
FROM chef AS planner

COPY ./Cargo.lock /app
COPY ./Cargo.toml /app
COPY ./crates /app/crates
COPY ./build.sh /app

# TODO This doesn't prepare the dependencies for the wasm-pack build, only for the binary build, so it's useless
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json

# Build application
COPY ./Cargo.lock /app
COPY ./Cargo.toml /app
COPY ./crates /app/crates
COPY ./build.sh /app

RUN bash /app/build.sh

FROM alpine AS runtime
RUN addgroup -S myuser && adduser -S myuser -G myuser
COPY --from=builder /app /app
USER myuser
# TODO For some reason, trying to run this executable doesn't work, but it does exist
ENTRYPOINT ["stat", "/app/target/release/sweeper-server"]
