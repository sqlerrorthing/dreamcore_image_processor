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

async fn fetch_images(query: &str, client: Client) -> Result<Vec<String>, reqwest::Error> {
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
                "bookmarks": [
                    "Y2JVSG81V2sxcmNHRlpWM1J5VFVaU1ZsWllaR3hpVmtreVZsZHpOVll4U2xoa2VrSlhVak5TVkZaSE1WZGphekZXVm14V1dGSXhTbEJYVm1RMFVtMVJlRlZzYUdwU1ZYQlBXVlJPVTJWV1pISlhhM1JYVm10V05sVldVbE5XVjBwSVZXeE9WVll6YUROV01GcFBaRWRPUms5V1RsTldia0l5Vm10a01GVXlSWGxUYTFwUVZtMW9WbGxzYUc5WlZuQllaRVYwYWsxWFVqQlpNRlV4VkRGYWRHVklhRmROVjJoNlZrUkdTMUpzVG5WVmJGWnBZbXRLVEZaR1dsWmxSbHBYVm14c2FWSXdXbGhVVmxaYVRVWlplR0ZJWkdoaVZrWXpWR3hTWVZaWFNsbFZhemxWVmpOT05GUnNXbE5rUjA1SVVtMW9UbEpHV2xkV2JYaFRWVEZTY2sxWVNtcFRSVnBXVm1wT1EyVnNiSEpXVkVacVZteGFWbFp0Y3pGaFZscFhZMGhrVjAxV1NsQlVhMXBTWlVaT2MxcEhSbE5TTWswMVdtdGFWMU5YU2paVmJYaFhWMGRvUmxkc1ZsZGhNV1J6VjFod2FGSkdjRmxaYTJSdVpXeFNjbFpVUmxkV2F6VmFXVlZWTlZVeFNsVlNWRXBYVW14YVZGWkhNVkprTURGWlVteGFWMUpWY0ZCWFZtUXdVbTFXVjFSWWJHdFNNMUpYV1d0YVMxSldhM2RWYlRsVlRWVndSMWt3YUU5V1ZscEdZMFU1VlZac1ZqUldNRnBQVmxaT2RGSnRhR2xTYkZZelZtdGtNRlV5VG5SV2ExcFBWbTFvVjFsc2FHOVZSbFp5Vm10YVRsWnNSak5YYTFaaFZHeGFWVkpyYkZkTmFrWXpWMVphV21WR1RuVlJiR2hwVWpGS1VGZHNWbUZrTVVwWFZHeHNZVkp1UW5CV2JHUXpUVlphUjJGSVpHcE5hM0JJV1d0b1UxZEhTbFZTYkVKV1lURmFNMWt3V210amJGWnpVMnMxYVZORlNscFdhMk4zWlVaa2NrMVlTbGRoYkZwWlZqQm9RMVF4Y0ZkV1dHaHFWbXMxV2xsVlZURlVhekI1WVVaR1YxWXphR2haVkVFMVVXeENWVTFVYUZCU1JsWTFWRlpTVW1Wck1UWlJXSEJPVWtWcmQxUnJVa3BrTURWd1kwVm9WbFpZWkhoYWExSnZZVEZyZVZSdE1VNVNNVVl6VjIxd2MyRldjSFJTYlhoUFZqRmFjVmRYY0Vwa01EbFZWMVJLVGxKRlJYbFhiVEZTWkRBMGVWTnRlRTVTUmxVd1ZGaHdjbVZyTkhsU1ZFcE9Va1pLY0ZkdE1WcGxSVFZGVjIxd1RtRnJXbkJVVjNCR1RVVTVXRlZ0ZUU5bGExVjNWMWh3VTA5R1VuSldiR2h0VVZRd09XWkVaekZOYWtVd1RYcE5kMDE2UVhsT1JGRjVUVVJaY1ZJeFJrMUxibmMxVDFkRmVGbFhTWGRaVkZrelRsUmpkMDVFV1ROT01sVTBUMGRKZUU1VVdUQk9hazVxVG5wT2FGcHFRWGxQVkdzeFdrUkJNVTVFVm0xTk1rMTNXbFJWZWxwRVl6VlBSR2h0VFdwck5WcHRTVEZhYWtGNVprVTFSbFl6ZHowPXxVSG81VDJJeU5XeG1SR2N4VFdwRk1FMTZUWGROZWtGNVRrUlJlVTFFV1hGU01VWk5TMjUzZVU1cVNYbFpla3ByVGxSQ2JWbFVhM2xPVjBac1RVUkZORTE2U1RGWmJWWm9Ua2ROZVU1RVNUTk9iVkpwVFdwV2FWbHRTVE5OTWtsNFRqSkZNRTlFYXpGYVJGRXpXWHBuTkZsNlRtcGFSRVYzVGxkRk1HWkZOVVpXTTNjOXxOb25lfDg1MjE0MzMwMzAyNDQyMDYqR1FMKnwxYmU5ZTg0MGQxOTJkMGYyMDI4MzY1Y2VhOTg0YTlhZTJiYWViMjMyMjIwYjhlNzc1NDU0MmYxOGZiZWJjODQ3fE5FV3w="
                ]
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

    Ok(extract_image_urls(res))
}

fn extract_image_urls(json: Value) -> Vec<String> {
    json.pointer("/resource_response/data/results")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| item.pointer("/images/orig/url").and_then(|v| v.as_str()))
                .map(|s| s.to_string())
                .collect()
        })
        .unwrap_or_default()
}

async fn download_image(client: Client, url: String, rest: usize) -> Result<DynamicImage, FetchBackgroundError> {
    info!("Downloading image {url}, {rest} images rest in pool");

    let bytes = client.get(&url).send().await?.bytes().await?;
    let img = image::load_from_memory(&bytes)?;

    Ok(img)
}

impl BackgroundProvider for PinterestProvider {
    async fn fetch_background(&self) -> Result<DynamicImage, FetchBackgroundError> {
        let mut pool = self.image_pool.lock().await;

        if pool.is_empty() {
            info!("Images pool empty, fetching new batch...");

            let images = fetch_images(self.query, self.client.clone()).await?;

            if images.is_empty() {
                return Err(FetchBackgroundError::NoImages);
            }

            pool.extend(images);
        }

        let image_link = pool.pop().unwrap();
        let rest = pool.len();

        drop(pool);

        download_image(self.client.clone(), image_link, rest).await
    }
}
