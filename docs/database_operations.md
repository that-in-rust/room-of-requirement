# Database Operations

This document describes the database operations module for the GitHub PostgreSQL Query tool.

## Overview

The `DatabaseManager` provides a comprehensive interface for managing PostgreSQL operations, including:

- Connection pool management
- Dynamic table creation with timestamped names
- Repository data insertion with conflict handling
- Query history tracking
- Table statistics and management

## Key Features

### 1. Connection Management

The `DatabaseManager` uses SQLx connection pooling for efficient database operations:

```rust
use github_pg_query::DatabaseManager;

let db = DatabaseManager::new("postgresql://user:pass@localhost/db").await?;
```

### 2. Timestamped Table Names

Tables are created with timestamps in the format `repos_YYYYMMDDHHMMSS`:

```rust
let table_name = DatabaseManager::generate_table_name();
// Example: "repos_20231201143022"
```

### 3. Dynamic Table Creation

Repository tables are created dynamically based on the GitHub API schema:

```rust
db.create_repository_table(&table_name).await?;
```

The table includes all repository fields:
- Basic info (id, name, description, URLs)
- Statistics (stars, forks, watchers, issues)
- Metadata (language, visibility, dates)
- Owner information (flattened)
- License information (flattened)
- Topics as PostgreSQL array
- Feature flags (has_issues, has_wiki, etc.)

### 4. Conflict Handling

Repository insertion uses `ON CONFLICT` to handle duplicates:

```rust
let inserted_count = db.insert_repositories(&table_name, &repositories).await?;
```

- New repositories are inserted
- Existing repositories (by `github_id`) are updated
- Returns the total number of affected rows

### 5. Query History Tracking

All queries are tracked in the `query_history` table:

```rust
let mut metadata = QueryMetadata::new(query, table_name);
metadata.mark_success(result_count, duration_ms);
db.save_query_metadata(&metadata).await?;
```

### 6. Table Statistics

Get comprehensive statistics for any repository table:

```rust
let stats = db.get_table_stats(&table_name).await?;
println!("Total repositories: {}", stats.total_repositories);
println!("Unique languages: {}", stats.unique_languages);
println!("Average stars: {:.1}", stats.avg_stars);
```

## Database Schema

### Repository Table Schema

```sql
CREATE TABLE repos_YYYYMMDDHHMMSS (
    id SERIAL PRIMARY KEY,
    github_id BIGINT UNIQUE NOT NULL,
    full_name VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    html_url VARCHAR(500) NOT NULL,
    clone_url VARCHAR(500) NOT NULL,
    ssh_url VARCHAR(500) NOT NULL,
    size_kb BIGINT NOT NULL DEFAULT 0,
    stargazers_count BIGINT NOT NULL DEFAULT 0,
    watchers_count BIGINT NOT NULL DEFAULT 0,
    forks_count BIGINT NOT NULL DEFAULT 0,
    open_issues_count BIGINT NOT NULL DEFAULT 0,
    language VARCHAR(100),
    default_branch VARCHAR(100) NOT NULL,
    visibility VARCHAR(20) NOT NULL,
    private BOOLEAN NOT NULL DEFAULT FALSE,
    fork BOOLEAN NOT NULL DEFAULT FALSE,
    archived BOOLEAN NOT NULL DEFAULT FALSE,
    disabled BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    pushed_at TIMESTAMPTZ,
    owner_id BIGINT NOT NULL,
    owner_login VARCHAR(255) NOT NULL,
    owner_type VARCHAR(50) NOT NULL,
    owner_avatar_url VARCHAR(500) NOT NULL,
    owner_html_url VARCHAR(500) NOT NULL,
    owner_site_admin BOOLEAN NOT NULL DEFAULT FALSE,
    license_key VARCHAR(100),
    license_name VARCHAR(255),
    license_spdx_id VARCHAR(100),
    license_url VARCHAR(500),
    topics TEXT[] DEFAULT '{}',
    has_issues BOOLEAN NOT NULL DEFAULT FALSE,
    has_projects BOOLEAN NOT NULL DEFAULT FALSE,
    has_wiki BOOLEAN NOT NULL DEFAULT FALSE,
    has_pages BOOLEAN NOT NULL DEFAULT FALSE,
    has_downloads BOOLEAN NOT NULL DEFAULT FALSE,
    fetched_at TIMESTAMPTZ DEFAULT NOW()
);
```

### Query History Table Schema

```sql
CREATE TABLE query_history (
    id UUID PRIMARY KEY,
    search_query TEXT NOT NULL,
    table_name VARCHAR(50) NOT NULL,
    result_count BIGINT NOT NULL DEFAULT 0,
    executed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    duration_ms BIGINT NOT NULL DEFAULT 0,
    success BOOLEAN NOT NULL DEFAULT FALSE,
    error_message TEXT
);
```

## Indexes

The following indexes are automatically created for performance:

