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
- ~~Confirm in AWS that the `github_actions_deploy` IAM role and the GitHub OIDC
  provider from the deleted `tf/` were actually destroyed (no AWS session was
  available during the audit).~~ Done 2026-08-23: the role was **not** destroyed —
  it was migrated into personal-infra's state and kept for `typst-resume`, still
  trusting this repository. Restricting its trust policy is **SEC-013**.
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

### SEC-006 — Mutable trust anchors in the Docker builds

**Severity:** Medium · **Status:** Fixed 2026-08-23 · **Where:** `Dockerfile`, `Dockerfile.dev`

**Finding.** Both stages pulled by tag (`rust:1.98-bookworm`, distroless `cc-debian12`) — tags any
registry compromise can move — and the cargo-leptos installer was `curl | sh` with no verification.

**Change.** Both `FROM`s carry tag **and** digest (Dependabot's docker ecosystem bumps them
together); `Dockerfile.dev` now uses the same digest-pinned base as the production builder. The
installer is downloaded to a file, verified against its pinned sha256
(`d12461e2…b796ee`, v0.2.46), and only then run — the script itself embeds a sha256 per platform
tarball, so the pin extends the chain of custody to the cargo-leptos binary. Bumping cargo-leptos
now means re-pinning this hash alongside the wasm-bindgen version (the comment says so in place).

**Verification.** A probe image built from the exact `FROM @digest` + installer instructions:
digest pull, `sha256sum -c`, install and `cargo leptos --version` all succeed; the distroless
digest pulls. (Build log in the entry's commit message context; the instructions are byte-identical
to the Dockerfile's.)

### SEC-008 — Release tags were unprotected and unsigned

**Severity:** Low · **Status:** Fixed 2026-08-23 · **Where:** GitHub ruleset, repo-local git config

**Finding.** Pushing any `v*` tag deploys to production within ~10 minutes, yet any write-scoped
credential could create one, and the existing tags were lightweight (unsigned) — the only
unauthenticated link in an otherwise 100 %-signed history.

**Change.** GitHub ruleset `protect-release-tags` (id 21245040, active): creating, moving or
deleting `refs/tags/v*` is restricted to repository admins. Locally, `tag.gpgSign = true` in this
repo's git config, so `git tag vX.Y.Z -m vX.Y.Z` produces a signed annotated tag (the `-m` is now
required); CLAUDE.md and the workflow header document the new one-liner.

