use std::convert::Infallible;
use hyper::{Body, Request, Response, Server, Client, Uri, HeaderMap};
use hyper::service::{make_service_fn, service_fn};
use hyper::client::HttpConnector;
use std::net::SocketAddr;
use crate::config::configuration;
use std::sync::Arc;

mod config;
mod fetch_token;

#[tokio::main]
async fn main() {
    let port = 8081;
    let addr = SocketAddr::from(([127,0,0,1], port));
    let config = Arc::new(configuration::load_configuration().unwrap());
    let server = Server::bind(&addr)
        .serve(
            make_service_fn(
                move |_conn| {
                    let config = config.clone();
                    async move { 
                        Ok::<_, Infallible>(
                            service_fn(
                                move |req| handle_request(req, config.clone())
                            )
                        )
                    }
                }
            )
        );
    if let Err(e) = server.await {
        println!("server error: {}", e);
    }
}

fn init_client() -> Client<HttpConnector> {
    let client = Client::new();
    client
}

async fn handle_request(req: Request<Body>, config: Arc<configuration::Config>) -> Result<Response<Body>, Infallible> {    
    let client = Client::new();
    let uri_str = config.redirect_url.clone() + req.uri().path_and_query().map_or("/", |pq| pq.as_str());
    let backend_uri: Uri = uri_str.parse().unwrap();
    let headers = fetch_token::collect_headers(req.headers(), &config).await;
    let mut proxy_req = Request::builder()
        .method(req.method())
        .uri(backend_uri)
        .body(req.into_body())
        .unwrap();
    *proxy_req.headers_mut() = headers;
    match client.request(proxy_req).await {
        Ok(res) => Ok(res),
        Err(_) => Ok(Response::builder().status(500).body(Body::from("Internal Server Error")).unwrap())
    }    
}