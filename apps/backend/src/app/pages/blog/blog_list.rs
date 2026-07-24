use crate::app::api::get_published_posts;
use crate::app::components::BlogPostList;
use leptos::prelude::*;

#[component]
pub fn BlogList() -> impl IntoView {
    let posts_resource = Resource::new(|| (), |_| async { get_published_posts(Some(20)).await });

    view! {
        <div class="blog-container">
            <Suspense fallback=move || {
                view! { <div class="loading">"Loading posts..."</div> }
            }>
                {move || Suspend::new(async move {
                    match posts_resource.await {
                        Ok(posts) => {
                            view! { <BlogPostList posts=posts/> }.into_any()
                        }
                        Err(e) => {
                            view! {
                                <div class="error">
                                    <p>"Error loading posts: " {e.to_string()}</p>
                                </div>
                            }.into_any()
                        }
                    }
                })}
            </Suspense>
        </div>
    }
}
