# GitHub API Client

The GitHub API client provides a robust, rate-limited interface to the GitHub Search API with comprehensive error handling and retry logic.

## Features

- **Authentication**: Secure token-based authentication
- **Rate Limiting**: Automatic rate limit detection with exponential backoff retry
- **Error Handling**: Comprehensive error mapping for all GitHub API responses
- **Validation**: Input validation and sanitization
- **Testing**: Extensive unit tests with mock HTTP responses
- **Async/Await**: Full async support with Tokio

## Quick Start

```rust
use github_pg_query::{GitHubClient, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Create client with GitHub token
    let client = GitHubClient::new("your_github_token".to_string())?;
    
    // Search for repositories
    let results = client.search_repositories(
        "language:rust stars:>1000",
        Some(10),  // per_page
        Some(1)    // page
    ).await?;
    
    println!("Found {} repositories", results.total_count);
    for repo in results.items {
        println!("- {} (⭐ {})", repo.full_name, repo.stargazers_count);
    }
    
    Ok(())
}
```

## Authentication

The client requires a GitHub personal access token or API token:

```rust
// From environment variable (recommended)
let token = std::env::var("GITHUB_TOKEN")?;
let client = GitHubClient::new(token)?;

// Validate token before use
client.validate_token().await?;
```

### Required Token Permissions

For public repository searches, no special permissions are required. For private repositories or higher rate limits, consider using a token with appropriate scopes.

## Search Queries

The client accepts any valid GitHub search query syntax:

```rust
// Language-specific search
client.search_repositories("language:rust", None, None).await?;

// Complex queries with multiple criteria
client.search_repositories(
    "language:rust stars:>1000 created:>2020-01-01",
    Some(50),
    Some(1)
).await?;

// Organization-specific search
client.search_repositories("org:rust-lang language:rust", None, None).await?;

// Topic-based search
client.search_repositories("topic:web-framework language:rust", None, None).await?;
```

### Query Validation

The client validates queries before sending requests:

- Empty queries are rejected
- Query length limits are enforced
- Invalid syntax is detected and reported

## Rate Limiting

The client automatically handles GitHub's rate limiting:

```rust
use github_pg_query::{GitHubClient, RateLimitConfig};

let config = RateLimitConfig {
    max_retries: 5,
    initial_backoff_ms: 2000,
    max_backoff_ms: 120000,
    backoff_multiplier: 2.0,
};

let results = client.search_repositories_with_config(
    "rust",
    None,
    None,
    &config
).await?;
```

### Rate Limit Features

- **Exponential Backoff**: Automatic retry with increasing delays
- **Jitter**: Random delay variation to prevent thundering herd
- **Respect Headers**: Uses GitHub's rate limit headers for optimal timing
- **Configurable**: Customizable retry behavior

## Error Handling

The client provides detailed error information:

```rust
match client.search_repositories("invalid query syntax", None, None).await {
    Ok(results) => { /* handle success */ }
    Err(AppError::InvalidQuery { query, reason }) => {
        println!("Query '{}' failed: {}", query, reason);
    }
    Err(AppError::RateLimit { reset_time }) => {
        println!("Rate limited until: {}", reset_time);
    }
    Err(AppError::Authentication { reason }) => {
        println!("Auth failed: {}", reason);
    }
    Err(e) => {
        println!("Other error: {}", e);
    }
}
```

### Error Types

- `InvalidQuery`: Malformed search queries
- `RateLimit`: API rate limit exceeded
- `Authentication`: Invalid or expired token
- `GitHubApi`: General API errors
- `Http`: Network connectivity issues
- `Json`: Response parsing errors

## Rate Limit Monitoring

Check your current rate limit status:

```rust
let status = client.get_rate_limit().await?;
println!("Remaining requests: {}/{}", status.remaining, status.limit);
println!("Reset time: {}", status.reset_at);
```

## Testing

The client includes comprehensive test coverage:

```bash
# Run all GitHub client tests
cargo test github

# Run with coverage
cargo test github --lib -- --nocapture
```

### Mock Testing

For integration tests, use the provided mock server:

```rust
use github_pg_query::github::tests::{MockGitHubServer, MockResponse};

#[tokio::test]
async fn test_custom_scenario() {
    let server = MockGitHubServer::new();
    let response = MockGitHubServer::create_success_response(vec![]);
    server.add_response(response).await;
    
    // Test your code with predictable responses
}
```

## Performance Considerations

- **Connection Pooling**: HTTP client reuses connections
- **Timeout Handling**: 30-second request timeout
- **Memory Efficiency**: Streaming JSON parsing for large responses
- **Async Design**: Non-blocking I/O operations

## Configuration

### Environment Variables

```bash
export GITHUB_TOKEN="your_token_here"
export GITHUB_API_URL="https://api.github.com"  # Optional, for testing
```

### Custom Configuration

```rust
// Custom base URL (for testing or GitHub Enterprise)
let client = GitHubClient::with_base_url(
    token,
    "https://api.github.example.com".to_string()
)?;

// Custom rate limiting
let config = RateLimitConfig {
    max_retries: 10,
    initial_backoff_ms: 500,
    max_backoff_ms: 300000,
    backoff_multiplier: 1.5,
};
```

## Examples

See the `examples/` directory for complete working examples:

- `github_client_demo.rs`: Basic usage demonstration
- Run with: `cargo run --example github_client_demo`

## API Reference

### GitHubClient

#### Methods

- `new(token: String) -> Result<GitHubClient>`
- `with_base_url(token: String, base_url: String) -> Result<GitHubClient>`
- `search_repositories(query: &str, per_page: Option<u32>, page: Option<u32>) -> Result<SearchResponse>`
- `search_repositories_with_config(query: &str, per_page: Option<u32>, page: Option<u32>, config: &RateLimitConfig) -> Result<SearchResponse>`
- `validate_token() -> Result<()>`
- `get_rate_limit() -> Result<RateLimitStatus>`

### RateLimitConfig

#### Fields

- `max_retries: u32` - Maximum retry attempts (default: 3)
- `initial_backoff_ms: u64` - Initial delay in milliseconds (default: 1000)
- `max_backoff_ms: u64` - Maximum delay in milliseconds (default: 60000)
- `backoff_multiplier: f64` - Exponential backoff multiplier (default: 2.0)

### RateLimitStatus

#### Fields

- `limit: u32` - Total rate limit
- `remaining: u32` - Remaining requests
- `reset_at: DateTime<Utc>` - When the rate limit resets

## Best Practices

1. **Token Security**: Store tokens in environment variables, never in code
2. **Error Handling**: Always handle rate limiting and authentication errors
3. **Query Optimization**: Use specific queries to reduce API usage
4. **Monitoring**: Check rate limit status before making many requests
5. **Testing**: Use mock responses for reliable testing

## Troubleshooting

### Common Issues

**Authentication Errors**
```
Error: GitHub API authentication failed: Invalid or expired GitHub token
```
- Verify your token is correct and has not expired
- Check token permissions for the repositories you're searching

**Rate Limiting**
```
Error: GitHub API rate limit exceeded: 2023-12-01 15:30:00 UTC
```
- Wait until the reset time or use authenticated requests for higher limits
- Implement exponential backoff in your application logic

**Invalid Queries**
```
Error: Invalid GitHub search query: "invalid syntax" - The search is longer than 256 characters
```
- Check GitHub's search syntax documentation
- Validate query length and format before sending

**Network Issues**
```
Error: HTTP request failed: connection timeout
```
- Check internet connectivity
- Verify GitHub API is accessible from your network
- Consider proxy configuration if needed