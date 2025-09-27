use github_pg_query::{CliConfig, DatabaseManager, GitHubClient};
use testcontainers::{clients::Cli, images::postgres::Postgres, Container};
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path, query_param, header};
use serde_json::json;
use serial_test::serial;
use std::process::Command;
use tempfile::NamedTempFile;
use std::io::Write;

// Test container setup
fn setup_test_database() -> (Cli, Container<'static, Postgres>) {
    let docker = Cli::default();
    let postgres_image = Postgres::default()
        .with_db_name("test_db")
        .with_user("test_user")
        .with_password("test_password");
    
    let container = docker.run(postgres_image);
    (docker, container)
}

async fn create_database_url(container: &Container<'_, Postgres>) -> String {
    let port = container.get_host_port_ipv4(5432);
    format!("postgresql://test_user:test_password@localhost:{}/test_db", port)
}

#[tokio::test]
#[serial]
async fn test_cli_dry_run_success() {
    let (_docker, container) = setup_test_database();
    let database_url = create_database_url(&container).await;
    
    // Setup mock GitHub server
    let mock_server = MockServer::start().await;
    
    // Mock the user endpoint for token validation
    Mock::given(method("GET"))
        .and(path("/user"))
        .and(header("Authorization", "Bearer test_token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&json!({
            "login": "testuser",
            "id": 12345
        })))
        .mount(&mock_server)
        .await;

    let args = vec![
        "github-pg-query",
        "rust language:rust",
        "--dry-run",
        "--verbose",
        "--github-token", "test_token",
        "--database-url", &database_url
    ];

    let config = CliConfig::parse_from(args).unwrap();
    assert!(config.dry_run);
    assert!(config.verbose);
    
    // Test that we can create the components for dry run
    let github_client = GitHubClient::with_base_url(
        config.github_token.clone(),
        mock_server.uri(),
    ).unwrap();
    
    let db_manager = DatabaseManager::new(&config.database_url).await.unwrap();
    
    // Validate token (should succeed with mock)
    let token_result = github_client.validate_token().await;
    assert!(token_result.is_ok());
    
    // Validate database connection
    let history = db_manager.get_query_history(Some(1), false).await;
    assert!(history.is_ok());
}

#[tokio::test]
#[serial]
async fn test_cli_search_workflow_success() {
    let (_docker, container) = setup_test_database();
    let database_url = create_database_url(&container).await;
    
    // Setup mock GitHub server
    let mock_server = MockServer::start().await;
    
    // Mock search repositories endpoint
    let search_response = json!({
        "total_count": 1,
        "incomplete_results": false,
        "items": [{
            "id": 123456789,
            "full_name": "octocat/Hello-World",
            "name": "Hello-World",
            "description": "This your first repo!",
            "html_url": "https://github.com/octocat/Hello-World",
            "clone_url": "https://github.com/octocat/Hello-World.git",
            "ssh_url": "git@github.com:octocat/Hello-World.git",
            "size": 108,
            "stargazers_count": 80,
            "watchers_count": 9,
            "forks_count": 9,
            "open_issues_count": 0,
            "language": "C",
            "default_branch": "master",
            "visibility": "public",
            "private": false,
            "fork": false,
            "archived": false,
            "disabled": false,
            "created_at": "2011-01-26T19:01:12Z",
            "updated_at": "2011-01-26T19:14:43Z",
            "pushed_at": "2011-01-26T19:06:43Z",
            "owner": {
                "id": 1,
                "login": "octocat",
                "type": "User",
                "avatar_url": "https://github.com/images/error/octocat_happy.gif",
                "html_url": "https://github.com/octocat",
                "site_admin": false
            },
            "license": {
                "key": "mit",
                "name": "MIT License",
                "spdx_id": "MIT",
                "url": "https://api.github.com/licenses/mit"
            },
            "topics": ["octocat", "atom"],
            "has_issues": true,
            "has_projects": true,
            "has_wiki": true,
            "has_pages": false,
            "has_downloads": true
        }]
    });
    
    Mock::given(method("GET"))
        .and(path("/search/repositories"))
        .and(query_param("q", "rust language:rust"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&search_response))
        .mount(&mock_server)
        .await;

    let args = vec![
        "github-pg-query",
        "rust language:rust",
        "--per-page", "30",
        "--page", "1",
        "--verbose",
        "--github-token", "test_token",
        "--database-url", &database_url
    ];

    let config = CliConfig::parse_from(args).unwrap();
    
    // Create GitHub client with mock server
    let github_client = GitHubClient::with_base_url(
        config.github_token.clone(),
        mock_server.uri(),
    ).unwrap();
    
    // Create database manager
    let db_manager = DatabaseManager::new(&config.database_url).await.unwrap();
    
    // Execute search
    let search_result = github_client.search_repositories(
        &config.search_query,
        Some(config.per_page),
        Some(config.page),
    ).await;
    
    assert!(search_result.is_ok());
    let search_response = search_result.unwrap();
    assert_eq!(search_response.total_count, 1);
    assert_eq!(search_response.items.len(), 1);
    
    // Create table and insert results
    let table_name = DatabaseManager::generate_table_name();
    db_manager.create_repository_table(&table_name).await.unwrap();
    
    let inserted_count = db_manager.insert_repositories(&table_name, &search_response.items).await.unwrap();
    assert_eq!(inserted_count, 1);
    
    // Verify data was inserted
    let stats = db_manager.get_table_stats(&table_name).await.unwrap();
    assert_eq!(stats.total_repositories, 1);
    
    // Cleanup
    db_manager.drop_table(&table_name).await.unwrap();
}

