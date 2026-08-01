//! Ingest CLI: content/posts/*.md → HTML → upsert into Postgres by slug.
//!
//! Reuses the exact same composition as the server (pool + migrations), so it
//! also bootstraps an empty database. The web app never writes; this bin is
//! the only write path.
//!
//! Usage: `DATABASE_URL=... ingest [content-dir] [--prune]` (default:
//! `content/posts`). `--prune` deletes every DB post whose slug has no
//! matching `.md` file, so the database mirrors the content dir exactly.
//! Typically run via `make blog/ingest` / `make blog/publish` (sops exec-env).

use std::path::Path;

use backend::composition;
use backend::configuration::Configuration;
use bc_blog::application::use_cases::UpsertPostCommand;
use pulldown_cmark::{Options, Parser, html};
use serde::Deserialize;

/// TOML frontmatter between `+++` fences (Zola-style).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Frontmatter {
    title: String,
    summary: String,
    /// RFC 3339, e.g. "2026-07-21T00:00:00Z". Becomes created_at (first
    /// ingest) and published_at (when status = "published").
    date: String,
    #[serde(default = "default_author")]
    author: String,
    #[serde(default)]
    tags: Vec<String>,
    /// Defaults to draft: nothing goes public by omission.
    #[serde(default)]
    status: Status,
    /// Defaults to the file stem. Set it explicitly to rename the file
    /// without breaking the URL (the slug is the upsert key).
    slug: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Status {
    #[default]
    Draft,
    Published,
}

fn default_author() -> String {
    "Ken Esparta".to_string()
}

/// Splits `+++\n<toml>\n+++` from the markdown body.
fn split_frontmatter(raw: &str) -> Result<(&str, &str), String> {
    let rest = raw
        .strip_prefix("+++")
        .ok_or("file must start with a `+++` frontmatter fence")?;
    let end = rest
        .find("\n+++")
        .ok_or("unclosed `+++` frontmatter fence")?;
    let body = rest[end + 4..].trim_start_matches('\n');
    Ok((&rest[..end], body))
}

fn render_markdown(markdown: &str) -> String {
    let options =
        Options::ENABLE_TABLES | Options::ENABLE_FOOTNOTES | Options::ENABLE_STRIKETHROUGH;
    let mut out = String::with_capacity(markdown.len() * 2);
    html::push_html(&mut out, Parser::new_ext(markdown, options));
    out
}

fn parse_post(path: &Path) -> Result<UpsertPostCommand, Box<dyn std::error::Error>> {
    let raw = std::fs::read_to_string(path)?;
    let (front, body) = split_frontmatter(&raw)?;
    let fm: Frontmatter = toml::from_str(front)?;
    let date = chrono::DateTime::parse_from_rfc3339(&fm.date)
        .map_err(|e| format!("invalid `date` (want RFC 3339): {e}"))?
        .timestamp();
    let slug = match fm.slug {
        Some(slug) => slug,
        None => path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or("file name is not valid UTF-8")?
            .to_string(),
    };

    Ok(UpsertPostCommand {
        title: fm.title,
        slug,
        content_html: render_markdown(body),
        // The body verbatim, minus the frontmatter: `/blog/<slug>.md` serves
        // it back to agents, so it stays the authored source, not a round-trip.
        content_md: body.to_string(),
        summary: fm.summary,
        author: fm.author,
        tags: fm.tags,
        published: matches!(fm.status, Status::Published),
        date,
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    backend::telemetry::init();

    let mut dir = "content/posts".to_string();
    let mut prune = false;
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--prune" => prune = true,
            flag if flag.starts_with('-') => {
                return Err(format!(
                    "unknown flag `{flag}` (usage: ingest [content-dir] [--prune])"
                )
                .into());
            }
            path => dir = path.to_string(),
        }
    }

    let config = Configuration::from_env()?;
    let container = composition::compose(&config).await?;

    let mut paths: Vec<_> = std::fs::read_dir(&dir)
        .map_err(|e| format!("cannot read content dir `{dir}`: {e}"))?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.extension().is_some_and(|ext| ext == "md"))
        .collect();
    paths.sort();

    if paths.is_empty() {
        // Deliberately also skips --prune: an empty (or mistyped) content dir
        // must never wipe the whole table.
        tracing::warn!(dir = %dir, "no .md files — nothing to do");
        return Ok(());
    }

    let mut slugs = Vec::with_capacity(paths.len());
    for path in &paths {
        let cmd = parse_post(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let slug = cmd.slug.clone();
        container
            .blog
            .upsert
            .execute(cmd)
            .await
            .map_err(|e| format!("{}: {e}", path.display()))?;
        tracing::info!(slug = %slug, "upserted");
        slugs.push(slug);
    }

    if prune {
        let pruned = container.blog.prune.execute(&slugs).await?;
        for slug in &pruned {
            tracing::info!(slug = %slug, "pruned");
        }
        tracing::info!(upserted = slugs.len(), pruned = pruned.len(), "done");
    } else {
        tracing::info!(upserted = slugs.len(), "done");
    }
    Ok(())
}
