use rataify::{config::Config, Error, App};

#[tokio::main]
async fn main() -> Result<(), Error> {
    App::new(Config::load()?)
        .await?
        .run()
        .await
}
