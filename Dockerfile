FROM rust:1.76-bookworm as builder

WORKDIR /usr/src/disrupted-data-node
COPY . .

RUN cargo clean
RUN cargo build --release

FROM debian:bookworm-20240130-slim

ARG IP_ADDRESS
ARG PORT
ARG NODE_KEY_LOCATION
ARG BOOTSTRAP_NODES
ARG LOG_FILE

RUN apt-get update && apt-get install -y inetutils-ping && apt-get install -y curl  && apt-get install -y gettext-base && apt-get clean && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/disrupted-data-node/target/release/disrupted-data /disrupted-data
COPY --from=builder /usr/src/disrupted-data-node/config.template.toml /config.template.toml

RUN #envsubst < /config.template.toml > /config.toml

EXPOSE 6969
CMD ["/disrupted-data"]
