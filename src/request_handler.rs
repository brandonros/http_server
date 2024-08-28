use std::future::Future;

use async_trait::async_trait;
use http::{Request, Response};
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
