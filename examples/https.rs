use std::sync::Arc;
use http::{Method, Request, Response, StatusCode, Version};
use http_server::{Router, HttpServer};
use async_executor::Executor;
use rcgen::{Certificate, CertificateParams, DnType, PKCS_ECDSA_P256_SHA256, SanType};
use simple_error::SimpleResult;
use smol::MainExecutor as _;

pub fn generate_cert_and_key() -> SimpleResult<(String, String)> {
    let mut params = CertificateParams::default();
    params.distinguished_name.push(DnType::CommonName, "localhost".to_string());
    params.distinguished_name.push(DnType::OrganizationName, "Test Org".to_string());
    params.distinguished_name.push(DnType::CountryName, "US".to_string());
    params.subject_alt_names = vec![
        SanType::DnsName("localhost".to_string()),
        SanType::IpAddress("127.0.0.1".parse().unwrap())
    ];
    params.alg = &PKCS_ECDSA_P256_SHA256;
    
    let cert = Certificate::from_params(params)?;
    Ok((cert.serialize_pem()?, cert.serialize_private_key_pem()))
}

async fn get_index(_executor: Arc<Executor<'static>>, _request: Request<Vec<u8>>) -> SimpleResult<Response<String>> {
    Ok(Response::builder()
        .status(StatusCode::OK)
        .version(Version::HTTP_11)
        .header("Content-Type", "text/plain")
        .body("Hello, World!".to_string())?)
}

async fn async_main(executor: Arc<Executor<'static>>) -> SimpleResult<()> {
    // Initialize logging
    env_logger::init();

    // Server settings
    let host = "0.0.0.0";
    let port = 8443;  // Standard HTTPS port
    
    // TLS configuration
    let (cert_pem, key_pem) = generate_cert_and_key()?;
    let tls_config = Some((
        cert_pem,
        key_pem
    ));

    // Build router
    let mut router = Router::new(executor.clone());
    router.add_routes(vec![
        (Method::GET, "/", Arc::new(move |executor, req| Box::pin(get_index(executor, req)))),
    ]);
    let router = Arc::new(router);

    // Run HTTPS server
    println!("HTTPS server listening on https://{}:{}", host, port);
    HttpServer::run_server(executor, host, port, router, tls_config).await
}

fn main() -> SimpleResult<()> {
    Arc::<Executor>::with_main(|ex| smol::block_on(async_main(ex.clone())))
}