use tupy::{api::{
    auth::OAuth, flow::{Credentials, Pkce}, scopes, PublicApi, Spotify, UserApi
}, Pagination};

static NOT_ANOTHER_DND_PODCAST: (&str, &str) = ("Not Another D&D Podcast", "5GcTIDkgnB9wP6CmUyOSqa");
static MYTHICAL_MONSTERS: (&str, &str) = ("Mythical Monsters", "5viQDUDBZ4tsPFlAwnn1v0");
static STUFF_YOU_SHOULD_KNOW: (&str, &str) = ("Stuff You Should Know", "0ofXAdFIQQRsCYj9754UFx");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let oauth = OAuth::from_env([
        scopes::USER_LIBRARY_READ,
        scopes::USER_LIBRARY_MODIFY,
    ]).unwrap();

    let spotify = Spotify::<Pkce>::new(Credentials::from_env().unwrap(), oauth, "tupy").await?;
    let shows = [
        NOT_ANOTHER_DND_PODCAST,
        MYTHICAL_MONSTERS,
        STUFF_YOU_SHOULD_KNOW,
    ];

    let show = spotify.api.show(shows[0].1, None).await?;
    println!("{}", show.name);
    println!();

    println!("[Shows]");
    for (show, info) in shows.iter().zip(spotify.api.shows(shows.iter().map(|e| e.1), None).await?) {
        println!(" - {} @ {}", show.0, info.id); 
    }
    println!();

    println!("[Episodes]");
    let mut episodes = spotify.api.show_episodes::<2, _, _>(shows[0].1, None)?;
    while let Some(page) = episodes.next().await? {
        for episode in page.items {
            println!(" - {} @ {}", shows[0].0, episode.name);
        }

        if episodes.progress() >= 15 {
            if episodes.progress() < episodes.total() {
                println!(" - ...{} More...", episodes.total() - episodes.progress());
            }
            break;
        }
    }
    println!();

    let mut saved_shows = spotify.api.saved_shows::<2>()?;
    println!("[Saved Shows]");
    while let Some(page) = saved_shows.next().await? {
        for saved_episode in page.items {
            println!(" - {}", saved_episode.show.name);
        }

        if saved_shows.progress() >= 6 {
            break;
        }
    }
    println!();

    let names = shows.iter().map(|e| e.0).collect::<Vec<&str>>();
    println!("Saving shows {names:?}");
    spotify.api.save_shows(shows.iter().map(|e| e.1)).await?;
    println!();

    println!("[Shows Saved]");
    for (episode, added) in names.iter().zip(spotify.api.check_saved_shows(shows.iter().map(|e| e.1)).await?) {
        println!(" - {}: {}", episode, added);
    }
    println!();
    _ = dialoguer::Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt(format!("Remove shows {names:?}?"))
        .interact()?;
    println!("Remove shows {names:?}");
    spotify.api.remove_saved_shows(shows.iter().map(|e| e.1)).await?;
    println!();

    Ok(())
}