#[tokio::test]
#[serial]
async fn test_cli_error_handling_invalid_token() {
    let (_docker, container) = setup_test_database();
    let database_url = create_database_url(&container).await;
    
    // Setup mock GitHub server that returns 401
    let mock_server = MockServer::start().await;
    
    Mock::given(method("GET"))
        .and(path("/search/repositories"))
        .respond_with(ResponseTemplate::new(401).set_body_json(&json!({
            "message": "Bad credentials"
        })))
        .mount(&mock_server)
        .await;

    let args = vec![
        "github-pg-query",
        "rust language:rust",
        "--github-token", "invalid_token",
        "--database-url", &database_url
    ];

    let config = CliConfig::parse_from(args).unwrap();
    
    let github_client = GitHubClient::with_base_url(
        config.github_token.clone(),
        mock_server.uri(),
    ).unwrap();
    
    let search_result = github_client.search_repositories(
        &config.search_query,
        Some(config.per_page),
        Some(config.page),
    ).await;
    
    assert!(search_result.is_err());
}

#[tokio::test]
#[serial]
async fn test_cli_error_handling_database_connection() {
    let invalid_database_url = "postgresql://invalid:invalid@nonexistent:5432/invalid";
    
    let args = vec![
        "github-pg-query",
        "rust language:rust",
        "--github-token", "test_token",
        "--database-url", invalid_database_url
    ];

    let config = CliConfig::parse_from(args).unwrap();
    
    let db_result = DatabaseManager::new(&config.database_url).await;
    assert!(db_result.is_err());
}

#[tokio::test]
#[serial]
async fn test_cli_rate_limit_handling() {
    let (_docker, container) = setup_test_database();
    let database_url = create_database_url(&container).await;
    
    // Setup mock GitHub server that returns rate limit error
    let mock_server = MockServer::start().await;
    
    Mock::given(method("GET"))
        .and(path("/search/repositories"))
        .respond_with(
            ResponseTemplate::new(403)
                .set_header("x-ratelimit-reset", "1640995200")
                .set_body_json(&json!({
                    "message": "API rate limit exceeded"
                }))
        )
        .mount(&mock_server)
        .await;

    let args = vec![
        "github-pg-query",
        "rust language:rust",
        "--github-token", "test_token",
        "--database-url", &database_url
    ];

    let config = CliConfig::parse_from(args).unwrap();
    
    let github_client = GitHubClient::with_base_url(
        config.github_token.clone(),
        mock_server.uri(),
    ).unwrap();
    
    let search_result = github_client.search_repositories(
        &config.search_query,
        Some(config.per_page),
        Some(config.page),
    ).await;
    
    assert!(search_result.is_err());
}

