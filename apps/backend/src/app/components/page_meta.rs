use crate::app::constants::{
    BUCKET_URL, CODEBERG_URL, GITHUB_URL, ICON_URL, LINKEDIN_URL, SITE_URL,
};
use leptos::prelude::*;
use leptos_meta::{Link, Meta, Title};

/// Per-page head block: title, description, canonical URL and Open Graph.
///
/// Every routable page mounts exactly one, so no two URLs share a
/// title/description (crawlers treat duplicates as boilerplate and skip the
/// page) and every URL declares its own canonical.
#[component]
pub fn PageMeta(
    #[prop(into)] title: String,
    #[prop(into)] description: String,
    /// Absolute path of this page ("/", "/blog", "/blog/<slug>"); the
    /// canonical and og:url are `SITE_URL` + this path.
    #[prop(into)]
    path: String,
    /// Open Graph object type; defaults to "website", posts pass "article".
    #[prop(optional)]
    og_type: Option<&'static str>,
) -> impl IntoView {
    let url = format!("{SITE_URL}{path}");
    view! {
        <Title text=title.clone()/>
        <Meta name="description" content=description.clone()/>
        <Link rel="canonical" href=url.clone()/>
        <Meta property="og:url" content=url/>
        <Meta property="og:type" content=og_type.unwrap_or("website")/>
        <Meta property="og:title" content=title/>
        <Meta property="og:description" content=description/>
        <Meta property="og:image" content=ICON_URL/>
    }
}

/// Serialize a schema.org value for embedding in a `<script>` element.
///
/// `<` is escaped as `\\u003c` (equivalent inside JSON strings) so content
/// containing "</script>" cannot terminate the element early.
pub fn json_ld(value: &serde_json::Value) -> String {
    value.to_string().replace('<', "\\u003c")
}

/// schema.org `Person` entity, linking the site to the public profiles.
/// JSON-LD is valid anywhere in the body, so this renders in place.
#[component]
pub fn PersonJsonLd() -> impl IntoView {
    let person = json_ld(&serde_json::json!({
        "@context": "https://schema.org",
        "@type": "Person",
        "name": "Ken Esparta",
        "jobTitle": "Senior Software Engineer",
        "url": SITE_URL,
        "image": format!("{BUCKET_URL}/img/photo.webp"),
        "sameAs": [GITHUB_URL, CODEBERG_URL, LINKEDIN_URL],
    }));
    view! { <script type="application/ld+json" inner_html=person></script> }
}
