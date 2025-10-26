use pulldown_cmark::{Parser, html};
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PostMeta {
    pub id: String,
    pub slug: String,
    pub background_image: String,
    pub title: String,
    pub author: String,
    pub tags: Vec<String>,
    pub date: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FullPost {
    pub meta: PostMeta,
    pub html_content: String,
}

// pub async fn get_post_content_client(slug: String) -> Result<FullPost, String> {
//     let base_url = format!("/posts/{}", slug);
//     let window = web_sys::window().ok_or("No window found")?;
//
//     let opts = RequestInit::new();
//     opts.set_method("GET");
//     opts.set_mode(RequestMode::Cors);
//
//     // Helper to fetch a URL and return the text content
//     let fetch_text = |url: String| async move {
//
//         let request = Request::new_with_str_and_init(&url, &opts)
//             .map_err(|e| format!("Request failed for {}: {:?}", url, e))?;
//
//         let response = JsFuture::from(window.fetch_with_request(&request))
//             .await
//             .map_err(|e| format!("Fetch failed for {}: {:?}", url, e))?;
//
//         let response: Response = response.dyn_into()
//             .map_err(|_| "Failed to cast response")?;
//
//         if !response.ok() {
//             return Err(format!("HTTP Error fetching {}: {}", url, response.status()));
//         }
//
//         JsFuture::from(response.text().map_err(|_| "Failed to get response text")?)
//             .await
//             .map_err(|_| "Text promise failed")?
//             .as_string()
//             .ok_or_else(|| "Failed to convert text to String".to_string())
//     };
//
//     // 1. Fetch Metadata (JSON)
//     let meta_text = fetch_text(format!("{}/meta.json", base_url)).await?;
//     let meta: PostMeta = serde_json::from_str(&meta_text)
//         .map_err(|e| format!("Failed to parse meta.json: {}", e))?;
//
//     // 2. Fetch Content (Markdown)
//     let markdown_content = fetch_text(format!("{}/content.md", base_url)).await?;
//
//     // 3. Parse Markdown to HTML
//     let parser = Parser::new(&markdown_content);
//     let mut html_output = String::new();
//     html::push_html(&mut html_output, parser);
//
//     Ok(FullPost {
//         meta,
//         html_content: html_output,
//     })
// }

pub async fn get_posts_index() -> Result<Vec<PostMeta>, String> {
    let window = web_sys::window().ok_or("No window found")?;

    let opts = RequestInit::new();
    opts.set_method("GET");
    opts.set_mode(RequestMode::Cors);

    let request = Request::new_with_str_and_init("/meta.json", &opts)
        .map_err(|e| format!("Request failed: {:?}", e))?;

    let response = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("Fetch failed: {:?}", e))?;

    let response: Response = response.dyn_into()
        .map_err(|_| "Failed to cast response")?;

    if !response.ok() {
        return Err(format!("HTTP Error: {}", response.status()));
    }

    let text = JsFuture::from(response.text().map_err(|_| "Failed to get response text")?)
        .await
        .map_err(|_| "Text promise failed")?
        .as_string()
        .ok_or_else(|| "Failed to convert text to String".to_string())?;

    let posts: Vec<PostMeta> = serde_json::from_str(&text)
        .map_err(|e| format!("Failed to parse meta.json: {}", e))?;

    Ok(posts)
}

pub async fn get_markdown_content(file_path: String) -> Result<String, String> {
    let window = web_sys::window().ok_or("No window found")?;

    let opts = RequestInit::new();
    opts.set_method("GET");
    opts.set_mode(RequestMode::Cors);

    let url = format!("/metadata/{}", file_path);
    let request = Request::new_with_str_and_init(&url, &opts)
        .map_err(|e| format!("Request failed: {:?}", e))?;

    let response = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("Fetch failed: {:?}", e))?;

    let response: Response = response.dyn_into()
        .map_err(|_| "Failed to cast response")?;

    if !response.ok() {
        return Err(format!("HTTP Error: {}", response.status()));
    }

    let markdown_text = JsFuture::from(response.text().map_err(|_| "Failed to get response text")?)
        .await
        .map_err(|_| "Text promise failed")?
        .as_string()
        .ok_or_else(|| "Failed to convert text to String".to_string())?;

    // Parse Markdown to HTML
    let parser = Parser::new(&markdown_text);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    Ok(html_output)
}