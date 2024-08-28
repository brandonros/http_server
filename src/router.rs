use std::{collections::HashMap, future::Future};

use async_trait::async_trait;
use http::{Request, Response, StatusCode, Version};
use crate::types::Result;

#[async_trait]
pub trait RequestHandler: Send + Sync {
    async fn handle(&self, request: Request<()>) -> Result<Response<String>>;
}

#[async_trait]
impl<F, Fut> RequestHandler for F
where
    F: Fn(Request<()>) -> Fut + Send + Sync,
    Fut: Future<Output = Result<Response<String>>> + Send,
{
    async fn handle(&self, request: Request<()>) -> Result<Response<String>> {
        (self)(request).await
    }
}


#[derive(Default)]
pub struct Router {
    routes: HashMap<String, Box<dyn RequestHandler + Send + Sync>>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
        }
    }

    pub fn add_route<H>(&mut self, method: &str, path: &str, handler: H)
    where
        H: RequestHandler + Send + Sync + 'static,
    {
        let key = format!("{method}:{path}");
        self.routes.insert(key, Box::new(handler));
    }

    pub async fn route(&self, request: Request<()>) -> Result<Response<String>> {
        let method = request.method().as_str();
        let path = request.uri().to_string();
        let key = format!("{method}:{path}");
        log::info!("request key = {key}");
        if let Some(handler) = self.routes.get(&key) {
            match handler.handle(request).await {
                Ok(response) => {
                    Ok(response)
                },
                Err(err) => {
                    log::error!("controller error key = {key} err = {err:?}");
                    Ok(Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .version(Version::HTTP_11)
                        .body(format!("{err:?}"))
                        .unwrap())
                },
            }
        } else {
            log::warn!("route not found key = {key}");
            Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .version(Version::HTTP_11)
                .body("Not Found".to_string())
                .unwrap())
        }
    }
}
