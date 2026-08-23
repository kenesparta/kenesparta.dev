# Security ledger

A running record of every security-relevant change to this repository and of
the findings that are still open. It exists so that a decision taken under
pressure ("why does `blog/publish` build before it opens the tunnel?") is never
lost, and so that the next audit starts from what the last one found rather than
from zero.

To report a vulnerability in the site itself, email `kenesparta@pm.me`.

## Conventions

- Every entry has a stable id, `SEC-NNN`, assigned once and never reused.
  Reference it from code comments, commit messages and the other docs
  (`CLAUDE.md`, `README.md`, the `Makefile`).
- Severity is the impact if exploited, not the effort to fix: **High** = credential
  or production compromise, **Medium** = meaningful hardening gap with a realistic
  path, **Low** = defence in depth / hygiene.
- Status is one of **Fixed** (with the date), **Pending** (decided, not yet
  applied — say what is blocking), **Open** (found, not yet decided), **Accepted**
  (deliberately not fixed — say why).
- An entry records: the finding, the change (files), how it was verified, and
  follow-ups. Add one whenever you touch secrets handling, the Makefile
  `blog/*`/`secrets*` targets, the workflows, CI trust (action pins, tokens),
  response headers, or bump a dependency because of an advisory.

## Entries

### SEC-001 — Production DB credentials were exposed to the whole build graph

**Severity:** High · **Status:** Fixed 2026-08-23 · **Where:** `Makefile`

**Finding.** `make blog/publish` ran
`sops exec-env secrets/prod.enc.env 'cargo run … --bin ingest …'`. `cargo run`
compiles before it runs, so every `build.rs` and every proc-macro of the ~370
crates in the dependency graph executed with the production `DATABASE_URL` in
its environment — and it did so while the SSH tunnel had production Postgres
reachable on `127.0.0.1:5433`. That is precisely the payload a malicious crate
harvests: a single compromised transitive dependency would have turned into a
production database compromise. `make blog/ingest` had the same shape with the
dev credentials. (The 54 build scripts in the current lockfile were read during
the audit and are benign; the fix removes the exposure regardless of that.)

**Change.**
- New target `blog/build` compiles the ingest binary with **no secret in its
  environment**: `env -u DATABASE_URL cargo build --locked -p backend
  --no-default-features --features ingest --bin ingest`. `env -u` also strips a
  `DATABASE_URL` inherited from the shell profile (one is exported there), so
  the build sees none at all. `--locked` makes the build fail instead of
  silently rewriting `Cargo.lock`, so the binary links only what the committed
  lockfile pins.
- `blog/ingest` and `blog/publish` now depend on `blog/build` and execute the
  built binary (`$(INGEST_BIN)`, `target/debug/ingest`, honouring
  `CARGO_TARGET_DIR`) under `sops exec-env`. For `blog/publish` this also means
  the tunnel is opened **after** the build, only for the seconds the ingest runs.
- `CLAUDE.md` and `README.md` document the invariant so it is not folded back
  into a `cargo run`.

**Verification.** `make -n blog/ingest` / `make -n blog/publish` show the build
step ahead of `sops exec-env` and no `cargo` invocation inside it;
`make blog/build` compiles with `DATABASE_URL` absent from the environment
(checked with the same `env -u` wrapper); `make blog/ingest` runs through sops to
the binary and fails only at the database connection when the dev Postgres is
down (`PoolTimedOut`).

**Follow-ups.** None required. If the ingest ever needs a release build, change
`INGEST_BIN` and add `--release` to `blog/build` — keep the build/run split.

### SEC-002 — Leftover GitHub Actions secrets `SOPS_AGE_KEY` and `AWS_ROLE_ARN`

**Severity:** High · **Status:** Pending — deletion needs the repository admin
(the agent that did the audit was not permitted to run it) · **Where:** GitHub →
Settings → Secrets and variables → Actions

