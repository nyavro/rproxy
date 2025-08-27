use reqwest::header;
use tokio::{io::BufStream, io::BufWriter, io::AsyncWriteExt, net::TcpListener};
use tokio_util::io::StreamReader;
use tracing::info;
use dashmap::DashMap;
use tokio::net::TcpStream;
use futures::StreamExt;

mod http;
mod config;
mod auth;

use crate::http::{req, resp, forward};
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
                    // header_map.insert(
                    // HeaderName::from_bytes("authorization".as_bytes()).unwrap(),
                    // HeaderName::from_str(authorization.as_str()).unwrap()
                    //)
                    let forward_response = forward::forward(&config.redirect_url, &req.headers.clone(), req).await.unwrap();
                    write_response_headers(&mut stream, &forward_response).await;
                    let mut buffered_writer = BufWriter::new(stream);
                    let mut res_stream = forward_response.bytes_stream().map(|result| {
                        result.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
                    });
                    let mut async_reader = StreamReader::new(res_stream);
                    tokio::io::copy(&mut async_reader, &mut buffered_writer).await;
                    buffered_writer.flush().await;
                }
                Err(e) => {
                    info!(?e, "failed to parse request");
                }
            }
        });
    }
}

async fn write_response_headers(
    stream: &mut BufStream<TcpStream>,
    response: &reqwest::Response,
) -> tokio::io::Result<()> {
    let status_line = format!(
        "HTTP/1.1 {} {}\r\n",
        response.status().as_u16(),
        response.status().canonical_reason().unwrap_or("Unknown")
    );
    stream.write_all(status_line.as_bytes()).await;
    for (name, value) in response.headers() {
        let header_line = format!("{}: {}\r\n", name, value.to_str().unwrap_or(""));
        stream.write_all(header_line.as_bytes()).await?;
    }
    stream.write_all(b"\r\n").await?;
    Ok(())
}