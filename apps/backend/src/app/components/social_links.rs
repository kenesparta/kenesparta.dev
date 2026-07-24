use super::icons::{CodebergIcon, GithubIcon};
use crate::app::constants::BUCKET_URL;
use leptos::prelude::*;
use leptos::{IntoView, component, view};

#[component]
fn SocialLink(
    #[prop(into)] href: String,
    #[prop(into)] aria_label: String,
    #[prop(optional, into)] download: Option<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <a
            href=href
            target="_blank"
            rel="noopener noreferrer"
            download=download
            class="social-links__element"
            aria-label=aria_label
        >
            {children()}
        </a>
    }
}

#[component]
pub fn SocialLinks() -> impl IntoView {
    let resume = format!("{}/cv/ken_esparta_cv.pdf", BUCKET_URL);
    view! {
        <div class="social-links">
            <SocialLink href="https://github.com/kenesparta" aria_label="Visit my GitHub profile">
                <GithubIcon/>
            </SocialLink>

            <SocialLink
                href="https://codeberg.org/kenesparta"
                aria_label="Visit my Codeberg profile"
            >
                <CodebergIcon/>
            </SocialLink>

            <SocialLink
                href="https://linkedin.com/in/kenesparta"
                aria_label="Visit my LinkedIn profile"
            >
                <svg
                    width="30"
                    height="30"
                    viewBox="0 0 30 30"
                    fill="currentColor"
                    xmlns="http://www.w3.org/2000/svg"
                >
                    <path d="M20.447 20.452h-3.554v-5.569c0-1.328-.027-3.037-1.852-3.037-1.853 0-2.136 1.445-2.136 2.939v5.667H9.351V9h3.414v1.561h.046c.477-.9 1.637-1.85 3.37-1.85 3.601 0 4.267 2.37 4.267 5.455v6.286zM5.337 7.433c-1.144 0-2.063-.926-2.063-2.065 0-1.138.92-2.063 2.063-2.063 1.14 0 2.064.925 2.064 2.063 0 1.139-.925 2.065-2.064 2.065zm1.782 13.019H3.555V9h3.564v11.452zM22.225 0H1.771C.792 0 0 .774 0 1.729v20.542C0 23.227.792 24 1.771 24h20.451C23.2 24 24 23.227 24 22.271V1.729C24 .774 23.2 0 22.222 0h.003z"/>
                </svg>
            </SocialLink>

            <SocialLink
                href=resume
                download="kenesparta-resume.pdf"
                aria_label="Download my Resume"
            >
                <svg
                    width="33"
                    height="33"
                    viewBox="0 0 30 30"
                    fill="currentColor"
                    xmlns="http://www.w3.org/2000/svg"
                >
                    <path d="M14 2H6c-1.1 0-1.99.9-1.99 2L4 20c0 1.1.89 2 1.99 2H18c1.1 0 2-.9 2-2V8l-6-6zm2 16H8v-2h8v2zm0-4H8v-2h8v2zm-3-5V3.5L18.5 9H13z"/>
                </svg>
            </SocialLink>
        </div>
    }
}
