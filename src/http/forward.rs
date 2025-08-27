use futures::future::{self, FutureExt};
use serde::Deserialize;
use reqwest::{Client, Response};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::collections::HashMap;

use crate::req::{Method, Request};

#[derive(Deserialize, Clone)]
pub struct AuthResponse {
    pub access_token: String,
    pub expires_in: u64,
}

pub async fn forward(
    redirect_url: &str,
    headers: &HashMap<String, String>,
    req: Request,
) -> Result<Response, Box<dyn std::error::Error + Send + Sync>> {
    let target_url = format!("{}/{}", redirect_url, req.path);
    let header_map = get_headers(headers)?;
    let client = Client::new();
    let body = reqwest::Body::from(req.body);
    
    let response = method_step(&client, &req.method, target_url)
        .headers(header_map)
        .body(body)
        .send()
        .await?;
    
    Ok(response)
}

fn method_step(client: &Client, method: &Method, url: String) -> reqwest::RequestBuilder {
    match method {
        Method::Get => client.get(url),
        Method::Post => client.post(url),
        Method::Put => client.put(url),
    }
}

fn get_headers(headers_map: &HashMap<String, String>) -> Result<HeaderMap, Box<dyn std::error::Error + Send + Sync>> {
    let mut header_map = HeaderMap::new();
    
    for (key, value) in headers_map {
        let header_name = HeaderName::from_bytes(key.as_bytes())
            .map_err(|e| format!("Invalid header name '{}': {}", key, e))?;
        
        let header_value = HeaderValue::from_str(value)
            .map_err(|e| format!("Invalid header value for '{}': {}", key, e))?;
        
        header_map.insert(header_name, header_value);
    }
    
    Ok(header_map)
}