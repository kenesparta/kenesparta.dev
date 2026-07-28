//! Crawler endpoints: `/sitemap.xml` and `/feed.xml` (RSS 2.0).
//!
//! Hand-rolled XML over the same use case the pages consume; two small
//! documents are not worth a feed/sitemap dependency. Emitted as a single
//! line — only parsers read these.

use axum::extract::State;
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use bc_blog::application::dto::BlogPostSummaryDTO;
use chrono::DateTime;

use crate::app::constants::{BLOG_DESCRIPTION, SITE_URL};
use crate::http::ServerState;

/// Routable pages that exist independent of database content.
const STATIC_PATHS: &[&str] = &["/", "/about", "/blog", "/experience", "/projects"];

pub async fn sitemap(State(state): State<ServerState>) -> Response {
    let posts = match published_posts(&state).await {
        Ok(posts) => posts,
        Err(response) => return response,
    };

    let mut body = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?><urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">"#,
    );
    for path in STATIC_PATHS {
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

    xml_response("application/xml; charset=utf-8", body)
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

    xml_response("application/rss+xml; charset=utf-8", body)
}

/// Every published post: sitemap and feed must be complete, so no page cap.
async fn published_posts(state: &ServerState) -> Result<Vec<BlogPostSummaryDTO>, Response> {
    state
        .container
        .blog
        .list_published
        .execute(i32::MAX)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

fn xml_response(content_type: &'static str, body: String) -> Response {
    ([(header::CONTENT_TYPE, content_type)], body).into_response()
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
