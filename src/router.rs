use std::{collections::HashMap, sync::Arc};

use http::{Request, Response, StatusCode, Version};
use crate::types::{BoxFuture, Result};

pub type RouteHandler = dyn Fn(Request<Vec<u8>>) -> BoxFuture<'static, Result<Response<String>>> + Send + Sync;

#[derive(Default)]
pub struct Router {
    routes: HashMap<String, Arc<RouteHandler>>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
        }
    }

    pub fn add_route(&mut self, method: &str, path: &str, handler: Arc<RouteHandler>) {
        let key = format!("{method}:{path}");
        self.routes.insert(key, handler);
    }

    pub async fn route(&self, request: Request<Vec<u8>>) -> Result<Response<String>> {
        let method = request.method().as_str();
        let path = request.uri().to_string();
        let key = format!("{method}:{path}");
        log::info!("request key = {key}");
        if let Some(handler) = self.routes.get(&key) {
            match handler(request).await {
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
