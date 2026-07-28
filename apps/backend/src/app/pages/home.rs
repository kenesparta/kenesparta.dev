use crate::app::components::{HeaderLinks, PageMeta, PersonJsonLd, SocialLinks};
use crate::app::constants::{BUCKET_URL, META_DESCRIPTION, META_TITLE};
use leptos::prelude::*;

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
        </div>
    }
}
