use async_io::Async;
use async_executor::Executor;
use futures_lite::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader, AsyncReadExt};
use http::{Method, Request, Uri, Version};
use simple_error::{box_err, SimpleResult};
use async_tls::TlsAcceptor;
use rustls::{Certificate, PrivateKey, ServerConfig};
use std::net::{TcpListener, TcpStream, ToSocketAddrs as _};
use std::str::FromStr as _;
use std::sync::Arc;

use crate::async_connection::AsyncConnection;
use crate::router::Router;

pub struct HttpServer {
    tls_acceptor: Option<TlsAcceptor>,
}

impl HttpServer {
    pub fn new() -> Self {
        Self { tls_acceptor: None }
    }

    pub fn with_tls(cert_pem: &str, key_pem: &str) -> SimpleResult<Self> {
        // Load certificate from string
        let mut cert_reader = std::io::BufReader::new(std::io::Cursor::new(cert_pem));
        let cert = rustls_pemfile::certs(&mut cert_reader)?
            .into_iter()
            .map(Certificate)
            .collect();

        // Load private key from string
        let mut key_reader = std::io::BufReader::new(std::io::Cursor::new(key_pem));
        let key = rustls_pemfile::pkcs8_private_keys(&mut key_reader)?
            .into_iter()
            .map(PrivateKey)
            .next()
            .ok_or("No private key found")?;

        // Create TLS config
        let config = ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(cert, key)?;

        Ok(Self {
            tls_acceptor: Some(TlsAcceptor::from(Arc::new(config))),
        })
    }

    async fn accept_connection(&self, stream: Async<TcpStream>) -> SimpleResult<Box<dyn AsyncConnection>> {
        if let Some(tls_acceptor) = &self.tls_acceptor {
            // Handle HTTPS connection
            let tls_stream = tls_acceptor.accept(stream).await?;
            Ok(Box::new(tls_stream))
        } else {
            // Handle HTTP connection
            Ok(Box::new(stream))
        }
    }

    async fn read_http_request<S: AsyncRead + AsyncWrite + Unpin>(
        stream: &mut S,
    ) -> SimpleResult<Request<Vec<u8>>> {
        // Wrap the stream with a BufReader for efficient reading
        let mut reader = BufReader::new(stream);

        // Read the request line (e.g., "GET /path HTTP/1.1")
        let mut request_line = String::new();
        reader.read_line(&mut request_line).await?;

        // Parse the request line into components
        let mut parts = request_line.trim().split_whitespace();
        let method = parts.next().ok_or(box_err!("Failed to parse method"))?;
        let uri = parts.next().ok_or(box_err!("Failed to parse URI"))?;
        let version = parts.next().ok_or(box_err!("Failed to parse version"))?;

        // Convert components into appropriate types for Request
        let method = Method::from_str(method)?;
        let uri = Uri::from_str(uri)?;
        let version = match version {
            "HTTP/1.0" => Version::HTTP_10,
            "HTTP/1.1" => Version::HTTP_11,
            "HTTP/2.0" => Version::HTTP_2,
            _ => return Err(box_err!("Unsupported HTTP version")),
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
            let key = header_parts.next().ok_or(box_err!("Failed to parse header key"))?;
            let value = header_parts.next().ok_or(box_err!("Failed to parse header value"))?;

            // Add the header to the request builder
            request_builder = request_builder.header(key.trim(), value.trim());
        }

        // Extract the Content-Length header if it exists
        let mut request_body = Vec::new();
        if let Some(length) = request_builder
            .headers_ref()
            .and_then(|headers| headers.get("content-length")) // TODO: case-sensitive?
        {
            let length = length
                .to_str()
                .map_err(|_| box_err!("Invalid Content-Length header"))?
                .parse::<usize>()
                .map_err(|_| box_err!("Content-Length is not a valid number"))?;

            // Read the specified number of bytes from the request body
            request_body.resize(length, 0);
            reader.read_exact(&mut request_body).await?;
        }

        // TODO: support more request body types like chunked, multipart, etc.

        // Build the request with the body
        let request = request_builder.body(request_body)?;

        Ok(request)
    }

    async fn handle_request<S: AsyncRead + AsyncWrite + Unpin>(
        router: Arc<Router>,
        mut stream: S,
    ) -> SimpleResult<()> {
        // read request
        let request = Self::read_http_request(&mut stream).await?;
    
        // Route requests by method + path
        let response = router.route(request).await?;
    
        // Write the status line
        let status_line = format!(
            "{:?} {} {}\r\n",
            response.version(),
            response.status().as_str(),
            response.status().canonical_reason().unwrap_or("")
        );
        stream.write_all(status_line.as_bytes()).await?;
    
        // Write headers
        for (name, value) in response.headers() {
            let header_line = format!("{}: {}\r\n", name, value.to_str()?);
            stream.write_all(header_line.as_bytes()).await?;
        }
    
        // Add Content-Length header if not present
        if !response.headers().contains_key("content-length") {
            let content_length = format!("Content-Length: {}\r\n", response.body().len());
            stream.write_all(content_length.as_bytes()).await?;
        }
    
        // Write the empty line that separates headers from body
        stream.write_all(b"\r\n").await?;
    
        // Write the body
        stream.write_all(response.body()).await?;
        stream.flush().await?;
        
        Ok(())
    }

    pub async fn run_server(
        executor: Arc<Executor<'static>>,
        host: &str,
        port: u16,
        router: Arc<Router>,
        tls_config: Option<(String, String)>,
    ) -> SimpleResult<()> {
        let server = if let Some((cert_path, key_path)) = tls_config {
            Self::with_tls(&cert_path, &key_path)?
        } else {
            Self::new()
        };

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
        
            match server.accept_connection(stream).await {
                Ok(connection) => {
                    let task = executor.spawn({
                        let router = router.clone();
                        async move {
                            if let Err(err) = Self::handle_request(router, connection).await {
                                log::error!("error handling request err = {err:?}");
                            }
                        }
                    });
                    task.detach();
                }
                Err(err) => {
                    log::warn!("Failed to establish connection: {:?}", err);
                    // Continue accepting new connections
                    continue;
                }
            }
        }
    }
}
