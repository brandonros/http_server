use futures_lite::io::{AsyncRead, AsyncWrite};

pub trait AsyncConnection: AsyncRead + AsyncWrite + Send + Unpin {}

impl AsyncConnection for async_io::Async<std::net::TcpStream> {}
impl AsyncConnection for async_tls::server::TlsStream<async_io::Async<std::net::TcpStream>> {}
