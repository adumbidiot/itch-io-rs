/// The client
mod client;
/// API types
mod types;

pub use self::client::Client;
pub use self::types::DownloadInfo;
pub use self::types::DownloadPage;
pub use self::types::DownloadPageUrlInfo;
pub use self::types::GamePage;
pub use self::types::Platform;
pub use self::types::PurchaseDialog;

/// The error type
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// HTTP Error
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    /// Tokio Join Error
    #[error(transparent)]
    TokioJoin(#[from] tokio::task::JoinError),

    /// Invalid Game page
    #[error("invalid game page")]
    InvalidGamePage(#[from] self::types::game_page::FromHtmlError),

    /// Invalid Download page
    #[error("invalid download page")]
    InvalidDownloadPage(#[from] self::types::download_page::FromHtmlError),
}

#[cfg(test)]
mod test {
    use super::*;

    const GAME_PAGE_URLS: &[&str] = &[
        "https://tumblewed.itch.io/doghouse-2",
        "https://jnohr.itch.io/mrk-borg-free",
        "https://maytch.itch.io/roll-racer",
    ];

    #[tokio::test]
    async fn download_game_page_works() {
        let client = Client::new();

        for url in GAME_PAGE_URLS {
            let game_page = client
                .get_game_page(url)
                .await
                .expect("failed to get game page");
            dbg!(&game_page);

            for download in game_page.downloads.iter() {
                let parsed_size = download.parse_size().expect("failed to parse size");
                dbg!(parsed_size);

                let id = match download.id {
                    Some(id) => id,
                    None => {
                        let download_page_url_info = client
                            .get_download_page_url(
                                game_page.twitter_url.as_str(),
                                game_page.csrf_token.as_str(),
                            )
                            .await
                            .expect("failed to get download page url");
                        let download_page = client
                            .get_download_page(download_page_url_info.url.as_str())
                            .await
                            .expect("failed to get download page");

                        let download = download_page
                            .downloads
                            .iter()
                            .find(|download_page_download| {
                                download.title == download_page_download.title
                            })
                            .expect("failed to locate download by title");

                        dbg!(&download);

                        download.id
                    }
                };

                let download_info = client
                    .get_download_info(game_page.twitter_url.as_str(), id, &game_page.csrf_token)
                    .await
                    .expect("failed to get download info");
                dbg!(download_info);
            }

            if let Some(view_html_url) = game_page.view_html_url {
                dbg!(view_html_url.as_str());

                // TODO: Download somehow?
            }
        }
    }
}
