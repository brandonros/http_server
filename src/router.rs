use std::{collections::HashMap, sync::Arc};

use async_executor::Executor;
use http::{Method, Request, Response, StatusCode, Version};
use simple_error::SimpleResult;

use crate::types::BoxFuture;

pub type RouteHandler = dyn Fn(Arc<Executor<'static>>, Request<Vec<u8>>) -> BoxFuture<'static, SimpleResult<Response<String>>> + Send + Sync;

#[derive(Default)]
pub struct Router {
    executor: Arc<Executor<'static>>,
    routes: HashMap<String, Arc<RouteHandler>>,
}

impl Router {
    pub fn new(executor: Arc<Executor<'static>>) -> Self {
        Self {
            executor,
            routes: HashMap::new(),
        }
    }

    pub fn add_routes(&mut self, routes: Vec<(Method, &str, Arc<RouteHandler>)>) {
        for (method, path, handler) in routes {
            self.add_route(method, path, handler);
        }
    }

    pub fn add_route(&mut self, method: Method, path: &str, handler: Arc<RouteHandler>) {
        let key = format!("{method}:{path}");
        self.routes.insert(key, handler);
    }

    pub async fn route(&self, request: Request<Vec<u8>>) -> SimpleResult<Response<String>> {
        let method = request.method().as_str();
        let path = request.uri().path().to_string();
        let key = format!("{method}:{path}");
        log::info!("request key = {key}");
        if let Some(handler) = self.routes.get(&key) {
            log::debug!("request = {request:02x?}");
            match handler(self.executor.clone(), request).await {
                Ok(response) => {
                    log::debug!("response = {response:02x?}");
                    Ok(response)
                },
                Err(err) => {
                    log::error!("controller error key = {key} err = {err:?}");
                    let response_body = format!("{err:?}");
                    Ok(Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .version(Version::HTTP_11)
                        .header("Content-Type", "text/plain")
                        .header("Content-Length", response_body.len().to_string())
                        .body(response_body)
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
