# Tag AND digest (SECURITY.md SEC-006): the digest is what is pulled — a moved
# or re-uploaded tag cannot change the build — and the tag keeps it readable.
# Dependabot (docker ecosystem) bumps both together.
FROM rust:1.98-bookworm@sha256:82150a52ec202c1b14d7817e14516c392bb7f5cfebd88f1ed531cb37ebd39922 AS site-builder

RUN rustup target add wasm32-unknown-unknown
# cargo-leptos 0.2.46 bundles the wasm-bindgen CLI 0.2.104, which MUST match the
# pinned `wasm-bindgen = "=0.2.104"` in Cargo.toml. If you bump one, bump both —
# and re-pin the installer hash (SECURITY.md SEC-006). The script embeds a
# sha256 per platform tarball it downloads, so verifying the script extends the
# chain of custody to the cargo-leptos binary itself.
RUN curl --proto '=https' --tlsv1.2 -fsSL -o /tmp/cargo-leptos-installer.sh \
      https://github.com/leptos-rs/cargo-leptos/releases/download/v0.2.46/cargo-leptos-installer.sh \
 && echo "d12461e2fd1be38e43dcf4b6ba43abf3f8ddf2689c06c2b0aa8bf499c0b796ee  /tmp/cargo-leptos-installer.sh" | sha256sum -c - \
 && sh /tmp/cargo-leptos-installer.sh \
 && rm /tmp/cargo-leptos-installer.sh
WORKDIR /app

# Whole workspace: the Leptos app (apps/backend) depends on the Bounded
# Contexts under crates/.
COPY . .

# Build the CSS bundle from its parts (no Sass), then compile the app.
RUN cat apps/backend/style/parts/*.css > apps/backend/style/main.css \
 && cd apps/backend && cargo leptos build --release


FROM gcr.io/distroless/cc-debian12:latest@sha256:e5d81ddde149641e2a9ba55be4545bc125c67de07508b03ba4c22e6eb0ded5aa AS runtime

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
