FROM rust

RUN rustup toolchain install nightly
RUN rustup component add --toolchain nightly rustfmt

ENTRYPOINT ["cargo"]
