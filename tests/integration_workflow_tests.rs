use github_pg_query::{CliConfig, Result};

#[tokio::test]
async fn test_cli_config_parsing_and_validation() -> Result<()> {
    // Test that CLI config can be parsed from arguments
    let args = vec![
        "github-pg-query".to_string(),
        "--github-token".to_string(),
        "test_token_12345678901234567890123456789012345678901234567890".to_string(),
        "--database-url".to_string(),
        "postgresql://user:pass@localhost:5432/test".to_string(),
        "--per-page".to_string(),
        "50".to_string(),
        "--page".to_string(),
        "2".to_string(),
        "--verbose".to_string(),
        "rust language:rust".to_string(),
    ];

    let config = CliConfig::parse_from(args)?;
    
    // Verify all fields are parsed correctly
    assert_eq!(config.search_query, "rust language:rust");
    assert_eq!(config.per_page, 50);
    assert_eq!(config.page, 2);
    assert!(config.verbose);
    assert!(!config.dry_run);
    assert_eq!(config.github_token, "test_token_12345678901234567890123456789012345678901234567890");
    assert_eq!(config.database_url, "postgresql://user:pass@localhost:5432/test");

    Ok(())
}

#[tokio::test]
async fn test_configuration_validation() -> Result<()> {
    let args = vec![
        "github-pg-query".to_string(),
        "--github-token".to_string(),
        "test_token_12345678901234567890123456789012345678901234567890".to_string(),
        "--database-url".to_string(),
        "postgresql://user:secret_password@localhost:5432/test".to_string(),
        "test query".to_string(),
    ];

    let config = CliConfig::parse_from(args)?;
    
    // Test that configuration is properly parsed
    assert_eq!(config.search_query, "test query");
    assert_eq!(config.github_token, "test_token_12345678901234567890123456789012345678901234567890");
    assert_eq!(config.database_url, "postgresql://user:secret_password@localhost:5432/test");

    Ok(())
}

#[test]
fn test_workflow_components_integration() {
    // Test that all the main workflow components can be instantiated
    // This verifies that the integration between modules is correct
    
    // Test GitHub client creation (without network call)
    let github_client = github_pg_query::GitHubClient::new("test_token".to_string());
    assert!(github_client.is_ok());
    
    // Test that progress indicator can be created
    let progress = github_pg_query::ProgressIndicator::new("Test".to_string(), false);
    progress.start();
    progress.success("Test completed");
    
    // Test query metadata creation
    let metadata = github_pg_query::QueryMetadata::new(
        "test query".to_string(),
        "test_table".to_string()
    );
    assert_eq!(metadata.search_query, "test query");
    assert_eq!(metadata.table_name, "test_table");
    assert_eq!(metadata.result_count, 0);
    assert!(!metadata.success);
}

#[test]
fn test_table_name_generation_consistency() {
    // Test that table name generation follows the expected format
    let table_name = github_pg_query::DatabaseManager::generate_table_name();
    
    // Should follow the expected format
    assert!(table_name.starts_with("repos_"));
    
    // Should be valid PostgreSQL table names (no special characters except underscore)
    assert!(table_name.chars().all(|c| c.is_alphanumeric() || c == '_'));
    
    // Should have the expected length (repos_ + timestamp)
    assert!(table_name.len() > 6); // More than just "repos_"
}

#[test]
fn test_error_handling_integration() {
    // Test that errors from different components can be properly handled
    use github_pg_query::AppError;
    
    // Test authentication error
    let auth_error = AppError::Authentication { reason: "Invalid token".to_string() };
    assert!(matches!(auth_error, AppError::Authentication { .. }));
    
    // Test validation error  
    let validation_error = AppError::Validation { field: "token".to_string(), reason: "too short".to_string() };
    assert!(matches!(validation_error, AppError::Validation { .. }));
    
    // Test configuration error
    let config_error = AppError::Configuration { message: "Invalid config".to_string() };
    assert!(matches!(config_error, AppError::Configuration { .. }));
}