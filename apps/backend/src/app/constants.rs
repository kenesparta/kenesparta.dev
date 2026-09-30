pub const SITE_URL: &str = "https://kenesparta.dev";
pub const BUCKET_URL: &str = "https://cdn.kenesparta.dev";
pub const ICON_URL: &str = "https://cdn.kenesparta.dev/img/icon.svg";
pub const META_TITLE: &str = "Ken Esparta - Senior Software Engineer";
pub const META_DESCRIPTION: &str = "A Senior Software Engineer with expertise in software development, cloud architecture, and designing scalable solutions. I am passionate about building innovative software and exploring cutting-edge technologies.";
pub const BLOG_DESCRIPTION: &str =
    "Articles by Ken Esparta on Rust, Go, backend engineering, and cloud infrastructure.";

// Public profiles: rendered as social links and as `sameAs` in the Person
// JSON-LD, so search engines resolve them to the same entity.
pub const GITHUB_URL: &str = "https://github.com/kenesparta";
pub const CODEBERG_URL: &str = "https://codeberg.org/kenesparta";
pub const LINKEDIN_URL: &str = "https://linkedin.com/in/kenesparta";

// Contact address: rendered as a `mailto:` social link, shown verbatim on
// /about, and declared as `email` in the Person JSON-LD.
pub const EMAIL: &str = "kenesparta@pm.me";

// Release stamp shown in the home-page footer. publish-image.yml passes the
// tag and commit as Docker build args (the build context excludes .git/, so
// the build cannot ask git) and option_env! reads them at compile time — into
// the server binary and the wasm bundle alike, so the hydrated footer matches
// the SSR one. Local builds have neither and show "dev".
pub const APP_VERSION: &str = match option_env!("APP_VERSION") {
    Some(version) if !version.is_empty() => version,
    _ => "dev",
};
pub const APP_BUILD: Option<&str> = match option_env!("APP_BUILD") {
    Some(commit) if !commit.is_empty() => Some(commit),
    _ => None,
};

pub const GLOBAL_FONTS: &[&str] = &[
    "solway-v19-latin-regular.woff2",
    "solway-v19-latin-700.woff2",
    "solway-v19-latin-800.woff2",
    "delius-swash-caps-v25-latin-regular.woff2",
    "mooli-v1-latin-regular.woff2",
];
