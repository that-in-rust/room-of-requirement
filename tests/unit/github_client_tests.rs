use github_pg_query::{GitHubClient, RateLimitConfig, AppError, SearchResponse, Repository, RepositoryOwner, RepositoryLicense};
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path, query_param, header};
use serde_json::json;
use proptest::prelude::*;

#[tokio::test]
async fn test_github_client_creation_success() {
    let client = GitHubClient::new("test_token_123".to_string());
    assert!(client.is_ok());
    
    let client = client.unwrap();
    // We can't access private fields directly, but we can test behavior
    assert!(client.validate_token().await.is_err()); // Will fail with real API, but tests creation
}

#[tokio::test]
async fn test_github_client_creation_empty_token() {
    let result = GitHubClient::new("".to_string());
    assert!(result.is_err());
    
    match result.unwrap_err() {
        AppError::Authentication { reason } => {
            assert!(reason.contains("token cannot be empty"));
        }
        _ => panic!("Expected Authentication error"),
    }
}

#[tokio::test]
async fn test_search_repositories_success() {
    let mock_server = MockServer::start().await;
    
    // Create test repository data
    let test_repo = create_test_repository();
    let search_response = SearchResponse {
        total_count: 1,
        incomplete_results: false,
        items: vec![test_repo.clone()],
    };
    
    // Setup mock response
    Mock::given(method("GET"))
        .and(path("/search/repositories"))
        .and(query_param("q", "rust language:rust"))
        .and(header("Authorization", "Bearer test_token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&search_response))
        .mount(&mock_server)
        .await;
    
    // Create client with mock server URL
    let client = GitHubClient::with_base_url(
        "test_token".to_string(),
        mock_server.uri(),
    ).unwrap();
    
    // Test search
    let result = client.search_repositories("rust language:rust", Some(30), Some(1)).await;
    assert!(result.is_ok());
    
    let response = result.unwrap();
    assert_eq!(response.total_count, 1);
    assert!(!response.incomplete_results);
    assert_eq!(response.items.len(), 1);
    assert_eq!(response.items[0], test_repo);
}

#[tokio::test]
async fn test_search_repositories_empty_query() {
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

#[tokio::test]
async fn test_search_repositories_rate_limit() {
    let mock_server = MockServer::start().await;
    
    // Setup rate limit response
    Mock::given(method("GET"))
        .and(path("/search/repositories"))
        .respond_with(
            ResponseTemplate::new(403)
                .set_header("x-ratelimit-reset", "1640995200")
                .set_body_json(&json!({
                    "message": "API rate limit exceeded",
                    "documentation_url": "https://docs.github.com/rest/overview/resources-in-the-rest-api#rate-limiting"
                }))
        )
        .mount(&mock_server)
        .await;
    
    let client = GitHubClient::with_base_url(
        "test_token".to_string(),
        mock_server.uri(),
    ).unwrap();
    
    // Use config with no retries for faster test
    let config = RateLimitConfig {
        max_retries: 0,
        initial_backoff_ms: 10,
        max_backoff_ms: 100,
        backoff_multiplier: 2.0,
    };
    
    let result = client.search_repositories_with_config(
        "test query", 
        Some(30), 
        Some(1), 
        &config
    ).await;
    
    assert!(result.is_err());
    match result.unwrap_err() {
        AppError::RateLimit { reset_time } => {
            assert!(!reset_time.is_empty());
        }
        _ => panic!("Expected RateLimit error"),
    }
}

#[tokio::test]
async fn test_search_repositories_authentication_error() {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("GET"))
        .and(path("/search/repositories"))
        .respond_with(
            ResponseTemplate::new(401)
                .set_body_json(&json!({
                    "message": "Bad credentials",
                    "documentation_url": "https://docs.github.com/rest"
                }))
        )
        .mount(&mock_server)
        .await;
    
    let client = GitHubClient::with_base_url(
        "invalid_token".to_string(),
        mock_server.uri(),
    ).unwrap();
    
    let result = client.search_repositories("test query", Some(30), Some(1)).await;
    assert!(result.is_err());
    
    match result.unwrap_err() {
        AppError::Authentication { reason } => {
            assert!(reason.contains("Invalid or expired GitHub token"));
        }
        _ => panic!("Expected Authentication error"),
    }
}

#[tokio::test]
async fn test_search_repositories_validation_error() {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("GET"))
        .and(path("/search/repositories"))
        .respond_with(
            ResponseTemplate::new(422)
                .set_body_json(&json!({
                    "message": "Validation Failed",
                    "errors": [
                        {
                            "message": "The search is longer than 256 characters.",
                            "code": "invalid"
                        }
                    ],
                    "documentation_url": "https://docs.github.com/v3/search/"
                }))
        )
        .mount(&mock_server)
        .await;
    
    let client = GitHubClient::with_base_url(
        "test_token".to_string(),
        mock_server.uri(),
    ).unwrap();
    
    let long_query = "a".repeat(300);
    let result = client.search_repositories(&long_query, Some(30), Some(1)).await;
    assert!(result.is_err());
    
    match result.unwrap_err() {
        AppError::InvalidQuery { query, reason } => {
            assert_eq!(query, long_query);
            assert!(reason.contains("The search is longer than 256 characters"));
        }
        _ => panic!("Expected InvalidQuery error"),
    }
}

#[tokio::test]
async fn test_search_repositories_with_retry_success() {
    let mock_server = MockServer::start().await;
    
    let test_repo = create_test_repository();
    let search_response = SearchResponse {
        total_count: 1,
        incomplete_results: false,
        items: vec![test_repo.clone()],
    };
    
    // First request fails with rate limit
    Mock::given(method("GET"))
        .and(path("/search/repositories"))
        .respond_with(
            ResponseTemplate::new(429)
                .set_header("x-ratelimit-reset", "1640995200")
        )
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;
    
    // Second request succeeds
    Mock::given(method("GET"))
        .and(path("/search/repositories"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&search_response))
        .mount(&mock_server)
        .await;
    
    let client = GitHubClient::with_base_url(
        "test_token".to_string(),
        mock_server.uri(),
    ).unwrap();
    
    let config = RateLimitConfig {
        max_retries: 2,
        initial_backoff_ms: 10,
        max_backoff_ms: 100,
        backoff_multiplier: 2.0,
    };
    
    let result = client.search_repositories_with_config(
        "test query", 
        Some(30), 
        Some(1), 
        &config
    ).await;
    
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.items.len(), 1);
}

