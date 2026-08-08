//! Crawler endpoints: `/sitemap.xml`, `/feed.xml` (RSS 2.0), `/llms.txt` and
//! the `/blog/<slug>.md` Markdown variants that llms.txt points at.
//!
//! Hand-rolled over the same use cases the pages consume; a handful of small
//! documents are not worth a feed/sitemap dependency. The XML is emitted as a
//! single line — only parsers read it.

use axum::extract::{Path, Request, State};
use axum::http::{StatusCode, Uri, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use bc_blog::application::dto::BlogPostSummaryDTO;
use bc_blog::domain::model::PostStatus;
use chrono::DateTime;

use crate::app::constants::{BLOG_DESCRIPTION, META_DESCRIPTION, SITE_URL};
use crate::http::ServerState;

/// Routable pages that exist independent of database content, as
/// `(path, label, description)` — the description is what llms.txt annotates
/// each link with; the sitemap only reads the path.
const PAGES: &[(&str, &str, &str)] = &[
    (
        "/",
        "Home",
        "Landing page: who I am, with links to my public profiles.",
    ),
    (
        "/about",
        "About",
        "Background, what I work on, and how I approach engineering.",
    ),
    ("/blog", "Blog", "Index of every published post."),
    (
        "/experience",
        "Experience",
        "Professional experience, role by role.",
    ),
    (
        "/projects",
        "Projects",
        "Selected personal and open-source projects.",
    ),
];

pub async fn sitemap(State(state): State<ServerState>) -> Response {
    let posts = match published_posts(&state).await {
        Ok(posts) => posts,
        Err(response) => return response,
    };

    let mut body = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?><urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">"#,
    );
    for (path, _, _) in PAGES {
        body.push_str(&format!("<url><loc>{SITE_URL}{path}</loc></url>"));
    }
    for post in &posts {
        let location = format!("{SITE_URL}/blog/{}", escape_xml(&post.slug));
        // lastmod is the ingest's updated_at: re-publishing an edited post
        // is what tells crawlers to come back.
        match rfc3339(post.updated_at) {
            Some(modified) => body.push_str(&format!(
                "<url><loc>{location}</loc><lastmod>{modified}</lastmod></url>"
            )),
            None => body.push_str(&format!("<url><loc>{location}</loc></url>")),
        }
    }
    body.push_str("</urlset>");

    text_response("application/xml; charset=utf-8", body)
}

pub async fn feed(State(state): State<ServerState>) -> Response {
    let posts = match published_posts(&state).await {
        Ok(posts) => posts,
        Err(response) => return response,
    };

    let mut body = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?><rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom"><channel>"#,
    );
    body.push_str(&format!(
        r#"<title>Ken Esparta - Blog</title><link>{SITE_URL}/blog</link><description>{}</description><language>en</language><atom:link href="{SITE_URL}/feed.xml" rel="self" type="application/rss+xml"/>"#,
        escape_xml(BLOG_DESCRIPTION)
    ));
    for post in &posts {
        let link = format!("{SITE_URL}/blog/{}", escape_xml(&post.slug));
        let item_date = rfc2822(post.published_at.unwrap_or(post.created_at));
        body.push_str(&format!(
            r#"<item><title>{}</title><link>{link}</link><guid isPermaLink="true">{link}</guid><pubDate>{item_date}</pubDate><description>{}</description></item>"#,
            escape_xml(&post.title),
            escape_xml(&post.summary),
        ));
    }
    body.push_str("</channel></rss>");

    text_response("application/rss+xml; charset=utf-8", body)
}

/// `/llms.txt` — the llmstxt.org index: an H1, a blockquote summary, then
/// H2-delimited link lists. Points at the `.md` variants rather than the
/// pages, so an agent that follows a link gets the source instead of hydrated
/// HTML.
pub async fn llms_txt(State(state): State<ServerState>) -> Response {
    let posts = match published_posts(&state).await {
        Ok(posts) => posts,
        Err(response) => return response,
    };

    let mut body = format!("# Ken Esparta\n\n> {}\n\n", one_line(META_DESCRIPTION));
    body.push_str(
        "Personal site and blog. Every post below is also available as clean Markdown at its \
         page URL plus a `.md` suffix — prefer those over the HTML pages.\n\n",
    );

    body.push_str("## Blog\n\n");
    if posts.is_empty() {
        body.push_str("No published posts yet.\n");
    }
    for post in &posts {
        body.push_str(&format!(
            "- [{}]({SITE_URL}/blog/{}.md): {}\n",
            link_text(&post.title),
            post.slug,
            one_line(&post.summary),
        ));
    }

    body.push_str("\n## Pages\n\n");
    for (path, label, description) in PAGES {
        body.push_str(&format!("- [{label}]({SITE_URL}{path}): {description}\n"));
    }

    body.push_str("\n## Optional\n\n");
    body.push_str(&format!(
        "- [RSS feed]({SITE_URL}/feed.xml): {}\n- [Sitemap]({SITE_URL}/sitemap.xml): Every \
         indexable URL on the site.\n",
        one_line(BLOG_DESCRIPTION),
    ));

    text_response("text/plain; charset=utf-8", body)
}

