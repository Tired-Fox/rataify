use tupy::{api::{
    auth::OAuth, flow::{Credentials, Pkce}, scopes, PublicApi, Spotify, UserApi
}, Pagination};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let oauth = OAuth::from_env([
        scopes::USER_LIBRARY_READ,
        scopes::USER_LIBRARY_MODIFY,
    ]).unwrap();

    let spotify = Spotify::<Pkce>::new(Credentials::from_env().unwrap(), oauth, "tupy").await?;
    println!(
        "{}",
        spotify
            .api
            .audiobook("7iHfbu1YPACw6oZPAFJtqe", None)
            .await?
            .name
    );
    println!();

    let audiobooks = spotify.api.audiobooks([
        "18yVqkdbdRvS24c0Ilj2ci",
        "1HGw3J3NxZO1TP1BTtVhpZ",
        "7iHfbu1YPACw6oZPAFJtqe"
    ], None).await?;
    let selection = dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Select an audiobook")
        .items(&audiobooks.iter().map(|a| a.name.clone()).collect::<Vec<String>>())
        .default(0)
        .interact()?;
    let audiobook = audiobooks.get(selection).unwrap();
    println!();

    println!("[{} Chapters]", audiobook.name);
    let mut chapters = spotify.api.audiobook_chapters::<10, _, _>(&audiobook.uri, None)?;
    if let Some(page) = chapters.next().await? {
        for chapter in page.items {
            println!(" - {} ({})", chapter.name, chapter.id);
        }

        if chapters.item_count() < page.total {
            println!(" - ...{} More Chapters...", page.total - chapters.item_count());
        }
    }
    println!();

    println!("[Saved Audiobooks]");
    let mut saved = spotify.api.saved_audiobooks::<2>()?;
    if let Some(page) = saved.next().await? {
        for audiobook in page.items {
            println!(" - {}", audiobook.name);
        }
    }
    println!();

    println!("Saving audiobook [Dune]");
    spotify.api.save_audiobooks(["7iHfbu1YPACw6oZPAFJtqe"]).await?;
    println!();

    println!("[Audiobooks saved]");
    for (audiobook, added) in ["Dune"].iter().zip(spotify.api.check_saved_audiobooks(["7iHfbu1YPACw6oZPAFJtqe"]).await?) {
        println!(" - {}: {}", audiobook, added);
    }
    println!();
    _ = dialoguer::Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Remove audiobook [Dune]?")
        .interact()?;
    println!("Remove audiobook [Dune]");
    spotify.api.remove_saved_audiobooks(["7iHfbu1YPACw6oZPAFJtqe"]).await?;
    println!();

    Ok(())
}
