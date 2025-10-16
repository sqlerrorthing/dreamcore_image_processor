use crate::provider::{BackgroundProvider, FetchBackgroundError};
use derive_new::new;
use image::DynamicImage;
use lazy_static::lazy_static;
use log::info;
use reqwest::header::HeaderMap;
use reqwest::{header, Client};
use serde_json::{json, Value};
use std::collections::HashMap;
use tokio::sync::Mutex;

lazy_static! {
    static ref HEADERS: HeaderMap = {
        let mut headers = HeaderMap::new();
        headers.insert(
            "User-Agent",
            "Mozilla/5.0 (X11; Linux x86_64; rv:143.0) Gecko/20100101 Firefox/143.0"
                .parse()
                .unwrap(),
        );
        headers.insert(
            "Accept",
            "application/json, text/javascript, */*, q=0.01"
                .parse()
                .unwrap(),
        );
        headers.insert("Referer", "https://www.pinterest.com/".parse().unwrap());
        headers.insert("X-Requested-With", "XMLHttpRequest".parse().unwrap());
        headers.insert("X-APP-VERSION", "0dd1dca".parse().unwrap());
        headers.insert(
            "Content-Type",
            "application/x-www-form-urlencoded".parse().unwrap(),
        );
        headers.insert(
            "X-CSRFToken",
            "720e1cf655eb75932e9b71ab79ec1d21".parse().unwrap(),
        );
        headers.insert("X-Pinterest-AppState", "background".parse().unwrap());
        headers.insert(
            "X-Pinterest-Source-Url",
            "/search/pins/?q=dreamcore%20background".parse().unwrap(),
        );
        headers.insert(
            "X-Pinterest-PWS-Handler",
            "www/search/[scope].js".parse().unwrap(),
        );
        headers.insert("screen-dpr", "1".parse().unwrap());
        headers.insert("X-B3-TraceId", "7425cd664415310a".parse().unwrap());
        headers.insert("X-B3-SpanId", "0ec54aeb368ea87b".parse().unwrap());
        headers.insert("X-B3-ParentSpanId", "7425cd664415310a".parse().unwrap());
        headers.insert("X-B3-Flags", "0".parse().unwrap());
        headers.insert("Origin", "https://www.pinterest.com".parse().unwrap());
        headers.insert("Sec-GPC", "1".parse().unwrap());
        headers.insert("Connection", "keep-alive".parse().unwrap());
        headers.insert(header::COOKIE, "csrftoken=720e1cf655eb75932e9b71ab79ec1d21; _pinterest_sess=TWc9PSZmK293QnQxeDE0ckZyU01ha1R4SndkZGtJV01FSUhvVTdGeXFqZTQxNHNuSjY2RnFxV1VpaXQrMTBxQjBJcWlDQ2N1Zks4ZStTcXIvSnMvMXkwb04ySUI4WmlaMkV2OU1JT1h0L2FBOTdxcz0mQjZDVFc0aEF5Q1hHNGFsWllLWS9JWDBETWNZPQ==; _auth=0; _routing_id=\"f2d1f224-013f-4541-ad36-a201ac4239be\"; g_state={\"i_l\":1,\"i_ll\":1760585975395,\"i_b\":\"zQ3ojrIu/3MF354oYANCjS0QXf7iq9Q886RCjgtXGdk\",\"i_p\":1760593179991}".parse().unwrap());
        headers.insert("Sec-Fetch-Dest", "empty".parse().unwrap());
        headers.insert("Sec-Fetch-Mode", "cors".parse().unwrap());
        headers.insert("Sec-Fetch-Site", "same-origin".parse().unwrap());
        headers.insert("Pragma", "no-cache".parse().unwrap());
        headers.insert("Cache-Control", "no-cache".parse().unwrap());
        headers.insert("TE", "trailers".parse().unwrap());

        headers
    };
}

#[derive(Debug, new)]
pub struct PinterestProvider {
    query: &'static str,
    #[new(default)]
    image_pool: Mutex<Vec<String>>,
    #[new(default)]
    client: Client,
}

async fn fetch_images(query: &str, client: Client, out: &mut Vec<String>) -> Result<(), reqwest::Error> {
    let source_url = format!("/search/pins/?q={query}");
    let source_url = urlencoding::encode(&source_url);

    let data = json!({
        "options": {
                "query": query,
                "scope": "pins",
                "appliedProductFilters": "---",
                "domains": null,
                "user": null,
                "seoDrawerEnabled": false,
                "applied_unified_filters": null,
                "auto_correction_disabled": false,
                "journey_depth": null,
                "source_id": null,
                "source_module_id": null,
                "source_url": source_url,
                "selected_one_bar_modules": null,
                "query_pin_sigs": null,
                "page_size": null,
                "price_max": null,
                "price_min": null,
                "request_params": null,
                "top_pin_ids": null,
                "article": null,
                "corpus": null,
                "customized_rerank_type": null,
                "filters": null,
                "rs": "direct_navigation",
                "redux_normalize_feed": true,
                "bookmarks": [""]
            },
        "context": {}
    });

    let mut form = HashMap::new();
    form.insert("source_url", source_url);
    form.insert("data", data.to_string().into());


    let res = client
        .post("https://www.pinterest.com/resource/BaseSearchResource/get/")
        .headers(HEADERS.clone())
        .form(&form)
        .send()
        .await?
        .error_for_status()?
        .json::<Value>()
        .await?;

    extract_image_urls(res, out);
    Ok(())
}


#[inline(always)]
fn extract_image_urls(json: Value, out: &mut Vec<String>) {
    if let Some(arr) = json.pointer("/resource_response/data/results").and_then(|v| v.as_array()) {
        for item in arr {
            if let Some(url) = item.pointer("/images/orig/url").and_then(|v| v.as_str()) {
                out.push(url.to_string());
            }
        }
    }
}

async fn download_image(client: Client, url: String) -> Result<DynamicImage, FetchBackgroundError> {
    let bytes = client.get(&url).send().await?.bytes().await?;
    let img = image::load_from_memory(&bytes)?;
    Ok(img)
}

impl BackgroundProvider for PinterestProvider {
    async fn fetch_background(&self) -> Result<DynamicImage, FetchBackgroundError> {
        let mut pool = self.image_pool.lock().await;

        if pool.is_empty() {
            info!("Images pool empty, fetching new batch...");

            fetch_images(self.query, self.client.clone(), &mut pool).await?;

            if pool.is_empty() {
                return Err(FetchBackgroundError::NoImages);
            }
        }

        let image_link = pool.pop().unwrap();
        let rest = pool.len();

        drop(pool);

        info!("Downloading image {image_link}, {rest} images rest in pool");
        download_image(self.client.clone(), image_link).await
    }
}
