-- Raw Markdown source of each post, alongside the rendered `content` HTML.
--
-- Feeds the `/blog/<slug>.md` variants that `/llms.txt` points crawlers at, so
-- agents read the authored source instead of re-parsing hydrated HTML. The
-- pages keep rendering `content`; nothing on the read path changes.
--
-- Rows ingested before this column existed backfill to '': re-run
-- `make blog/ingest` (or `blog/publish`) to fill them. Until then the .md
-- endpoint 404s rather than serving a blank document.
ALTER TABLE blog_posts ADD COLUMN content_md text NOT NULL DEFAULT '';
