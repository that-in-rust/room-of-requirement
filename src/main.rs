//! # GitHub PostgreSQL Query Tool
//! 
//! A command-line tool that executes GitHub API search queries and stores
//! the results in timestamped PostgreSQL tables.
//! 
//! ## Features
//! 
//! - Supports any valid GitHub repository search query
//! - Creates timestamped tables with format `repos_YYYYMMDDHHMMSS`
//! - Handles duplicate repositories with upsert operations
//! - Tracks query metadata and performance metrics
//! - Provides comprehensive error handling and progress indicators
//! - Supports dry-run mode for configuration validation
//! 
//! ## Usage
//! 
//! ```bash
//! # Basic usage
//! github-pg-query "language:rust stars:>1000"
//! 
//! # With pagination
//! github-pg-query "user:octocat" --per-page 50 --page 2
//! 
//! # Verbose output
//! github-pg-query "topic:machine-learning" --verbose
//! 
//! # Dry run (validate only)
//! github-pg-query "created:>2023-01-01" --dry-run
//! ```
//! 
//! ## Environment Variables
//! 
//! - `GITHUB_TOKEN`: GitHub personal access token (required)
//! - `DATABASE_URL`: PostgreSQL connection string (required)
//! 
//! ## Examples
//! 
//! See `EXAMPLES.md` for comprehensive query examples and `SETUP.md` for
//! detailed setup instructions.

use github_pg_query::{
    CliConfig, DatabaseManager, GitHubClient, ProgressIndicator, 
    QueryMetadata, Result
};
use std::time::Instant;

/// Main entry point for the GitHub PostgreSQL Query tool.
/// 
/// This function orchestrates the complete workflow:
/// 1. Loads environment variables from .env file if present
/// 2. Parses and validates CLI arguments
/// 3. Handles dry-run mode for configuration validation
/// 4. Executes the main search and storage workflow
/// 
/// # Error Handling
/// 
/// All errors are handled gracefully with user-friendly messages.
/// The application exits with code 1 on any error.
#[tokio::main]
async fn main() {
    // Load environment variables from .env file if it exists
    dotenvy::dotenv().ok();
    // Parse CLI arguments and validate configuration
    let config = match CliConfig::parse() {
        Ok(config) => config,
        Err(error) => {
            CliConfig::display_error(&error);
            std::process::exit(1);
        }
    };

    // Display configuration summary if verbose
    if config.verbose {
        config.display_summary();
    }

    // Handle dry run mode
    if config.dry_run {
        if let Err(error) = validate_dry_run(&config).await {
            CliConfig::display_error(&error);
            std::process::exit(1);
        }
        println!("✅ Dry run completed successfully - configuration is valid");
        return;
    }

    // Execute the main workflow
    if let Err(error) = execute_search_workflow(&config).await {
        CliConfig::display_error(&error);
        std::process::exit(1);
    }
}

/// Validates configuration in dry-run mode without executing queries.
/// 
/// This function performs comprehensive validation of:
/// - GitHub token validity and permissions
/// - Database connectivity and accessibility
/// - Search query format and syntax
/// 
/// # Arguments
/// 
/// * `config` - The parsed CLI configuration to validate
/// 
/// # Returns
/// 
/// * `Ok(())` if all validations pass
/// * `Err(AppError)` if any validation fails
/// 
/// # Example
/// 
/// ```bash
/// github-pg-query "language:rust" --dry-run
/// ```
async fn validate_dry_run(config: &CliConfig) -> Result<()> {
    let progress = ProgressIndicator::new("Dry run validation".to_string(), config.verbose);
    progress.start();

    // Validate GitHub client
    progress.update("Validating GitHub token");
    let github_client = GitHubClient::new(config.github_token.clone())?;
    github_client.validate_token().await?;
    progress.update("GitHub token is valid");

    // Validate database connection
    progress.update("Validating database connection");
    let _db_manager = DatabaseManager::new(&config.database_url).await?;
    progress.update("Database connection is valid");

    // Validate search query format (basic validation)
    progress.update("Validating search query format");
    // The query validation is already done in CliConfig::parse()
    progress.update("Search query format is valid");

    progress.success("All validations passed");
    Ok(())
}

