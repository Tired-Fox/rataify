use color_eyre::Report;
use http_body_util::{Empty, Full};
use hyper::{Request, Response};
use hyper::body::{Body, Incoming, Bytes};
use hyper_util::rt::TokioIo;
use serde::Deserialize;
use tokio::net::TcpStream;

mod auth;
mod credentials;

pub use auth::OAuth;
pub use credentials::Credentials;


#[macro_export]
macro_rules! scopes {
    ($($scope: ident),* $(,)?) => {
        std::collections::HashSet::from([$(stringify!($scope).replace("_", "-"),)*])
    };
    ($scopes: literal) => {
        std::collections::HashSet::from_iter($scopes.split(" ").map(|s| s.to_string()))
    };
}
#[macro_export]
macro_rules! browser {
    ($base: literal ? $($param: ident = $value: expr),* $(,)?) => {
        open::that(format!("{}?{}",
            $base,
            vec![
                $(format!("{}={}", stringify!($param), $value)),*
            ].join("&")
        ))
    };
    ($base: literal) => {
        open::that($base)
    };
}
#[macro_export]
macro_rules! query {
    ($base: literal ? $($param: ident = $value: expr),* $(,)?) => {
        format!("{}?{}",
            $base,
            vec![
                $(format!("{}={}", stringify!($param), $value)),*
            ].join("&")
        )
    };
    ($base: literal) => {
        $base.to_string()
    };
}
pub use crate::browser;
pub use crate::scopes;
pub use crate::query;

pub(crate) trait SendRequest {
    async fn send(&self) -> color_eyre::Result<Response<Incoming>>;
}

impl SendRequest for Request<Full<Bytes>>
{
    async fn send(&self) -> color_eyre::Result<Response<Incoming>> {
        let url = self.uri().clone();
        let host = url.host().ok_or(Report::msg("uri has no host"))?;
        let port = url
            .port_u16()
            .unwrap_or(80);
        let address = format!("{}:{}", host, port);

        let stream = TcpStream::connect(address).await?;
        let io = TokioIo::new(stream);
        let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;

        tokio::task::spawn(async move {
            if let Err(err) = conn.await {
                eprintln!("Error making connection to spotify for authentication: {:?}", err);
            }
        });

        let authority = url.authority().unwrap().clone();
        println!("{:?}", authority);
        let mut req = self.clone();
        req.headers_mut().insert(hyper::header::HOST, authority.as_str().parse()?);

        Ok(sender.send_request(req).await?)
    }
}


impl SendRequest for Request<Empty<Bytes>>
{
    async fn send(&self) -> color_eyre::Result<Response<Incoming>> {
        let url = self.uri().clone();
        let host = url.host().ok_or(Report::msg("uri has no host"))?;
        let port = url
            .port_u16()
            .unwrap_or(80);
        let address = format!("{}:{}", host, port);

        let stream = TcpStream::connect(address).await?;
        let io = TokioIo::new(stream);
        let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;

        tokio::task::spawn(async move {
            if let Err(err) = conn.await {
                eprintln!("Error making connection to spotify for authentication: {:?}", err);
            }
        });

        let authority = url.authority().unwrap().clone();
        let mut req = self.clone();
        req.headers_mut().insert(hyper::header::HOST, authority.as_str().parse()?);

        Ok(sender.send_request(req).await?)
    }
}

