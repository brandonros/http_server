#[cfg(not(any(feature = "futures", feature = "futures-lite")))]
compile_error!(
    "You must enable either the `futures` or `futures-lite` feature to build this crate."
);

#[cfg(feature = "futures")]
use futures as futures_provider;

#[cfg(feature = "futures-lite")]
use futures_lite as futures_provider;

pub mod router;
mod types;

use async_io::Async;
use async_executor::Executor;
use futures_provider::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use http::{Method, Request, Uri, Version};
use std::{
    net::{TcpListener, ToSocketAddrs},
    str::FromStr,
    sync::Arc,
};

use crate::router::Router;
use crate::types::Result;

pub struct HttpServer;

impl HttpServer {
    async fn read_http_request<S: AsyncRead + AsyncWrite + Unpin>(
        stream: &mut S,
    ) -> Result<Request<()>> {
        // Wrap the stream with a BufReader for efficient reading
        let mut reader = BufReader::new(stream);

        // Read the request line (e.g., "GET /path HTTP/1.1")
        let mut request_line = String::new();
        reader.read_line(&mut request_line).await?;

        // Parse the request line into components
        let mut parts = request_line.trim().split_whitespace();
        let method = parts.next().ok_or("Failed to parse method")?;
        let uri = parts.next().ok_or("Failed to parse URI")?;
        let version = parts.next().ok_or("Failed to parse version")?;

        // Convert components into appropriate types for Request
        let method = Method::from_str(method)?;
        let uri = Uri::from_str(uri)?;
        let version = match version {
            "HTTP/1.0" => Version::HTTP_10,
            "HTTP/1.1" => Version::HTTP_11,
            "HTTP/2.0" => Version::HTTP_2,
            _ => return Err("Unsupported HTTP version".into()),
        };

        // Create a new request builder
        let mut request_builder = Request::builder().method(method).uri(uri).version(version);

        // Read the HTTP headers
        loop {
            let mut header_line = String::new();
            reader.read_line(&mut header_line).await?;

            // An empty line indicates the end of the headers
            if header_line == "\r\n" {
                break;
            }

            // Split the header into key and value
            let mut header_parts = header_line.trim().splitn(2, ':');
            let key = header_parts.next().ok_or("Failed to parse header key")?;
            let value = header_parts.next().ok_or("Failed to parse header value")?;

            // Add the header to the request builder
            request_builder = request_builder.header(key.trim(), value.trim());
        }

        // Build the request without a body (`()`)
        let request = request_builder.body(())?;

        Ok(request)
    }

    async fn handle_request<S: AsyncRead + AsyncWrite + Unpin>(
        router: Arc<Router>,
        mut stream: S,
    ) -> Result<()> {
        // read request
        let request = Self::read_http_request(&mut stream).await?;

        // Route requests by method + path
        let response = router.route(request).await?;

        // Serialize and send the response
        let response_str = format!(
            "{:?} {} {}\r\nContent-Length: {}\r\n\r\n{}",
            response.version(),
            response.status(),
            response.status().canonical_reason().unwrap_or(""),
            response.body().len(),
            response.body()
        );

        stream.write_all(response_str.as_bytes()).await?;
        stream.flush().await?;
        Ok(())
    }

    pub async fn run_server(executor: &Arc<Executor<'static>>, host: &str, port: u16, router: Arc<Router>) -> Result<()> {
        // bind listener
        let addr = format!("{host}:{port}")
            .to_socket_addrs()?
            .next()
            .ok_or("Failed to build host")?;
        let listener = Async::<TcpListener>::bind(addr)?;

        // handle request
        loop {
            let (stream, _) = listener.accept().await?;
            log::info!("accepted new connection");

            // Spawn a task to handle each client connection
            let task = executor.spawn({
                let router = router.clone();
                async move {
                    match Self::handle_request(router, stream).await {
                        Ok(()) => (),
                        Err(err) => {
                            log::error!("error handling request err = {err:?}");
                        },
                    }
                }
            });

            // run in background
            task.detach();
        }
    }
}
