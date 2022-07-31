use anyhow::Context;
use url::Url;

#[derive(argh::FromArgs)]
#[argh(description = "a CLI for interacting with itch.io")]
struct Options {
    #[argh(subcommand)]
    subcommand: Subcommand,
}

#[derive(argh::FromArgs)]
#[argh(subcommand)]
enum Subcommand {
    GameInfo(GameInfoOptions),
}

#[derive(argh::FromArgs)]
#[argh(subcommand, name = "game-info", description = "get game info")]
struct GameInfoOptions {
    #[argh(positional, description = "the url of the game")]
    url: Url,
}

fn main() -> anyhow::Result<()> {
    let options = argh::from_env();

    let tokio_rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("failed to build tokio runtime")?;
    tokio_rt.block_on(async_main(options))?;

    Ok(())
}

async fn async_main(options: Options) -> anyhow::Result<()> {
    let client = itch_io::Client::new();

    match options.subcommand {
        Subcommand::GameInfo(options) => {
            let game_page = client
                .get_game_page(options.url.as_str())
                .await
                .context("failed to get game page")?;

            println!("Title: {}", game_page.title);
            println!("Url: {}", game_page.twitter_url);
            println!("CSRF Token: {}", game_page.csrf_token);
            println!(
                "Html View Url: {}",
                game_page
                    .view_html_url
                    .as_ref()
                    .map(|url| url.as_str())
                    .unwrap_or("None")
            );

            println!("Downloads:");
            if game_page.downloads.is_empty() {
                println!("  None");
            }

            for download in game_page.downloads {
                let mut id_str_buffer = itoa::Buffer::new();

                println!("  Title: {}", download.title);
                println!("  Size: {}", download.size);
                println!(
                    "  Id: {}",
                    download
                        .id
                        .map(|id| id_str_buffer.format(id))
                        .unwrap_or("unknown")
                );
                println!("  Platforms:");
                if download.platforms.is_empty() {
                    println!("    None");
                }
                for platform in download.platforms {
                    let platform_str = match platform {
                        itch_io::Platform::Windows => "Windows",
                        itch_io::Platform::Linux => "Linux",
                        itch_io::Platform::MacOs => "MacOs",
                    };
                    println!("    {platform_str}");
                    println!();
                }
                println!();
            }
        }
    }

    Ok(())
}