**Finding.** The repository still stores two Actions secrets from the retired
ECR/Terraform pipeline: `AWS_ROLE_ARN` (OIDC role, present since `19cfd66`,
2025-10-28) and `SOPS_AGE_KEY` (referenced from `e0ef34d`, 2026-07-24). Neither
has been referenced by any workflow since `c9b205c` (2026-07-27, "publish images
to GHCR instead of ECR"); the current pipeline authenticates with the repo's
own `GITHUB_TOKEN` only. `SOPS_AGE_KEY` is the **age private key** — it
decrypts `secrets/prod.enc.env`, i.e. the production `DATABASE_URL`. Stored but
unused, it is pure attack surface: anyone able to push a workflow file (a stolen
token with `workflow` scope, a compromised action in a workflow that references
it) can exfiltrate it. `AWS_ROLE_ARN` is not secret in itself, but it points at
an IAM role whose trust policy may still accept this repository's OIDC tokens.

**Change (to apply).**

```bash
gh secret delete SOPS_AGE_KEY -R kenesparta/kenesparta.dev
gh secret delete AWS_ROLE_ARN -R kenesparta/kenesparta.dev
gh secret list   -R kenesparta/kenesparta.dev   # must print nothing
```

Then flip this entry to **Fixed** with the date. `README.md` ("Required
Secrets") already states that no secret is required and that these two must not
be re-created; the age private key belongs only in
`~/.config/sops/age/keys.txt`.

**Follow-ups.**
- Confirm in AWS that the `github_actions_deploy` IAM role and the GitHub OIDC
  provider from the deleted `tf/` were actually destroyed (no AWS session was
  available during the audit). If the role still exists, delete it or restrict
  its trust policy — an orphaned role that trusts `repo:kenesparta/kenesparta.dev:*`
  lets any future workflow here assume it.
- Optional, cheap: rotate the age key (`age-keygen`, replace the recipient in
  `.sops.yaml`, `make secrets-rotate`). The key was stored in GitHub for about a
  month and used by CI for three days, all under the owner's control, so this is
  belt-and-braces rather than a response to a known exposure.

## Open findings (audit of 2026-08-23)

Found during the same audit, not yet addressed. Each gets its id now so the fix
can reference it; move an item into *Entries* when it is fixed or accepted.

| Id | Sev. | Finding | Where | Fix |
|---|---|---|---|---|
| SEC-003 | Medium | All six Actions are pinned to mutable major tags; repo setting *require SHA pinning* is off and `allowed_actions` is `all`. `publish-image` holds `packages: write` and a moved `latest` auto-deploys within 10 min, so a retagged action (the 2025 `tj-actions/changed-files` pattern) is a direct path to production. | `.github/workflows/*.yml`, repo settings | Pin to full SHAs (resolved 2026-08-23: `actions/checkout` `3d3c42e5aac5ba805825da76410c181273ba90b1` v7.0.1 · `taiki-e/install-action` `6cd13508893c0e7eab5f273c2575d3859bd7229a` v2.86.6 · `docker/login-action` `c94ce9fb468520275223c153574b00df6fe4bcc9` v3.7.0 · `docker/setup-buildx-action` `8d2750c68a42422c14e847fe6c8ac0403b4cbd6f` v3.12.0 · `docker/build-push-action` `10e90e3645eae34f1e60eeb005ba3a3d33f178e8` v6.19.2); unify `audit.yml` on checkout v7; enable the SHA-pinning setting; add Dependabot for `github-actions`, `cargo`, `npm`, `docker`. |
| SEC-004 | Medium | No security response headers anywhere in the chain — verified live (`HTTP/2 200`, only `content-type` and `vary`). The app sets none, Caddy only gates on `X-Origin-Verify`, the blog CloudFront distribution has no response-headers policy. | `apps/backend/src/main.rs` (the app owns its markup, so set them here) | `tower_http::set_header::SetResponseHeaderLayer` (feature `set-header`) for `Strict-Transport-Security`, `X-Content-Type-Options: nosniff`, `Referrer-Policy`, `X-Frame-Options: DENY`, `Permissions-Policy`. CSP needs Leptos nonce support for the hydration and JSON-LD inline scripts — start as `Content-Security-Policy-Report-Only`. |
| SEC-005 | Medium | Raw database errors are rendered to visitors: `RepositoryError::Infrastructure(sqlx_err.to_string())` travels through `ServerFnError::new(error)` into `"Error loading post: " {e.to_string()}`. | `persistence/blog_postgres.rs:52` → `app/api.rs:40,51` → `app/pages/blog/blog_post.rs:50`, `blog_list.rs:24` | Keep the `tracing::error!`; return an opaque `ServerFnError::new("service unavailable")` and render a generic message. |
| SEC-006 | Medium | Docker trust anchors are mutable: the cargo-leptos installer is `curl \| sh` with no hash (the script itself embeds SHA-256s for the tarballs it downloads, so pinning the script closes the gap); base images are referenced by tag. | `Dockerfile:1,7,19`, `Dockerfile.dev` | Verify the installer against `sha256 d12461e2fd1be38e43dcf4b6ba43abf3f8ddf2689c06c2b0aa8bf499c0b796ee` (v0.2.46, as of 2026-08-23) before piping to `sh`; pin `rust:1.98-bookworm@sha256:e70e2eec3d495fd5c8e0be74adda86507dfac7f51a724fbf9813ff59b2b247c7` and `gcr.io/distroless/cc-debian12@sha256:e5d81ddde149641e2a9ba55be4545bc125c67de07508b03ba4c22e6eb0ded5aa`; let Dependabot move the digests. |
| SEC-007 | Medium | `spin 0.9.8` is yanked yet compiled into the SSR binary (`multer` → `axum`); `event-listener 5.4.1` is unsound (RUSTSEC-2026-0221, 5.4.2 available); `paste` and `proc-macro-error2` are unmaintained. CI's `cargo audit` passes anyway because warnings are not denied, and the `.cargo/audit.toml` its comments reference does not exist. 82 crates had updates pending. | `Cargo.lock`, `.github/workflows/audit.yml` | `cargo update -p spin -p event-listener`; run `cargo audit --deny yanked --deny unsound` in CI; either create `.cargo/audit.toml` with documented ignores or drop the comment. Consider `cargo deny` for `sources`/`bans`/licence policy. |
| SEC-008 | Low | No tag ruleset and release tags are lightweight (unsigned); pushing any `v*` tag is a production deploy. Commits are 100 % signed and GitHub-verified, tags are not. | repo settings, release process | Tag ruleset restricting `v*` to the owner; `git tag -s`. |
| SEC-009 | Low | The dev compose publishes Postgres (`blog:blog`) and ports 3000/3001 on all interfaces. | `docker-compose.dev.yml` | Prefix the port mappings with `127.0.0.1:`. |
| SEC-010 | Low | `.dockerignore` does not exclude `secrets/` (encrypted, but any `*.dec*` left behind would be copied into the builder layer) or `.claude/` (≈2 200 files). | `.dockerignore` | Add `secrets/` and `.claude/`. |
| SEC-011 | Low | The publish tunnel's control socket is a fixed path in world-writable `/tmp`; if it pre-exists, ssh silently disables multiplexing and the `trap` can no longer close the tunnel, leaving production Postgres forwarded on `:5433` after `make` exits. Host keys are TOFU (`accept-new`). | `Makefile` (`TUNNEL_SOCK`) | Put the socket under `$(HOME)/.ssh/` or a `mktemp -d`; pre-seed `known_hosts` and use `StrictHostKeyChecking=yes`. |
| SEC-012 | Low | `get_published_posts(limit)` is a public server function with an unclamped limit (`i32::MAX` returns the whole table). The table is tiny today. | `apps/backend/src/app/api.rs:31` | `.clamp(1, 100)` server-side. |

## Supply-chain verification record — 2026-08-23

Conclusion: **no evidence of a supply-chain compromise.** Recorded here so the
checks can be repeated, and so the next audit knows what was already covered.

- **Cargo.lock** — 373 third-party crates, all `registry+https://github.com/rust-lang/crates.io-index`, no git/path/http sources. **373/373 checksums match the live crates.io sparse index** (`index.crates.io/<prefix>/<name>`, `cksum` field). 343/343 locally cached `.crate` archives (`~/.cargo/registry/cache`) hash to the lockfile checksum (cargo never re-hashes an already cached archive, so this is the only check that catches a tampered local cache). Across all seven commits that ever touched `Cargo.lock` (2025-10-20 → 2026-08-01), no `(name, version)` ever changed checksum, and no commit changed the lockfile without a matching `Cargo.toml` change. `sqlx-sqlite`/`ring` appear in the lockfile only through optional features and are in neither the `ssr` nor the `ingest` build graph.
- **Build scripts** — all 54 `build.rs` in the locked graph were read: `rustc --version` probes, `aws-lc-rs` reading its own feature variables, `ring`'s Windows-only nasm call, `wasm-bindgen-shared` running `git rev-parse`. Nothing opens sockets or reads credentials.
- **cargo audit** — 0 vulnerabilities; 4 warnings (see SEC-007).
- **pnpm** (`apps/backend/end2end`) — 7 packages, 7/7 `sha512` integrity values match `registry.npmjs.org`; none on the 2025 npm compromise lists (`chalk`/`debug`/Shai-Hulud, `nx`, `eslint-config-prettier`).
- **Git history** — 93 commits, a single author, every commit signed and reported `verified: true, reason: valid` by GitHub (31 show `E` locally only because the older key and GitHub's web-flow key are not in the local keyring). Local `main` == `origin/main` (the Codeberg mirror was 3 commits behind).
- **GitHub** — public repo, 1 collaborator (owner), 0 webhooks, 0 deploy keys, default workflow token permission `read`; all 35 workflow runs in history were triggered by the owner.
- **GHCR** — `:latest` digest equals `:v0.4.3` (`sha256:f09e74fb21855f38c3b49dfd72e4ff29ccab0f49aaf402b782538b3fa5091aaa`), image created 6 minutes after that tag's workflow run, `USER nonroot`, buildx provenance attestation attached, labels/env/cmd as expected. Three SHA-named tags (`0f8fa66…`, `b904879…`, `3a16452…`) are leftovers of the July pipeline iterations and map to real commits.
- **Application code** — all SQL parameterised; the trailing-slash open-redirect fix confirmed live (`//evil.com/` → `Location: /evil.com`); JSON-LD `<` escaping correct; drafts gated by slug and by the `.md` endpoint; `unsafe_code = "forbid"` on every crate; `content/posts` contains no raw HTML/script.