#[tokio::test]
#[serial]
async fn test_cli_pagination_parameters() {
    let (_docker, container) = setup_test_database();
    let database_url = create_database_url(&container).await;
    
    // Setup mock GitHub server
    let mock_server = MockServer::start().await;
    
    Mock::given(method("GET"))
        .and(path("/search/repositories"))
        .and(query_param("per_page", "50"))
        .and(query_param("page", "2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&json!({
            "total_count": 0,
            "incomplete_results": false,
            "items": []
        })))
        .mount(&mock_server)
        .await;

    let args = vec![
        "github-pg-query",
        "rust language:rust",
        "--per-page", "50",
        "--page", "2",
        "--github-token", "test_token",
        "--database-url", &database_url
    ];

    let config = CliConfig::parse_from(args).unwrap();
    assert_eq!(config.per_page, 50);
    assert_eq!(config.page, 2);
    
    let github_client = GitHubClient::with_base_url(
        config.github_token.clone(),
        mock_server.uri(),
    ).unwrap();
    
    let search_result = github_client.search_repositories(
        &config.search_query,
        Some(config.per_page),
        Some(config.page),
    ).await;
    
    assert!(search_result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_cli_environment_variable_usage() {
    let (_docker, container) = setup_test_database();
    let database_url = create_database_url(&container).await;
    
    // Create temporary .env file
    let mut env_file = NamedTempFile::new().unwrap();
    writeln!(env_file, "GITHUB_TOKEN=env_test_token").unwrap();
    writeln!(env_file, "DATABASE_URL={}", database_url).unwrap();
    env_file.flush().unwrap();
    
    // Test that environment variables can be used
    std::env::set_var("GITHUB_TOKEN", "env_test_token");
    std::env::set_var("DATABASE_URL", &database_url);
    
    let args = vec![
        "github-pg-query",
        "rust language:rust"
    ];

    let config = CliConfig::parse_from(args).unwrap();
    assert_eq!(config.github_token, "env_test_token");
    assert_eq!(config.database_url, database_url);
    
    // Cleanup
    std::env::remove_var("GITHUB_TOKEN");
    std::env::remove_var("DATABASE_URL");
}

#[tokio::test]
#[serial]
async fn test_cli_query_validation() {
    let (_docker, container) = setup_test_database();
    let database_url = create_database_url(&container).await;
    
    // Test empty query
    let args = vec![
        "github-pg-query",
        "",
        "--github-token", "test_token",
        "--database-url", &database_url
    ];

    let result = CliConfig::parse_from(args);
    assert!(result.is_err());
    
    // Test very long query
    let long_query = "a".repeat(300);
    let args = vec![
        "github-pg-query",
        &long_query,
        "--github-token", "test_token",
        "--database-url", &database_url
    ];

    let result = CliConfig::parse_from(args);
    assert!(result.is_err());
}

#[tokio::test]
#[serial]
async fn test_cli_complete_workflow_with_metadata() {
    let (_docker, container) = setup_test_database();
    let database_url = create_database_url(&container).await;
    
    // Setup mock GitHub server
    let mock_server = MockServer::start().await;
    
    let search_response = json!({
        "total_count": 2,
        "incomplete_results": false,
        "items": [
            {
                "id": 123456789,
                "full_name": "rust-lang/rust",
                "name": "rust",
                "description": "Empowering everyone to build reliable and efficient software.",
                "html_url": "https://github.com/rust-lang/rust",
                "clone_url": "https://github.com/rust-lang/rust.git",
                "ssh_url": "git@github.com:rust-lang/rust.git",
                "size": 200000,
                "stargazers_count": 80000,
                "watchers_count": 80000,
                "forks_count": 10000,
                "open_issues_count": 9000,
                "language": "Rust",
                "default_branch": "master",
                "visibility": "public",
                "private": false,
                "fork": false,
                "archived": false,
                "disabled": false,
                "created_at": "2010-06-16T21:30:36Z",
                "updated_at": "2023-12-01T12:00:00Z",
                "pushed_at": "2023-12-01T11:30:00Z",
                "owner": {
                    "id": 5430905,
                    "login": "rust-lang",
                    "type": "Organization",
                    "avatar_url": "https://avatars.githubusercontent.com/u/5430905?v=4",
                    "html_url": "https://github.com/rust-lang",
                    "site_admin": false
                },
                "license": {
                    "key": "apache-2.0",
                    "name": "Apache License 2.0",
                    "spdx_id": "Apache-2.0",
                    "url": "https://api.github.com/licenses/apache-2.0"
                },
                "topics": ["rust", "compiler", "programming-language"],
                "has_issues": true,
                "has_projects": true,
                "has_wiki": false,
                "has_pages": false,
                "has_downloads": true
            },
            {
                "id": 987654321,
                "full_name": "tokio-rs/tokio",
                "name": "tokio",
                "description": "A runtime for writing reliable asynchronous applications with Rust.",
                "html_url": "https://github.com/tokio-rs/tokio",
                "clone_url": "https://github.com/tokio-rs/tokio.git",
                "ssh_url": "git@github.com:tokio-rs/tokio.git",
                "size": 50000,
                "stargazers_count": 20000,
                "watchers_count": 20000,
                "forks_count": 3000,
                "open_issues_count": 200,
                "language": "Rust",
                "default_branch": "master",
                "visibility": "public",
                "private": false,
                "fork": false,
                "archived": false,
                "disabled": false,
                "created_at": "2016-02-16T21:30:36Z",
                "updated_at": "2023-12-01T12:00:00Z",
                "pushed_at": "2023-12-01T11:30:00Z",
                "owner": {
                    "id": 6180040,
                    "login": "tokio-rs",
                    "type": "Organization",
                    "avatar_url": "https://avatars.githubusercontent.com/u/6180040?v=4",
                    "html_url": "https://github.com/tokio-rs",
                    "site_admin": false
                },
                "license": {
                    "key": "mit",
                    "name": "MIT License",
                    "spdx_id": "MIT",
                    "url": "https://api.github.com/licenses/mit"
                },
                "topics": ["rust", "async", "tokio"],
                "has_issues": true,
                "has_projects": true,
                "has_wiki": true,
                "has_pages": false,
                "has_downloads": true
            }
        ]
    });
    
    Mock::given(method("GET"))
        .and(path("/search/repositories"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&search_response))
        .mount(&mock_server)
        .await;

    let args = vec![
        "github-pg-query",
        "rust language:rust stars:>1000",
        "--verbose",
        "--github-token", "test_token",
        "--database-url", &database_url
    ];

    let config = CliConfig::parse_from(args).unwrap();
    
    // Execute complete workflow
    let github_client = GitHubClient::with_base_url(
        config.github_token.clone(),
        mock_server.uri(),
    ).unwrap();
    
    let db_manager = DatabaseManager::new(&config.database_url).await.unwrap();
    
    // Generate table name and create table
    let table_name = DatabaseManager::generate_table_name();
    db_manager.create_repository_table(&table_name).await.unwrap();
    
    // Execute search
    let search_result = github_client.search_repositories(
        &config.search_query,
        Some(config.per_page),
        Some(config.page),
    ).await.unwrap();
    
    // Insert repositories
    let inserted_count = db_manager.insert_repositories(&table_name, &search_result.items).await.unwrap();
    assert_eq!(inserted_count, 2);
    
    // Create and save query metadata
    let mut query_metadata = github_pg_query::QueryMetadata::new(
        config.search_query.clone(),
        table_name.clone(),
    );
    query_metadata.mark_success(search_result.items.len() as i64, 1500);
    
    db_manager.save_query_metadata(&query_metadata).await.unwrap();
    
    // Verify results
    let stats = db_manager.get_table_stats(&table_name).await.unwrap();
    assert_eq!(stats.total_repositories, 2);
    assert_eq!(stats.unique_languages, 1); // Both Rust
    assert_eq!(stats.unique_owners, 2); // rust-lang and tokio-rs
    
    // Verify query history
    let history = db_manager.get_query_history(Some(10), false).await.unwrap();
    let our_query = history.iter().find(|h| h.id == query_metadata.id).unwrap();
    assert_eq!(our_query.search_query, config.search_query);
    assert_eq!(our_query.result_count, 2);
    assert!(our_query.success);
    
    // Cleanup
    db_manager.drop_table(&table_name).await.unwrap();
}

#[test]
fn test_cli_binary_execution() {
    // Test that the binary can be executed (will fail due to missing args, but should not crash)
    let output = Command::new("cargo")
        .args(&["run", "--", "--help"])
        .output();
    
    // Should succeed in showing help
    assert!(output.is_ok());
    let output = output.unwrap();
    assert!(output.status.success() || output.status.code() == Some(0) || output.status.code() == Some(2)); // Help exits with code 0 or 2
}

#[test]
fn test_cli_version_output() {
    let output = Command::new("cargo")
        .args(&["run", "--", "--version"])
        .output();
    
    assert!(output.is_ok());
    let output = output.unwrap();
    // Version command should exit successfully or with code 0
    assert!(output.status.success() || output.status.code() == Some(0));
}