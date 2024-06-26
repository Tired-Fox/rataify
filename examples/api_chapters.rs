use tupy::api::{
    auth::OAuth,
    flow::{Credentials, Pkce},
    PublicApi, Spotify,
};

static CHAPTER_1: &str = "5xyvsBA200doffnyJwcJLI";
static CHAPTER_2: &str = "4Q1n87UCGyfD5R3lcXhRjU";
static CHAPTER_3: &str = "0T1gxYOYyuh1oKP6epDJtm";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let oauth = OAuth::from_env(()).unwrap();

    let spotify = Spotify::<Pkce>::new(Credentials::from_env().unwrap(), oauth, "tupy").await?;

    let c1 = spotify.api.chapter(CHAPTER_1, None).await?;
    println!("[A Fire Apon The Deep]");
    println!(" - {}", c1.name);

    for c in spotify.api.chapters([CHAPTER_2, CHAPTER_3], None).await? {
        println!(" - {}", c.name);
    }
    Ok(())
}
