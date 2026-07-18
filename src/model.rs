use std::{fmt, str::FromStr};

use jiff::Timestamp;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use url::Url;

use crate::{Error, Result};

macro_rules! string_enum {
    ($name:ident { $($variant:ident => $value:literal),+ $(,)? }) => {
        #[doc = concat!("A typed ", stringify!($name), " value accepted by Google Play.")]
        #[derive(Clone, Debug, Eq, Hash, PartialEq)]
        #[non_exhaustive]
        pub enum $name {
            $(#[doc = concat!("The `", $value, "` wire value.")]
            $variant,)+
            /// A forward-compatible raw value.
            Custom(String),
        }

        impl $name {
            /// Returns the wire-format value.
            pub fn as_str(&self) -> &str {
                match self {
                    $(Self::$variant => $value,)+
                    Self::Custom(value) => value,
                }
            }

            /// Creates a custom value, rejecting empty or control-character input.
            ///
            /// # Errors
            ///
            /// Returns an error if the value is empty or contains control characters.
            pub fn custom(value: impl Into<String>) -> Result<Self> {
                let value = value.into();
                if value.trim().is_empty() || value.chars().any(char::is_control) {
                    return Err(Error::invalid(stringify!($name), "value must be nonempty and printable"));
                }
                Ok(Self::Custom(value))
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl FromStr for $name {
            type Err = Error;

            fn from_str(value: &str) -> Result<Self> {
                Ok(match value {
                    $($value => Self::$variant,)+
                    other => Self::custom(other)?,
                })
            }
        }

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
            where S: Serializer {
                serializer.serialize_str(self.as_str())
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
            where D: Deserializer<'de> {
                let value = String::deserialize(deserializer)?;
                value.parse().map_err(de::Error::custom)
            }
        }
    };
}

/// Validated Google Play application identifier.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AppId(String);

impl AppId {
    /// Validates and creates an application ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the ID is empty or contains whitespace or control characters.
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        if value.is_empty()
            || value.chars().any(char::is_whitespace)
            || value.chars().any(char::is_control)
        {
            return Err(Error::invalid(
                "app_id",
                "must be nonempty and contain no whitespace or control characters",
            ));
        }
        Ok(Self(value))
    }

    /// Returns the application ID as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for AppId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for AppId {
    type Err = Error;
    fn from_str(value: &str) -> Result<Self> {
        Self::new(value)
    }
}

/// Validated Google Play language code.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Language(String);

impl Language {
    /// Creates a normalized language code.
    ///
    /// # Errors
    ///
    /// Returns an error if the input is not a nonempty BCP-47-style code.
    pub fn new(value: impl AsRef<str>) -> Result<Self> {
        let value = value.as_ref().trim().to_ascii_lowercase();
        let valid = !value.is_empty()
            && value.len() <= 35
            && value
                .split('-')
                .all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_alphanumeric()));
        if !valid {
            return Err(Error::invalid(
                "language",
                "must be a BCP-47-style language code",
            ));
        }
        Ok(Self(value))
    }
    /// Returns the wire value.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Validated two-letter country code.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Country(String);

impl Country {
    /// Creates a normalized country code.
    ///
    /// # Errors
    ///
    /// Returns an error unless the input contains exactly two ASCII letters.
    pub fn new(value: impl AsRef<str>) -> Result<Self> {
        let value = value.as_ref().trim().to_ascii_lowercase();
        if value.len() != 2 || !value.chars().all(|c| c.is_ascii_alphabetic()) {
            return Err(Error::invalid(
                "country",
                "must contain exactly two ASCII letters",
            ));
        }
        Ok(Self(value))
    }
    /// Returns the wire value.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Language and country used for localized Google Play responses.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Locale {
    /// Response language.
    pub language: Language,
    /// Store country.
    pub country: Country,
}

impl Locale {
    /// Creates a validated locale.
    ///
    /// # Errors
    ///
    /// Returns an error when either locale component is invalid.
    pub fn new(language: impl AsRef<str>, country: impl AsRef<str>) -> Result<Self> {
        Ok(Self {
            language: Language::new(language)?,
            country: Country::new(country)?,
        })
    }
}

impl Default for Locale {
    fn default() -> Self {
        Self {
            language: Language("en".into()),
            country: Country("us".into()),
        }
    }
}

/// Price filter for search operations.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PriceFilter {
    /// Return free and paid applications.
    #[default]
    All,
    /// Return only applications available at no cost.
    Free,
    /// Return only paid applications.
    Paid,
}

string_enum!(Collection {
    TopFree => "topselling_free",
    TopPaid => "topselling_paid",
    TopGrossing => "topgrossing",
});

