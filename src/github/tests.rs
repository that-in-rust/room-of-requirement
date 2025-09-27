use super::*;
use crate::models::{Repository, RepositoryOwner, RepositoryLicense, SearchResponse};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Mock HTTP server for testing GitHub API interactions
pub struct MockGitHubServer {
    responses: Arc<Mutex<Vec<MockResponse>>>,
    request_log: Arc<Mutex<Vec<MockRequest>>>,
}

#[derive(Debug, Clone)]
pub struct MockResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

#[derive(Debug, Clone)]
pub struct MockRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub query_params: Vec<(String, String)>,
}

impl MockGitHubServer {
    pub fn new() -> Self {
        Self {
            responses: Arc::new(Mutex::new(Vec::new())),
            request_log: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn add_response(&self, response: MockResponse) {
        self.responses.lock().await.push(response);
    }

    pub async fn get_requests(&self) -> Vec<MockRequest> {
        self.request_log.lock().await.clone()
    }

    pub async fn clear(&self) {
        self.responses.lock().await.clear();
        self.request_log.lock().await.clear();
    }

    // In a real implementation, this would start an HTTP server
    // For unit tests, we'll simulate the responses
    pub fn create_success_response(repositories: Vec<Repository>) -> MockResponse {
        let search_response = SearchResponse {
            total_count: repositories.len() as i64,
            incomplete_results: false,
            items: repositories,
        };

        MockResponse {
            status: 200,
            headers: vec![
                ("content-type".to_string(), "application/json".to_string()),
                ("x-ratelimit-limit".to_string(), "30".to_string()),
                ("x-ratelimit-remaining".to_string(), "29".to_string()),
                ("x-ratelimit-reset".to_string(), "1640995200".to_string()),
            ],
            body: serde_json::to_string(&search_response).unwrap(),
        }
    }

    pub fn create_rate_limit_response() -> MockResponse {
        MockResponse {
            status: 403,
            headers: vec![
                ("x-ratelimit-limit".to_string(), "30".to_string()),
                ("x-ratelimit-remaining".to_string(), "0".to_string()),
                ("x-ratelimit-reset".to_string(), "1640995200".to_string()),
            ],
            body: json!({
                "message": "API rate limit exceeded",
                "documentation_url": "https://docs.github.com/rest/overview/resources-in-the-rest-api#rate-limiting"
            }).to_string(),
        }
    }

    pub fn create_auth_error_response() -> MockResponse {
        MockResponse {
            status: 401,
            headers: vec![],
            body: json!({
                "message": "Bad credentials",
                "documentation_url": "https://docs.github.com/rest"
            }).to_string(),
        }
    }

    pub fn create_validation_error_response() -> MockResponse {
        MockResponse {
            status: 422,
            headers: vec![],
            body: json!({
                "message": "Validation Failed",
                "errors": [
                    {
                        "message": "The search is longer than 256 characters.",
                        "code": "invalid"
                    }
                ],
                "documentation_url": "https://docs.github.com/v3/search/"
            }).to_string(),
        }
    }
}

fn create_test_repository() -> Repository {
    Repository {
        id: 123456789,
        full_name: "octocat/Hello-World".to_string(),
        name: "Hello-World".to_string(),
        description: Some("This your first repo!".to_string()),
        html_url: "https://github.com/octocat/Hello-World".to_string(),
        clone_url: "https://github.com/octocat/Hello-World.git".to_string(),
        ssh_url: "git@github.com:octocat/Hello-World.git".to_string(),
        size: 108,
        stargazers_count: 80,
        watchers_count: 9,
        forks_count: 9,
        open_issues_count: 0,
        language: Some("C".to_string()),
        default_branch: "master".to_string(),
        visibility: "public".to_string(),
        private: false,
        fork: false,
        archived: false,
        disabled: false,
        created_at: "2011-01-26T19:01:12Z".parse().unwrap(),
        updated_at: "2011-01-26T19:14:43Z".parse().unwrap(),
        pushed_at: Some("2011-01-26T19:06:43Z".parse().unwrap()),
        owner: RepositoryOwner {
            id: 1,
            login: "octocat".to_string(),
            owner_type: "User".to_string(),
            avatar_url: "https://github.com/images/error/octocat_happy.gif".to_string(),
            html_url: "https://github.com/octocat".to_string(),
            site_admin: false,
        },
        license: Some(RepositoryLicense {
            key: "mit".to_string(),
            name: "MIT License".to_string(),
            spdx_id: Some("MIT".to_string()),
            url: Some("https://api.github.com/licenses/mit".to_string()),
        }),
        topics: vec!["octocat".to_string(), "atom".to_string()],
        has_issues: true,
        has_projects: true,
        has_wiki: true,
        has_pages: false,
        has_downloads: true,
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_mock_server_creation() {
        let _server = MockGitHubServer::new();
        // Just test that we can create the server without panicking
    }

    #[test]
    fn test_create_success_response() {
        let repo = create_test_repository();
        let response = MockGitHubServer::create_success_response(vec![repo.clone()]);
        
        assert_eq!(response.status, 200);
        assert!(response.headers.iter().any(|(k, v)| k == "content-type" && v == "application/json"));
        
        // Verify the response body can be deserialized
        let search_response: SearchResponse = serde_json::from_str(&response.body).unwrap();
        assert_eq!(search_response.total_count, 1);
        assert_eq!(search_response.items.len(), 1);
        assert_eq!(search_response.items[0], repo);
    }

    #[test]
    fn test_create_rate_limit_response() {
        let response = MockGitHubServer::create_rate_limit_response();
        
        assert_eq!(response.status, 403);
        assert!(response.headers.iter().any(|(k, v)| k == "x-ratelimit-remaining" && v == "0"));
        
        let body_json: serde_json::Value = serde_json::from_str(&response.body).unwrap();
        assert_eq!(body_json["message"], "API rate limit exceeded");
    }

    #[test]
    fn test_create_auth_error_response() {
        let response = MockGitHubServer::create_auth_error_response();
        
        assert_eq!(response.status, 401);
        
        let body_json: serde_json::Value = serde_json::from_str(&response.body).unwrap();
        assert_eq!(body_json["message"], "Bad credentials");
    }

    #[test]
    fn test_create_validation_error_response() {
        let response = MockGitHubServer::create_validation_error_response();
        
        assert_eq!(response.status, 422);
        
        let body_json: serde_json::Value = serde_json::from_str(&response.body).unwrap();
        assert_eq!(body_json["message"], "Validation Failed");
        assert!(body_json["errors"].is_array());
    }

    #[tokio::test]
    async fn test_mock_server_add_response() {
        let server = MockGitHubServer::new();
        let response = MockGitHubServer::create_success_response(vec![]);
        
        server.add_response(response.clone()).await;
        
        let responses = server.responses.lock().await;
        assert_eq!(responses.len(), 1);
        assert_eq!(responses[0].status, response.status);
    }

    #[tokio::test]
    async fn test_mock_server_clear() {
        let server = MockGitHubServer::new();
        let response = MockGitHubServer::create_success_response(vec![]);
        
        server.add_response(response).await;
        assert_eq!(server.responses.lock().await.len(), 1);
        
        server.clear().await;
        assert_eq!(server.responses.lock().await.len(), 0);
        assert_eq!(server.request_log.lock().await.len(), 0);
    }
}

// Integration-style tests that test the actual GitHubClient logic
// These tests focus on the client's behavior rather than HTTP interactions
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_github_client_creation() {
        let client = GitHubClient::new("test_token".to_string());
        assert!(client.is_ok());
        
        let client = client.unwrap();
        assert_eq!(client.token, "test_token");
        assert_eq!(client.base_url, "https://api.github.com");
    }

    #[test]
    fn test_github_client_empty_token_error() {
        let result = GitHubClient::new("".to_string());
        assert!(result.is_err());
        
        match result.unwrap_err() {
            AppError::Authentication { reason } => {
                assert!(reason.contains("token cannot be empty"));
            }
            _ => panic!("Expected Authentication error"),
        }
    }

    #[test]
    fn test_github_client_with_custom_base_url() {
        let client = GitHubClient::with_base_url(
            "test_token".to_string(),
            "https://api.example.com".to_string(),
        );
        assert!(client.is_ok());
        
        let client = client.unwrap();
        assert_eq!(client.base_url, "https://api.example.com");
    }

    #[tokio::test]
    async fn test_search_repositories_empty_query_error() {
        let client = GitHubClient::new("test_token".to_string()).unwrap();
        let result = client.search_repositories("", None, None).await;
        
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::InvalidQuery { query, reason } => {
                assert_eq!(query, "");
                assert!(reason.contains("Query cannot be empty"));
            }
            _ => panic!("Expected InvalidQuery error"),
        }
    }

    #[test]
    fn test_rate_limit_config_default_values() {
        let config = RateLimitConfig::default();
        
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.initial_backoff_ms, 1000);
        assert_eq!(config.max_backoff_ms, 60000);
        assert_eq!(config.backoff_multiplier, 2.0);
    }

