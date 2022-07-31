/// Download page
pub mod download_page;
/// Game Page
pub mod game_page;
/// Purchase dialog
pub mod purchase_dialog;

pub use self::download_page::DownloadPage;
pub use self::game_page::GamePage;
pub use self::purchase_dialog::PurchaseDialog;
use url::Url;

/// Download info
#[derive(Debug, serde::Deserialize)]
pub struct DownloadInfo {
    /// ?
    pub external: bool,

    /// html?
    pub lightbox: String,

    /// The download url
    pub url: Url,
}

/// The download page url
#[derive(Debug, serde::Deserialize)]
pub struct DownloadPageUrlInfo {
    /// The download page url
    pub url: Url,
}