string_enum!(Category {
    Game => "GAME", Family => "FAMILY", Application => "APPLICATION",
    AndroidWear => "ANDROID_WEAR", ArtAndDesign => "ART_AND_DESIGN",
    AutoAndVehicles => "AUTO_AND_VEHICLES", Beauty => "BEAUTY",
    BooksAndReference => "BOOKS_AND_REFERENCE", Business => "BUSINESS",
    Comics => "COMICS", Communication => "COMMUNICATION", Dating => "DATING",
    Education => "EDUCATION", Entertainment => "ENTERTAINMENT", Events => "EVENTS",
    Finance => "FINANCE", FoodAndDrink => "FOOD_AND_DRINK",
    HealthAndFitness => "HEALTH_AND_FITNESS", HouseAndHome => "HOUSE_AND_HOME",
    LibrariesAndDemo => "LIBRARIES_AND_DEMO", Lifestyle => "LIFESTYLE",
    MapsAndNavigation => "MAPS_AND_NAVIGATION", Medical => "MEDICAL",
    MusicAndAudio => "MUSIC_AND_AUDIO", NewsAndMagazines => "NEWS_AND_MAGAZINES",
    Parenting => "PARENTING", Personalization => "PERSONALIZATION",
    Photography => "PHOTOGRAPHY", Productivity => "PRODUCTIVITY", Shopping => "SHOPPING",
    Social => "SOCIAL", Sports => "SPORTS", Tools => "TOOLS",
    TravelAndLocal => "TRAVEL_AND_LOCAL", VideoPlayers => "VIDEO_PLAYERS",
    WatchFace => "WATCH_FACE", Weather => "WEATHER", GameAction => "GAME_ACTION",
    GameAdventure => "GAME_ADVENTURE", GameArcade => "GAME_ARCADE", GameBoard => "GAME_BOARD",
    GameCard => "GAME_CARD", GameCasino => "GAME_CASINO", GameCasual => "GAME_CASUAL",
    GameEducational => "GAME_EDUCATIONAL", GameMusic => "GAME_MUSIC",
    GamePuzzle => "GAME_PUZZLE", GameRacing => "GAME_RACING",
    GameRolePlaying => "GAME_ROLE_PLAYING", GameSimulation => "GAME_SIMULATION",
    GameSports => "GAME_SPORTS", GameStrategy => "GAME_STRATEGY",
    GameTrivia => "GAME_TRIVIA", GameWord => "GAME_WORD",
});

/// Optional child age range for list requests.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgeRange {
    /// Content intended for children aged five and under.
    FiveAndUnder,
    /// Content intended for children aged six through eight.
    SixToEight,
    /// Content intended for children aged nine and older.
    NineAndUp,
}

impl AgeRange {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::FiveAndUnder => "AGE_RANGE1",
            Self::SixToEight => "AGE_RANGE2",
            Self::NineAndUp => "AGE_RANGE3",
        }
    }
}

/// Review ordering.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewSort {
    /// Most recent reviews first.
    #[default]
    Newest,
    /// Reviews ordered by rating.
    Rating,
    /// Most helpful reviews first.
    Helpfulness,
}

impl ReviewSort {
    pub(crate) const fn wire_value(self) -> u8 {
        match self {
            Self::Newest => 2,
            Self::Rating => 3,
            Self::Helpfulness => 1,
        }
    }
}

/// Exact price representation used by Google Play.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Money {
    /// Millionths of a currency unit.
    pub micros: i64,
    /// ISO-style currency code when provided.
    pub currency: Option<String>,
    /// Localized price text when provided.
    pub formatted: Option<String>,
}

impl Money {
    /// Creates an exact price from integer micros and optional display metadata.
    #[must_use]
    pub fn new(micros: i64, currency: Option<String>, formatted: Option<String>) -> Self {
        Self {
            micros,
            currency,
            formatted,
        }
    }

    /// Returns an approximate major-unit value for display only.
    #[allow(clippy::cast_precision_loss)]
    pub fn as_major_units(&self) -> f64 {
        self.micros as f64 / 1_000_000.0
    }
    /// Returns whether the exact price is zero.
    pub const fn is_free(&self) -> bool {
        self.micros == 0
    }
}

/// Rating counts by star value.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct RatingHistogram {
    /// Number of one-star ratings.
    pub one_star: u64,
    /// Number of two-star ratings.
    pub two_star: u64,
    /// Number of three-star ratings.
    pub three_star: u64,
    /// Number of four-star ratings.
    pub four_star: u64,
    /// Number of five-star ratings.
    pub five_star: u64,
}

