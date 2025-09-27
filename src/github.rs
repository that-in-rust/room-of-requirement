use crate::{AppError, Result, SearchResponse};

#[cfg(test)]
mod tests;
use reqwest::{Client, StatusCode};
use serde_json::Value;
use std::time::Duration;
use tokio::time::sleep;

/// GitHub API client with authentication and rate limiting
#[derive(Debug, Clone)]
pub struct GitHubClient {
    client: Client,
    token: String,
    base_url: String,
}

/// Rate limiting configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Initial backoff delay in milliseconds
    pub initial_backoff_ms: u64,
    /// Maximum backoff delay in milliseconds
    pub max_backoff_ms: u64,
    /// Backoff multiplier for exponential backoff
    pub backoff_multiplier: f64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_backoff_ms: 1000,
            max_backoff_ms: 60000,
            backoff_multiplier: 2.0,
        }
    }
}

impl GitHubClient {
    /// Create a new GitHub client with authentication token
    /// 
    /// # Arguments
    /// * `token` - GitHub personal access token or API token
    /// 
    /// # Returns
    /// * `Result<GitHubClient>` - Configured client or error
    pub fn new(token: String) -> Result<Self> {
        if token.is_empty() {
            return Err(AppError::authentication("GitHub token cannot be empty"));
        }

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("github-pg-query/0.1.0")
            .build()
            .map_err(|e| AppError::configuration(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            client,
            token,
            base_url: "https://api.github.com".to_string(),
        })
    }

    /// Create a new GitHub client with custom base URL (for testing)
    pub fn with_base_url(token: String, base_url: String) -> Result<Self> {
        let mut client = Self::new(token)?;
        client.base_url = base_url;
        Ok(client)
    }

