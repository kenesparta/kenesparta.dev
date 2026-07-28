use crate::app::components::blog::tags::Tags;
use crate::app::components::blog::utils::{published_date, rfc3339};
use crate::app::components::go_back::GoBack;
use crate::app::components::page_meta::{PageMeta, json_ld};
use crate::app::constants::SITE_URL;
use bc_blog::application::dto::BlogPostDTO;
use leptos::prelude::*;
use leptos::{IntoView, component, view};
use leptos_meta::Meta;

#[component]
pub fn Article(post: BlogPostDTO) -> impl IntoView {
    let path = format!("/blog/{}", post.slug);
    let date_text = published_date(post.published_at);
    let published_iso = rfc3339(post.published_at);
    let modified_iso = rfc3339(Some(post.updated_at));

    let mut schema = serde_json::json!({
        "@context": "https://schema.org",
        "@type": "BlogPosting",
        "headline": post.title,
        "description": post.summary,
        "mainEntityOfPage": {"@type": "WebPage", "@id": format!("{SITE_URL}{path}")},
        "author": {"@type": "Person", "name": post.author, "url": SITE_URL},
        "keywords": post.tags.join(", "),
    });
    if let Some(published) = &published_iso {
        schema["datePublished"] = serde_json::json!(published);
    }
    if let Some(modified) = &modified_iso {
        schema["dateModified"] = serde_json::json!(modified);
    }
    let schema = json_ld(&schema);

    // Drafts stay reachable by slug for preview; keep them out of indexes.
    let draft = post.status != "published";

    view! {
        <article class="blog-post">
            <PageMeta
                title=format!("{} - Ken Esparta", post.title)
                description=post.summary.clone()
                path=path
                og_type="article"
            />
            <Meta name="author" content=post.author.clone()/>
            {published_iso
                .clone()
                .map(|iso| view! { <Meta property="article:published_time" content=iso/> })}
            {modified_iso.map(|iso| view! { <Meta property="article:modified_time" content=iso/> })}
            {draft.then(|| view! { <Meta name="robots" content="noindex"/> })}
            <script type="application/ld+json" inner_html=schema></script>

            <header class="post-header">
                <GoBack go_to="blog" text="Back to Blog"/>
                <h1 class="post-title">{post.title}</h1>
                <div class="post-meta">
                    <span class="post-author">{post.author}</span>
                    <time class="post-date" datetime=published_iso>{date_text}</time>
                </div>

                <Tags tags=post.tags/>
            </header>

            <div class="post-content" inner_html=post.content></div>

            <footer class="post-footer">
                <GoBack go_to="blog" text="All posts"/>
            </footer>
        </article>
    }
    .into_any()
}