### Repository Table Indexes
- `idx_<table>_github_id` - Unique constraint and fast lookups
- `idx_<table>_full_name` - Repository name searches
- `idx_<table>_language` - Language filtering
- `idx_<table>_stargazers` - Sorting by popularity
- `idx_<table>_created_at` - Date range queries
- `idx_<table>_owner_login` - Owner-based queries

### Query History Indexes
- `idx_query_history_executed_at` - Time-based queries
- `idx_query_history_table_name` - Table-specific history
- `idx_query_history_success` - Success/failure filtering

## Error Handling

The database module uses structured error handling:

```rust
use github_pg_query::{AppError, Result};

match db.create_repository_table(&table_name).await {
    Ok(()) => println!("Table created successfully"),
    Err(AppError::Database(e)) => eprintln!("Database error: {}", e),
    Err(AppError::TableCreation { table_name, reason }) => {
        eprintln!("Failed to create table {}: {}", table_name, reason);
    }
    Err(e) => eprintln!("Other error: {}", e),
}
```

## Performance Considerations

### Batch Operations
- Repository insertion uses transactions for consistency
- Large batches are processed efficiently
- Connection pooling prevents connection exhaustion

### Memory Usage
- Streaming results for large datasets
- Bounded connection pools
- Efficient SQL generation

### Concurrency
- Thread-safe connection pool
- Concurrent read/write operations supported
- Proper transaction isolation

## Usage Examples

### Basic Usage

```rust
use github_pg_query::{DatabaseManager, Repository};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to database
    let db = DatabaseManager::new(&database_url).await?;
    
    // Generate table name
    let table_name = DatabaseManager::generate_table_name();
    
    // Create table
    db.create_repository_table(&table_name).await?;
    
    // Insert repositories
    let repositories: Vec<Repository> = fetch_from_github().await?;
    let count = db.insert_repositories(&table_name, &repositories).await?;
    
    println!("Inserted {} repositories into {}", count, table_name);
    
    Ok(())
}
```

### Query History Tracking

```rust
use github_pg_query::{QueryMetadata, DatabaseManager};

async fn track_query(
    db: &DatabaseManager,
    query: &str,
    table_name: &str,
    repositories: &[Repository],
) -> Result<(), AppError> {
    let start = std::time::Instant::now();
    
    // Create metadata
    let mut metadata = QueryMetadata::new(query.to_string(), table_name.to_string());
    
    // Perform operation
    match db.insert_repositories(table_name, repositories).await {
        Ok(count) => {
            let duration = start.elapsed().as_millis() as i64;
            metadata.mark_success(count, duration);
        }
        Err(e) => {
            let duration = start.elapsed().as_millis() as i64;
            metadata.mark_failure(e.to_string(), duration);
        }
    }
    
    // Save metadata
    db.save_query_metadata(&metadata).await?;
    
    Ok(())
}
```

### Table Management

```rust
use github_pg_query::DatabaseManager;

async fn manage_tables(db: &DatabaseManager) -> Result<(), AppError> {
    // List all repository tables
    let tables = db.list_repository_tables().await?;
    println!("Found {} tables", tables.len());
    
    for table in &tables {
        // Get statistics for each table
        let stats = db.get_table_stats(table).await?;
        println!("{}: {} repositories, {} languages", 
                 table, stats.total_repositories, stats.unique_languages);
    }
    
    // Clean up old tables (example: keep only last 10)
    if tables.len() > 10 {
        for old_table in &tables[10..] {
            println!("Dropping old table: {}", old_table);
            db.drop_table(old_table).await?;
        }
    }
    
    Ok(())
}
```

## Testing

The database module includes comprehensive tests:

### Unit Tests
- Table name generation
- Data structure validation
- Error handling

### Integration Tests
- Full database operations (requires PostgreSQL)
- Concurrent access patterns
- Large batch processing
- Error scenarios

Run tests with:
```bash
# Unit tests (no database required)
cargo test database::tests

# Integration tests (requires DATABASE_URL)
export TEST_DATABASE_URL="postgresql://postgres:password@localhost/test_db"
cargo test --test database_integration_tests
```

## Environment Variables

- `DATABASE_URL` - PostgreSQL connection string for production
- `TEST_DATABASE_URL` - PostgreSQL connection string for testing

Example:
```bash
export DATABASE_URL="postgresql://username:password@localhost:5432/github_pg_query"
export TEST_DATABASE_URL="postgresql://username:password@localhost:5432/github_pg_query_test"
```

## Migration Strategy

The database module automatically creates tables and indexes as needed. For production deployments:

1. Ensure PostgreSQL is running and accessible
2. Set appropriate `DATABASE_URL`
3. The application will create the `query_history` table on first run
4. Repository tables are created dynamically as needed

## Security Considerations

- SQL injection prevention through parameterized queries
- Table name validation for drop operations
- Connection string security (use environment variables)
- Proper error handling to avoid information leakage