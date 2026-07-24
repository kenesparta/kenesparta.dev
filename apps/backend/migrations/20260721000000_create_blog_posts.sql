CREATE TABLE blog_posts (
    post_id      uuid PRIMARY KEY,
    title        text NOT NULL,
    slug         text NOT NULL UNIQUE,
    content      text NOT NULL,
    summary      text NOT NULL,
    author       text NOT NULL,
    tags         text[] NOT NULL DEFAULT '{}',
    status       text NOT NULL CHECK (status IN ('draft', 'published')),
    created_at   timestamptz NOT NULL DEFAULT now(),
    updated_at   timestamptz NOT NULL DEFAULT now(),
    published_at timestamptz
);

-- Partial index matching the only hot query:
-- WHERE status = 'published' ORDER BY created_at DESC LIMIT n.
-- Drafts never pollute it; smaller than a composite (status, created_at).
CREATE INDEX blog_posts_published_created_at_idx
    ON blog_posts (created_at DESC)
    WHERE status = 'published';
