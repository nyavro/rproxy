use hyper::Client;
use hyper::client::{HttpConnector};
use rustls::{ClientConfig, RootCertStore};
use rustls::client::{ServerCertVerifier, ServerCertVerified};
use hyper_rustls::HttpsConnector;
use std::sync::Arc;

struct NoCertificateVerification {}

impl ServerCertVerifier for NoCertificateVerification {
    fn verify_server_cert(
            &self,
            _: &rustls::Certificate,
            _: &[rustls::Certificate],
            _: &rustls::ServerName,
            _: &mut dyn Iterator<Item = &[u8]>,
            _: &[u8],
            _: std::time::SystemTime,
        ) -> Result<rustls::client::ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }
}

pub fn init_client() -> Client<HttpsConnector<HttpConnector>> {    
    let mut cfg = ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(RootCertStore::empty())
        .with_no_client_auth();
    let mut dangerous_config = ClientConfig::dangerous(&mut cfg);
    dangerous_config.set_certificate_verifier(Arc::new(NoCertificateVerification {}));
    let https = hyper_rustls::HttpsConnectorBuilder::new()
        .with_tls_config(cfg)
        .https_or_http()
        .enable_http1()
        .build();
    let client = Client::builder().build(https);
    client
}