    #[test]
    fn test_extract_validation_error_parsing() {
        let client = GitHubClient::new("test_token".to_string()).unwrap();
        
        // Test with message field
        let error_with_message = json!({
            "message": "Validation Failed",
            "errors": [{"message": "Invalid syntax"}]
        }).to_string();
        
        let result = client.extract_validation_error(&error_with_message);
        assert_eq!(result, "Validation Failed");
        
        // Test with errors array only
        let error_with_errors = json!({
            "errors": [
                {"message": "Error 1"},
                {"message": "Error 2"}
            ]
        }).to_string();
        
        let result = client.extract_validation_error(&error_with_errors);
        assert_eq!(result, "Error 1, Error 2");
        
        // Test with invalid JSON
        let result = client.extract_validation_error("invalid json");
        assert_eq!(result, "Invalid query format");
    }

    #[test]
    fn test_parameter_validation() {
        // Test per_page clamping
        let test_cases = vec![
            (Some(0), 1),      // Below minimum
            (Some(1), 1),      // At minimum
            (Some(50), 50),    // Valid value
            (Some(100), 100),  // At maximum
            (Some(150), 100),  // Above maximum
            (None, 30),        // Default
        ];

        for (input, expected) in test_cases {
            let per_page = input.unwrap_or(30).clamp(1, 100);
            assert_eq!(per_page, expected, "per_page clamping failed for {:?}", input);
        }

        // Test page minimum
        let page_cases = vec![
            (Some(0), 1),   // Below minimum
            (Some(1), 1),   // At minimum
            (Some(5), 5),   // Valid value
            (None, 1),      // Default
        ];

        for (input, expected) in page_cases {
            let page = input.unwrap_or(1).max(1);
            assert_eq!(page, expected, "page validation failed for {:?}", input);
        }
    }

