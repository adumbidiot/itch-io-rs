use once_cell::sync::Lazy;
use scraper::ElementRef;
use scraper::Html;
use scraper::Selector;

static DOWNLOAD_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse(".upload").expect("invalid DOWNLOAD_SELECTOR"));
static TITLE_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse(".name").expect("invalid TITLE_SELECTOR"));
static FILE_SIZE_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse(".file_size > span").expect("invalid FILE_SIZE_SELECTOR"));
static DOWNLOAD_BTN_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse(".download_btn").expect("invalid DOWNLOAD_BTN_SELECTOR"));
static PLATFORMS_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse(".download_platforms").expect("invalid PLATFORM_SELECTOR"));
static ICON_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse("span.icon").expect("invalid ICON_SELECTOR"));

/// An error that occurs while parsing from html
#[derive(Debug, thiserror::Error)]
pub enum FromHtmlError {
    #[error("invalid download")]
    InvalidDownload(#[from] FromElementError),
}

/// A page of downloads
#[derive(Debug)]
pub struct DownloadPage {
    /// Downloads
    pub downloads: Vec<Download>,
}

impl DownloadPage {
    /// Parse this from html
    pub(crate) fn from_html(html: &Html) -> Result<Self, FromHtmlError> {
        let downloads = html
            .select(&DOWNLOAD_SELECTOR)
            .map(Download::from_element)
            .collect::<Result<_, _>>()?;

        Ok(Self { downloads })
    }
}

/// An error that may occur while parsing an element.
#[derive(Debug, thiserror::Error)]
pub enum FromElementError {
    /// Missing Title
    #[error("missing title")]
    MissingTitle,

    /// Missing file size
    #[error("missing file size")]
    MissingFileSize,

    /// Missing id
    #[error("missing id")]
    MissingId,

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
    /// The title of the download
    pub title: String,

    /// Download size
    ///
    /// This is a string representation.
    /// The format is something like `N {specifier}`
    /// where `specifier` is 'MB' or 'kB'.
    pub size: String,

    /// Download id
    pub id: u64,

    // The platforms this download is for
    pub platforms: Vec<Platform>,
}

impl Download {
    /// Make this from an element
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
            .ok_or(FromElementError::MissingId)?
            .parse()
            .map_err(FromElementError::InvalidId)?;

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

/// A platform
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Platform {
    /// Windows
    Windows,

    /// Linux
    Linux,

    /// MacOs
    MacOs,
}