/// `/blog/<slug>.md` (reaches here as `/blog-md/{slug}`, see
/// [`rewrite_markdown_suffix`]) — the authored Markdown source of one post.
pub async fn post_markdown(State(state): State<ServerState>, Path(slug): Path<String>) -> Response {
    let post = match state.container.blog.get_markdown.execute(&slug).await {
        Ok(Some(post)) => post,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(error) => {
            tracing::error!(error = %error, slug = %slug, "loading post markdown failed");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // Drafts are public nowhere else, so the .md variant must not leak them.
    // An empty source means the row predates the content_md column: 404 until
    // the next ingest rather than serve a blank document.
    if post.status != PostStatus::Published.as_str() || post.content_md.trim().is_empty() {
        return StatusCode::NOT_FOUND.into_response();
    }

    // The title lives in the frontmatter, not the body: prepend it so the
    // served document stands on its own.
    let body = format!("# {}\n\n{}", post.title, post.content_md);
    text_response("text/markdown; charset=utf-8", body)
}

/// Redirects any non-root path with a trailing slash to its slash-free form
/// (`/blog/` → `/blog`) with a permanent, method-preserving 308, before routing.
///
/// Two reasons. Canonical hygiene: the site advertises one URL per page
/// (per-page canonicals, sitemap, llms.txt), so a trailing-slash duplicate
/// should collapse onto the canonical. And safety: Leptos's SSR matcher accepts
/// a trailing slash, so `/blog/` resolves to the async blog route, but the Axum
/// route generated for it is the slash-free `/blog` — the trailing-slash form
/// would fall through to the context-less error handler and, before this,
/// panic the worker inside the data resource. Normalizing first keeps every
/// data route on the path that carries its dependencies. Runs ahead of
/// [`rewrite_markdown_suffix`] so `/blog/<slug>.md/` normalizes before rewrite.
///
/// The redirect target is always a single-slash, same-origin absolute path.
/// Trimming only the trailing slash is not enough: `//evil.com/` would yield
/// `Location: //evil.com`, a protocol-relative URL browsers resolve off-site —
/// an open redirect. Collapsing the leading slashes (and a leading backslash,
/// which browsers fold to `/`) closes that.
pub async fn redirect_trailing_slash(request: Request, next: Next) -> Response {
    let path = request.uri().path();
    if path.len() > 1 && path.ends_with('/') {
        // Strip leading and trailing `/` and `\`, then re-prefix exactly one
        // `/`. `//evil.com/` → `/evil.com` (same origin), `/blog/` → `/blog`,
        // an all-slash path (`//`, `///`) → `/`.
        let body = path.trim_matches(|c| c == '/' || c == '\\');
        let target = format!("/{body}");
        let location = match request.uri().query() {
            Some(query) => format!("{target}?{query}"),
            None => target,
        };
        return (StatusCode::PERMANENT_REDIRECT, [(header::LOCATION, location)]).into_response();
    }

    next.run(request).await
}

/// Rewrites `/blog/<slug>.md` to `/blog-md/<slug>` before routing.
///
/// llms.txt asks for Markdown variants at the page URL plus `.md`, but matchit
/// (axum's router) cannot express a dynamic segment with a literal suffix —
/// "dynamic suffixes are not currently supported" — and `/blog/{slug}` is
/// already claimed by the Leptos page route. Rewriting keeps the public URL
/// canonical while the router only ever sees a statically-prefixed path.
pub async fn rewrite_markdown_suffix(mut request: Request, next: Next) -> Response {
    // Scoped so the borrow of `request` ends before `uri_mut()`.
    let rewritten = {
        let uri = request.uri();
        uri.path()
            .strip_prefix("/blog/")
            .and_then(|rest| rest.strip_suffix(".md"))
            // A nested or empty slug is not a post: leave it to the 404 path.
            .filter(|slug| !slug.is_empty() && !slug.contains('/'))
            .map(|slug| match uri.query() {
                Some(query) => format!("/blog-md/{slug}?{query}"),
                None => format!("/blog-md/{slug}"),
            })
    };

    if let Some(rewritten) = rewritten
        && let Ok(uri) = rewritten.parse::<Uri>()
    {
        *request.uri_mut() = uri;
    }

    next.run(request).await
}

/// Every published post: sitemap and feed must be complete, so no page cap.
async fn published_posts(state: &ServerState) -> Result<Vec<BlogPostSummaryDTO>, Response> {
    state
        .container
        .blog
        .list_published
        .execute(i32::MAX)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "listing published posts failed");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        })
}

fn text_response(content_type: &'static str, body: String) -> Response {
    ([(header::CONTENT_TYPE, content_type)], body).into_response()
}

/// llms.txt list items are one line each: fold any newline in author-written
/// text so a multi-line summary cannot break out of its bullet.
fn one_line(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Brackets in a title would truncate the Markdown link wrapping it.
fn link_text(text: &str) -> String {
    one_line(text).replace('[', "\\[").replace(']', "\\]")
}

fn escape_xml(raw: &str) -> String {
    raw.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn rfc3339(timestamp: i64) -> Option<String> {
    DateTime::from_timestamp(timestamp, 0).map(|date| date.to_rfc3339())
}

fn rfc2822(timestamp: i64) -> String {
    DateTime::from_timestamp(timestamp, 0)
        .map(|date| date.to_rfc2822())
        .unwrap_or_default()
}
