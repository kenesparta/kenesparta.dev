use crate::app::api::get_published_posts;
use crate::app::components::{BlogPostList, PageMeta};
use crate::app::constants::BLOG_DESCRIPTION;
use leptos::prelude::*;

#[component]
pub fn BlogList() -> impl IntoView {
    let posts_resource = Resource::new(|| (), |_| async { get_published_posts(Some(20)).await });

    view! {
        <PageMeta title="Blog - Ken Esparta" description=BLOG_DESCRIPTION path="/blog"/>
        <div class="blog-container">
            <Suspense fallback=move || {
                view! { <div class="loading">"Loading posts..."</div> }
            }>
                {move || Suspend::new(async move {
                    match posts_resource.await {
                        Ok(posts) => {
                            view! { <BlogPostList posts=posts/> }.into_any()
                        }
                        // Logged server-side (api.rs); the text never reaches
                        // the page — it could describe the database.
                        Err(_) => {
                            #[cfg(feature = "ssr")]
                            crate::app::set_response_status(
                                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                            );
                            view! {
                                <div class="error">
                                    <p>"The posts could not be loaded right now. Please try again later."</p>
                                </div>
                            }.into_any()
                        }
                    }
                })}
            </Suspense>
        </div>
    }
}
