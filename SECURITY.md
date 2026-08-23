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

**Severity:** High · **Status:** Fixed 2026-08-23 · **Where:** GitHub →
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

**Change.** Both secrets deleted on 2026-08-23 with
`gh secret delete SOPS_AGE_KEY` / `gh secret delete AWS_ROLE_ARN`
(`-R kenesparta/kenesparta.dev`). The repository now has **no** Actions
secrets; the workflow needs none (`GITHUB_TOKEN` only). `README.md` ("Required
Secrets") states that these two must not be re-created — the age private key
belongs only in `~/.config/sops/age/keys.txt`.

**Verification.** `gh secret list -R kenesparta/kenesparta.dev` prints nothing;
`grep -rn 'SOPS_AGE_KEY\|AWS_ROLE_ARN' .github/` matches nothing.

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

### SEC-007 — Yanked and unsound crates shipped because `cargo audit` only warned

**Severity:** Medium · **Status:** Fixed 2026-08-23 · **Where:** `Cargo.lock`,
`.github/workflows/audit.yml`

**Finding.** The release binary compiled `spin 0.9.8`, a **yanked** version
(via `multer` → `axum`), and `event-listener 5.4.1`, flagged **unsound**
(RUSTSEC-2026-0221, via `async-lock` → `reactive_graph` → `leptos`). CI's
`cargo audit` reported both on every run and still exited 0: yanked/unsound
are warnings by default, and nothing denied them. The workflow's comments also
referred to a `.cargo/audit.toml` that does not exist.

**Change.**
- `cargo update -p spin -p event-listener`: `spin 0.9.8 → 0.9.9`
  (published 2026-07-13 by `zesterer`, spin's long-time maintainer) and
  `event-listener 5.4.1 → 5.4.2` (2026-07-27, `zeenix`, smol-rs), which also
  dropped `concurrent-queue`. Targeted on purpose — a blanket `cargo update`
  would have moved 82 crates at once; Dependabot (SEC-003) now does that
  incrementally under review.
- `audit.yml` runs `cargo audit --deny yanked --deny unsound`. `unmaintained`
  deliberately stays a warning: the two today (`paste`, `proc-macro-error2`)
  arrive through leptos and cannot be fixed here. Ignores, if ever needed, go in
  `.cargo/audit.toml` with a reason and a SEC entry; the flags are not loosened.

**Verification.** Both new versions' lockfile checksums match the crates.io
index. `cargo audit --deny yanked --deny unsound` exits 0 with only the two
unmaintained warnings; `cargo check --locked` passes for `ssr` and for
`hydrate` on `wasm32-unknown-unknown`; the 4 unit tests pass; `make blog/build`
(`--locked`) still builds.

**Follow-ups.** `paste` and `proc-macro-error2` disappear when leptos drops
them — watch the leptos 0.8.x changelog. Consider `cargo deny` (`sources`,
`bans`, licences) as the lockfile policy this audit enforced by hand.

### SEC-003 — Actions pinned to mutable tags, with `packages: write` and auto-deploy behind them

**Severity:** Medium · **Status:** Fixed 2026-08-23 · **Where:**
`.github/workflows/*.yml`, `.github/dependabot.yml`, repository settings

**Finding.** All six Actions were referenced by major tag (`@v4`, `@v7`, `@v2`,
`@v3`, `@v6`). A tag is a pointer whoever controls the action's repository can
move — the 2025 `tj-actions/changed-files` compromise did exactly that. Here the
`publish-image` workflow runs with `packages: write`, and the host redeploys
whatever `:latest` becomes within 10 minutes, so a moved tag was a direct path
to production. The repository allowed any action (`allowed_actions: all`) and
did not require SHA pinning; `audit.yml` and `publish-image.yml` also disagreed
on `actions/checkout` (v4 vs v7).

**Change.**
- Every `uses:` is a full commit SHA with the release as a trailing comment:
  `actions/checkout@3d3c42e5…` (v7.0.1, both workflows),
  `taiki-e/install-action@288e7469…` (v2.86.1 — deliberately a week-old
  release rather than the one cut the same day),
  `docker/login-action@c94ce9fb…` (v3.7.0),
  `docker/setup-buildx-action@8d2750c6…` (v3.12.0),
  `docker/build-push-action@10e90e36…` (v6.19.2). Each SHA was resolved from
  the tag through the GitHub API and checked to be a commit object.
- Repository settings (Actions → General): **Require actions to be pinned to
  a full-length commit SHA** = on (`sha_pinning_required: true`), and
  **Allow select actions** = GitHub-owned + verified creators +
  `docker/*`, `taiki-e/install-action@*`. Both fail closed: a workflow that
  regresses to a tag, or pulls an action outside that list, does not run.
- `.github/dependabot.yml`: weekly, grouped updates for `github-actions`
  (bumps SHA and comment together), `cargo` (one PR for minor/patch,
  `wasm-bindgen` excluded because it is pinned to the cargo-leptos CLI), `npm`
  (`apps/backend/end2end`) and `docker`. Dependabot security updates enabled
  (alerts already were). `audit.yml` runs on every Dependabot PR.

**Verification.** Push of the pinned workflows triggered `audit.yml`
(run 32653534901): success, all steps green. After the settings change a
`workflow_dispatch` run (32653595115) succeeded under
`sha_pinning_required` + the allow-list. Dependabot reports 0 open alerts; the
13 in its history are all *fixed* and belong to npm lockfiles that no longer
exist (`site/`, `web/`, the pre-pnpm `package-lock.json`).

**Follow-ups.** Dependabot PRs need a human review, not an auto-merge: a
bumped SHA is only as trustworthy as the release behind it. When a
`publish-image` change needs a new action, add it to the allow-list pattern
before tagging, or the release will fail closed.

### SEC-005 — Raw database errors were rendered to visitors

**Severity:** Medium · **Status:** Fixed 2026-08-23 · **Where:**
`apps/backend/src/app/api.rs`, `app/pages/blog/{blog_post,blog_list}.rs`

**Finding.** A repository failure surfaced as
`RepositoryError::Infrastructure(sqlx_err.to_string())`, travelled through
`ServerFnError::new(error)` and was rendered on the page
(`"Error loading post: " {e.to_string()}`) and in the server-function JSON —
Postgres/SQLx wording ("error returned from database: relation … does not
exist", pool timeouts) shown to any visitor.

**Change.** The server functions map every use-case failure to one constant,
`service unavailable`; the full error keeps going to the server log
(`tracing::error!`) and nowhere else. The two pages render a generic message
and no longer interpolate the error at all — belt and braces should a future
server function leak again. The blog list error branch now also sets HTTP 500
(it returned 200 before), matching the post page.

**Verification.** `api.rs` regression tests drive the real server functions
against a repository that fails with Postgres-flavoured text, inside a reactive
owner carrying the DI container: the client-visible message must contain
`service unavailable` and must not contain the database wording. Reverting the
`map_err` makes the test fail ("database detail leaked"), so the test does
catch the bug it guards against.

**Follow-ups.** None. If a new server function is added, route its errors
through the same constant.

### SEC-004 — No security response headers anywhere in the chain

**Severity:** Medium · **Status:** Fixed 2026-08-23 · **Where:**
`apps/backend/src/security.rs` (new), `http.rs`, `app.rs`, the e2e suite

**Finding.** Verified live: `https://kenesparta.dev/` answered with only
`content-type` and `vary`. The app set nothing, Caddy only gates on
`X-Origin-Verify`, and the blog CloudFront distribution has no
response-headers policy — no HSTS, no `nosniff`, no CSP, no referrer or
framing policy, end to end.

**Change.** The app owns its markup, so it sets the headers
(`src/security.rs`, mounted in `http::build_app` outside the redirect/rewrite
middlewares so their responses are covered too):

- On every response: `Strict-Transport-Security` (belt-and-braces — `.dev` is
  browser-preloaded), `X-Content-Type-Options: nosniff`,
  `X-Frame-Options: DENY`, `Referrer-Policy: strict-origin-when-cross-origin`,
  `Permissions-Policy` (camera, microphone, geolocation, payment all off) and
  `Cross-Origin-Opener-Policy: same-origin`.
- A **Content-Security-Policy, enforced**, per request. Leptos hydrates via an
  inline `<script type="module">`, so the leptos `nonce` feature (ssr-only) has
  `leptos_axum` generate a nonce per render; `shell()` builds the policy around
  it (`security::provide_csp`) and `<HydrationScripts>` stamps it on the
  script — Async-mode resource-serialization scripts get it too. The policy:
  `script-src 'self' 'nonce-…' 'wasm-unsafe-eval'` (no `unsafe-inline`
  anywhere), fonts/images from the CDN, `connect-src 'self'` (plus the
  cargo-leptos reload websocket, only under `LEPTOS_WATCH`),
  `frame-ancestors 'none'`, `object-src 'none'`, `base-uri`/`form-action`
  `'self'`. Responses that are not rendered pages — assets, 308 redirects, the
  crawler endpoints — get `default-src 'none'; frame-ancestors 'none'`.
- `main.rs`'s router moved to `http::build_app` so tests can drive the real
  app without a socket; `composition::wire(pool)` split out of `compose` so
  they can build the container over a lazy pool.

**Verification.** Enforced, not report-only, on the strength of: unit tests on
the policy string; router tests (`tower::ServiceExt::oneshot` over the real
app) asserting the nonce in the header equals the nonce on the hydration
script, fresh per request, on `/` and on the 404 page, with the fallback
policy on redirects and crawler endpoints; and a Playwright chromium suite
against the running server + dev DB — hydration on `/`, `/blog` and a
published post page with **zero** CSP violations or page errors, wasm loading
under `'wasm-unsafe-eval'`, client-side routing and the server-function fetch
working under the policy (18/18, three consecutive runs). The post-page tests
self-skip when the database has no published post, so they do not depend on
this machine's drafts.

**Follow-ups.** After the next deploy, click through the live site once with
the browser console open — the only environmental difference from the verified
setup is CloudFront/Caddy in front. If leptos ever adds a second inline-script
mechanism, the nonce covers it only if the framework stamps it; the e2e suite
would catch that as a violation.

## Open findings (audit of 2026-08-23)

Found during the same audit, not yet addressed. Each gets its id now so the fix
can reference it; move an item into *Entries* when it is fixed or accepted.

| Id | Sev. | Finding | Where | Fix |
|---|---|---|---|---|
| SEC-006 | Medium | Docker trust anchors are mutable: the cargo-leptos installer is `curl \| sh` with no hash (the script itself embeds SHA-256s for the tarballs it downloads, so pinning the script closes the gap); base images are referenced by tag. | `Dockerfile:1,7,19`, `Dockerfile.dev` | Verify the installer against `sha256 d12461e2fd1be38e43dcf4b6ba43abf3f8ddf2689c06c2b0aa8bf499c0b796ee` (v0.2.46, as of 2026-08-23) before piping to `sh`; pin `rust:1.98-bookworm@sha256:e70e2eec3d495fd5c8e0be74adda86507dfac7f51a724fbf9813ff59b2b247c7` and `gcr.io/distroless/cc-debian12@sha256:e5d81ddde149641e2a9ba55be4545bc125c67de07508b03ba4c22e6eb0ded5aa`; let Dependabot move the digests. |
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
