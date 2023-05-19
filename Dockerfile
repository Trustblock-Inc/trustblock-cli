FROM rust:1.69-slim-buster as builder

RUN apt update && apt install -y pkg-config libssl-dev
# create a new empty shell project
RUN cargo new --bin trustblock-cli
WORKDIR /trustblock-cli

# copy over your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse

# this build step will cache your dependencies
RUN cargo build --release
RUN rm src/*.rs

# copy your source tree
COPY ./src ./src

# build for release
RUN rm ./target/release/deps/trustblock*

RUN cargo build --release 

# our final base
FROM rust:1.69-slim-buster

RUN useradd -ms /bin/bash trustblock

# copy the build artifact from the build stage
COPY --chown=trustblock:trustblock --from=builder /trustblock-cli/target/release/trustblock .

USER trustblock

# set the entrypoint
ENTRYPOINT [ "./trustblock" ]