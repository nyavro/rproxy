use serde::Deserialize;
use figment::{Figment, providers::{Format, Toml, Json, Env}};
use std::collections::HashMap;

#[derive(Deserialize, Clone)]
pub struct ProviderConfig {
    pub method: String,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub form: HashMap<String, String>
}

#[derive(Deserialize, Clone)]
pub struct Config {
    pub auth_providers: HashMap<String, ProviderConfig>,
    pub redirect_url: String
}

pub fn load_configuration() -> Result<Config, figment::Error> {    
    Ok(
        Figment::new()
            .join(Json::file("config.json"))
            .extract()?
    )    
}