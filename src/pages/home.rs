use crate::SocialLinks;
use leptos::prelude::*;

pub struct Experience {
    pub situation: String,
    pub task: String,
    pub action: String,
    pub result: String,
}

#[component]
pub fn HomePage() -> impl IntoView {
    let description = r#"
Engineer with 8+ years of experience, specializing in Go and Rust microservices. I architect and implement highly efficient, secure backend systems across energy, VOD, and finance. My focus is on optimizing API performance and bolstering network security to maximize system availability.
"#;

    view! {
        <div class="">
            <img src="/img/photo.webp" alt="Logo" class="main__logo" />
            <h1 class="delius-swash-caps main__title">"Ken Esparta"</h1>
            <h2 class="mooli main__subtitle">"Senior Software Engineer"</h2>
            <SocialLinks/>
            <p class="main__description">
                {description.to_string()}
            </p>
        </div>
    }
}
