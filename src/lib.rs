#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

mod client;
mod config;
mod error;
mod model;
mod parser;
mod protocol;
mod request;
mod transport;

#[cfg(test)]
#[path = "../tests/support/mod.rs"]
mod test_support;

#[cfg(feature = "blocking")]
pub mod blocking;

pub use client::{Client, ClientBuilder};
pub use config::{Proxy, RetryPolicy};
pub use error::{Error, ErrorKind, Result};
pub use model::{
    AgeRange, AppDetails, AppId, AppOverview, Category, Collection, Country, DeveloperReply,
    Language, Locale, Money, Page, PageToken, PriceFilter, RatingHistogram, Review, ReviewSort,
};
pub use request::{AppRequest, ListQuery, ReviewQuery, SearchQuery, SuggestionQuery};
