-- no-transaction
-- The blog's whole schema, as ONE statement (personal-infra §5.22, G51).
--
-- It replaces the two migrations of the Postgres era, which Aurora DSQL cannot
-- run: `tags text[]` (DSQL stores no array columns — tags are jsonb now) and
-- `ADD COLUMN content_md ... NOT NULL DEFAULT ''` (DSQL refuses ADD COLUMN with
-- a constraint). The `-- no-transaction` line above must stay first: sqlx then
-- runs this statement on its own and records it separately, the only shape
-- DSQL accepts, and that works on the local Postgres just the same.
--
-- The partial index on published posts is gone. DSQL builds indexes only with
-- CREATE INDEX ASYNC, which plain Postgres does not parse, and at this table's
-- size a scan is what either planner picks anyway. Revisit with hundreds of
-- posts, as a DSQL-only migration.
--
-- A local database created before this file must be recreated
-- (`docker compose -f docker-compose.dev.yml down -v`): its history names
-- migrations that no longer exist, and its `blog_posts` has the old shape.
CREATE TABLE blog_posts (
    post_id      uuid PRIMARY KEY,
    title        text NOT NULL,
    slug         text NOT NULL UNIQUE,
    content      text NOT NULL,
    content_md   text NOT NULL DEFAULT '',
    summary      text NOT NULL,
    author       text NOT NULL,
    tags         jsonb NOT NULL DEFAULT '[]'::jsonb,
    status       text NOT NULL CHECK (status IN ('draft', 'published')),
    created_at   timestamptz NOT NULL DEFAULT now(),
    updated_at   timestamptz NOT NULL DEFAULT now(),
    published_at timestamptz
)