    #[test]
    fn test_rate_limit_status_creation() {
        let status = RateLimitStatus {
            limit: 30,
            remaining: 25,
            reset_at: chrono::Utc::now(),
        };

        assert_eq!(status.limit, 30);
        assert_eq!(status.remaining, 25);
        assert!(status.reset_at <= chrono::Utc::now());
    }

    #[test]
    fn test_backoff_calculation() {
        let config = RateLimitConfig::default();
        let mut backoff_ms = config.initial_backoff_ms;

        // Test exponential backoff progression
        let expected_progression = vec![1000, 2000, 4000, 8000];
        
        for expected in expected_progression {
            assert_eq!(backoff_ms, expected);
            backoff_ms = ((backoff_ms as f64 * config.backoff_multiplier) as u64)
                .min(config.max_backoff_ms);
        }

        // Test that it caps at max_backoff_ms
        for _ in 0..10 {
            backoff_ms = ((backoff_ms as f64 * config.backoff_multiplier) as u64)
                .min(config.max_backoff_ms);
            assert!(backoff_ms <= config.max_backoff_ms);
        }
    }
}

// Property-based tests for invariants
#[cfg(test)]
mod property_tests {
    use super::*;

    #[test]
    fn test_per_page_always_in_valid_range() {
        // Test that per_page clamping always produces valid values
        for input in 0..200u32 {
            let clamped = Some(input).unwrap_or(30).clamp(1, 100);
            assert!(clamped >= 1 && clamped <= 100, "Invalid per_page: {}", clamped);
        }
    }

    #[test]
    fn test_page_always_positive() {
        // Test that page validation always produces positive values
        for input in 0..100u32 {
            let page = Some(input).unwrap_or(1).max(1);
            assert!(page >= 1, "Invalid page: {}", page);
        }
    }

    #[test]
    fn test_backoff_progression_monotonic() {
        let config = RateLimitConfig::default();
        let mut backoff_ms = config.initial_backoff_ms;
        let mut previous = 0;

        // Test that backoff always increases (until cap)
        for _ in 0..10 {
            assert!(backoff_ms >= previous, "Backoff decreased: {} -> {}", previous, backoff_ms);
            previous = backoff_ms;
            backoff_ms = ((backoff_ms as f64 * config.backoff_multiplier) as u64)
                .min(config.max_backoff_ms);
        }
    }

    #[test]
    fn test_token_validation_invariants() {
        // Empty token should always fail
        assert!(GitHubClient::new("".to_string()).is_err());
        
        // Non-empty token should always succeed in creation
        assert!(GitHubClient::new("test".to_string()).is_ok());
        assert!(GitHubClient::new("a".to_string()).is_ok());
        assert!(GitHubClient::new("very_long_token_string_that_might_be_realistic".to_string()).is_ok());
    }
}