FROM rust:1.44 AS BUILDER

RUN rustup default nightly-2020-06-09
WORKDIR /usr/src/poke_shakespeare
COPY . .
RUN cargo install --path .

FROM debian:buster-slim
COPY --from=builder /usr/local/cargo/bin/poke_shakespeare /usr/local/bin/poke_shakespeare

RUN mkdir /usr/srv
WORKDIR /usr/srv
ENV ROCKET_ADDRESS=0.0.0.0
CMD ["poke_shakespeare"]
