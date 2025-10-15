use std::convert::Infallible;
use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let port = 7877;
    let addr = SocketAddr::from(([127,0,0,1], port));
    let server = Server::bind(&addr)
        .serve(
            make_service_fn(|_conn| async {Ok::<_, Infallible>(service_fn(handle_request))})
        );
    if let Err(e) = server.await {
        println!("server error: {}", e);
    }
}

async fn handle_request(_: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(
        Response::builder()
            .status(200)
            .body(Body::from("{\"access_token\": \"acc_tok_123\", \"expires_in\": 1234}"))
            .unwrap()
    )
}