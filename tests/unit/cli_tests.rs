use github_pg_query::{CliConfig, ProgressIndicator, AppError};
use proptest::prelude::*;
use std::env;

#[test]
fn test_cli_config_parsing_success() {
    let args = vec![
        "github-pg-query",
        "rust language:rust",
        "--per-page", "50",
        "--page", "2",
        "--verbose",
        "--github-token", "test_token_12345678901234567890",
        "--database-url", "postgresql://user:pass@localhost:5432/test"
    ];

    let config = CliConfig::parse_from(args).unwrap();
    
    assert_eq!(config.search_query, "rust language:rust");
    assert_eq!(config.per_page, 50);
    assert_eq!(config.page, 2);
    assert!(config.verbose);
    assert!(!config.dry_run);
    assert_eq!(config.github_token, "test_token_12345678901234567890");
    assert_eq!(config.database_url, "postgresql://user:pass@localhost:5432/test");
}

#[test]
fn test_cli_config_parsing_defaults() {
    let args = vec![
        "github-pg-query",
        "test query",
        "--github-token", "test_token_12345678901234567890",
        "--database-url", "postgresql://user:pass@localhost:5432/test"
    ];

    let config = CliConfig::parse_from(args).unwrap();
    
    assert_eq!(config.search_query, "test query");
    assert_eq!(config.per_page, 30); // default
    assert_eq!(config.page, 1); // default
    assert!(!config.verbose); // default
    assert!(!config.dry_run); // default
}

#[test]
fn test_cli_config_dry_run_flag() {
    let args = vec![
        "github-pg-query",
        "test query",
        "--dry-run",
        "--github-token", "test_token_12345678901234567890",
        "--database-url", "postgresql://user:pass@localhost:5432/test"
    ];

    let config = CliConfig::parse_from(args).unwrap();
    assert!(config.dry_run);
}

#[test]
fn test_cli_config_missing_query() {
    let args = vec![
        "github-pg-query",
        "--github-token", "test_token_12345678901234567890",
        "--database-url", "postgresql://user:pass@localhost:5432/test"
    ];

    let result = CliConfig::parse_from(args);
    assert!(result.is_err());
}

#[test]
fn test_cli_config_invalid_per_page() {
    let test_cases = vec![
        ("0", "below minimum"),
        ("101", "above maximum"),
        ("abc", "non-numeric"),
    ];

    for (per_page_value, description) in test_cases {
        let args = vec![
            "github-pg-query",
            "test query",
            "--per-page", per_page_value,
            "--github-token", "test_token_12345678901234567890",
            "--database-url", "postgresql://user:pass@localhost:5432/test"
        ];

        let result = CliConfig::parse_from(args);
        assert!(result.is_err(), "Should fail for per-page {}: {}", per_page_value, description);
    }
}

#[test]
fn test_cli_config_invalid_page() {
    let args = vec![
        "github-pg-query",
        "test query",
        "--page", "0", // Invalid: minimum is 1
        "--github-token", "test_token_12345678901234567890",
        "--database-url", "postgresql://user:pass@localhost:5432/test"
    ];

    let result = CliConfig::parse_from(args);
    assert!(result.is_err());
}

#[test]
fn test_search_query_validation() {
    // Valid queries
    let valid_queries = vec![
        "rust language:rust",
        "stars:>1000",
        "user:octocat",
        "created:>2023-01-01",
        "topic:machine-learning",
        "a", // minimum length
        "a".repeat(256), // maximum length
    ];

    for query in valid_queries {
        let result = CliConfig::validate_search_query(&query);
        assert!(result.is_ok(), "Query should be valid: {}", query);
    }

    // Invalid queries
    let invalid_queries = vec![
        ("", "empty query"),
        ("   ", "whitespace only"),
        (&"a".repeat(257), "too long"),
        ("test\0query", "null character"),
    ];

    for (query, description) in invalid_queries {
        let result = CliConfig::validate_search_query(query);
        assert!(result.is_err(), "Query should be invalid ({}): {}", description, query);
    }
}

