use std::convert::Infallible;
use hyper::{Body, Request, Response, Server, HeaderMap};
use hyper::service::{make_service_fn, service_fn};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let port = 7879;
    let addr = SocketAddr::from(([127,0,0,1], port));
    let server = Server::bind(&addr)
        .serve(
            make_service_fn(|_conn| async {Ok::<_, Infallible>(service_fn(handle_request))})
        );
    if let Err(e) = server.await {
        println!("server error: {}", e);
    }
}

fn get_authorization_template(headers: &HeaderMap) -> Option<String> {
    headers.iter()
        .filter(|(header, _)| header.to_string().to_lowercase() == "authorization")
        .flat_map(|(_, value)|
            match value.to_str() {
                Ok(v) => Some(v.to_string()),
                Err(_) => None
            }
        )
        .next()
}

async fn handle_request(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let auth_header = get_authorization_template(&req.headers());
    Ok(
        Response::builder()
            .status(200)
            .body(Body::from(req.uri().to_string() + "::::" + &auth_header.unwrap_or("none".to_string())))
            .unwrap()
    )
}