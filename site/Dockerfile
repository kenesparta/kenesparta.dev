FROM rust:1.90-bookworm AS site-builder

RUN rustup target add wasm32-unknown-unknown
RUN curl --proto '=https' --tlsv1.2 -LsSf https://github.com/leptos-rs/cargo-leptos/releases/download/v0.2.47/cargo-leptos-installer.sh | sh
WORKDIR /app

COPY Cargo.toml ./
COPY src ./src
COPY style ./style
COPY public ./public

ENV LEPTOS_SASS_VERSION=1.93.2

RUN cargo leptos build --release


FROM gcr.io/distroless/cc-debian12 AS runtime

WORKDIR /app

COPY --from=site-builder /app/target/release/kenespartadev /app/
COPY --from=site-builder /app/target/kdevsite /app/kdevsite

USER nonroot:nonroot

ENV RUST_LOG="info"
ENV LEPTOS_SITE_ADDR="0.0.0.0:3000"
ENV LEPTOS_SITE_ROOT=./kdevsite
ENV LEPTOS_SASS_VERSION=1.93.3

EXPOSE 3000

CMD ["/app/kenespartadev"]
