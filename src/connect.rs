//! Connection helper.
use tokio::net::TcpStream;

use tungstenite::{
    error::{Error, UrlError},
    handshake::client::Response,
    protocol::WebSocketConfig,
};

use crate::{domain, stream::MaybeTlsStream, IntoClientRequest, WebSocketStream};

#[cfg(any(feature = "native-tls", feature = "__rustls-tls"))]
use crate::Connector;

/// Connect to a given URL.
pub async fn connect_async<R>(
    request: R,
) -> Result<(WebSocketStream<MaybeTlsStream<TcpStream>>, Response), Error>
where
    R: IntoClientRequest + Unpin,
{
    connect_async_with_config(request, None, None).await
}

/// Connect to a given URL and trust the certificate always
pub async fn connect_async_trust_certificate<R>(
    request: R,
) -> Result<(WebSocketStream<MaybeTlsStream<TcpStream>>, Response), Error>
where
    R: IntoClientRequest + Unpin,
{
    // #[cfg(feature = "native-tls")]

    use native_tls_crate::TlsConnector;

    let mut builder = TlsConnector::builder();
    builder.danger_accept_invalid_certs(true);
    let connector = builder.build().unwrap();

    connect_async_with_config(request, None, Some(Connector::NativeTls(connector))).await
}

/// The same as `connect_async()` but the one can specify a websocket configuration.
/// Please refer to `connect_async()` for more details.
pub async fn connect_async_with_config<R>(
    request: R,
    config: Option<WebSocketConfig>,
    connector: Option<Connector>,
) -> Result<(WebSocketStream<MaybeTlsStream<TcpStream>>, Response), Error>
where
    R: IntoClientRequest + Unpin,
{
    let request = request.into_client_request()?;

    let domain = domain(&request)?;
    let port = request
        .uri()
        .port_u16()
        .or_else(|| match request.uri().scheme_str() {
            Some("wss") => Some(443),
            Some("ws") => Some(80),
            _ => None,
        })
        .ok_or(Error::Url(UrlError::UnsupportedUrlScheme))?;

    let addr = format!("{}:{}", domain, port);
    let try_socket = TcpStream::connect(addr).await;
    let socket = try_socket.map_err(Error::Io)?;

    crate::tls::client_async_tls_with_config(request, socket, config, connector).await
}

/// The same as `connect_async()` but the one can specify a websocket configuration,
/// and a TLS connector to use.
/// Please refer to `connect_async()` for more details.
#[cfg(any(feature = "native-tls", feature = "__rustls-tls"))]
pub async fn connect_async_tls_with_config<R>(
    request: R,
    config: Option<WebSocketConfig>,
    connector: Option<Connector>,
) -> Result<(WebSocketStream<MaybeTlsStream<TcpStream>>, Response), Error>
where
    R: IntoClientRequest + Unpin,
{
    let request = request.into_client_request()?;

    let domain = domain(&request)?;
    let port = request
        .uri()
        .port_u16()
        .or_else(|| match request.uri().scheme_str() {
            Some("wss") => Some(443),
            Some("ws") => Some(80),
            _ => None,
        })
        .ok_or(Error::Url(UrlError::UnsupportedUrlScheme))?;

    let addr = format!("{}:{}", domain, port);
    let try_socket = TcpStream::connect(addr).await;
    let socket = try_socket.map_err(Error::Io)?;
    crate::tls::client_async_tls_with_config(request, socket, config, connector).await
}
