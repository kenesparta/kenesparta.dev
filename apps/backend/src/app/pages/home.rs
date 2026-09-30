use crate::app::components::{HeaderLinks, PageMeta, PersonJsonLd, SocialLinks};
use crate::app::constants::{APP_BUILD, APP_VERSION, BUCKET_URL, META_DESCRIPTION, META_TITLE};
use leptos::prelude::*;

/// "v0.5.3 · build 17b2c7e", or just the version when no commit was stamped.
/// One string, so the footer is a single text node on both sides of hydration.
fn release_stamp(version: &str, build: Option<&str>) -> String {
    match build {
        Some(commit) => format!("{version} · build {}", commit.get(..7).unwrap_or(commit)),
        None => version.to_string(),
    }
}

#[component]
pub fn HomePage() -> impl IntoView {
    let description = r#"
Engineer with 8+ years of experience, specializing in Go and Rust microservices. I architect and implement highly efficient, secure backend systems across energy, VOD, and finance. My focus is on optimizing API performance and bolstering network security to maximize system availability.
"#;
    let photo = format!("{}/img/photo.webp", BUCKET_URL);

    view! {
        <PageMeta title=META_TITLE description=META_DESCRIPTION path="/"/>
        <div class="home-container">
            <img src={photo} alt="Logo" class="home__logo" />
            <h1 class="delius-swash-caps home__title">"Ken Esparta"</h1>
            <h2 class="mooli home__subtitle">"Senior Software Engineer"</h2>
            <SocialLinks/>
            <HeaderLinks/>
            <p class="home__description">
                {description.to_string()}
            </p>
            <PersonJsonLd/>
            <footer class="home__footer">{release_stamp(APP_VERSION, APP_BUILD)}</footer>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::release_stamp;

    #[test]
    fn stamp_shows_the_version_and_the_short_commit() {
        let commit = "17b2c7ea3355dcab87b1b3a794d1268663d87fbf";
        assert_eq!(
            release_stamp("v0.5.3", Some(commit)),
            "v0.5.3 · build 17b2c7e"
        );
    }

    #[test]
    fn unstamped_builds_show_only_the_version() {
        assert_eq!(release_stamp("dev", None), "dev");
    }
}
