FROM rust

WORKDIR /usr/src/app

COPY . .

RUN cargo install --path ./debian_bridge

ENTRYPOINT ["debian_bridge"]
