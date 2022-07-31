mod types;

pub use self::types::DownloadInfo;
pub use self::types::DownloadPage;
pub use self::types::DownloadPageUrlInfo;
pub use self::types::GamePage;
pub use self::types::Platform;
pub use self::types::PurchaseDialog;
use scraper::Html;

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

/// The client
#[derive(Debug, Clone)]
pub struct Client {
    /// The inner http client.
    ///
    /// Probably shouldn't be used by you.
    pub client: reqwest::Client,
}

impl Client {
    /// Make a new client
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .cookie_store(true)
                .build()
                .expect("failed to build itch.io client"),
        }
    }

    /// Get a page and parse it
    async fn get_html<F, T>(&self, url: &str, f: F) -> Result<T, Error>
    where
        F: FnOnce(Html) -> T + Send + 'static,
        T: Send + 'static,
    {
        let text = self
            .client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        Ok(tokio::task::spawn_blocking(move || {
            let html = Html::parse_document(&text);
            f(html)
        })
        .await?)
    }

    /// Get a game page.
    ///
    /// The `url` parameter should be a url for the game page, like `https://tumblewed.itch.io/doghouse-2`.
    pub async fn get_game_page(&self, url: &str) -> Result<GamePage, Error> {
        Ok(self
            .get_html(url, |html| GamePage::from_html(&html))
            .await??)
    }

    /// Get the download info for a given game download by id.
    pub async fn get_download_info(
        &self,
        game_page_url: &str,
        download_id: u64,
        csrf_token: &str,
    ) -> Result<DownloadInfo, Error> {
        let url = format!("{game_page_url}/file/{download_id}?after_download_lightbox=true");

        Ok(self
            .client
            .post(url.as_str())
            .form(&[("csrf_token", csrf_token)])
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    /// Get the purchase dialog for a game.
    ///
    /// This is the download that appears when clicking "download now".
    pub async fn get_purchase_dialog(&self, game_page_url: &str) -> Result<PurchaseDialog, Error> {
        let url = format!("{game_page_url}/purchase?lightbox=true");
        Ok(self
            .client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    /// Get the download page url
    pub async fn get_download_page_url(
        &self,
        game_page_url: &str,
        csrf_token: &str,
    ) -> Result<DownloadPageUrlInfo, Error> {
        let url = format!("{game_page_url}/download_url");
        Ok(self
            .client
            .post(url)
            .form(&[("csrf_token", csrf_token)])
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    /// Get the download page from a url.
    ///
    /// The url must be from the `DownloadPageUrlInfo` struct.
    pub async fn get_download_page(&self, url: &str) -> Result<DownloadPage, Error> {
        Ok(self
            .get_html(url, |html| DownloadPage::from_html(&html))
            .await??)
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
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
