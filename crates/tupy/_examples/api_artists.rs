use tupy::{
    api::{
        auth::OAuth, flow::{Credentials, Pkce}, request::IncludeGroup, scopes, PublicApi, Spotify, UserApi
    },
    Pagination,
};

static ADO: (&str, &str) = ("Ado", "6mEQK9m2krja6X1cfsAjfl");
static ONE_REPUBLIC: (&str, &str) = ("OneRepublic", "5Pwc4xIPtQLFEnJriah9YJ");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let oauth = OAuth::from_env(())
    .unwrap();

    let spotify = Spotify::<Pkce>::new(Credentials::from_env().unwrap(), oauth, "tupy").await?;

    let ado = spotify.api.artist(ADO.1).await?;
    println!("{} [{}/100] {}", ado.name, ado.popularity, match ado.images.is_empty() {
        true => String::new(),
        false => format!("\x1b]8;;{}\x1b\\PFP\x1b]8;;\x1b\\", &ado.images[0].url),
    });
    println!();

    let artists = spotify.api.artists([
        ADO.1, // Ado
        ONE_REPUBLIC.1, // OneRepublic
    ]).await?;
    let selection = dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Select an artist")
        .items(&artists.iter().map(|a| format!("{} [{}/100]", a.name, a.popularity)).collect::<Vec<String>>())
        .default(0)
        .interact()?;
    let artist = artists.get(selection).unwrap();
    println!();

    println!("[{} Albums]", artist.name);
    let mut albums = spotify.api.artist_albums::<10, _, _>(&artist.id, None, &[IncludeGroup::Album])?;
    while let Some(page) = albums.next().await? {
        for album in page.items {
            println!(" - {}", album.name);
        }
    }
    println!();

    println!("[{} Top Tracks]", artist.name);
    for track in spotify.api.artist_top_tracks(&artist.id, None).await? {
        println!(" - {}", track.name);
    }
    println!();

    println!("[Artists Related to {}]", artist.name);
    for artist in spotify.api.related_artists(&artist.uri).await? {
        println!(" - {}", artist.name);
    }
    println!();

    Ok(())
}
