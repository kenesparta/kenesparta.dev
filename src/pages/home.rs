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
Senior software engineer with 8+ years of experience. Specializes in the leading, architecting, implementing of highly
efficient, highly secure backend microservices in Go and Rust. My expertise spans the full Software Development Life Cycle
across diverse industries, including energy, video-on-demand, and finance.
I focus on driving system efficiency, notably by optimizing API calls efficiency and architecting secure network
infrastructure to improve application availability.
I am committed to building robust, high-quality software with keen attention to detail.
"#;

    view! {
        <div class="">
            <img src="/img/photo.webp" alt="Logo" class="main__logo" />
            <h1 class="delius-swash-caps main__title">"Ken Esparta"</h1>
            <h2 class="mooli main__subtitle">"Software Engineer"</h2>
            <SocialLinks/>
            <p class="main__description">
                {description.to_string()}
            </p>
        </div>
    }
}
