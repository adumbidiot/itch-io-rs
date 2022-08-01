use crate::DownloadInfo;
use crate::DownloadPage;
use crate::DownloadPageUrlInfo;
use crate::Error;
use crate::GamePage;
use crate::PurchaseDialog;
use scraper::Html;

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
