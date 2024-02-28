use color_eyre::eyre::eyre;
use color_eyre::Report;
use html_to_string_macro::html;
use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use hyper::{Method, Request, Response};
use hyper::service::Service;
use serde::Deserialize;
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub(crate) struct AuthCodeResponse {
    pub code: Option<String>,
    pub error: Option<String>,
    pub state: String,
}

pub struct Callback {
    pub state: String,
    pub tx: UnboundedSender<String>,
}

impl Clone for Callback {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            tx: self.tx.clone(),
        }
    }
}

macro_rules! layout {
    ($($html: tt)*) => {
        layout(html! { $($html)*})
    };
}

impl Callback {
    pub fn new(uuid: Uuid, tx: UnboundedSender<String>) -> Self {
        Self {
            state: uuid.to_string(),
            tx
        }
    }

    async fn handler(query: Option<&str>, state: String, result: UnboundedSender<String>) -> color_eyre::Result<Response<Full<Bytes>>> {
        match query {
            Some(query) => {
                let response: AuthCodeResponse = serde_qs::from_str(query)?;
                if let Some(err) = response.error {
                    return Err(eyre!(err));
                }

                // Validate State for cross-site request forgery
                match response.state == state {
                    false => {
                        result.send(String::new()).unwrap();
                        return Err(eyre!("Invalid response state"));
                    }
                    true => {
                        result.send(response.code.unwrap()).unwrap();
                        Ok(
                            Response::builder()
                                .body(Full::new(Bytes::from(layout! {
                                   <h1>
                                       "Successfully authenticated Rataify with "
                                       <span class="green">"Spotify"</span>
                                   </h1>
                                   <h3>"This tab may now be closed"</h3>
                                })))
                                .unwrap()
                        )
                    }
                }
            }
            None => {
                result.send(String::new()).unwrap();
                return Err(eyre!("Spotify did not send a response"));
            }
        }
    }
}

impl Service<Request<Incoming>> for Callback {
    type Response = Response<Full<Bytes>>;
    type Error = Report;
    type Future = futures::future::BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn call(&self, req: Request<Incoming>) -> Self::Future {
        match (req.method().clone(), req.uri().path()) {
            (Method::GET, "/Rataify/auth") => {
                let state = self.state.clone();
                let tx = self.tx.clone();
                Box::pin(async move {
                    match Callback::handler(req.uri().query(), state, tx).await {
                        Ok(response) => Ok(response),
                        Err(err) => {
                            eprintln!("{:?}", err);
                            Ok(
                                Response::builder()
                                    .status(500)
                                    .body(Full::new(Bytes::from("<h1>500 Internal Server Error<h1>")))
                                    .unwrap()
                            )
                        }
                    }
                })
            },
            _ => {
                Box::pin(async {
                    Ok(
                        Response::builder()
                            .status(404)
                            .body(Full::new(Bytes::from(layout! {
                                <h1>"404 Page not found"</h1>
                            })))
                            .unwrap()
                    )
                })
            }
        }
    }
}

fn layout(body: String) -> String {
    html! {
        <html>
            <head>
                <title>"Rataify"</title>
                <style>"
                * {
                    box-sizing: border-box
                }
                html {
                    font-family: Arial;
                    background-color: #191414;
                    color: #FFFFFF
                }
                :is(h1, h3) {
                    text-align: center;
                }
                body {
                    padding: 1.5rem;
                }
                .green {
                    color: #1DB954
                }
                "</style>
            </head>
            <body>
                {body}
            </body>
        </html>
    }
}
// TODO: Create custom pages that can close automatically