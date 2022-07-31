use crate::types::Platform;
use once_cell::sync::Lazy;
use scraper::ElementRef;
use scraper::Html;
use scraper::Selector;
use url::Url;

static GAME_TITLE_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse(".game_title").expect("invalid GAME_TITLE_SELECTOR"));
static TWITTER_URL_SELECTOR: Lazy<Selector> = Lazy::new(|| {
    Selector::parse("meta[name=\"twitter:url\"]").expect("invalid TWITTER_URL_SELECTOR")
});
static CSRF_TOKEN_SELECTOR: Lazy<Selector> = Lazy::new(|| {
    Selector::parse("meta[name=\"csrf_token\"]").expect("invalid CSRF_TOKEN_SELECTOR")
});
static DOWNLOAD_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse(".upload").expect("invalid DOWNLOAD_SELECTOR"));
static DOWNLOAD_BTN_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse(".download_btn").expect("invalid DOWNLOAD_BTN_SELECTOR"));
static TITLE_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse(".name").expect("invalid TITLE_SELECTOR"));
static FILE_SIZE_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse(".file_size > span").expect("invalid FILE_SIZE_SELECTOR"));
static PLATFORMS_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse(".download_platforms").expect("invalid PLATFORM_SELECTOR"));
static ICON_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse("span.icon").expect("invalid ICON_SELECTOR"));
static VIEW_HTML_SELECTOR: Lazy<Selector> = Lazy::new(|| {
    Selector::parse(".view_html_game_page .iframe_placeholder").expect("invalid VIEW_HTML_SELECTOR")
});
static IFRAME_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse("iframe").expect("invalid IFRAME_SELECTOR"));

///  Error that may occur while parsing a game page
#[derive(Debug, thiserror::Error)]
pub enum FromHtmlError {
    #[error("missing title")]
    MissingTitle,

    #[error("missing twitter url")]
    MissingTwitterUrl,

