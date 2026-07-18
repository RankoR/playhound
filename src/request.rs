use crate::{AgeRange, AppId, Category, Collection, Locale, PageToken, PriceFilter, ReviewSort};

/// Request for complete application details.
#[must_use = "a request has no effect until passed to a client"]
#[derive(Clone, Debug)]
pub struct AppRequest {
    pub(crate) app_id: String,
    pub(crate) locale: Option<Locale>,
}
impl AppRequest {
    /// Creates a request. Validation occurs before transmission.
    pub fn new(app_id: impl Into<String>) -> Self {
        Self {
            app_id: app_id.into(),
            locale: None,
        }
    }
    /// Overrides the client locale.
    pub fn locale(mut self, locale: Locale) -> Self {
        self.locale = Some(locale);
        self
    }
}
impl From<&str> for AppRequest {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}
impl From<String> for AppRequest {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

/// Google Play search request.
#[must_use = "a query has no effect until passed to a client"]
#[derive(Clone, Debug)]
pub struct SearchQuery {
    pub(crate) term: String,
    pub(crate) limit: usize,
    pub(crate) price: PriceFilter,
    pub(crate) locale: Option<Locale>,
}
impl SearchQuery {
    /// Creates a search with a default limit of 20.
    pub fn new(term: impl Into<String>) -> Self {
        Self {
            term: term.into(),
            limit: 20,
            price: PriceFilter::All,
            locale: None,
        }
    }
    /// Sets the maximum number of results, following continuation pages as needed.
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }
    /// Restricts results by price.
    pub fn price(mut self, price: PriceFilter) -> Self {
        self.price = price;
        self
    }
    /// Overrides the client locale.
    pub fn locale(mut self, locale: Locale) -> Self {
        self.locale = Some(locale);
        self
    }
}
impl From<&str> for SearchQuery {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}
impl From<String> for SearchQuery {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

/// Google Play collection request.
#[must_use = "a query has no effect until passed to a client"]
#[derive(Clone, Debug)]
pub struct ListQuery {
    pub(crate) collection: Collection,
    pub(crate) category: Category,
    pub(crate) age: Option<AgeRange>,
    pub(crate) limit: usize,
    pub(crate) locale: Option<Locale>,
}
impl ListQuery {
    /// Creates a list request with a default limit of 50.
    pub fn new(collection: Collection, category: Category) -> Self {
        Self {
            collection,
            category,
            age: None,
            limit: 50,
            locale: None,
        }
    }
    /// Restricts family content to an age range.
    pub fn age(mut self, age: AgeRange) -> Self {
        self.age = Some(age);
        self
    }
    /// Sets the maximum number of results requested.
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }
    /// Overrides the client locale.
    pub fn locale(mut self, locale: Locale) -> Self {
        self.locale = Some(locale);
        self
    }
}
impl Default for ListQuery {
    fn default() -> Self {
        Self::new(Collection::TopFree, Category::Application)
    }
}

/// Review page request.
#[must_use = "a query has no effect until passed to a client"]
#[derive(Clone, Debug)]
pub struct ReviewQuery {
    pub(crate) app_id: String,
    pub(crate) sort: ReviewSort,
    pub(crate) page_size: usize,
    pub(crate) page_token: Option<PageToken>,
    pub(crate) locale: Option<Locale>,
}
impl ReviewQuery {
    /// Creates a request with newest-first sorting and a page size of 100.
    pub fn new(app_id: impl Into<String>) -> Self {
        Self {
            app_id: app_id.into(),
            sort: ReviewSort::Newest,
            page_size: 100,
            page_token: None,
            locale: None,
        }
    }
    /// Sets the review order.
    pub fn sort(mut self, sort: ReviewSort) -> Self {
        self.sort = sort;
        self
    }
    /// Sets the number of reviews requested for this page.
    pub fn page_size(mut self, page_size: usize) -> Self {
        self.page_size = page_size;
        self
    }
    /// Continues from an opaque token returned by an earlier page.
    pub fn page_token(mut self, page_token: PageToken) -> Self {
        self.page_token = Some(page_token);
        self
    }
    /// Overrides the client locale.
    pub fn locale(mut self, locale: Locale) -> Self {
        self.locale = Some(locale);
        self
    }
}

/// Search-suggestion request.
#[must_use = "a query has no effect until passed to a client"]
#[derive(Clone, Debug)]
pub struct SuggestionQuery {
    pub(crate) term: String,
    pub(crate) locale: Option<Locale>,
}
impl SuggestionQuery {
    /// Creates a suggestion request for the supplied partial search term.
    pub fn new(term: impl Into<String>) -> Self {
        Self {
            term: term.into(),
            locale: None,
        }
    }
    /// Overrides the client locale.
    pub fn locale(mut self, locale: Locale) -> Self {
        self.locale = Some(locale);
        self
    }
}
impl From<&str> for SuggestionQuery {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}
impl From<String> for SuggestionQuery {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

pub(crate) fn validate_app_id(value: &str) -> crate::Result<AppId> {
    AppId::new(value)
}
pub(crate) fn validate_nonempty(field: &'static str, value: &str) -> crate::Result<()> {
    if value.trim().is_empty() {
        Err(crate::Error::invalid(field, "must not be empty"))
    } else {
        Ok(())
    }
}
pub(crate) fn validate_limit(field: &'static str, value: usize) -> crate::Result<()> {
    if value == 0 || value > i32::MAX as usize {
        Err(crate::Error::invalid(
            field,
            "must be nonzero and fit in a signed 32-bit wire value",
        ))
    } else {
        Ok(())
    }
}