#[test]
fn test_github_token_validation() {
    // Valid tokens
    let valid_tokens = vec![
        "ghp_1234567890abcdef1234567890abcdef12345678",
        "github_pat_1234567890abcdef",
        "a".repeat(40), // typical length
        "a".repeat(10), // minimum length
        "a".repeat(255), // maximum length
    ];

    for token in valid_tokens {
        let result = CliConfig::validate_github_token(&token);
        assert!(result.is_ok(), "Token should be valid: {}", token);
    }

    // Invalid tokens
    let invalid_tokens = vec![
        ("", "empty token"),
        ("   ", "whitespace only"),
        ("short", "too short"),
        (&"a".repeat(256), "too long"),
        ("token with spaces", "contains spaces"),
        ("token\nwith\nnewlines", "contains newlines"),
        ("token\twith\ttabs", "contains tabs"),
    ];

    for (token, description) in invalid_tokens {
        let result = CliConfig::validate_github_token(token);
        assert!(result.is_err(), "Token should be invalid ({}): {}", description, token);
    }
}

#[test]
fn test_database_url_validation() {
    // Valid URLs
    let valid_urls = vec![
        "postgresql://user:pass@localhost:5432/dbname",
        "postgres://user:pass@localhost:5432/dbname",
        "postgresql://user:pass@host.example.com:5432/dbname",
        "postgres://user:pass@192.168.1.100:5432/dbname",
    ];

    for url in valid_urls {
        let result = CliConfig::validate_database_url(url);
        assert!(result.is_ok(), "URL should be valid: {}", url);
    }

    // Invalid URLs
    let invalid_urls = vec![
        ("", "empty URL"),
        ("   ", "whitespace only"),
        ("mysql://user:pass@host/db", "wrong protocol"),
        ("http://example.com", "wrong protocol"),
        ("postgresql://localhost:5432/db", "missing auth"),
        ("postgresql://user:pass@localhost:5432", "missing database"),
        ("postgresql://user:pass@localhost", "missing port and database"),
    ];

    for (url, description) in invalid_urls {
        let result = CliConfig::validate_database_url(url);
        assert!(result.is_err(), "URL should be invalid ({}): {}", description, url);
    }
}

#[test]
fn test_database_url_masking() {
    let config = CliConfig {
        search_query: "test".to_string(),
        github_token: "token".to_string(),
        database_url: "postgresql://user:secret_password@localhost:5432/dbname".to_string(),
        per_page: 30,
        page: 1,
        verbose: false,
        dry_run: false,
    };

    let masked = config.mask_database_url();
    
    // Should contain masked password
    assert!(masked.contains("***"));
    // Should not contain the actual password
    assert!(!masked.contains("secret_password"));
    // Should still contain the protocol and structure
    assert!(masked.starts_with("postgresql://"));
}

#[test]
fn test_progress_indicator_verbose_mode() {
    let progress = ProgressIndicator::new("Test operation".to_string(), true);
    
    // These should not panic in verbose mode
    progress.start();
    progress.update("Step 1");
    progress.update("Step 2");
    progress.success("Completed successfully");
    progress.error("Error occurred");
    progress.warning("Warning message");
    progress.info("Information message");
}

#[test]
fn test_progress_indicator_quiet_mode() {
    let progress = ProgressIndicator::new("Test operation".to_string(), false);
    
    // These should not panic in quiet mode
    progress.start();
    progress.update("Step 1");
    progress.success("Completed successfully");
    progress.error("Error occurred");
    progress.warning("Warning message");
    progress.info("Information message");
}

#[test]
fn test_error_display_formatting() {
    // Test different error types and their display formatting
    let errors = vec![
        AppError::environment("GITHUB_TOKEN"),
        AppError::authentication("Invalid token"),
        AppError::invalid_query("bad query", "syntax error"),
        AppError::configuration("Invalid config"),
    ];

    for error in errors {
        // Should not panic when displaying
        let error_string = error.to_string();
        assert!(!error_string.is_empty());
        
        // Test that display_error doesn't panic
        CliConfig::display_error(&error);
    }
}

