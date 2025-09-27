# API Documentation

This document provides comprehensive API documentation for the GitHub PostgreSQL Query tool's Rust library components.

## Table of Contents

- [Core Modules](#core-modules)
- [CLI Module](#cli-module)
- [GitHub Client](#github-client)
- [Database Manager](#database-manager)
- [Data Models](#data-models)
- [Error Handling](#error-handling)
- [Usage Examples](#usage-examples)

## Core Modules

### Library Structure

```rust
github_pg_query/
├── cli.rs          // CLI argument parsing and progress indication
├── github.rs       // GitHub API client with rate limiting
├── database.rs     // PostgreSQL operations and table management
├── models.rs       // Data structures and serialization
├── errors.rs       // Error types and handling
└── lib.rs          // Public API exports
```

## CLI Module

### CliConfig

The main configuration structure for the application.

```rust
pub struct CliConfig {
    pub search_query: String,
    pub github_token: String,
    pub database_url: String,
    pub per_page: u32,
    pub page: u32,
    pub verbose: bool,
    pub dry_run: bool,
}
```

#### Methods

##### `parse() -> Result<CliConfig>`

Parses command line arguments and environment variables.

```rust
use github_pg_query::CliConfig;

let config = CliConfig::parse()?;
println!("Query: {}", config.search_query);
```

##### `parse_from<I, T>(args: I) -> Result<CliConfig>`

Parses from provided arguments (useful for testing).

```rust
let args = vec!["github-pg-query", "rust", "--verbose"];
let config = CliConfig::parse_from(args)?;
```

##### `display_summary(&self)`

Displays configuration summary with masked sensitive information.

```rust
config.display_summary();
// Output: Configuration details with masked tokens/passwords
```

##### `display_error(error: &AppError)`

Displays user-friendly error messages with actionable suggestions.

```rust
if let Err(error) = config.parse() {
    CliConfig::display_error(&error);
}
```

### ProgressIndicator

Provides user-friendly progress feedback during operations.

```rust
pub struct ProgressIndicator {
    message: String,
    verbose: bool,
}
```

#### Methods

##### `new(message: String, verbose: bool) -> Self`

Creates a new progress indicator.

```rust
let progress = ProgressIndicator::new("Connecting".to_string(), true);
```

##### `start(&self)`

Starts the progress indicator.

```rust
progress.start();
// Output: 🔄 Connecting...
```

##### `update(&self, status: &str)`

Updates progress with a status message (verbose mode only).

```rust
progress.update("Establishing connection pool");
// Output (verbose): ↳ Establishing connection pool
```

##### `success(&self, message: &str)`

Completes progress with success message.

```rust
progress.success("Connected successfully");
// Output: ✅ Connected successfully
```

##### `error(&self, message: &str)`

Shows error message.

```rust
progress.error("Connection failed");
// Output: ❌ Connection failed
```

##### `warning(&self, message: &str)`

Shows warning message.

```rust
progress.warning("Rate limit approaching");
// Output: ⚠️ Rate limit approaching
```

##### `info(&self, message: &str)`

Shows informational message (verbose mode only).

```rust
progress.info("Using cached credentials");
// Output (verbose): ℹ️ Using cached credentials
```

## GitHub Client

### GitHubClient

Provides authenticated access to the GitHub Search API with rate limiting and retry logic.

```rust
pub struct GitHubClient {
    // Internal fields
}
```

#### Methods

##### `new(token: String) -> Result<GitHubClient>`

Creates a new GitHub client with authentication.

```rust
use github_pg_query::GitHubClient;

let client = GitHubClient::new("your_github_token".to_string())?;
```

##### `search_repositories(&self, query: &str, per_page: Option<u32>, page: Option<u32>) -> Result<SearchResponse>`

Searches for repositories using GitHub's search API.

```rust
let results = client.search_repositories(
    "language:rust stars:>1000",
    Some(30),  // per_page
    Some(1)    // page
).await?;

println!("Found {} repositories", results.total_count);
for repo in results.items {
    println!("- {} (⭐ {})", repo.full_name, repo.stargazers_count);
}
```

##### `validate_token(&self) -> Result<()>`

Validates the GitHub token by making a test API call.

```rust
client.validate_token().await?;
println!("Token is valid");
```

##### `get_rate_limit(&self) -> Result<RateLimitStatus>`

Gets current rate limit status.

```rust
let status = client.get_rate_limit().await?;
println!("Remaining: {}/{}", status.remaining, status.limit);
println!("Reset at: {}", status.reset_at);
```

### SearchResponse

Response structure from GitHub's search API.

```rust
pub struct SearchResponse {
    pub total_count: u64,
    pub incomplete_results: bool,
    pub items: Vec<Repository>,
}
```

### RateLimitStatus

Current rate limit information.

```rust
pub struct RateLimitStatus {
    pub limit: u32,
    pub remaining: u32,
    pub reset_at: DateTime<Utc>,
}
```

## Database Manager

### DatabaseManager

Manages PostgreSQL operations including table creation, data insertion, and query tracking.

```rust
pub struct DatabaseManager {
    // Internal connection pool
}
```

#### Methods

##### `new(database_url: &str) -> Result<DatabaseManager>`

Creates a new database manager with connection pooling.

```rust
use github_pg_query::DatabaseManager;

let db = DatabaseManager::new("postgresql://user:pass@localhost/db").await?;
```

##### `generate_table_name() -> String`

Generates a timestamped table name.

```rust
let table_name = DatabaseManager::generate_table_name();
// Returns: "repos_20231201143022"
```

##### `create_repository_table(&self, table_name: &str) -> Result<()>`

Creates a new repository table with full schema and indexes.

```rust
let table_name = DatabaseManager::generate_table_name();
db.create_repository_table(&table_name).await?;
```

##### `insert_repositories(&self, table_name: &str, repositories: &[Repository]) -> Result<i64>`

Inserts repositories with conflict handling (upsert).

```rust
let count = db.insert_repositories(&table_name, &repositories).await?;
println!("Inserted/updated {} repositories", count);
```

##### `save_query_metadata(&self, metadata: &QueryMetadata) -> Result<()>`

Saves query execution metadata.

```rust
let mut metadata = QueryMetadata::new(query, table_name);
metadata.mark_success(result_count, duration_ms);
db.save_query_metadata(&metadata).await?;
```

##### `get_table_stats(&self, table_name: &str) -> Result<TableStats>`

Gets comprehensive statistics for a repository table.

```rust
let stats = db.get_table_stats(&table_name).await?;
println!("Total repositories: {}", stats.total_repositories);
println!("Unique languages: {}", stats.unique_languages);
println!("Average stars: {:.1}", stats.avg_stars);
```

##### `list_repository_tables(&self) -> Result<Vec<String>>`

Lists all repository tables in the database.

```rust
let tables = db.list_repository_tables().await?;
for table in tables {
    println!("Table: {}", table);
}
```

##### `get_query_history(&self, limit: Option<i64>, success_only: bool) -> Result<Vec<QueryMetadata>>`

Retrieves query execution history.

```rust
let history = db.get_query_history(Some(10), false).await?;
for query in history {
    println!("{}: {} results", query.search_query, query.result_count);
}
```

##### `drop_table(&self, table_name: &str) -> Result<()>`

Drops a repository table (use with caution).

```rust
db.drop_table("old_table_name").await?;
```

### QueryMetadata

Tracks query execution metadata and performance.

```rust
pub struct QueryMetadata {
    pub id: Uuid,
    pub search_query: String,
    pub table_name: String,
    pub result_count: i64,
    pub executed_at: DateTime<Utc>,
    pub duration_ms: i64,
    pub success: bool,
    pub error_message: Option<String>,
}
```

#### Methods

##### `new(search_query: String, table_name: String) -> Self`

Creates new query metadata.

```rust
let metadata = QueryMetadata::new(
    "language:rust".to_string(),
    "repos_20231201143022".to_string()
);
```

##### `mark_success(&mut self, result_count: i64, duration_ms: i64)`

Marks query as successful with results.

```rust
metadata.mark_success(25, 1500);
```

##### `mark_failure(&mut self, error_message: String, duration_ms: i64)`

Marks query as failed with error details.

```rust
metadata.mark_failure("Rate limit exceeded".to_string(), 500);
```

### TableStats

Statistics for a repository table.

```rust
pub struct TableStats {
    pub total_repositories: i64,
    pub unique_languages: i64,
    pub unique_owners: i64,
    pub avg_stars: f64,
    pub max_stars: i64,
    pub avg_forks: f64,
    pub max_forks: i64,
}
```

## Data Models

### Repository

Main repository data structure matching GitHub's API response.

```rust
pub struct Repository {
    pub id: i64,
    pub full_name: String,
    pub name: String,
    pub description: Option<String>,
    pub html_url: String,
    pub clone_url: String,
    pub ssh_url: String,
    pub size: i64,
    pub stargazers_count: i64,
    pub watchers_count: i64,
    pub forks_count: i64,
    pub open_issues_count: i64,
    pub language: Option<String>,
    pub default_branch: String,
    pub visibility: String,
    pub private: bool,
    pub fork: bool,
    pub archived: bool,
    pub disabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub pushed_at: Option<DateTime<Utc>>,
    pub owner: RepositoryOwner,
    pub license: Option<RepositoryLicense>,
    pub topics: Vec<String>,
    pub has_issues: bool,
    pub has_projects: bool,
    pub has_wiki: bool,
    pub has_pages: bool,
    pub has_downloads: bool,
}
```

### RepositoryOwner

Repository owner information.

```rust
pub struct RepositoryOwner {
    pub id: i64,
    pub login: String,
    pub owner_type: String,
    pub avatar_url: String,
    pub html_url: String,
    pub site_admin: bool,
}
```

### RepositoryLicense

Repository license information.

```rust
pub struct RepositoryLicense {
    pub key: String,
    pub name: String,
    pub spdx_id: Option<String>,
    pub url: Option<String>,
}
```

## Error Handling

### AppError

Comprehensive error type with specific variants for different failure modes.

```rust
pub enum AppError {
    Environment { var_name: String },
    Authentication { reason: String },
    InvalidQuery { query: String, reason: String },
    RateLimit { reset_time: DateTime<Utc> },
    GitHubApi { status: u16, message: String },
    Database(sqlx::Error),
    Http(reqwest::Error),
    Json(serde_json::Error),
    Configuration { message: String },
    TableCreation { table_name: String, reason: String },
    Io(std::io::Error),
}
```

#### Error Creation Methods

```rust
// Environment variable errors
AppError::environment("GITHUB_TOKEN")

// Authentication errors
AppError::authentication("Invalid token format")

// Query validation errors
AppError::invalid_query("", "Query cannot be empty")

// Rate limiting errors
AppError::rate_limit(reset_time)

// Configuration errors
AppError::configuration("Invalid database URL format")
```

#### Error Display

All errors implement `Display` with user-friendly messages:

```rust
match error {
    AppError::Environment { var_name } => {
        println!("Environment variable {} is not set", var_name);
    }
    AppError::RateLimit { reset_time } => {
        println!("Rate limited until: {}", reset_time);
    }
    // ... other variants
}
```

## Usage Examples

### Basic Library Usage

```rust
use github_pg_query::{GitHubClient, DatabaseManager, QueryMetadata};
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize clients
    let github_client = GitHubClient::new("your_token".to_string())?;
    let db = DatabaseManager::new("postgresql://user:pass@localhost/db").await?;
    
    // Create table
    let table_name = DatabaseManager::generate_table_name();
    db.create_repository_table(&table_name).await?;
    
    // Search repositories
    let start = Instant::now();
    let results = github_client.search_repositories(
        "language:rust stars:>1000",
        Some(30),
        Some(1)
    ).await?;
    
    // Store results
    let count = db.insert_repositories(&table_name, &results.items).await?;
    
    // Track metadata
    let mut metadata = QueryMetadata::new(
        "language:rust stars:>1000".to_string(),
        table_name
    );
    metadata.mark_success(count, start.elapsed().as_millis() as i64);
    db.save_query_metadata(&metadata).await?;
    
    println!("Stored {} repositories", count);
    Ok(())
}
```

### Error Handling Example

```rust
use github_pg_query::{GitHubClient, AppError};

async fn search_with_error_handling(query: &str) {
    let client = GitHubClient::new("token".to_string())?;
    
    match client.search_repositories(query, None, None).await {
        Ok(results) => {
            println!("Found {} repositories", results.total_count);
        }
        Err(AppError::RateLimit { reset_time }) => {
            println!("Rate limited until: {}", reset_time);
            // Wait and retry logic
        }
        Err(AppError::InvalidQuery { query, reason }) => {
            println!("Invalid query '{}': {}", query, reason);
            // Query correction logic
        }
        Err(AppError::Authentication { reason }) => {
            println!("Authentication failed: {}", reason);
            // Token refresh logic
        }
        Err(e) => {
            println!("Other error: {}", e);
        }
    }
}
```

### Progress Indication Example

```rust
use github_pg_query::ProgressIndicator;

async fn operation_with_progress(verbose: bool) {
    let progress = ProgressIndicator::new("Processing data".to_string(), verbose);
    
    progress.start();
    
    // Step 1
    progress.update("Validating input");
    // ... validation logic
    
    // Step 2
    progress.update("Connecting to services");
    // ... connection logic
    
    // Step 3
    progress.update("Processing results");
    // ... processing logic
    
    progress.success("Operation completed successfully");
}
```

### Database Statistics Example

```rust
use github_pg_query::DatabaseManager;

async fn analyze_repository_data(db: &DatabaseManager, table_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let stats = db.get_table_stats(table_name).await?;
    
    println!("Repository Analysis for {}", table_name);
    println!("================================");
    println!("Total repositories: {}", stats.total_repositories);
    println!("Unique languages: {}", stats.unique_languages);
    println!("Unique owners: {}", stats.unique_owners);
    println!("Average stars: {:.1}", stats.avg_stars);
    println!("Max stars: {}", stats.max_stars);
    println!("Average forks: {:.1}", stats.avg_forks);
    println!("Max forks: {}", stats.max_forks);
    
    Ok(())
}
```

## Type Aliases

```rust
/// Result type alias for the application
pub type Result<T> = std::result::Result<T, AppError>;
```

## Feature Flags

The library supports optional features:

```toml
[dependencies]
github-pg-query = { version = "0.1", features = ["metrics"] }
```

Available features:
- `metrics`: Enable performance metrics collection
- `tracing`: Enable detailed tracing support

## Thread Safety

All public types are thread-safe:
- `GitHubClient`: Uses internal connection pooling (Send + Sync)
- `DatabaseManager`: Uses SQLx connection pool (Send + Sync)
- `CliConfig`: Immutable after creation (Send + Sync)
- `Repository` and related types: Immutable data structures (Send + Sync)

## Performance Considerations

- **Connection Pooling**: Both GitHub client and database manager use connection pooling
- **Async Operations**: All I/O operations are async for better concurrency
- **Batch Processing**: Repository insertion is optimized for large batches
- **Memory Efficiency**: Streaming JSON parsing for large API responses
- **Rate Limiting**: Built-in exponential backoff for GitHub API limits

## Testing

The library includes comprehensive test utilities:

```rust
#[cfg(test)]
mod tests {
    use github_pg_query::*;
    
    #[tokio::test]
    async fn test_github_client() {
        // Test with mock server
    }
    
    #[tokio::test]
    async fn test_database_operations() {
        // Test with test containers
    }
}
```

See the `tests/` directory for complete integration and unit tests.