/// Executes the complete search and storage workflow.
/// 
/// This function orchestrates the main application workflow:
/// 1. Initializes GitHub client with authentication
/// 2. Establishes database connection and creates table
/// 3. Executes GitHub search query with pagination
/// 4. Stores repository data with conflict handling
/// 5. Records query metadata for tracking and analysis
/// 
/// # Arguments
/// 
/// * `config` - The validated CLI configuration
/// 
/// # Returns
/// 
/// * `Ok(())` if the workflow completes successfully
/// * `Err(AppError)` if any step fails
/// 
/// # Workflow Steps
/// 
/// 1. **GitHub Client**: Creates authenticated client with rate limiting
/// 2. **Database Setup**: Connects and creates timestamped table
/// 3. **Search Execution**: Queries GitHub API with specified parameters
/// 4. **Data Storage**: Inserts repositories with duplicate handling
/// 5. **Metadata Tracking**: Records query statistics and performance
/// 
/// # Error Handling
/// 
/// Errors at any step are propagated with context. Failed queries
/// are recorded in the query history for analysis.
async fn execute_search_workflow(config: &CliConfig) -> Result<()> {
    let start_time = Instant::now();
    
    // Initialize GitHub client
    let progress = ProgressIndicator::new("Initializing GitHub client".to_string(), config.verbose);
    progress.start();
    let github_client = GitHubClient::new(config.github_token.clone())?;
    progress.success("GitHub client initialized");

    // Initialize database manager
    let progress = ProgressIndicator::new("Connecting to database".to_string(), config.verbose);
    progress.start();
    let db_manager = DatabaseManager::new(&config.database_url).await?;
    progress.success("Database connection established");

    // Generate table name for this query
    let table_name = DatabaseManager::generate_table_name();
    let progress = ProgressIndicator::new(
        format!("Creating table: {}", table_name), 
        config.verbose
    );
    progress.start();
    
    // Create query metadata
    let mut query_metadata = QueryMetadata::new(
        config.search_query.clone(),
        table_name.clone()
    );

    // Create repository table
    db_manager.create_repository_table(&table_name).await?;
    progress.success(&format!("Table {} created", table_name));

    // Execute GitHub search
    let progress = ProgressIndicator::new(
        format!("Searching GitHub: '{}'", config.search_query), 
        config.verbose
    );
    progress.start();
    
    let search_start = Instant::now();
    let search_result = github_client.search_repositories(
        &config.search_query,
        Some(config.per_page),
        Some(config.page)
    ).await;

    let search_duration = search_start.elapsed();

    match search_result {
        Ok(search_response) => {
            let result_count = search_response.items.len() as i64;
            progress.success(&format!(
                "Found {} repositories (total: {}, page: {})", 
                result_count, 
                search_response.total_count,
                config.page
            ));

            if config.verbose {
                progress.info(&format!("Search completed in {:.2}s", search_duration.as_secs_f64()));
                if search_response.incomplete_results {
                    progress.warning("Search results may be incomplete due to timeout");
                }
            }

            // Store repositories in database
            if !search_response.items.is_empty() {
                let progress = ProgressIndicator::new(
                    format!("Storing {} repositories", result_count), 
                    config.verbose
                );
                progress.start();

                let inserted_count = db_manager.insert_repositories(
                    &table_name, 
                    &search_response.items
                ).await?;

                progress.success(&format!("Stored {} repositories", inserted_count));

                if config.verbose && inserted_count != result_count {
                    progress.info(&format!(
                        "Note: {} repositories were updated (duplicates)", 
                        result_count - inserted_count
                    ));
                }
            } else {
                let progress = ProgressIndicator::new("No repositories found".to_string(), config.verbose);
                progress.warning("No repositories matched the search query");
            }

            // Update query metadata with success
            query_metadata.mark_success(result_count, search_duration.as_millis() as i64);
        }
        Err(error) => {
            // Update query metadata with failure
            query_metadata.mark_failure(
                error.to_string(), 
                search_duration.as_millis() as i64
            );
            
            progress.error(&format!("Search failed: {}", error));
            
            // Save the failed query metadata before returning error
            if let Err(save_error) = db_manager.save_query_metadata(&query_metadata).await {
                progress.warning(&format!("Failed to save query metadata: {}", save_error));
            }
            
            return Err(error);
        }
    }

    // Save query metadata
    let progress = ProgressIndicator::new("Saving query metadata".to_string(), config.verbose);
    progress.start();
    db_manager.save_query_metadata(&query_metadata).await?;
    progress.success("Query metadata saved");

    // Display final summary
    let total_duration = start_time.elapsed();
    println!();
    println!("🎉 Search completed successfully!");
    println!("   Table name: {}", table_name);
    println!("   Results: {} repositories", query_metadata.result_count);
    println!("   Total time: {:.2}s", total_duration.as_secs_f64());
    
    if config.verbose {
        println!("   Search time: {:.2}s", search_duration.as_secs_f64());
        println!("   Query ID: {}", query_metadata.id);
    }

    Ok(())
}