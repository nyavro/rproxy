use tokio::{io::BufStream, net::TcpListener};
use tracing::info;
use dashmap::DashMap;

mod http;
mod config;

use crate::http::{req, resp};
use crate::config::configuration;

static DEFAULT_PORT: &str = "7878";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let port: u16 = std::env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_PORT.to_string())
        .parse()?;

    let listener = TcpListener::bind(format!("0.0.0.0:{port}")).await.unwrap();
    info!("listening on: {}", listener.local_addr()?);
    let config = configuration::load_configuration().unwrap();
    let auth_map: DashMap<String, auth::AuthResponse> = DashMap::new();
    loop {
        let (stream, addr) = listener.accept().await?;
        let mut stream = BufStream::new(stream);        
        let config = config.clone();        
        tokio::spawn(async move {
            info!(?addr, "new connection");
            match req::parse_request(&mut stream).await {
                Ok(req) => {
                    info!("incoming request auth {}", req.headers["Authorization"]);
                    let auth_response = auth::fetch_token(&config.auth_providers["puz_auth_token"]).await.unwrap();
                    info!("puz token {}, {}", auth_response.access_token, auth_response.expires_in);
                }
                Err(e) => {
                    info!(?e, "failed to parse request");
                }
            }
            let resp = resp::Response::from_html(
                resp::Status::NotFound,
                include_str!("../static/404.html"),
            );
            resp.write(&mut stream).await.unwrap();
        });
    }
}