**Verification.** Ruleset readback shows `enforcement=active`, admin bypass
(`current_user_can_bypass: always` — the owner's release flow keeps working); `git config
tag.gpgSign` → true. The signing itself rides the same key every commit already uses.

### SEC-009 — Dev containers published ports on every interface

**Severity:** Low · **Status:** Fixed 2026-08-23 · **Where:** `docker-compose.dev.yml`, `docker-compose.yml`

**Finding.** `"5432:5432"` (Postgres, `blog`/`blog`), `"3000:3000"`, `"3001:3001"` bind 0.0.0.0 —
on a laptop that means the coffee-shop LAN can reach the dev database with known credentials.

**Change.** Every published port is now `127.0.0.1:`-prefixed, including the root compose's
smoke-test port. That file's healthcheck was also removed: distroless has no curl and no shell, so
it had reported "unhealthy" since the day it was written (health in production is
CloudFront/Caddy's concern).

**Verification.** `docker compose config` validates both files; `docker ps` on the dev database
shows `127.0.0.1:5432->5432/tcp`.

### SEC-010 — Build context included `secrets/` and `.claude/`

**Severity:** Low · **Status:** Fixed 2026-08-23 · **Where:** `.dockerignore`

**Finding.** `COPY . .` shipped `secrets/` (encrypted — but a decrypted `*.dec*` left behind by
tooling would ride along into a builder layer) and the ~2 200-file `.claude/` directory into every
build context.

**Change.** Both excluded. The runtime stage never copied them, so this is exposure trimming for
the builder stage plus faster context uploads.

### SEC-011 — Publish tunnel: control socket in /tmp, TOFU host keys

**Severity:** Low · **Status:** Fixed 2026-08-23 · **Where:** `Makefile` (`blog/publish`)

**Finding.** The ssh ControlMaster socket lived at a fixed path in world-writable `/tmp`: any
local process could pre-create it, ssh would then silently disable multiplexing, and the EXIT trap
could no longer close the tunnel — leaving production Postgres forwarded on `127.0.0.1:5433` after
`make` returned. Both ssh invocations also used `StrictHostKeyChecking=accept-new` (trust on first
use).

**Change.** The socket moved to `~/.ssh/kdev-pg-tunnel.sock` (0700 directory), with a stale-socket
`rm -f` after the initial `-O exit`. Host key checking is `yes`: the origin's ed25519 key is
already in `known_hosts`, so nothing ever TOFUs again — if the host is ever rebuilt, update
`known_hosts` deliberately (personal-infra owns the host key).

**Verification.** `ssh-keygen -F origin.kenesparta.dev` confirms the pinned key; `make -n
blog/publish` shows socket path, `rm -f`, and `StrictHostKeyChecking=yes` on both invocations.

### SEC-012 — Public server function accepted an unbounded limit

**Severity:** Low · **Status:** Fixed 2026-08-23 · **Where:** `apps/backend/src/app/api.rs`

**Finding.** `get_published_posts(limit)` is a public endpoint and handed the client's `limit`
straight to `LIMIT $1` — `i32::MAX` returned the whole table (harmless today, unbounded by
contract).

**Change.** The client value is folded into `[1, 100]` at the boundary. The complete listings
(sitemap, feed, llms.txt) call the use case directly server-side and are unaffected.

**Verification.** A probe repository asserts any limit it receives is within `[1, 100]`; the test
feeds `i32::MAX`, `101`, `0`, `-5`, `i32::MIN` and `None` through the real server function.
Reverting the clamp makes it fail ("unclamped client limit reached the repository: 2147483647").

### SEC-013 — The migrated OIDC deploy role still trusted this repository

**Severity:** Medium · **Status:** Fixed 2026-08-31 (applied in `../personal-infra` as
`58c0c95`, confirmed live in AWS) · **Where:** AWS IAM role `github-actions-ecr-ecs-deploy`,
fixed via `../personal-infra` (`terraform/iam.tf`, spec AD-5)

**Finding.** Chasing SEC-002's follow-up with a live AWS session: the old pipeline's IAM role was
never destroyed. Its state was migrated verbatim into personal-infra (AD-9, addresses preserved)
and the role deliberately survives Phase 7 because `typst-resume` publishes the CV through it —
its only remaining permission is write/delete on the `cdn.kenesparta.dev` S3 bucket. But its
trust policy still accepted `repo:kenesparta/kenesparta.dev:ref:refs/heads/main` and
`:ref:refs/tags/*`. This repository's CI has needed no AWS access since the GHCR migration, so
the entries were pure leftover: any workflow added here — or a compromised action running inside
one (the SEC-003 scenario) — could assume the role over OIDC and overwrite or delete the
published CV and the blog's static assets. The GitHub OIDC *provider* remains in the account,
correctly: two roles legitimately federate through it (`typst-resume` → CDN bucket,
`cnayp-discord-bot` → its legal-pages bucket), and neither trusts this repository.

**Change.** In `../personal-infra` (all infra changes are made there): the two `kenesparta.dev`
`sub` entries removed from the role's trust policy, leaving `typst-resume` alone; spec
`03-decisions.md` AD-5 amended to record the scoping so it is not re-added. Nothing to change in
this repository — no workflow has referenced the role since SEC-002 deleted `AWS_ROLE_ARN`.

**Verification.** Before: enumerating all roles whose trust policy names
`token.actions.githubusercontent.com` found exactly the two above, with this repository present
only on `github-actions-ecr-ecs-deploy`. Plan reviewed: the only diff is the two removed `sub`
lines. After: confirmed against live AWS on 2026-08-31 (SSO profile
`AdministratorAccess-711387133796`, account 711387133796).
`aws iam get-role --role-name github-actions-ecr-ecs-deploy` returns a trust policy whose `sub`
list is exactly `repo:kenesparta/typst-resume:ref:refs/heads/main` and
`…:ref:refs/tags/*` — the two `kenesparta.dev` entries are gone. The Before sweep was re-run over
**every** role in the account: still exactly two federate through GitHub OIDC
(`github-actions-ecr-ecs-deploy` → typst-resume, `github-actions-cnayp-bot-site` →
cnayp-discord-bot), and no role's trust policy names `kenesparta.dev` at all. The Terraform code
behind it landed as `58c0c95` in personal-infra, so state and reality agree.

**Follow-ups.**
- ~~Run `make apply` in `../personal-infra` (the saved `tf.plan`), then flip this entry to
  Fixed.~~ Done — applied and verified against live AWS on 2026-08-31 (see Verification).
- Still open, and **not this repository's to fix** — carried here only because this audit found
  it. The same live sweep confirmed the asymmetry below is real: `github-actions-cnayp-bot-site`
  carries **both** subject spellings, while `github-actions-ecr-ecs-deploy` carries only the
  plain one. GitHub has begun issuing **immutable** OIDC subject claims
  (`repo:kenesparta@8525741/<repo>@<id>:…` — see the `cnayp-bot` role's comment in `iam.tf`,
  which lists both spellings because the plain-name form was already being denied for that
  repo). `typst-resume`'s trust lists only the plain spelling; if its claims switch, the CV
  publish starts failing with an opaque `Not authorized to perform
  sts:AssumeRoleWithWebIdentity`. Adding the id-form spellings for `typst-resume` mirrors the
  scoping commit `8f19d8e` in personal-infra.

### SEC-014 — A yanked transitive turned the weekly audit red; pins drifted for a week

**Severity:** Low · **Status:** Fixed 2026-08-31 · **Where:** `Cargo.lock`, `Cargo.toml`,
`Dockerfile`, `Dockerfile.dev`, `.github/workflows/*.yml`

**Finding.** The scheduled `audit.yml` run of 2026-08-31 (33393110703) **failed**: `chacha20
0.10.1` had been yanked upstream and sat in the graph via `rand 0.10.2` ← `sqlx-postgres 0.9.0`.
No vulnerability — a yank, caught exactly as SEC-007 intended when it turned `--deny yanked` on;
this is the control firing, not a gap. Alongside it, a week of unattended drift: the
`rust:1.98-bookworm` digest had moved (the tag was rebuilt 2026-08-25 over a newer
`buildpack-deps:bookworm`, so the pin was holding the build on an older Debian package set), the
five pinned Actions were behind, and all five Dependabot PRs (#1–#5) were closed unmerged on
2026-08-31 in favour of one reviewed manual pass — the SEC-007 precedent.

**Change.** All applied by hand and verified together:
- **Yanked crate:** `cargo update -p chacha20` → 0.10.2 (0.10.0 and 0.10.1 are both yanked; 0.10.2
  is the only clean 0.10.x). One package, no other movement.
- **Cargo group** (what Dependabot #3 had batched): `tokio 1.52.3 → 1.53.1`,
  `async-trait 0.1.89 → 0.1.92`, `serde_json 1.0.150 → 1.0.151`, `thiserror 2.0.18 → 2.0.20`,
  `uuid 1.23.5 → 1.26.0`. `async-trait` and `thiserror-impl` bring `syn 3.0.4` (proc-macro only).
- **`toml 0.8 → 1`** (Dependabot #4). The crate has exactly one caller,
  `toml::from_str` in `bin/ingest.rs`; `from_str` is unchanged in 1.x. This also **de-duplicated**
  the lockfile — 0.8 and 1.1 were both present — dropping `toml_edit`, `toml_write`,
  `toml_datetime 0.6`, `serde_spanned 0.6` and `winnow 0.7`. Net: 375 → 371 crates.
- **Docker base** (Dependabot #5): the `rust:1.98-bookworm` digest `e70e2eec…` → `82150a52…` in
  **both** `Dockerfile` and `Dockerfile.dev`, keeping them identical as SEC-006 requires. The
  distroless `cc-debian12` digest was re-checked against the live tag and is **unchanged**, so it
  stays. cargo-leptos stays at **0.2.46** deliberately — see the follow-up.
- **Action pins** (Dependabot #2), each SHA resolved from its release tag through the GitHub API
  and confirmed to be a commit object, SHA and `# vX.Y.Z` comment moved together:
  `docker/login-action` v3.7.0 → **v4.6.0**, `docker/setup-buildx-action` v3.12.0 → **v4.3.0**,
  `docker/build-push-action` v6.19.2 → **v7.3.0**, `taiki-e/install-action` v2.86.1 → **v2.86.7**.
  `actions/checkout` was already at the current v7.0.1. The three Docker majors are all the same
  upstream change — Node 24 runtime, ESM, and removal of deprecated inputs/envs; the workflow was
  read against those removals and uses none of them (`setup-buildx` is invoked with no inputs at
  all, and neither `DOCKER_BUILD_NO_SUMMARY` nor `DOCKER_BUILD_EXPORT_RETENTION_DAYS` appears),
  so the majors are inert here. `install-action` is v2.86.7 (2026-08-24), not the current v2.87.2
  (2026-08-30): SEC-003 deliberately takes a week-old release of this action rather than a fresh
  cut, and that rule is kept.

**Verification.** `cargo audit --deny yanked --deny unsound` exits **0** (previously 1), leaving
only the two known `unmaintained` warnings that arrive through leptos. `cargo check --locked`
passes for `ssr` and for `hydrate` on `wasm32-unknown-unknown`; **10/10** unit tests pass,
including the SEC-004, SEC-005 and SEC-012 regression tests; `make blog/build` (`--locked`,
`DATABASE_URL` stripped) links the new `toml`. Because the ingest connects to Postgres *before* it
parses, compiling proves nothing about parsing — so `toml 1.1.4` was run against the real
`content/posts/*.md` frontmatter in isolation, confirming the inline comment after
`status = "draft"`, the `default_author` fallback, the tags array and `deny_unknown_fields` all
still behave. The lockfile delta was diffed package-by-package against its baseline: every
addition and removal above is accounted for and nothing else moved. Both new image digests were
re-resolved from their live tags; the new rust index is the official `rust-lang/docker-rust`
build of 2026-08-25 and carries a `linux/amd64` manifest.

**Follow-ups.**
- The **Docker image was not built** as part of this change — the local Docker daemon was down —
  and the Playwright e2e suite was not run for the same reason (it needs the dev database). Both
  are exercised by the next `v*` tag; the first release after this commit is worth watching rather
  than assuming, since it is the first to run the three Docker action majors.
- **cargo-leptos stays on 0.2.46** (0.3.7 is current) as a deliberate security decision, not
  neglect. 0.2.x carries `wasm-bindgen-cli-support` as a compiled-in Cargo dependency, so the
  pinned installer sha256 covers the whole chain down to the bindgen binary. 0.3.x dropped that
  dependency: it reads the wasm-bindgen version out of `Cargo.lock` and **downloads the matching
  CLI tarball from GitHub releases at build time** (`src/ext/exe.rs:618`, `:677`), which would put
  an unverified binary back inside the Docker builder — precisely what SEC-006 closed. Revisit
  only with a way to pin that download; the payoff would be dropping the manual
  `wasm-bindgen = "=0.2.104"` coupling.
- `paste` and `proc-macro-error2` remain `unmaintained` warnings via leptos (SEC-007), unchanged.

### SEC-015 — TypeScript 7 stopped including `@types/node` implicitly; the week's pins reviewed

**Severity:** Low · **Status:** Fixed 2026-09-13 · **Where:** `Cargo.lock`,
`apps/backend/end2end/{package.json,pnpm-lock.yaml,tsconfig.json}`, `.github/workflows/audit.yml`

**Finding.** The week's three Dependabot PRs (#6 actions, #7 npm, #8 cargo), reviewed as SEC-003
requires — a bumped pin is only as trustworthy as the release behind it — and applied as one manual
pass on `main` with the PRs closed by the commits, the SEC-014 precedent. Two of the three are
routine. The npm one crosses **two TypeScript majors** (5.9.3 → 7.0.2, the native compiler) and
**six `@types/node` majors** (20 → 26), and TypeScript 6.0 changed the default of `types` from
"every package under `node_modules/@types`" to `[]`. The e2e `tsconfig.json` relied on the old
default: under 7.0.2, `tsc --noEmit` fails with three `TS2591: Cannot find name 'process'` in
`playwright.config.ts`. Nothing in CI runs `tsc` — Playwright compiles the specs with its own
pipeline — so merged as-is the bump would have degraded silently into editor errors and left the
suite's only static check broken without anyone noticing.

**Change.**
- **Cargo group (#8):** `serde`/`serde_core`/`serde_derive` 1.0.228 → 1.0.229 (`serde_derive`
  moves to `syn 3`; 3.0.4 was already in the lock via `async-trait`/`thiserror-impl` since SEC-014,
  so no new crate), `toml` 1.1.4 → 1.1.5 (a `DeValue::make_owned` fix; the only caller is
  `toml::from_str` in `bin/ingest.rs`), `tower-http` 0.7.0 → 0.7.1. Every 0.7.1 change lives in a
  module this app does not use — `fs` (`ServeDir` now propagates I/O errors instead of answering
  404; `leptos_axum`'s own `ServeDir` is tower-http **0.6.11**, untouched), `decompression`,
  `request-id`, `set-header` — while this crate is used for `CompressionLayer` and `TraceLayer`
  only, so the bump is inert. Side effect in the lock: `errno 0.3.14` and `winapi-util 0.1.11`
  re-resolved `windows-sys` 0.52.0 → 0.61.2; both versions stay in the graph, both are Windows-only
  and absent from the linux/amd64 image and the macOS builds. 371 → 371 crates.
- **npm group (#7):** `typescript` 5.9.3 → 7.0.2 and `@types/node` 20.19.43 → 26.4.1 (the major of
  the Node that runs the suite, 26.8.2 here), pulling `undici-types` 8.3.0 and the twenty
  `@typescript/typescript-<os>-<arch>` optional platform binaries TS 7 ships as. Plus the fix:
  `"types": ["node"]` in `tsconfig.json`, with a comment saying why. The rest of that config survives
  7.0 — `target: es2016` (only ES5 went), `module: commonjs` (only amd/umd/system/none went),
  `esModuleInterop: true` (`false` is now an error), `strict`, `skipLibCheck`.
- **Actions group (#6):** `taiki-e/install-action` v2.86.7 → **v2.87.5** in `audit.yml`, SHA and
  comment together. Released 2026-09-04 — nine days old, so the SEC-003 week-old rule holds;
  v2.87.6 … v2.87.12 (2026-09-05 … 09-12) exist and were skipped for that reason. Between the two
  pins the changelog is `@latest` manifest updates plus support for one new tool (`kache`, 2.87.0);
  `cargo-audit@latest` did not move, so the job installs the same cargo-audit as before.

**Verification.**
- Pin: `5bf6ce01…` is the commit object tag `v2.87.5` points at (`Release 2.87.5`, Taiki Endo, an
  ancestor of the action's `main`; unsigned, as every release commit of that action is,
  `b6ff5808…` included). `audit.yml` ran on the PR with the new pin (run 34166118815) and passed —
  the pin was exercised end to end, not just resolved.
- Cargo: 8/8 lockfile checksums — the five bumped crates plus `syn 3.0.4`, `windows-sys 0.61.2`,
  `http-range-header 0.4.2` — match the crates.io sparse index, and the publishers are the crates'
  owners (`dtolnay` for serde and syn, `epage` for toml, `seanmonstar` for tower-http, `kennykerr`
  for windows-sys); serde 1.0.229 dates from 2026-07-18, tower-http 0.7.1 from 2026-08-31, toml
  1.1.5 from 2026-09-02. `cargo check --locked` passes for `ssr` and for `hydrate` on
  `wasm32-unknown-unknown`; **13/13** unit tests pass (10 backend, 3 bc-blog); `make blog/build`
  (`--locked`, `DATABASE_URL` stripped) links; `cargo audit --deny yanked --deny unsound` exits 0
  with the two known `unmaintained` warnings. Cargo's four `future-incompat` warnings are all
  `proc-macro-error2 2.0.1`, the same leptos transitive SEC-007 tracks at the same version — not
  new. The PR's own audit run (34166139973) was green as well.
- npm: **27/27** `sha512` integrity values in the new lockfile match `registry.npmjs.org`.
  `@playwright/test`, `playwright`, `playwright-core` and `undici-types` carry npm provenance
  attestations; `typescript`, its platform packages and `@types/node` do not (neither did 5.9.3).
  Dependabot's "new releaser" note: 7.0.2 was pushed by `microsoft1es` (`npmjs@microsoft.com`),
  5.9.3 by `typescript-bot`, 7.0.1-rc by `typescript-deploys` — all three sit in the package's
  maintainer list, TS 7 is built and released from `microsoft/typescript-go`, and 7.0.2 has been
  `latest` since 2026-07-08. With the fix `tsc --noEmit` exits 0 under 7.0.2 (1 without it);
  `playwright test --list` still enumerates 54 tests in 3 files.
- Not run: the Playwright suite itself, for the reason SEC-014 gives — the local Docker daemon was
  down, so no dev database. The bump cannot change its behaviour: Playwright never loads the
  `typescript` package, and `@types/node` is declarations only.

**Follow-ups.**
- Dependabot proposes this action weekly and each bump has to clear the week-old rule by hand:
  v2.87.12 is the current tip and becomes eligible on 2026-09-19.
- `@types/node` 26.5.1 was already `latest` at review time (Dependabot had resolved a week earlier)
  and will come next week. Keep it on the major of the Node that runs the suite.
- The e2e `tsconfig.json` is still the full `tsc --init` scaffold with every option commented out;
  TS 7 tolerates it, but it could shrink to the seven options it actually sets. Cosmetic.

### SEC-016 — `rustls` TLS 1.3 advisory kept the audit red for 16 days; the week's pins reviewed

**Severity:** Low · **Status:** Fixed 2026-09-30 · **Where:** `Cargo.lock`,
`apps/backend/end2end/pnpm-lock.yaml`, `.github/workflows/{audit,publish-image}.yml`

**Finding.** `cargo audit` has failed on every run since the evening of 2026-09-14 — four
Dependabot PR runs and the scheduled `main` runs of 09-21 (35597087367) and 09-28 (36424986386) —
on **RUSTSEC-2026-0285** (CVSS 5.3): `rustls` 0.23.13 … 0.23.44 accepted a plaintext TLS 1.3
handshake message placed in the same record after a key-changing one (e.g. `EncryptedExtensions`
behind `ServerHello`), against RFC 8446 §5.1. The transcript stays authenticated, so a
network-position attacker cannot alter or complete a handshake with it; a peer just gets to skip
encryption for those messages without the connection being torn down. The lock had `rustls
0.23.41`, reached only through sqlx's `tls-rustls-aws-lc-rs` — the Postgres client, in the server
and the ingest CLI. **Not reachable in practice:** personal-infra's `postgres:18` sets no `ssl`
option (image default `ssl = off`, no certificate — `postgres_settings` in its
`group_vars/all.yml`), and the `DATABASE_URL` its `env.j2` renders carries no `sslmode`, so sqlx's
default `prefer` gets `N` to its `SSLRequest` and continues in plaintext on the internal `web`
network; `rustls` never runs a handshake. The dev database is the same (`show ssl` → `off`). Hence
Low, and fixed by upgrading rather than by an ignore in `.cargo/audit.toml`. The red scheduled
runs went unaddressed for two weeks; it surfaced through the failing Dependabot PRs.
Those three PRs (#10 npm, #12 actions, #13 cargo) were reviewed as SEC-003 requires and applied as
one manual pass on `main`, the SEC-014 / SEC-015 precedent.

**Change.**
- **Advisory:** `cargo update -p rustls` → `rustls` 0.23.41 → **0.23.45**. It requires
  `aws-lc-rs ^1.18` and `rustls-webpki ^0.103.14`, so `aws-lc-rs` 1.17.1 → 1.18.1, `aws-lc-sys`
  0.42.0 → 0.45.0 (its `^0.45` requirement) and `rustls-webpki` 0.103.13 → 0.103.15 move with it —
  forced, nothing else. 371 → 371 crates.
- **Cargo group (#13):** `toml` 1.1.5 → 1.1.6 (parser/display allocation work; the only caller is
  `toml::from_str` in `bin/ingest.rs`), `thiserror`/`thiserror-impl` 2.0.20 → 2.0.21 (a parsing fix
  for generic unit variants in `#[error]` strings; the 1.0.69 copy leptos pulls is untouched),
  `uuid` 1.26.0 → 1.26.1 (v7 counter placement, a `Timestamp` → `SystemTime` overflow panic). This
  code calls only `Uuid::new_v4` and `Uuid::parse_str`, so the uuid fixes are inert here.
- **npm group (#10):** `@playwright/test` / `playwright` / `playwright-core` 1.62.1 → 1.63.0 and
  `@types/node` 26.4.1 → 26.5.1 (still the Node 26 major that runs the suite), with `undici-types`
  8.3.0 → 8.9.0. Playwright 1.63 no longer lists `fsevents` as an optional dependency, so the lock
  loses it: 27 → 26 packages. Lockfile only; `package.json` ranges already allowed both.
- **Actions group (#12)**, SHA and `# vX.Y.Z` comment together:
  `taiki-e/install-action` v2.87.5 → **v2.87.15** in `audit.yml`; `docker/setup-buildx-action`
  v4.3.0 → **v4.4.1** and `docker/build-push-action` v7.3.0 → **v7.4.0** in `publish-image.yml`.
  install-action v2.87.15 was released 2026-09-18, twelve days old, so the SEC-003 week-old rule
  holds. The PR's pin is taken as is rather than re-resolved to v2.87.19 (2026-09-23, the newest
  release a week old today): the rule sets a minimum age, not a newest-eligible target. Every
  changelog line from v2.87.6 through v2.87.15 is an `Update <tool>@latest` manifest bump, and
  `cargo-audit@latest` is not among them. The Docker minors add a BuildKit image pre-pull before
  builder creation (skipped for explicit endpoints in 4.4.1), use official Buildx releases for the
  cloud driver, and in build-push v7.4.0 **stop workflow-command injection through metadata log
  output**. They remove no input or env var, and the workflow passes `setup-buildx` no inputs.

**Verification.**
- Pins: each SHA was resolved from its tag through the GitHub API, is a commit object and equals
  the PR's: `4076c08d…` is `Release 2.87.15` (Taiki Endo, an ancestor of the action's `main`;
  unsigned, like every release commit of that action); `f87e5991…` and `c3c9e263…` are
  GitHub-verified signed merge commits, identical to each Docker repo's `master` HEAD. The PR's
  own audit run could not exercise the install-action pin (it was red for the reason above); the
  push of this change runs `audit.yml` with it.
- Cargo: the group delta was diffed against PR #13 and is **identical** line for line. All 8
  bumped checksums in the lock match the crates.io sparse index, and none of the versions is
  yanked. Each was published by that crate's own maintainer: `ctz` (rustls, 2026-09-14), `cpu`
  (rustls-webpki, 08-21), `justsmth` (aws-lc-rs and aws-lc-sys, 09-01), `epage` (toml, 09-10),
  `KodrAus` (uuid, 09-10), `dtolnay` (thiserror, 09-23). `cargo audit --deny yanked --deny unsound`
  exits **0**, down from 1, with only the two known `unmaintained` warnings. `cargo check --locked`
  passes for `ssr` and for `hydrate` on `wasm32-unknown-unknown`. **13/13** unit tests pass
  (10 backend, 3 bc-blog), and `make blog/build` links.
- **Docker image built** this time (the daemon was up, unlike SEC-014/015): the production
  `Dockerfile` compiled `aws-lc-sys` 0.45.0, `aws-lc-rs` 1.18.1, `rustls` 0.23.45 and
  `rustls-webpki` 0.103.15 in the `rust:1.98-bookworm` builder with no new system packages, and
  finished both the `release` and `wasm-release` profiles. That was a native arm64 build, not the
  CI's linux/amd64, and the image was deleted afterwards.
- Runtime: `make blog/ingest` against the dev database ran the migrations over the new sqlx/TLS
  stack and parsed the real `content/posts/*.md` frontmatter with `toml` 1.1.6 (1 upserted). That
  image then ran on the dev network against the same database: migrations applied, `/` and `/blog`
  answered 200.
- npm: **26/26** `sha512` integrity values in the new lock match `registry.npmjs.org`. The three
  Playwright packages and `undici-types` carry npm provenance attestations (published from GitHub
  Actions); `@types/node` does not, as before. `pnpm install --frozen-lockfile` is clean,
  `tsc --noEmit` exits 0, and `playwright test --list` enumerates 54 tests in 3 files. **The suite
  itself ran for the first time since SEC-014**, Playwright 1.63.0 on Chromium against the
  image above: **16 passed, 2 skipped**. The two skips are the post-page tests, which skip by
  design when the database has no published post (the dev DB holds one draft). Firefox and
  WebKit were not run; their browsers are not installed locally.

**Follow-ups.**
- A new advisory looked exactly like "Dependabot PRs are failing CI" for two weeks. Check the
  weekly scheduled `audit.yml` result as part of the Monday Dependabot pass. GitHub sends
  scheduled-workflow failure notifications only to whoever last edited the cron line.
- sqlx will attempt TLS the day the Postgres server offers it, with no client change. If
  personal-infra ever enables `ssl` on the shared Postgres, set `sslmode=verify-full` in the
  rendered `DATABASE_URL` rather than relying on `prefer`, which accepts an unauthenticated TLS
  downgrade to plaintext.
- `@types/node` 26.6.3 is current. Dependabot resolved #10 on 2026-09-14 and did not refresh it, so
  the 26.6 line comes next week.

### SEC-017 — The home page discloses the running release tag and commit

**Severity:** Low · **Status:** Accepted 2026-09-30 · **Where:** `.github/workflows/publish-image.yml`,
`Dockerfile`, `apps/backend/src/app/{constants.rs,pages/home.rs}`

**Finding.** The home page now ends in a footer naming the release and commit it was built from
(`v0.5.3 · build 17b2c7e`), at the owner's request. On a closed-source service that is
fingerprinting help: it maps the live site to a known version. Here it discloses almost nothing
new. The repository is public, every release is a public signed tag, and the image already
carries `org.opencontainers.image.source`. The one new fact is *which* release is live, i.e.
whether a fix has rolled out yet. That window is the host's 10-minute poll (personal-infra
AD-5), and the site has no login, admin surface or write path to aim at. Accepted on that basis.
Revisit if the repository ever goes private.

**Change.**
- `publish-image.yml` passes `APP_VERSION=${{ github.ref_name }}` and `APP_BUILD=${{ github.sha }}`
  as `build-args` of `docker/build-push-action`. No new action, permission, token or secret.
- `Dockerfile` declares both as `ARG` just above the compile `RUN`, which sees them as environment
  variables. They are not interpolated into its command text. `.git/` stays out of the build
  context (SEC-010); the build args are how the build learns the tag and commit.
- `app/constants.rs` reads them with `option_env!` at compile time into `APP_VERSION` (falling back
  to `"dev"`) and `APP_BUILD` (`None`). Unset and empty both count as absent. `pages/home.rs` renders
  them as one text node in `<footer class="home__footer">`, shortening the commit to seven
  characters. Nothing is read at runtime.

**Why the new inputs cannot inject anything.** `github.ref_name` can only be a tag the `on:` filter
`v[0-9]+.[0-9]+.[0-9]+` admits (digits and dots; in Actions filter patterns `+` repeats the
preceding character and `.` is literal). Only repository admins can create `v*` tags (SEC-008).
`github.sha` is 40 hex characters. Both reach the action as `with:` inputs, never through a
`run:` shell. Inside the build they are plain environment variables of a `RUN` that does not
mention them. In the page they are a compile-time `&'static str`, which Leptos escapes as text.

**Verification.**
- The production `Dockerfile` was built locally with `--build-arg APP_VERSION=v0.0.0
  --build-arg APP_BUILD=<HEAD>` (native arm64, not CI's linux/amd64) and run against the dev
  database. The SSR HTML of `/` carries `<footer class="home__footer">v0.0.0 · build 17b2c7e</footer>`.
  `/about` and `/blog` have no footer.
- Both values, the version and the full 40-character commit, are present in the image's
  `kenespartadev.wasm`, so the hydrate build saw the same build args as the server build.
- Hydration: `/` hydrates with no console error or warning. Navigating from `/about` to `/` by its
  link made **zero** document requests, so `HomePage` was rendered by the wasm alone, and it
  rendered the identical footer.
- Tests: 12/12 backend unit tests, including two new `release_stamp` tests (full commit
  shortened to seven characters; no commit → version only). The Chromium e2e suite passed 19 with
  2 skipped against that image. The 19 include three new tests: the stamp is on `/` in either
  its `dev` or its `vX.Y.Z · build <7 hex>` form, and absent from `/about` and `/blog`. The two
  skips are the post-page tests, as in SEC-016. `cargo fmt --check`, the `hydrate` wasm check and
  `tsc --noEmit` are clean. Clippy's one warning (`result_large_err`, `seo.rs:246`) predates this
  change.
- The workflow side runs only on the next `v*` tag. That release's footer is its verification: it
  must read `vX.Y.Z · build <first 7 of the tagged commit>`.

**Follow-ups.** None. The unstamped path (local `cargo leptos watch`, `Dockerfile.dev`, a
`docker build` without args) renders `dev`. The constants treat unset and empty build args alike,
so it does not matter which one Docker produces for an `ARG` given no value.

## Open findings

**None.** SEC-013 was the last one open and closed on 2026-08-31, verified against live AWS
rather than against Terraform state — every finding of the 2026-08-23 audit (SEC-001 … SEC-012)
plus SEC-013 … SEC-016 is now Fixed above, and SEC-017 is Accepted.

Two things are *tracked but not findings against this repository*, both recorded in full on their
entries: `typst-resume`'s OIDC trust lists only the plain subject spelling and will break opaquely
if GitHub switches it to the immutable form (SEC-013 follow-up, fixed in personal-infra, not
here); and `paste`/`proc-macro-error2` remain `unmaintained` warnings arriving through leptos
(SEC-007). New findings get the next id and start here until fixed or accepted.

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
