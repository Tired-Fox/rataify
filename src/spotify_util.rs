use http_body_util::Full;
use hyper::{
    body::{Bytes, Incoming},
    service::Service,
    Method, Request, Response,
};
use std::{future::Future, net::SocketAddr, pin::Pin, str::FromStr};
use tokio::{net::TcpListener, sync::mpsc::UnboundedSender};

use tupy::Error;

#[derive(Debug, serde::Deserialize)]
pub struct AuthResponse {
    pub code: Option<String>,
    pub error: Option<String>,
    pub state: String,
}

pub struct Callback {
    pub state: String,
    pub path: String,
    pub tx: UnboundedSender<String>,
}

impl Clone for Callback {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            path: self.path.clone(),
            tx: self.tx.clone(),
        }
    }
}

impl Callback {
    pub fn new(uuid: String, path: &str, tx: UnboundedSender<String>) -> Self {
        Self {
            state: uuid.to_string(),
            path: path.to_string(),
            tx,
        }
    }

    fn handler(
        query: Option<&str>,
        state: String,
        result: UnboundedSender<String>,
    ) -> Result<Response<Full<Bytes>>, Box<dyn std::error::Error>> {
        match query {
            Some(query) => {
                let response: AuthResponse = serde_qs::from_str(query)?;
                if let Some(err) = response.error {
                    return Err(err.into());
                }

                // Validate State for cross-site request forgery
                match response.state == state {
                    false => {
                        result.send(String::new()).unwrap();
                        Err("Invalid response state".into())
                    }
                    true => {
                        result.send(response.code.unwrap()).unwrap();
                        Ok(Response::builder()
                            .body(Full::new(Bytes::from(layout(indoc::indoc! {r#"
                                    <h1>
                                       Successfully granted access to
                                       <span class="green">Spotify</span>
                                        for Rotify
                                    </h1>
                                    <h3>This tab may now be closed</h3>
                                "#}))))
                            .unwrap())
                    }
                }
            }
            None => {
                result.send(String::new()).unwrap();
                Err("Spotify did not send a response".into())
            }
        }
    }
}

impl Service<Request<Incoming>> for Callback {
    type Response = Response<Full<Bytes>>;
    type Error = String;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<Incoming>) -> Self::Future {
        match (req.method().clone(), req.uri().path()) {
            (Method::GET, path) if path == self.path.as_str() => {
                let state = self.state.clone();
                let tx = self.tx.clone();
                Box::pin(async move {
                    match Callback::handler(req.uri().query(), state, tx) {
                        Ok(response) => Ok(response),
                        Err(err) => {
                            log::error!("{:?}", err);
                            Ok(Response::builder()
                                .status(500)
                                .body(Full::new(Bytes::from("<h1>500 Internal Server Error<h1>")))
                                .unwrap())
                        }
                    }
                })
            }
            _ => Box::pin(async {
                Ok(Response::builder()
                    .status(404)
                    .body(Full::new(Bytes::from(layout(indoc::indoc! {r#"
                                <h1>"404 Page not found"</h1>
                            "#}))))
                    .unwrap())
            }),
        }
    }
}

/// Direct the user to their browser for authentication. Then automatically capture the redirect
/// uri and capture the authentication code
pub async fn listen_for_authentication_code(
    redirect: &str,
    auth_url: &str,
    state: &str,
) -> Result<String, Error> {
    let uri = hyper::Uri::from_str(redirect).map_err(Error::custom)?;

    // Mini http server to serve callback and parse auth code from spotify
    let addr = SocketAddr::from(([127, 0, 0, 1], uri.port_u16().unwrap_or(8888)));
    let listener = TcpListener::bind(addr).await?;

    println!("Listening on {}", addr);

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

    let callback = Callback::new(state.to_string(), uri.path(), tx);
    let handle = tokio::spawn(async move {
        loop {
            let (stream, _) = listener.accept().await.unwrap();
            let io = hyper_util::rt::TokioIo::new(stream);

            let cb = callback.clone();
            tokio::spawn(async move {
                if let Err(err) = hyper::server::conn::http1::Builder::new()
                    .serve_connection(io, cb)
                    .await
                {
                    eprintln!("Error serving connection to spotify callback: {:?}", err);
                }
            });
        }
    });

    // Open the default browser to the spotify login/authentication page.
    // When it is successful, the callback will be triggered and the result is returned
    open::that(auth_url)?;

    let result = rx.recv().await.ok_or("Spotify did not send a response")?;
    handle.abort();
    Ok(result)
}

fn layout<S: AsRef<str>>(body: S) -> String {
    format!(
        indoc::indoc! {r#"
            <html>
                <head>
                    <title>Rataify</title>
                    <style>
                    * {{
                        box-sizing: border-box
                    }}
                    html {{
                        font-family: Arial;
                        background-color: #191414;
                        color: #FFFFFF
                    }}
                    :is(h1, h3) {{
                        text-align: center;
                    }}
                    body {{
                        padding: 1.5rem;
                    }}
                    .green {{
                        color: #1DB954
                    }}
                    </style>
                </head>
                <body>
                    {}
                </body>
            </html>
        "#},
        body.as_ref()
    )
}
