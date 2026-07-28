use crate::app::api::get_post_by_slug;
use crate::app::components::{Article, GoBack};
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_params_map;

#[component]
pub fn BlogPost() -> impl IntoView {
    let params = use_params_map();
    let slug = move || params.read().get("slug").unwrap_or_default();

    let post_resource = Resource::new(slug, |slug| async move { get_post_by_slug(slug).await });

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
                            // A real 404: without it every made-up slug is a
                            // "soft 404" that crawlers index as a page.
                            #[cfg(feature = "ssr")]
                            crate::app::set_response_status(axum::http::StatusCode::NOT_FOUND);
                            view! {
                                <Title text="Post not found"/>
                                <div class="not-found">
                                    <h1>"Post Not Found"</h1>
                                    <p>"The blog post you're looking for doesn't exist."</p>
                                    <GoBack go_to="blog" text="Back to Blog"/>
                                </div>
                            }
                                .into_any()
                        }

                        Err(e) => {
                            #[cfg(feature = "ssr")]
                            crate::app::set_response_status(
                                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                            );
                            view! {
                                <Title text="Error"/>
                                <div class="error">
                                    <h1>"Error"</h1>
                                    <p>"Error loading post: " {e.to_string()}</p>
                                    <GoBack go_to="blog" text="Back to Blog"/>
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
