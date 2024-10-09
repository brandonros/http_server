use std::{error::Error, sync::Arc};

use http::{Request, Response, StatusCode, Version};
use http_server::{Router, HttpServer};
use async_executor::Executor;
use tradingview_client::{DefaultTradingViewMessageProcessor, TradingViewClient, TradingViewClientConfig, TradingViewMessageProcessor};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

fn build_error_response(err: Box<dyn Error + Send + Sync>) -> Response<String> {
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .version(Version::HTTP_11)
        .header("Content-Type", "text/plain")
        .body(format!("{err}"))
        .expect("failed to build error response")
}

async fn post_scrape(request: Request<Vec<u8>>) -> Result<Response<String>> {
    log::info!("request = {request:02x?}");

    // build message processor
    let message_processor: Arc<Box<dyn TradingViewMessageProcessor + Send + Sync>> = Arc::new(Box::new(DefaultTradingViewMessageProcessor {}));

    // build config
    let request_body = request.body();
    let request_body = String::from_utf8(request_body.clone())?;
    let config: TradingViewClientConfig = match miniserde::json::from_str(&request_body) {
        Ok(config) => config,
        Err(err) => {
            return Ok(build_error_response(err.into()));
        },
    };

    // build client from config
    let client: TradingViewClient = config.to_client(message_processor);

    match client.run().await {
        Ok(scrape_result) => {
            // stringify scrape result
            let stringified_scrape_result = miniserde::json::to_string(&scrape_result);

            // return response
            let response = Response::builder()
                .status(StatusCode::OK)
                .version(Version::HTTP_11)
                .header("Content-Type", "application/json")
                .body(stringified_scrape_result)?;
            Ok(response)
        },
        Err(err) => {
            return Ok(build_error_response(err.into()));
        },
    }
}

#[macro_rules_attribute::apply(smol_macros::main!)]
async fn main(executor: &Arc<Executor<'static>>) -> Result<()> {
    // logging
    let logging_env = env_logger::Env::default().default_filter_or("debug,websocket_client=info,rustls=info,http_client=info");
    env_logger::Builder::from_env(logging_env).init();

    // settings
    let host = "127.0.0.1";
    let port = 3000;

    // build router
    let mut router = Router::new();
    router.add_route("POST", "/scrape", Arc::new(move |req| Box::pin(post_scrape(req)))); // TODO: get rid of this non-async wrapper?
    let router = Arc::new(router);

    // run server
    HttpServer::run_server(executor, host, port, router).await
}
