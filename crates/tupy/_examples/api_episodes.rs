use tupy::{api::{
    auth::OAuth, flow::{Credentials, Pkce}, scopes, PublicApi, Spotify, UserApi
}, Pagination};

static NOT_ANOTHER_DND_PODCAST: (&str, &str) = ("Not Another D&D Podcast", "6BTJ30JeHZ9qTduhSsFYDc");
static MYTHICAL_MONSTERS: (&str, &str) = ("Mythical Monsters", "5Bz1f8bZa0Fh6gOCsQkQqy");
static STUFF_YOU_SHOULD_KNOW: (&str, &str) = ("Stuff You Should Know", "3O5eqSuK2PRS3RWfBgXeId");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let oauth = OAuth::from_env([
        scopes::USER_LIBRARY_READ,
        scopes::USER_LIBRARY_MODIFY,
    ]).unwrap();

    let spotify = Spotify::<Pkce>::new(Credentials::from_env().unwrap(), oauth, "tupy").await?;
    let episodes = [
        NOT_ANOTHER_DND_PODCAST,
        MYTHICAL_MONSTERS,
        STUFF_YOU_SHOULD_KNOW,
    ];

    let episode = spotify.api.episode(episodes[0].1, None).await?;
    println!("Not Another D&D Podcast @ {}", episode.name);
    println!();

    println!("[Episodes]");
    for (podcast, episode) in episodes.iter().zip(spotify.api.episodes(episodes.iter().map(|e| e.1), None).await?) {
        println!(" - {} @ {}", podcast.0, episode.name); 
    }
    println!();

    let mut saved_episodes = spotify.api.saved_episodes::<2, _>(None)?;
    println!("[Saved Episodes]");
    while let Some(page) = saved_episodes.next().await? {
        for saved_episode in page.items {
            println!(" - {} @ {}", saved_episode.episode.show.name, saved_episode.episode.name);
        }

        if saved_episodes.progress() >= 6 {
            break;
        }
    }
    println!();

    let names = episodes.iter().map(|e| e.0).collect::<Vec<&str>>();
    println!("Saving Episodes {names:?}");
    spotify.api.save_episodes(episodes.iter().map(|e| e.1)).await?;
    println!();

    println!("[Episodes saved]");
    for (episode, added) in names.iter().zip(spotify.api.check_saved_episodes(episodes.iter().map(|e| e.1)).await?) {
        println!(" - {}: {}", episode, added);
    }
    println!();
    _ = dialoguer::Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt(format!("Remove episodes {names:?}?"))
        .interact()?;
    println!("Remove episodes {names:?}");
    spotify.api.remove_saved_episodes(episodes.iter().map(|e| e.1)).await?;
    println!();

    Ok(())
}