/// Summary information returned by search and list operations.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AppOverview {
    /// Application package identifier.
    pub app_id: AppId,
    /// Localized application title.
    pub title: String,
    /// Canonical Google Play details URL assembled by PlayHound.
    pub store_url: Url,
    /// Application icon URL, when present.
    pub icon_url: Option<Url>,
    /// Localized developer name, when present.
    pub developer: Option<String>,
    /// Google Play developer identifier, when present.
    pub developer_id: Option<String>,
    /// Numeric average score, when present.
    pub score: Option<f64>,
    /// Localized score text, when present.
    pub score_text: Option<String>,
    /// Exact price and display metadata, when present.
    pub price: Option<Money>,
    /// Whether the application is free, when determinable.
    pub is_free: Option<bool>,
    /// Localized short summary, when present.
    pub summary: Option<String>,
}

/// Complete application metadata.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AppDetails {
    #[serde(flatten)]
    /// Fields shared with search and collection results.
    pub overview: AppOverview,
    /// Localized plain-text description.
    pub description: Option<String>,
    /// Localized description markup returned by Google Play.
    pub description_html: Option<String>,
    /// Localized install-count text.
    pub installs_text: Option<String>,
    /// Lower bound of the install count.
    pub min_installs: Option<u64>,
    /// Upper estimate of the install count.
    pub max_installs: Option<u64>,
    /// Total rating count.
    pub ratings: Option<u64>,
    /// Total written-review count.
    pub reviews: Option<u64>,
    /// Rating distribution by star value.
    pub histogram: Option<RatingHistogram>,
    /// Whether the application is available in the selected locale.
    pub available: Option<bool>,
    /// Whether the listing declares in-app purchases.
    pub offers_in_app_purchases: Option<bool>,
    /// Required Android version text.
    pub android_version: Option<String>,
    /// Developer contact email.
    pub developer_email: Option<String>,
    /// Developer website URL.
    pub developer_website: Option<Url>,
    /// Developer postal address.
    pub developer_address: Option<String>,
    /// Privacy-policy URL.
    pub privacy_policy: Option<Url>,
    /// Localized genre name.
    pub genre: Option<String>,
    /// Google Play genre identifier.
    pub genre_id: Option<String>,
    /// Header artwork URL.
    pub header_image_url: Option<Url>,
    /// Screenshot URLs in listing order.
    pub screenshot_urls: Vec<Url>,
    /// Promotional video URL.
    pub video_url: Option<Url>,
    /// Localized content-rating label.
    pub content_rating: Option<String>,
    /// Localized original release date.
    pub released: Option<String>,
    /// Last-update instant.
    pub updated: Option<Timestamp>,
    /// Current version text.
    pub version: Option<String>,
    /// Localized release notes.
    pub recent_changes: Option<String>,
    /// Additional listing comments, when supplied.
    pub comments: Vec<String>,
}

/// Developer response to a user review.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DeveloperReply {
    /// Reply text.
    pub text: String,
    /// Reply instant.
    pub date: Option<Timestamp>,
}

/// A normalized Google Play review.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Review {
    /// Stable review identifier supplied by Google Play.
    pub id: String,
    /// Display name of the reviewer.
    pub user_name: String,
    /// Reviewer avatar URL.
    pub user_image_url: Option<Url>,
    /// Review creation or update instant.
    pub date: Option<Timestamp>,
    /// Star score from one through five.
    pub score: u8,
    /// Review title, when supplied separately.
    pub title: Option<String>,
    /// Review body.
    pub text: Option<String>,
    /// Developer response, when present.
    pub developer_reply: Option<DeveloperReply>,
    /// Application version associated with the review.
    pub app_version: Option<String>,
    /// Helpful-vote count.
    pub thumbs_up: Option<u64>,
}

/// Opaque continuation token.
#[derive(Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PageToken(String);

impl PageToken {
    /// Creates a nonempty token.
    ///
    /// # Errors
    ///
    /// Returns an error for an empty token.
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        if value.is_empty() {
            return Err(Error::invalid("page_token", "must not be empty"));
        }
        Ok(Self(value))
    }
    /// Exposes the token for a subsequent request.
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for PageToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PageToken(len={})", self.0.len())
    }
}

/// One page of results and its continuation token.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Page<T> {
    /// Results in this page.
    pub items: Vec<T>,
    /// Opaque token for the next page, if one exists.
    pub next_page_token: Option<PageToken>,
}

#[cfg(test)]
#[path = "../tests/unit/model.rs"]
mod tests;
