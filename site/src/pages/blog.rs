use leptos::prelude::*;
use crate::server::post::{get_markdown_content, get_posts_index, PostMeta};

#[component]
pub fn Blog() -> impl IntoView {
    let posts = LocalResource::new(|| async move {
        get_posts_index().await
    });

    view! {
        <div class="writing-page">
            <h1>"Writing"</h1>
            <Suspense fallback=move || view! { <p>"Loading posts..."</p> }>
                {move || Suspend::new(async move {
                    match posts.await {
                        Ok(posts_list) => view! {
                            <div class="posts-list">
                                <For
                                    each=move || posts_list.clone()
                                    key=|post| post.id.clone()
                                    let:post
                                >
                                    <PostItem post=post />
                                </For>
                            </div>
                        }.into_any(),
                        Err(e) => view! { <p>"Error loading posts: " {e.to_string()}</p> }.into_any(),
                    }
                })}
            </Suspense>
        </div>
    }
}

#[component]
fn PostItem(post: PostMeta) -> impl IntoView {
    let (show_content, set_show_content) = signal(false);
    let post_id = post.id.clone();
    let content = LocalResource::new(move || {
        let id = post_id.clone();
        async move {
            get_markdown_content(format!("{}.md", id)).await
        }
    });

    view! {
        <article class="post-item">
            <div class="post-meta">
                <h2>{post.title.clone()}</h2>
                <div class="post-info">
                    <span class="author">"By " {post.author.clone()}</span>
                    <span class="date">{post.date.clone()}</span>
                </div>
                <div class="tags">
                    {post.tags.iter().map(|t| {
                        view! { <span class="tag">{t.clone()}</span> }
                    }).collect::<Vec<_>>()}
                </div>
            </div>

            <button
                on:click=move |_| set_show_content.update(|v| *v = !*v)
                class="toggle-content"
            >
                {move || if show_content.get() { "Hide Content" } else { "Show Content" }}
            </button>

            {move || {
                show_content.get().then(|| view! {
                    <Suspense fallback=move || view! { <p>"Loading content..."</p> }>
                        {move || {
                            Suspend::new(async move {
                                match content.await {
                                    Ok(html) => view! {
                                        <div
                                            class="post-content"
                                            inner_html=html
                                        />
                                    }.into_any(),
                                    Err(e) => view! { <p>"Error loading content: " {e.to_string()}</p> }.into_any(),
                                }
                            })
                        }}
                    </Suspense>
                })
            }}
        </article>
    }
}