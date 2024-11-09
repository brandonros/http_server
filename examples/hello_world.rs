use std::sync::Arc;

use http::{Request, Response, StatusCode, Version};
use http_server::{Router, HttpServer};
use async_executor::Executor;
use simple_error::SimpleResult;
use smol::MainExecutor;

async fn get_index(_executor: Arc<Executor<'static>>, _request: Request<Vec<u8>>) -> SimpleResult<Response<String>> {
    Ok(Response::builder()
    .status(StatusCode::OK)
    .version(Version::HTTP_11)
    .header("Content-Type", "text/plain")
    .body("Hello, World!".to_string())?)
}

async fn async_main(executor: Arc<Executor<'static>>) -> SimpleResult<()> {
    // logging
    env_logger::init();

    // settings
    let host = "0.0.0.0";
    let port = 8080;

    // build router
    let mut router = Router::new(executor.clone());
    router.add_route("GET", "/", Arc::new(move |executor, req| Box::pin(get_index(executor, req)))); // TODO: get rid of this non-async wrapper?
    let router = Arc::new(router);

    // run server
    HttpServer::run_server(executor, host, port, router).await
}

fn main() -> SimpleResult<()> {
    Arc::<Executor>::with_main(|ex| smol::block_on(async_main(ex.clone())))
}

