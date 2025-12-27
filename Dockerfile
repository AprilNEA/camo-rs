FROM rust:1.83-alpine AS builder

RUN apk add --no-cache musl-dev

WORKDIR /app

COPY Cargo.toml Cargo.lock* ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && mkdir -p src
RUN echo "pub fn dummy() {}" > src/lib.rs
RUN cargo build --release --features server || true
RUN rm -rf src

COPY src ./src
RUN touch src/main.rs src/lib.rs && cargo build --release --features server

FROM alpine:3.21

RUN apk add --no-cache ca-certificates

COPY --from=builder /app/target/release/camo-rs /usr/local/bin/

EXPOSE 8080

ENTRYPOINT ["camo-rs"]
