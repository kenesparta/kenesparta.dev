use crate::blog::get_post_by_slug;
use crate::components::Article;
use leptos::prelude::*;
use leptos_router::{components::A, hooks::use_params_map};

#[component]
pub fn BlogPost() -> impl IntoView {
    let params = use_params_map();
    let slug = move || params.read().get("slug").unwrap_or_default();

    let post_resource = Resource::new(
        move || slug(),
        |slug| async move { get_post_by_slug(slug).await },
    );

    view! {
        <div class="blog-post-container">
            <Suspense fallback=move || {
                view! { <div class="loading">"Loading post..."</div> }
            }>
                {move || Suspend::new(async move {
                    match post_resource.await {
                        Ok(Some(post)) => {
                            view! { <Article post=post/> }.into_any()
                        }

                        Ok(None) => {
                            view! {
                                <div class="not-found">
                                    <h1>"Post Not Found"</h1>
                                    <p>"The blog post you're looking for doesn't exist."</p>
                                    <A href="/blog" attr:class="back-link">
                                        "Go back to blog"
                                    </A>
                                </div>
                            }
                                .into_any()
                        }

                        Err(e) => {
                            view! {
                                <div class="error">
                                    <h1>"Error"</h1>
                                    <p>"Error loading post: " {e.to_string()}</p>
                                    <A href="/blog" attr:class="back-link">
                                        "Go back to blog"
                                    </A>
                                </div>
                            }
                                .into_any()
                        }
                    }
                })}

            </Suspense>
        </div>
    }
}
