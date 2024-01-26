FROM rust:slim as builder
RUN apt-get update && apt-get install -y libpq-dev libssl-dev pkg-config musl-tools perl make && rm -rf /var/lib/apt/lists/*
RUN rustup target add x86_64-unknown-linux-musl
WORKDIR /app
COPY . /app
RUN cargo build --release --target x86_64-unknown-linux-musl
RUN strip /app/target/x86_64-unknown-linux-musl/release/microservice

FROM scratch
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/microservice /microservice
WORKDIR /
CMD ["/microservice"]