    /// Search repositories using GitHub API with rate limiting and retry logic
    /// 
    /// # Arguments
    /// * `query` - GitHub search query string (e.g., "rust language:rust")
    /// * `per_page` - Number of results per page (1-100, default: 30)
    /// * `page` - Page number to retrieve (default: 1)
    /// 
    /// # Returns
    /// * `Result<SearchResponse>` - Search results or error
    /// 
    /// # Examples
    /// ```rust,no_run
    /// use github_pg_query::GitHubClient;
    /// 
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = GitHubClient::new("your_token".to_string())?;
    /// let results = client.search_repositories("rust language:rust", Some(50), Some(1)).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn search_repositories(
        &self,
        query: &str,
        per_page: Option<u32>,
        page: Option<u32>,
    ) -> Result<SearchResponse> {
        self.search_repositories_with_config(query, per_page, page, &RateLimitConfig::default())
            .await
    }

    /// Search repositories with custom rate limiting configuration
    pub async fn search_repositories_with_config(
        &self,
        query: &str,
        per_page: Option<u32>,
        page: Option<u32>,
        config: &RateLimitConfig,
    ) -> Result<SearchResponse> {
        if query.is_empty() {
            return Err(AppError::invalid_query(query, "Query cannot be empty"));
        }

        let per_page = per_page.unwrap_or(30).clamp(1, 100);
        let page = page.unwrap_or(1).max(1);

        let url = format!("{}/search/repositories", self.base_url);
        
        let mut attempt = 0;
        let mut backoff_ms = config.initial_backoff_ms;

        loop {
            let response = self
                .client
                .get(&url)
                .header("Authorization", format!("Bearer {}", self.token))
                .header("Accept", "application/vnd.github.v3+json")
                .header("X-GitHub-Api-Version", "2022-11-28")
                .query(&[
                    ("q", query),
                    ("per_page", &per_page.to_string()),
                    ("page", &page.to_string()),
                    ("sort", "updated"),
                    ("order", "desc"),
                ])
                .send()
                .await?;

            match response.status() {
                StatusCode::OK => {
                    let search_response: SearchResponse = response.json().await?;
                    return Ok(search_response);
                }
                StatusCode::FORBIDDEN | StatusCode::TOO_MANY_REQUESTS => {
                    if attempt >= config.max_retries {
                        let reset_time = self.extract_rate_limit_reset(&response).await;
                        return Err(AppError::rate_limit(reset_time));
                    }

                    // Exponential backoff with jitter
                    let jitter = fastrand::u64(0..=backoff_ms / 4);
                    let delay = Duration::from_millis(backoff_ms + jitter);
                    sleep(delay).await;

                    backoff_ms = ((backoff_ms as f64 * config.backoff_multiplier) as u64)
                        .min(config.max_backoff_ms);
                    attempt += 1;
                }
                StatusCode::UNAUTHORIZED => {
                    return Err(AppError::authentication("Invalid or expired GitHub token"));
                }
                StatusCode::UNPROCESSABLE_ENTITY => {
                    let error_body = response.text().await.unwrap_or_default();
                    let reason = self.extract_validation_error(&error_body);
                    return Err(AppError::invalid_query(query, reason));
                }
                status => {
                    let error_body = response.text().await.unwrap_or_default();
                    let message = format!("HTTP {}: {}", status, error_body);
                    return Err(AppError::github_api(message));
                }
            }
        }
    }

    /// Extract rate limit reset time from response headers
    async fn extract_rate_limit_reset(&self, response: &reqwest::Response) -> String {
        if let Some(reset_header) = response.headers().get("x-ratelimit-reset") {
            if let Ok(reset_str) = reset_header.to_str() {
                if let Ok(reset_timestamp) = reset_str.parse::<i64>() {
                    let reset_time = chrono::DateTime::from_timestamp(reset_timestamp, 0)
                        .unwrap_or_else(|| chrono::Utc::now());
                    return reset_time.format("%Y-%m-%d %H:%M:%S UTC").to_string();
                }
            }
        }
        "unknown".to_string()
    }

    /// Extract validation error message from GitHub API error response
    fn extract_validation_error(&self, error_body: &str) -> String {
        if let Ok(error_json) = serde_json::from_str::<Value>(error_body) {
            if let Some(message) = error_json.get("message").and_then(|m| m.as_str()) {
                return message.to_string();
            }
            if let Some(errors) = error_json.get("errors").and_then(|e| e.as_array()) {
                let error_messages: Vec<String> = errors
                    .iter()
                    .filter_map(|e| e.get("message").and_then(|m| m.as_str()))
                    .map(|s| s.to_string())
                    .collect();
                if !error_messages.is_empty() {
                    return error_messages.join(", ");
                }
            }
        }
        "Invalid query format".to_string()
    }

    /// Validate GitHub token by making a test API call
    pub async fn validate_token(&self) -> Result<()> {
        let url = format!("{}/user", self.base_url);
        
        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => Ok(()),
            StatusCode::UNAUTHORIZED => {
                Err(AppError::authentication("Invalid or expired GitHub token"))
            }
            status => {
                let error_body = response.text().await.unwrap_or_default();
                Err(AppError::github_api(format!("Token validation failed: HTTP {}: {}", status, error_body)))
            }
        }
    }

    /// Get current rate limit status
    pub async fn get_rate_limit(&self) -> Result<RateLimitStatus> {
        let url = format!("{}/rate_limit", self.base_url);
        
        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => {
                let rate_limit: RateLimitResponse = response.json().await?;
                Ok(RateLimitStatus {
                    limit: rate_limit.resources.search.limit,
                    remaining: rate_limit.resources.search.remaining,
                    reset_at: chrono::DateTime::from_timestamp(rate_limit.resources.search.reset, 0)
                        .unwrap_or_else(|| chrono::Utc::now()),
                })
            }
            status => {
                let error_body = response.text().await.unwrap_or_default();
                Err(AppError::github_api(format!("Rate limit check failed: HTTP {}: {}", status, error_body)))
            }
        }
    }
}

/// Rate limit status information
#[derive(Debug, Clone)]
pub struct RateLimitStatus {
    pub limit: u32,
    pub remaining: u32,
    pub reset_at: chrono::DateTime<chrono::Utc>,
}

/// GitHub API rate limit response structure
#[derive(Debug, serde::Deserialize)]
struct RateLimitResponse {
    resources: RateLimitResources,
}

#[derive(Debug, serde::Deserialize)]
struct RateLimitResources {
    search: RateLimitInfo,
}

#[derive(Debug, serde::Deserialize)]
struct RateLimitInfo {
    limit: u32,
    remaining: u32,
    reset: i64,
}

