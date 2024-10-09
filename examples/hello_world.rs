use std::sync::Arc;

use http::{Request, Response, StatusCode, Version};
use http_server::{Router, HttpServer};
use async_executor::Executor;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

async fn get_index(_request: Request<()>) -> Result<Response<String>> {
    Ok(Response::builder()
    .status(StatusCode::OK)
    .version(Version::HTTP_11)
    .header("Content-Type", "text/plain")
    .body("Hello, World!".to_string())?)
}

#[macro_rules_attribute::apply(smol_macros::main!)]
async fn main(ex: &Arc<Executor<'static>>) -> Result<()> {
    // logging
    let logging_env = env_logger::Env::default().default_filter_or("debug");
    env_logger::Builder::from_env(logging_env).init();

    // settings
    let host = "127.0.0.1";
    let port = 3000;

    // build router
    let mut router = Router::new();
    router.add_route("GET", "/", Arc::new(move |req| Box::pin(get_index(req)))); // TODO: get rid of this non-async wrapper?
    let router = Arc::new(router);

    // run server
    HttpServer::run_server(ex, host, port, router).await
}
