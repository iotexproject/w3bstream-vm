FROM rust:1.70.0 AS builder

RUN update-ca-certificates

RUN apt update
RUN apt install -y curl libpq-dev libssl-dev libpq-dev librust-openssl-dev librust-openssl-sys-dev
RUN apt -y install openssl libssl1.1 libpq5
RUN apt update && apt install -y libprotobuf-dev protobuf-compiler

RUN cargo install diesel_cli --no-default-features --features postgres

RUN curl -L https://foundry.paradigm.xyz | bash

RUN cp /root/.foundry/bin/* /usr/bin/

RUN foundryup

WORKDIR /rust/src
COPY ./ ./

RUN export CARGO_NET_GIT_FETCH_WITH_CLI=true
RUN export RUST_BACKTRACE=1

RUN cargo build --release



FROM ubuntu:20.04

RUN apt update && apt -y install openssl libssl1.1 libpq5 ca-certificates
RUN apt update && apt -y install curl && apt -y install build-essential
RUN apt update install -y libpq-dev

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

RUN cargo install cargo-binstall
RUN echo yes | cargo binstall cargo-risczero
RUN cargo risczero install


WORKDIR /risc0server/


COPY --from=builder /rust/src/target/release/risc0server ./
COPY ./Cargo.toml /risc0server/
COPY ./diesel.toml /risc0server/
COPY ./verify_contract_abi.json /risc0server/

#RUN diesel setup
#RUN diesel migration generate risc0-server
#RUN diesel migration run

CMD ["/risc0server/risc0server"]
