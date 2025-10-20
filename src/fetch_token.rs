use hyper::header::{HeaderName, HeaderValue};
use hyper::{HeaderMap, Request, Method, Body};
use hyper::body::to_bytes;
use tokio::sync::Mutex;
use serde::Deserialize;
use crate::client::init_client;
use crate::config::configuration;
use std::{collections::HashMap, sync::Arc, time::{SystemTime, UNIX_EPOCH}};

#[derive(Deserialize, Clone, Debug)]
struct AuthResponse {
    pub access_token: String,
    pub expires_in: u64
}

#[derive(Deserialize, Clone, Debug)]
pub struct Token {
    pub access_token: String,
    pub valid_until: u128
}

pub async fn collect_headers(req_headers: &HeaderMap, cfg: &configuration::Config, token_cache: Arc<Mutex<HashMap<String, Token>>>) -> HeaderMap {
    let mut headers = HeaderMap::new();
    for (name, value) in req_headers.iter() {
        headers.insert(name.clone(), value.clone());
    }
    let headers = match get_authorization_template(&headers) {
        Some((header,value)) => 
            match parse_auth_template(&value) {
                Some(placeholder) => {
                    let now = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_millis();
                    let has_active_token_in_cache = {
                        let lock = token_cache.lock().await;
                        lock.contains_key(&placeholder) && lock.get(&placeholder).filter(|token| now < token.valid_until).is_some()
                    };
                    if has_active_token_in_cache {
                        let auth_header = {
                            let lock = token_cache.lock().await;
                            let cached = lock.get(&placeholder).unwrap();
                            "Bearer ".to_owned() + &cached.access_token
                        };
                        println!("Fetched from cache: {}", &auth_header[0..20]);
                        headers.insert(header.clone(), HeaderValue::from_str(&auth_header).unwrap());
                        headers
                    } else {
                        println!("No active token found in cache, requesting...");
                        match cfg.auth_providers.get(&placeholder) {
                            Some(provider_config) => {
                                let token = fetch_token(provider_config).await.unwrap();
                                println!("token {}, expires_in {}", &token.access_token[0..20], token.expires_in);
                                let mut lock = token_cache.lock().await;
                                lock.insert(placeholder, Token {access_token: token.access_token.clone(), valid_until: now + token.expires_in as u128});
                                let token = "Bearer ".to_owned() + &token.access_token;
                                headers.insert(header.clone(), HeaderValue::from_str(&token).unwrap());
                                headers
                            },
                            None => headers
                        }
                    }
                },
                None => headers,
            }
        None => headers
    };
    headers
}

async fn fetch_token(config: &configuration::ProviderConfig) -> Result<AuthResponse, Box<dyn std::error::Error + Send + Sync>> {
    let client = init_client();
    let mut request = Request::builder()
        .method(Method::POST) 
        .uri(config.url.clone());
    for (header, value) in config.headers.iter() {
        request = request.header(header, value);
    }        
    let request = request.body(Body::from(config.body.clone())).unwrap();
    match client.request(request).await {
        Ok(res) => Ok(serde_json::from_slice(&to_bytes(res.into_body()).await.unwrap())?),
        Err(e) => Err(Box::new(e))
    }
}

fn get_authorization_template(headers: &HeaderMap) -> Option<(HeaderName, String)> {
    headers.iter()
        .filter(|(header, _)| header.to_string().to_lowercase() == "authorization")
        .flat_map(|(header, value)|
            match value.to_str() {
                Ok(v) => Some((header.clone(), v.to_string())),
                Err(_) => None
            }
        )
        .next()
}

fn parse_auth_template(template: &str) -> Option<String> {
    if template.starts_with("Bearer {{") && template.ends_with("}}") {
        Some(template[9..template.len()-2].to_string())
    } else {
        None
    }
}