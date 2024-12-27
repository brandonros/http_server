use std::{collections::HashMap, sync::Arc};
use regex::Regex;

use async_executor::Executor;
use http::{Method, Request, Response, StatusCode, Version};
use simple_error::SimpleResult;

use crate::types::BoxFuture;

pub type RouteHandler = dyn Fn(Arc<Executor<'static>>, Request<Vec<u8>>) -> BoxFuture<'static, SimpleResult<Response<String>>> + Send + Sync;

struct RouteInfo {
    handler: Arc<RouteHandler>,
    pattern: Regex,
    path_params: Vec<String>,
}

#[derive(Default)]
pub struct Router {
    executor: Arc<Executor<'static>>,
    routes: HashMap<(Method, String), RouteInfo>,
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
        let key = (method, path.to_string());
        
        log::debug!("Adding route: {:?}", key);
        let mut path_params = Vec::new();
        let pattern_str = path
            .split('/')
            .map(|segment| {
                if segment.starts_with(':') {
                    path_params.push(segment[1..].to_string());
                    "([^/]+)".to_string()
                } else {
                    regex::escape(segment)
                }
            })
            .collect::<Vec<_>>()
            .join("/");

        let pattern = Regex::new(&format!("^{}$", pattern_str)).unwrap();
        
        self.routes.insert(key, RouteInfo {
            handler,
            pattern,
            path_params,
        });
    }

    pub async fn route(&self, request: Request<Vec<u8>>) -> SimpleResult<Response<String>> {
        let method = request.method().clone();
        let path = request.uri().path().to_string();
        
        // Get all routes for this method
        for ((route_method, _), route_info) in self.routes.iter() {
            if route_method != &method {
                continue;
            }

            if let Some(captures) = route_info.pattern.captures(&path) {
                let mut params = HashMap::new();
                for (i, param_name) in route_info.path_params.iter().enumerate() {
                    if let Some(value) = captures.get(i + 1) {
                        params.insert(param_name.clone(), value.as_str().to_string());
                    }
                }

                let mut request = request;
                request.extensions_mut().insert(params);

                return match (route_info.handler)(self.executor.clone(), request).await {
                    Ok(response) => {
                        log::debug!("Response: {:?}", response);
                        Ok(response)
                    },
                    Err(err) => {
                        log::error!("Controller error: {:?}", err);
                        let response_body = format!("{:?}", err);
                        Ok(Response::builder()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .version(Version::HTTP_11)
                            .header("Content-Type", "text/plain")
                            .header("Content-Length", response_body.len().to_string())
                            .body(response_body)
                            .unwrap())
                    },
                };
            }
        }

        // No matching route found
        log::warn!("Route not found: ({:?}, {})", method, path);
        let response_body = "Not Found".to_string();
        Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .version(Version::HTTP_11)
            .header("Content-Type", "text/plain")
            .header("Content-Length", response_body.len().to_string())
            .body(response_body)
            .unwrap())
    }
}
