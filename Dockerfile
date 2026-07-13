FROM rust:1.97-bookworm AS site-builder

RUN rustup target add wasm32-unknown-unknown
# cargo-leptos 0.2.46 bundles the wasm-bindgen CLI 0.2.104, which MUST match the
# pinned `wasm-bindgen = "=0.2.104"` in Cargo.toml. If you bump one, bump both.
RUN curl --proto '=https' --tlsv1.2 -LsSf https://github.com/leptos-rs/cargo-leptos/releases/download/v0.2.46/cargo-leptos-installer.sh | sh
WORKDIR /app

# Whole workspace: the Leptos app (apps/backend) depends on the Bounded
# Contexts under crates/.
COPY . .

# Build the CSS bundle from its parts (no Sass), then compile the app.
RUN cat apps/backend/style/parts/*.css > apps/backend/style/main.css \
 && cd apps/backend && cargo leptos build --release


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
