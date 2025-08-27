use futures::future::{self, FutureExt};
use serde::{Deserialize};
use reqwest::{Client};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::collections::HashMap;

use crate::config::configuration;

#[derive(Deserialize, Clone)]
pub struct AuthResponse {
    pub access_token: String,
    pub expires_in: u64
}

pub async fn fetch_token(provider_config: &configuration::ProviderConfig) -> Result<AuthResponse, Box<dyn std::error::Error + Send + Sync>> {
    let client = Client::new();
    let response = client
        .post(&provider_config.url)
        .headers(get_headers(&provider_config.headers))
        .form(&provider_config.form)
        .send()
        .await?
        .text()
        .await?;
    let auth_response: AuthResponse = serde_json::from_str(&response.to_string())?;
    Ok(auth_response)
}

fn get_headers(headers_map: &HashMap<String, String>) -> HeaderMap {
    headers_map.iter().fold(
        HeaderMap::new(),
        |mut map, (key,value)| {
            map.insert(
                HeaderName::from_bytes(key.as_bytes()).unwrap(),
                HeaderValue::from_str(value).unwrap(),
            );
            map
        }
    )
}