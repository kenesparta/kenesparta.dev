# Tag AND digest (SECURITY.md SEC-006): the digest is what is pulled — a moved
# or re-uploaded tag cannot change the build — and the tag keeps it readable.
# Dependabot (docker ecosystem) bumps both together.
FROM rust:1.100-bookworm@sha256:ade39a710ef85f3506cffcfec3e8a24ba090badef1dff2fb798f71d878833fea AS site-builder

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

# Release stamp for the home-page footer (src/app/constants.rs): the tag and
# commit, passed by publish-image.yml because .git/ is not in the context.
# ARGs reach the RUN below as environment variables, where option_env! reads
# them at compile time. Unset in local builds, which render "dev".
ARG APP_VERSION
ARG APP_BUILD

# Build the CSS bundle from its parts (no Sass), then compile the app.
# - LEPTOS_HASH_FILES names the bundle after its contents
#   (pkg/kenespartadev.<hash>.{css,js,wasm}) and records the three hashes in
#   hash.txt, which the server reads to link them (app.rs shell). Every release
#   is then a new set of URLs, and caching.rs gives them a year: what lets
#   personal-infra's edge keep /pkg/* (its §5.16, G31). The path is absolute so
#   it does not depend on where cargo-leptos resolves a relative one.
# - --precompress writes a .br and a .gz beside every file under kdevsite, and
#   the file handler serves those as they are instead of compressing the wasm
#   again on every request.
RUN cat apps/backend/style/parts/*.css > apps/backend/style/main.css \
 && cd apps/backend \
 && LEPTOS_HASH_FILES=true LEPTOS_HASH_FILE_NAME=/app/target/release/hash.txt \
    cargo leptos build --release --precompress


FROM gcr.io/distroless/cc-debian12:latest@sha256:e5d81ddde149641e2a9ba55be4545bc125c67de07508b03ba4c22e6eb0ded5aa AS runtime

WORKDIR /app

# Binary name is the crate name (`backend`); site assets live at the workspace
# target root under kdevsite (site-root in apps/backend/Cargo.toml).
COPY --from=site-builder /app/target/release/backend /app/backend
COPY --from=site-builder /app/target/kdevsite /app/kdevsite
# hash.txt goes NEXT TO THE BINARY: leptos looks for it in the directory of the
# running executable, not in the site root. Without it, a hashed build links
# bundle names that do not exist.
COPY --from=site-builder /app/target/release/hash.txt /app/hash.txt

# The AWS Lambda Web Adapter, as a Lambda extension (personal-infra AD-18,
# spec §5.21). On Lambda it receives each invocation and replays it as an HTTP
# request to the server below on AWS_LWA_PORT, so the server is unchanged and
# still just listens. Off Lambda the extension directory is ignored, so
# `docker run` and `make dev/up` behave as before. Pinned by the digest of the
# multi-platform index (SECURITY.md SEC-003); bump the version and digest
# together.
COPY --from=public.ecr.aws/awsguru/aws-lambda-adapter:1.1.0@sha256:17cfd08eff1dfea3f6a9a1e9c65fdac80aa4919b6085e746615530f43f57d2f1 /lambda-adapter /opt/extensions/lambda-adapter

USER nonroot:nonroot

ENV RUST_LOG="info"
ENV LEPTOS_OUTPUT_NAME=kenespartadev
ENV LEPTOS_SITE_ADDR="0.0.0.0:3000"
ENV LEPTOS_SITE_ROOT=/app/kdevsite
ENV LEPTOS_SITE_PKG_DIR=pkg
# The build above hashed the bundle's names; this makes the server link the
# hashed names and give them a year of cache. Unset, it would look for
# pkg/kenespartadev.css and friends, which this image does not contain.
ENV LEPTOS_HASH_FILES=true

EXPOSE 3000

CMD ["/app/backend"]
