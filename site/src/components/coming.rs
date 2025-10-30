use leptos::prelude::*;

#[component]
pub fn ComingSoon(title: &'static str) -> impl IntoView {
    let title = format!("\"{}\" is under construction", title);
    view! {
        <div class="coming-soon-page">
            <div class="coming-soon-container">
                <div class="coming-soon-content">
                    <div class="coming-soon-icon">
                        <svg xmlns="http://www.w3.org/2000/svg" width="80" height="80" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                            <circle cx="12" cy="12" r="10"></circle>
                            <polyline points="12 6 12 12 16 14"></polyline>
                        </svg>
                    </div>
                    <p class="coming-soon-title">{title}</p>
                </div>
            </div>
        </div>
    }
}
