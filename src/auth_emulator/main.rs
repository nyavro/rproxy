use tokio::{io::BufStream, net::TcpListener};
use tracing::info;
use serde::Serialize;
use serde_json;

use http::resp;
use http::req;

static DEFAULT_PORT: &str = "7877";

// Return mock token
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let port: u16 = std::env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_PORT.to_string())
        .parse()?;

    let listener = TcpListener::bind(format!("0.0.0.0:{port}")).await.unwrap();

    info!("listening on: {}", listener.local_addr()?);

    loop {
        let (stream, addr) = listener.accept().await?;
        let mut stream = BufStream::new(stream);
        tokio::spawn(async move {
            info!(?addr, "new connection");

            match req::parse_request(&mut stream).await {
                Ok(req) => info!(?req, "incoming request"),
                Err(e) => {
                    info!(?e, "failed to parse request");
                }
            }

            let resp = resp::Response::from_string(
                resp::Status::Ok,
                String::from("mock_token"),
            );

            resp.write(&mut stream).await.unwrap();
        });
    }
}