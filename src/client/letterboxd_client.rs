//! The asynchronous client that fetches Letterboxd RSS feeds.

use std::time::Duration;

use crate::client::build_feed_url::build_feed_url;
use crate::client::default_timeout::DEFAULT_TIMEOUT;
use crate::client::fetch_report::FetchReport;
use crate::error::LetterboxdError;
use crate::parse::parse_feed::parse_feed;
use crate::result::Result;
use crate::types::LetterboxdFeed;

/// An asynchronous client for Letterboxd's public RSS feeds.
///
/// The client is cheap to clone and safe to share between tasks. It holds a
/// `reqwest::Client`, which pools connections, so create one and reuse it.
///
/// ```no_run
/// # use letterboxd_rss_client_rs::LetterboxdClient;
/// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
/// let client = LetterboxdClient::new();
/// let feed = client.fetch_feed("jsiewert").await?;
/// println!("{} items", feed.items.len());
/// # Ok(())
/// # }
/// ```
///
/// # Builder
///
/// The settings are applied in the order they are called, and each one keeps
/// the others intact:
///
/// ```
/// # use letterboxd_rss_client_rs::LetterboxdClient;
/// # use std::time::Duration;
/// let client = LetterboxdClient::new()
///     .with_timeout(Duration::from_secs(10))
///     .with_user_agent("my-tool/1.0");
/// ```
#[derive(Debug, Clone)]
pub struct LetterboxdClient {
    http: reqwest::Client,
    user_agent: String,
    timeout: Duration,
}

impl LetterboxdClient {
    /// Creates a client with the default settings.
    ///
    /// The default timeout is 30 seconds.
    pub fn new() -> Self {
        let user_agent = default_user_agent();
        let http = build_http_client(&user_agent, DEFAULT_TIMEOUT);

        Self {
            http,
            user_agent,
            timeout: DEFAULT_TIMEOUT,
        }
    }

    /// Sets the request timeout.
    ///
    /// The timeout applies to each individual request.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self.http = build_http_client(&self.user_agent, timeout);
        self
    }

    /// Sets the `User-Agent` header sent with every request.
    ///
    /// Letterboxd serves the public RSS feed to any client, but sending a
    /// descriptive user agent is good manners and makes the client easier to
    /// identify in server logs.
    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = user_agent.into();
        self.http = build_http_client(&self.user_agent, self.timeout);
        self
    }

    /// Returns the `User-Agent` the client sends.
    pub fn user_agent(&self) -> &str {
        &self.user_agent
    }

    /// Returns the request timeout of the client.
    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    /// Fetches and parses the RSS feed of a member.
    ///
    /// # Errors
    ///
    /// Returns [`LetterboxdError::MemberNotFound`] when the member does not
    /// exist, [`LetterboxdError::Http`] when the request fails,
    /// [`LetterboxdError::HttpStatus`] when Letterboxd answers with a
    /// non-success status, and [`LetterboxdError::MalformedFeed`] when the
    /// response is not a Letterboxd feed.
    pub async fn fetch_feed(&self, username: &str) -> Result<LetterboxdFeed> {
        let xml = self.fetch_feed_xml(username).await?;
        parse_feed(&xml)
    }

    /// Fetches the raw RSS XML of a member without parsing it.
    ///
    /// # Errors
    ///
    /// Returns [`LetterboxdError::MemberNotFound`] when Letterboxd answers
    /// with a 404, which is how it reports an unknown member.
    /// [`LetterboxdError::Http`] covers a failed request and
    /// [`LetterboxdError::HttpStatus`] covers any other non-success status.
    pub async fn fetch_feed_xml(&self, username: &str) -> Result<String> {
        let url = build_feed_url(username)?;
        let response = self.http.get(&url).send().await?;

        let status = response.status();
        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(LetterboxdError::MemberNotFound(username.to_string()));
        }
        if !status.is_success() {
            return Err(LetterboxdError::HttpStatus {
                status: status.as_u16(),
                url,
            });
        }

        response.text().await.map_err(LetterboxdError::Http)
    }

    /// Fetches the feeds of several members concurrently.
    ///
    /// The members are fetched in parallel and the results are returned in the
    /// same order as the input. A member whose feed cannot be fetched yields
    /// an empty feed in its slot and its error is collected, so one bad
    /// username does not lose the data of the others.
    ///
    /// # Errors
    ///
    /// Returns [`LetterboxdError::MemberNotFound`] when at least one member
    /// does not exist. The successfully fetched feeds are still returned
    /// through the [`FetchReport`] in the error's [`LetterboxdError::MemberNotFound`]
    /// variant only for a single member; for multiple members use
    /// [`LetterboxdClient::fetch_feeds_report`].
    pub async fn fetch_feeds(&self, usernames: &[impl AsRef<str>]) -> Result<Vec<LetterboxdFeed>> {
        Ok(self.fetch_feeds_report(usernames).await.feeds)
    }

    /// Fetches the feeds of several members concurrently, keeping the errors
    /// of individual members alongside the successful results.
    ///
    /// This never fails: a member that cannot be fetched produces an error
    /// entry in [`FetchReport::errors`] instead.
    pub async fn fetch_feeds_report(&self, usernames: &[impl AsRef<str>]) -> FetchReport {
        let mut feeds = Vec::with_capacity(usernames.len());
        let mut errors = Vec::new();

        for username in usernames {
            let username = username.as_ref();
            match self.fetch_feed(username).await {
                Ok(feed) => feeds.push(feed),
                Err(error) => {
                    errors.push((username.to_string(), error));
                    feeds.push(LetterboxdFeed::empty());
                }
            }
        }

        FetchReport { feeds, errors }
    }
}

impl Default for LetterboxdClient {
    fn default() -> Self {
        Self::new()
    }
}

fn build_http_client(user_agent: &str, timeout: Duration) -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent(user_agent.to_string())
        .timeout(timeout)
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}

fn default_user_agent() -> String {
    format!(
        "letterboxd-rss-client-rs/{} (+https://github.com/sleet-ai-lab/letterboxd-rss-client-rs)",
        env!("CARGO_PKG_VERSION")
    )
}
