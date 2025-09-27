use github_pg_query::{CliConfig, GitHubClient, QueryMetadata, ProgressIndicator, Result};

#[tokio::test]
async fn test_complete_workflow_components_integration() -> Result<()> {
    // Test 1: CLI Configuration parsing and validation
    let args = vec![
        "github-pg-query".to_string(),
        "--github-token".to_string(),
        "test_token_12345678901234567890123456789012345678901234567890".to_string(),
        "--database-url".to_string(),
        "postgresql://user:pass@localhost:5432/test".to_string(),
        "--per-page".to_string(),
        "25".to_string(),
        "--page".to_string(),
        "1".to_string(),
        "--verbose".to_string(),
        "rust language:rust stars:>100".to_string(),
    ];

    let config = CliConfig::parse_from(args)?;
    
    // Verify configuration is parsed correctly
    assert_eq!(config.search_query, "rust language:rust stars:>100");
    assert_eq!(config.per_page, 25);
    assert_eq!(config.page, 1);
    assert!(config.verbose);
    assert!(!config.dry_run);

    // Test 2: GitHub client initialization
    let _github_client = GitHubClient::new(config.github_token.clone())?;
    
    // Test 3: Progress indicator functionality
    let progress = ProgressIndicator::new("Testing workflow integration".to_string(), config.verbose);
    progress.start();
    progress.update("Initializing components");
    progress.success("Components initialized");

    // Test 4: Query metadata creation and manipulation
    let table_name = format!("test_table_{}", chrono::Utc::now().timestamp());
    let mut query_metadata = QueryMetadata::new(
        config.search_query.clone(),
        table_name.clone()
    );

    // Verify initial state
    assert_eq!(query_metadata.search_query, config.search_query);
    assert_eq!(query_metadata.table_name, table_name);
    assert_eq!(query_metadata.result_count, 0);
    assert!(!query_metadata.success);
    assert!(query_metadata.error_message.is_none());

    // Test success scenario
    query_metadata.mark_success(42, 1500);
    assert_eq!(query_metadata.result_count, 42);
    assert!(query_metadata.success);
    assert_eq!(query_metadata.duration_ms, 1500);
    assert!(query_metadata.error_message.is_none());

    // Test failure scenario
    let mut failure_metadata = QueryMetadata::new(
        "test query".to_string(),
        "test_table".to_string()
    );
    failure_metadata.mark_failure("Test error".to_string(), 500);
    assert_eq!(failure_metadata.result_count, 0);
    assert!(!failure_metadata.success);
    assert_eq!(failure_metadata.duration_ms, 500);
    assert_eq!(failure_metadata.error_message, Some("Test error".to_string()));

    // Test 5: Table name generation consistency
    let table_name1 = github_pg_query::DatabaseManager::generate_table_name();
    
    // Verify table name format
    assert!(table_name1.starts_with("repos_"));
    assert!(table_name1.len() > 6);
    assert!(table_name1.chars().all(|c| c.is_alphanumeric() || c == '_'));

    progress.success("All workflow components integrated successfully");

    Ok(())
}

#[tokio::test]
async fn test_error_propagation_through_workflow() -> Result<()> {
    // Test that errors propagate correctly through the workflow
    
    // Test that GitHub client can be created (validation happens during API calls)
    let _github_client = GitHubClient::new("invalid".to_string())?;

    // Test invalid CLI arguments
    let invalid_args = vec![
        "github-pg-query".to_string(),
        "--per-page".to_string(),
        "200".to_string(), // Invalid: max is 100
        "test query".to_string(),
    ];
    
    let invalid_config_result = CliConfig::parse_from(invalid_args);
    assert!(invalid_config_result.is_err());

    // Test empty search query
    let empty_query_args = vec![
        "github-pg-query".to_string(),
        "--github-token".to_string(),
        "test_token_12345678901234567890123456789012345678901234567890".to_string(),
        "--database-url".to_string(),
        "postgresql://user:pass@localhost:5432/test".to_string(),
        "".to_string(), // Empty query
    ];
    
    let empty_query_result = CliConfig::parse_from(empty_query_args);
    assert!(empty_query_result.is_err());

    Ok(())
}

#[test]
fn test_progress_indicator_states() {
    // Test progress indicator in different modes
    let verbose_progress = ProgressIndicator::new("Verbose test".to_string(), true);
    verbose_progress.start();
    verbose_progress.update("Processing");
    verbose_progress.info("Information message");
    verbose_progress.warning("Warning message");
    verbose_progress.success("Completed successfully");

    let quiet_progress = ProgressIndicator::new("Quiet test".to_string(), false);
    quiet_progress.start();
    quiet_progress.update("Processing");
    quiet_progress.error("Error message");
    
    // These should not panic and should handle both verbose and quiet modes
}

#[test]
fn test_query_metadata_lifecycle() {
    // Test the complete lifecycle of query metadata
    let query = "rust language:rust";
    let table = "test_table_123";
    
    // Create new metadata
    let mut metadata = QueryMetadata::new(query.to_string(), table.to_string());
    
    // Verify initial state
    assert_eq!(metadata.search_query, query);
    assert_eq!(metadata.table_name, table);
    assert_eq!(metadata.result_count, 0);
    assert!(!metadata.success);
    assert!(metadata.error_message.is_none());
    assert_eq!(metadata.duration_ms, 0);
    
    // Test successful completion
    metadata.mark_success(150, 2500);
    assert_eq!(metadata.result_count, 150);
    assert!(metadata.success);
    assert_eq!(metadata.duration_ms, 2500);
    assert!(metadata.error_message.is_none());
    
    // Test that we can create another metadata instance
    let mut metadata2 = QueryMetadata::new("another query".to_string(), "another_table".to_string());
    metadata2.mark_failure("Connection timeout".to_string(), 30000);
    
    assert_eq!(metadata2.result_count, 0);
    assert!(!metadata2.success);
    assert_eq!(metadata2.duration_ms, 30000);
    assert_eq!(metadata2.error_message, Some("Connection timeout".to_string()));
}