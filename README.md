# GitHub PostgreSQL Query Tool

A simple, robust CLI tool that executes GitHub API search queries and stores results in timestamped PostgreSQL tables.

## Features

- **Any GitHub Search Query**: Supports the full GitHub repository search syntax
- **Timestamped Tables**: Automatically creates tables with format `repos_YYYYMMDDHHMMSS`
- **Conflict Handling**: Handles duplicate repositories with upsert operations
- **Query History**: Tracks all queries with metadata and performance metrics
- **Rate Limiting**: Automatic retry with exponential backoff for GitHub API limits
- **Progress Indicators**: Real-time feedback on operation progress
- **Comprehensive Error Handling**: Clear, actionable error messages
- **Dry Run Mode**: Validate configuration without executing queries

## Quick Start

### Prerequisites

1. **PostgreSQL Database**: Running and accessible
2. **GitHub Token**: Personal access token with `public_repo` scope

### Installation

```bash
# Clone the repository
git clone <repository-url>
cd github-pg-query

# Build the project
cargo build --release

# The binary will be available at target/release/github-pg-query
```

### Setup

1. **Create GitHub Token**:
   - Go to [GitHub Settings > Tokens](https://github.com/settings/tokens)
   - Generate a new token with `public_repo` scope
   - Copy the token

2. **Set Up Local Database** (Choose one option):

   **Option A: Docker (Recommended - Easiest)**
   ```bash
   # Run the setup script
   ./scripts/setup-local-db.sh
   ```

   **Option B: Native PostgreSQL (macOS)**
   ```bash
   # Install and configure PostgreSQL via Homebrew
   ./scripts/setup-native-postgres.sh
   ```

   **Option C: Manual Setup**
   ```bash
   # Install PostgreSQL, then:
   createdb github_pg_query
   psql postgres -c "CREATE USER github_user WITH PASSWORD 'secure_password';"
   psql postgres -c "GRANT ALL PRIVILEGES ON DATABASE github_pg_query TO github_user;"
   ```

3. **Set Environment Variables**:
   ```bash
   # The setup scripts create a .env file, or set manually:
   export GITHUB_TOKEN="your_github_token_here"
   export DATABASE_URL="postgresql://github_user:secure_password@localhost:5432/github_pg_query"
   ```

3. **Test Configuration**:
   ```bash
   ./target/release/github-pg-query "rust language:rust" --dry-run
   ```

### Basic Usage

```bash
# Search for Rust repositories with more than 1000 stars
./target/release/github-pg-query "language:rust stars:>1000"

# Search with pagination
./target/release/github-pg-query "user:octocat" --per-page 50 --page 2

# Verbose output with detailed progress
./target/release/github-pg-query "topic:machine-learning" --verbose

# Dry run to validate without executing
./target/release/github-pg-query "created:>2023-01-01" --dry-run
```

## GitHub Search Query Examples

The tool supports any valid GitHub repository search syntax:

### Language-Based Searches
```bash
# Rust repositories
github-pg-query "language:rust"

# JavaScript repositories with more than 100 stars
github-pg-query "language:javascript stars:>100"

# Python repositories created in 2023
github-pg-query "language:python created:>2023-01-01"
```

### Popularity-Based Searches
```bash
# Repositories with more than 10,000 stars
github-pg-query "stars:>10000"

# Repositories with 100-1000 forks
github-pg-query "forks:100..1000"

# Most starred repositories this year
github-pg-query "stars:>1000 created:>2023-01-01"
```

### User and Organization Searches
```bash
# All repositories from a specific user
github-pg-query "user:octocat"

# Repositories from an organization
github-pg-query "org:rust-lang"

# User's Rust repositories
github-pg-query "user:octocat language:rust"
```

### Topic-Based Searches
```bash
# Machine learning repositories
github-pg-query "topic:machine-learning"

# Web framework repositories in Rust
github-pg-query "topic:web-framework language:rust"

# Multiple topics
github-pg-query "topic:api topic:rest"
```

### Date-Based Searches
```bash
# Repositories created after 2023
github-pg-query "created:>2023-01-01"

# Repositories updated in the last month
github-pg-query "pushed:>2023-11-01"

# Repositories created in a specific year
github-pg-query "created:2023-01-01..2023-12-31"
```

### Complex Queries
```bash
# Popular Rust web frameworks
github-pg-query "language:rust topic:web-framework stars:>500"

# Recent TypeScript projects with good documentation
github-pg-query "language:typescript has:wiki created:>2023-06-01"

# Active Python data science projects
github-pg-query "language:python topic:data-science pushed:>2023-10-01 stars:>100"
```

## Command Line Options

```
github-pg-query [OPTIONS] <QUERY>

Arguments:
  <QUERY>  GitHub search query (e.g., 'rust language:rust', 'stars:>1000')

Options:
  -p, --per-page <COUNT>     Number of results per page (1-100) [default: 30]
      --page <NUMBER>        Page number to retrieve (starts from 1) [default: 1]
  -v, --verbose              Enable verbose output with detailed progress information
      --dry-run              Validate configuration and query without executing the search
      --github-token <TOKEN> GitHub API token (overrides GITHUB_TOKEN environment variable)
      --database-url <URL>   PostgreSQL database URL (overrides DATABASE_URL environment variable)
  -h, --help                 Print help
  -V, --version              Print version
```

## Environment Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `GITHUB_TOKEN` | GitHub personal access token | `ghp_xxxxxxxxxxxxxxxxxxxx` |
| `DATABASE_URL` | PostgreSQL connection string | `postgresql://user:pass@localhost:5432/dbname` |

## Database Schema

### Repository Tables (`repos_YYYYMMDDHHMMSS`)

Each query creates a new timestamped table with the following schema:

```sql
CREATE TABLE repos_20231201143022 (
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

### Query History Table

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

## Output Examples

### Successful Query
```
🔄 Initializing GitHub client... ✅ GitHub client initialized
🔄 Connecting to database... ✅ Database connection established
🔄 Creating table: repos_20231201143022... ✅ Table repos_20231201143022 created
🔄 Searching GitHub: 'language:rust stars:>1000'... ✅ Found 30 repositories (total: 15420, page: 1)
🔄 Storing 30 repositories... ✅ Stored 30 repositories
🔄 Saving query metadata... ✅ Query metadata saved

🎉 Search completed successfully!
   Table name: repos_20231201143022
   Results: 30 repositories
   Total time: 3.45s
```

### Verbose Output
```
🔄 Initializing GitHub client
   ↳ Validating GitHub token format
   ↳ Creating HTTP client with rate limiting
✅ GitHub client initialized

🔄 Connecting to database
   ↳ Establishing connection pool
   ↳ Validating database schema
✅ Database connection established

🔄 Creating table: repos_20231201143022
   ↳ Generating table schema
   ↳ Creating indexes for performance
✅ Table repos_20231201143022 created

🔄 Searching GitHub: 'language:rust stars:>1000'
   ↳ Executing API request
   ↳ Processing search results
✅ Found 30 repositories (total: 15420, page: 1)
ℹ️  Search completed in 1.23s

🔄 Storing 30 repositories
   ↳ Validating repository data
   ↳ Inserting with conflict resolution
✅ Stored 30 repositories

🔄 Saving query metadata
   ↳ Recording query statistics
✅ Query metadata saved

🎉 Search completed successfully!
   Table name: repos_20231201143022
   Results: 30 repositories
   Total time: 3.45s
   Search time: 1.23s
   Query ID: 550e8400-e29b-41d4-a716-446655440000
```

## Error Handling

The tool provides clear, actionable error messages:

### Missing GitHub Token
```
❌ Environment variable GITHUB_TOKEN is not set

💡 To fix this:
   1. Go to https://github.com/settings/tokens
   2. Generate a new token with 'public_repo' scope
   3. Run: export GITHUB_TOKEN=your_token_here
   4. Or use: --github-token your_token_here
```

### Invalid Database URL
```
❌ Configuration error: DATABASE_URL must start with 'postgres://' or 'postgresql://'

💡 Run with --help to see all available options
```

### Rate Limiting
```
❌ GitHub API rate limit exceeded: 2023-12-01 15:30:00 UTC

💡 To fix this:
   1. Wait until the reset time or use authenticated requests for higher limits
   2. Implement exponential backoff in your application logic
```

## Performance and Limits

### GitHub API Limits
- **Unauthenticated**: 60 requests per hour
- **Authenticated**: 5,000 requests per hour
- **Search API**: 30 requests per minute (authenticated)

### Pagination
- **Maximum per page**: 100 repositories
- **Maximum pages**: 1000 (GitHub API limit)
- **Total results**: Up to 100,000 repositories per query

### Database Performance
- **Batch inserts**: Optimized for large result sets
- **Conflict handling**: Efficient upsert operations
- **Indexes**: Automatic creation for common queries
- **Connection pooling**: Prevents connection exhaustion

## Development

### Building from Source

```bash
# Clone the repository
git clone <repository-url>
cd github-pg-query

# Install dependencies
cargo build

# Run tests
cargo test

# Run with example
cargo run -- "language:rust stars:>100" --dry-run
```

### Running Tests

```bash
# Unit tests (no external dependencies)
cargo test --lib

# Integration tests (requires PostgreSQL)
export TEST_DATABASE_URL="postgresql://postgres:password@localhost/test_db"
cargo test --test database_integration_tests

# All tests
cargo test

# Performance benchmarks
cargo bench
```

### Examples

The `examples/` directory contains working demonstrations:

```bash
# GitHub client demo
export GITHUB_TOKEN="your_token"
cargo run --example github_client_demo

# Database operations demo
export DATABASE_URL="postgresql://user:pass@localhost/db"
cargo run --example database_demo

# Complete workflow demo
cargo run --example workflow_demo
```

## Troubleshooting

### Common Issues

#### "Connection refused" Database Error
```bash
# Check if PostgreSQL is running
pg_isready -h localhost -p 5432

# Start PostgreSQL (macOS with Homebrew)
brew services start postgresql

# Start PostgreSQL (Linux systemd)
sudo systemctl start postgresql
```

#### "Authentication failed" GitHub Error
```bash
# Verify token is set
echo $GITHUB_TOKEN

# Test token validity
curl -H "Authorization: token $GITHUB_TOKEN" https://api.github.com/user
```

#### "Invalid query" Error
- Check GitHub's [search syntax documentation](https://docs.github.com/en/search-github/searching-on-github/searching-for-repositories)
- Ensure query is properly quoted in shell
- Verify query length is under 256 characters

#### "Table already exists" Error
- This shouldn't happen due to timestamped table names
- If it does, check system clock synchronization
- Consider adding microseconds to table name generation

### Getting Help

1. **Check the logs**: Use `--verbose` for detailed output
2. **Validate configuration**: Use `--dry-run` to test setup
3. **Check GitHub status**: Visit [GitHub Status](https://www.githubstatus.com/)
4. **Database connectivity**: Test with `psql` or similar tools

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## Changelog

### v0.1.0
- Initial release
- GitHub API integration with rate limiting
- PostgreSQL storage with timestamped tables
- CLI interface with comprehensive error handling
- Query history tracking
- Comprehensive test suite