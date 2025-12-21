FROM rust:alpine AS build
WORKDIR /app

COPY src src
COPY Cargo* .

RUN rustup target add x86_64-unknown-linux-musl \
    && apk add openssl-dev build-base openssl-libs-static pkgconfig

ENV OPENSSL_STATIC=1
ENV PKG_CONFIG_ALLOW_CROSS=1

RUN cargo build --release --target x86_64-unknown-linux-musl

FROM alpine:latest
WORKDIR /app

COPY web web
COPY --from=build /app/target/x86_64-unknown-linux-musl/release/link-ie .
RUN touch links.toml

EXPOSE 80
CMD [ "/app/link-ie" ]