    #[error("invalid twitter url")]
    InvalidTwitterUrl(#[source] url::ParseError),

    #[error("missing csrf token")]
    MissingCsrfToken,

    #[error("missing download")]
    MissingDownload,

    #[error("invalid download")]
    InvalidDownload(#[from] FromElementError),

    #[error("missing iframe data")]
    MissingIFrameData,

    #[error("missing iframe data src")]
    MissingIFrameDataSrc,

    #[error("invalid iframe data src")]
    InvalidIFrameDataSrc(#[source] url::ParseError),
}

/// The page for a game
#[derive(Debug)]
pub struct GamePage {
    /// The title of this game
    pub title: String,

    /// The url of this page.
    ///
    /// This is called `twitter_url` as it is scraped from twitter metadata on the page.
    pub twitter_url: Url,

    /// A csrf token
    pub csrf_token: String,

    /// Download ids
    pub downloads: Vec<Download>,

    /// The view html url
    pub view_html_url: Option<Url>,
}

impl GamePage {
    /// Parse a game page
    pub fn from_html(html: &Html) -> Result<Self, FromHtmlError> {
        let title = html
            .select(&GAME_TITLE_SELECTOR)
            .next()
            .and_then(|title_el| title_el.text().next())
            .ok_or(FromHtmlError::MissingTitle)?
            .to_string();

        let twitter_url = Url::parse(
            html.select(&TWITTER_URL_SELECTOR)
                .next()
                .and_then(|element| element.value().attr("content"))
                .ok_or(FromHtmlError::MissingTwitterUrl)?,
        )
        .map_err(FromHtmlError::InvalidTwitterUrl)?;

        let csrf_token = html
            .select(&CSRF_TOKEN_SELECTOR)
            .next()
            .and_then(|element| element.value().attr("value"))
            .ok_or(FromHtmlError::MissingCsrfToken)?
            .to_string();

        let downloads = html
            .select(&DOWNLOAD_SELECTOR)
            .map(Download::from_element)
            .collect::<Result<_, _>>()?;

        let view_html_url = html
            .select(&VIEW_HTML_SELECTOR)
            .next()
            .map(|view_html_el| {
                let iframe_data = view_html_el
                    .value()
                    .attr("data-iframe")
                    .ok_or(FromHtmlError::MissingIFrameData)?;
                let html = Html::parse_fragment(iframe_data);
                html.select(&IFRAME_SELECTOR)
                    .next()
                    .ok_or(FromHtmlError::MissingIFrameData)?
                    .value()
                    .attr("src")
                    .ok_or(FromHtmlError::MissingIFrameDataSrc)
                    .map(Url::parse)?
                    .map_err(FromHtmlError::InvalidIFrameDataSrc)
            })
            .transpose()?;

        Ok(Self {
            title,
            twitter_url,
            csrf_token,
            downloads,
            view_html_url,
        })
    }
}

/// Error that may occur while parsing a download
#[derive(Debug, thiserror::Error)]
pub enum FromElementError {
    /// Missing Title
    #[error("missing title")]
    MissingTitle,

    /// Missing file size
    #[error("missing file size")]
    MissingFileSize,

    /// Invalid id
    #[error("invalid id")]
    InvalidId(#[source] std::num::ParseIntError),

    /// Missing platforms
    #[error("missing platforms")]
    MissingPlatforms,

    /// Missing a platform string
    #[error("missing platform string")]
    MissingPlatformString,

    /// Invalid platform
    #[error("invalid platform string `{0}`")]
    InvalidPlatformString(String),
}

/// A download
#[derive(Debug)]
pub struct Download {
    /// Download title
    pub title: String,

    /// Download size
    ///
    /// This is a string representation.
    /// The format is something like `N {specifier}`
    /// where `specifier` is 'MB' or 'kB'.
    pub size: String,

    /// Download id, if it exists
    pub id: Option<u64>,

    /// The platforms this download is for
    pub platforms: Vec<Platform>,
}

impl Download {
    /// Parse a download from an elemnt
    fn from_element(element: ElementRef) -> Result<Self, FromElementError> {
        let title = element
            .select(&TITLE_SELECTOR)
            .next()
            .and_then(|element| element.text().next())
            .ok_or(FromElementError::MissingTitle)?
            .to_string();

        let size = element
            .select(&FILE_SIZE_SELECTOR)
            .next()
            .and_then(|element| element.text().next())
            .ok_or(FromElementError::MissingFileSize)?
            .to_string();

        let id = element
            .select(&DOWNLOAD_BTN_SELECTOR)
            .next()
            .and_then(|element| element.value().attr("data-upload_id"))
            .map(|id_str| id_str.parse().map_err(FromElementError::InvalidId))
            .transpose()?;

        let platforms = element
            .select(&PLATFORMS_SELECTOR)
            .next()
            .ok_or(FromElementError::MissingPlatforms)?
            .select(&ICON_SELECTOR)
            .map(|icon_el| {
                let platform_str = icon_el
                    .value()
                    .classes()
                    .find_map(|class| class.strip_prefix("icon-"))
                    .ok_or(FromElementError::MissingPlatformString)?;

                match platform_str {
                    "windows8" => Ok(Platform::Windows),
                    "tux" => Ok(Platform::Linux),
                    "apple" => Ok(Platform::MacOs),
                    _ => Err(FromElementError::InvalidPlatformString(platform_str.into())),
                }
            })
            .collect::<Result<_, _>>()?;

        Ok(Self {
            title,
            size,
            id,
            platforms,
        })
    }

    /// Attempt to parse the size of this download.
    ///
    /// This is a rough approximation,
    /// and usually a lower limit as the value has little precision.
    ///
    /// # Return
    /// * Returns the value in bytes, otherwise None.
    pub fn parse_size(&self) -> Option<u64> {
        let (value, modifier) = self.size.split_once(' ')?;
        let value: u64 = value.parse().ok()?;
        let modifier = match modifier {
            "MB" => 1_000_000,
            "kB" => 1_000,
            _ => return None,
        };

        Some(value * modifier)
    }
}