#[tokio::test]
async fn test_validate_token_success() {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("GET"))
        .and(path("/user"))
        .and(header("Authorization", "Bearer valid_token"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(&json!({
                    "login": "testuser",
                    "id": 12345
                }))
        )
        .mount(&mock_server)
        .await;
    
    let client = GitHubClient::with_base_url(
        "valid_token".to_string(),
        mock_server.uri(),
    ).unwrap();
    
    let result = client.validate_token().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_validate_token_failure() {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("GET"))
        .and(path("/user"))
        .respond_with(
            ResponseTemplate::new(401)
                .set_body_json(&json!({
                    "message": "Bad credentials"
                }))
        )
        .mount(&mock_server)
        .await;
    
    let client = GitHubClient::with_base_url(
        "invalid_token".to_string(),
        mock_server.uri(),
    ).unwrap();
    
    let result = client.validate_token().await;
    assert!(result.is_err());
    
    match result.unwrap_err() {
        AppError::Authentication { reason } => {
            assert!(reason.contains("Invalid or expired GitHub token"));
        }
        _ => panic!("Expected Authentication error"),
    }
}

#[tokio::test]
async fn test_get_rate_limit_success() {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("GET"))
        .and(path("/rate_limit"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(&json!({
                    "resources": {
                        "search": {
                            "limit": 30,
                            "remaining": 25,
                            "reset": 1640995200
                        }
                    }
                }))
        )
        .mount(&mock_server)
        .await;
    
    let client = GitHubClient::with_base_url(
        "test_token".to_string(),
        mock_server.uri(),
    ).unwrap();
    
    let result = client.get_rate_limit().await;
    assert!(result.is_ok());
    
    let rate_limit = result.unwrap();
    assert_eq!(rate_limit.limit, 30);
    assert_eq!(rate_limit.remaining, 25);
}

#[test]
fn test_rate_limit_config_default() {
    let config = RateLimitConfig::default();
    
    assert_eq!(config.max_retries, 3);
    assert_eq!(config.initial_backoff_ms, 1000);
    assert_eq!(config.max_backoff_ms, 60000);
    assert_eq!(config.backoff_multiplier, 2.0);
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

// Property-based tests
proptest! {
    #[test]
    fn test_per_page_clamping_invariants(per_page in 0u32..200) {
        let clamped = Some(per_page).unwrap_or(30).clamp(1, 100);
        prop_assert!(clamped >= 1 && clamped <= 100);
    }

    #[test]
    fn test_page_validation_invariants(page in 0u32..1000) {
        let validated = Some(page).unwrap_or(1).max(1);
        prop_assert!(validated >= 1);
    }

    #[test]
    fn test_backoff_calculation_invariants(
        initial_backoff in 100u64..10000,
        multiplier in 1.1f64..5.0,
        max_backoff in 10000u64..100000
    ) {
        let config = RateLimitConfig {
            max_retries: 3,
            initial_backoff_ms: initial_backoff,
            max_backoff_ms: max_backoff,
            backoff_multiplier: multiplier,
        };

        let mut backoff = config.initial_backoff_ms;
        let mut previous = 0;

        // Test monotonic increase until cap
        for _ in 0..10 {
            prop_assert!(backoff >= previous);
            prop_assert!(backoff <= config.max_backoff_ms);
            
            previous = backoff;
            backoff = ((backoff as f64 * config.backoff_multiplier) as u64)
                .min(config.max_backoff_ms);
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