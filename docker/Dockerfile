FROM rust:1.89 AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock* ./
COPY apps ./apps
COPY crates ./crates
COPY workers ./workers
COPY scripts ./scripts
COPY schemas ./schemas
COPY docs ./docs
COPY README.md ./

RUN cargo build --release -p dn-cli

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN useradd --create-home --uid 10001 --user-group --shell /usr/sbin/nologin dn

WORKDIR /workspace

COPY --from=builder /app/target/release/dn-cli /usr/local/bin/dn-cli

RUN chmod +x /usr/local/bin/dn-cli

USER dn

ENTRYPOINT ["/usr/local/bin/dn-cli"]
CMD ["scan", "/workspace", "--profile", "quick"]
