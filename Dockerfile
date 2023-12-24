FROM rust:1.74.1 AS base

WORKDIR /build

FROM base AS build

RUN apt update \
    && apt install -y \
    musl-dev \
    libssl-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# statically link against openssl
ENV OPENSSL_STATIC=1

ARG VERSION
ENV CARGO_PKG_VERSION=${VERSION:-dev}

COPY . .

RUN cargo build --target x86_64-unknown-linux-gnu --release

FROM base AS build-entrypoint

COPY scripts/docker-entrypoint.sh .
RUN chmod +x docker-entrypoint.sh

FROM gcr.io/distroless/base-nossl AS runtime

WORKDIR /app

COPY --from=build-entrypoint /build/docker-entrypoint.sh .

COPY --from=build /build/target/x86_64-unknown-linux-gnu/release/winterjam-mc-link /usr/local/bin

EXPOSE 3000

ENTRYPOINT ["./docker-entrypoint.sh"]