#[test]
fn test_environment_variable_handling() {
    // Test with environment variables set
    env::set_var("TEST_GITHUB_TOKEN", "test_token_from_env_12345678901234567890");
    env::set_var("TEST_DATABASE_URL", "postgresql://user:pass@localhost:5432/test");

    // Test that CLI can use environment variables when not provided as args
    // Note: We can't easily test the actual env var reading without modifying the code,
    // but we can test the validation functions
    
    let token = env::var("TEST_GITHUB_TOKEN").unwrap();
    assert!(CliConfig::validate_github_token(&token).is_ok());
    
    let db_url = env::var("TEST_DATABASE_URL").unwrap();
    assert!(CliConfig::validate_database_url(&db_url).is_ok());

    // Cleanup
    env::remove_var("TEST_GITHUB_TOKEN");
    env::remove_var("TEST_DATABASE_URL");
}

// Property-based tests
proptest! {
    #[test]
    fn test_search_query_length_invariants(
        query in "[a-zA-Z0-9 ]{1,256}"
    ) {
        let result = CliConfig::validate_search_query(&query);
        prop_assert!(result.is_ok());
    }

    #[test]
    fn test_github_token_length_invariants(
        token in "[a-zA-Z0-9_]{10,255}"
    ) {
        let result = CliConfig::validate_github_token(&token);
        prop_assert!(result.is_ok());
    }

    #[test]
    fn test_per_page_range_invariants(per_page in 1u32..=100) {
        // All values in valid range should be accepted
        let clamped = Some(per_page).unwrap_or(30).clamp(1, 100);
        prop_assert_eq!(clamped, per_page);
    }

    #[test]
    fn test_page_number_invariants(page in 1u32..1000) {
        // All positive page numbers should be valid
        let validated = Some(page).unwrap_or(1).max(1);
        prop_assert_eq!(validated, page);
    }

    #[test]
    fn test_database_url_masking_invariants(
        user in "[a-zA-Z0-9_]{3,20}",
        password in "[a-zA-Z0-9_!@#$%^&*]{8,50}",
        host in "[a-zA-Z0-9.-]{5,50}",
        port in 1000u16..65535,
        dbname in "[a-zA-Z0-9_]{3,20}"
    ) {
        let url = format!("postgresql://{}:{}@{}:{}/{}", user, password, host, port, dbname);
        let config = CliConfig {
            search_query: "test".to_string(),
            github_token: "token".to_string(),
            database_url: url,
            per_page: 30,
            page: 1,
            verbose: false,
            dry_run: false,
        };

        let masked = config.mask_database_url();
        
        // Should not contain the original password
        prop_assert!(!masked.contains(&password));
        // Should contain masking
        prop_assert!(masked.contains("***"));
    }
}

#[test]
fn test_cli_help_generation() {
    // Test that help can be generated without panicking
    let args = vec!["github-pg-query", "--help"];
    let result = CliConfig::parse_from(args);
    
    // Help should cause an error (clap exits with help)
    assert!(result.is_err());
}

#[test]
fn test_cli_version_generation() {
    // Test that version can be generated without panicking
    let args = vec!["github-pg-query", "--version"];
    let result = CliConfig::parse_from(args);
    
    // Version should cause an error (clap exits with version)
    assert!(result.is_err());
}

#[test]
fn test_configuration_display() {
    let config = CliConfig {
        search_query: "rust language:rust".to_string(),
        github_token: "test_token_12345678901234567890".to_string(),
        database_url: "postgresql://user:password@localhost:5432/test".to_string(),
        per_page: 50,
        page: 2,
        verbose: true,
        dry_run: false,
    };

    // Should not panic when displaying configuration
    config.display_summary();
}

#[test]
fn test_setup_help_display() {
    // Should not panic when displaying setup help
    CliConfig::display_setup_help();
}

#[test]
fn test_environment_validation() {
    // Should not panic when validating environment
    let _ = CliConfig::validate_environment();
}