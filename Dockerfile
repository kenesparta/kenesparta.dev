FROM rust:1.97-bookworm AS site-builder

RUN rustup target add wasm32-unknown-unknown
RUN curl --proto '=https' --tlsv1.2 -LsSf https://github.com/leptos-rs/cargo-leptos/releases/download/v0.2.47/cargo-leptos-installer.sh | sh
WORKDIR /app

# Whole workspace: the Leptos app (apps/backend) depends on the Bounded
# Contexts under crates/.
COPY . .

ENV LEPTOS_SASS_VERSION=1.93.2

RUN cd apps/backend && cargo leptos build --release


FROM gcr.io/distroless/cc-debian12 AS runtime

WORKDIR /app

# Binary name is the crate name (`backend`); site assets live at the workspace
# target root under kdevsite (site-root in apps/backend/Cargo.toml).
COPY --from=site-builder /app/target/release/backend /app/backend
COPY --from=site-builder /app/target/kdevsite /app/kdevsite

USER nonroot:nonroot

ENV RUST_LOG="info"
ENV LEPTOS_OUTPUT_NAME=kenespartadev
ENV LEPTOS_SITE_ADDR="0.0.0.0:3000"
ENV LEPTOS_SITE_ROOT=/app/kdevsite
ENV LEPTOS_SITE_PKG_DIR=pkg

EXPOSE 3000

CMD ["/app/backend"]
