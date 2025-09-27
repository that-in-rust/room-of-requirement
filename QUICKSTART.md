# Quick Start Guide

Get up and running with the GitHub PostgreSQL Query tool in under 5 minutes.

## Prerequisites

- [Docker](https://docs.docker.com/get-docker/) (recommended) OR PostgreSQL installed locally
- [Rust](https://rustup.rs/) (1.70+)
- GitHub Personal Access Token

## 1. Clone and Setup Database

```bash
# Clone the repository
git clone <your-repo-url>
cd github-pg-query

# Set up local database (Docker - easiest option)
./scripts/setup-local-db.sh
```

This script will:
- Start a PostgreSQL container
- Create the database and user
- Generate a `.env` file template

## 2. Get GitHub Token

1. Go to [GitHub Settings > Personal Access Tokens](https://github.com/settings/tokens)
2. Click "Generate new token (classic)"
3. Select scope: `public_repo`
4. Copy the generated token

## 3. Configure Environment

```bash
# Edit the .env file and add your GitHub token
nano .env

# Or set it directly:
export GITHUB_TOKEN="your_github_token_here"
```

Your `.env` file should look like:
```bash
DATABASE_URL=postgresql://github_user:secure_password@localhost:5432/github_pg_query
GITHUB_TOKEN=ghp_your_actual_token_here
```

## 4. Build and Test

```bash
# Build the application
cargo build --release

# Test configuration (dry run)
cargo run -- "language:rust" --dry-run

# Run your first query
cargo run -- "language:rust stars:>1000" --per-page 5 --verbose
```

## 5. Verify Results

```bash
# Connect to database
docker-compose exec postgres psql -U github_user -d github_pg_query

# List tables
\dt

# View data
SELECT full_name, stargazers_count, language FROM repos_20231201143022 LIMIT 5;

# Exit
\q
```

## Common Commands

```bash
# Search repositories
cargo run -- "language:python stars:>100"

# Search with more results
cargo run -- "topic:machine-learning" --per-page 50

# View query history
cargo run -- --history

# Get help
cargo run -- --help
```

## Troubleshooting

### Database Connection Issues
```bash
# Check if PostgreSQL is running
docker-compose ps

# Restart database
docker-compose restart postgres

# View logs
docker-compose logs postgres
```

### GitHub API Issues
```bash
# Test your token
curl -H "Authorization: token $GITHUB_TOKEN" https://api.github.com/user

# Check rate limits
curl -H "Authorization: token $GITHUB_TOKEN" https://api.github.com/rate_limit
```

### Build Issues
```bash
# Clean and rebuild
cargo clean
cargo build --release
```

## Next Steps

- Read [EXAMPLES.md](EXAMPLES.md) for more query examples
- Check [API.md](API.md) for detailed API documentation
- See [SETUP.md](SETUP.md) for comprehensive setup guide

## Alternative Database Setup

If you prefer not to use Docker:

### macOS (Homebrew)
```bash
./scripts/setup-native-postgres.sh
```

### Manual Setup
```bash
# Install PostgreSQL for your OS, then:
createdb github_pg_query
psql postgres -c "CREATE USER github_user WITH PASSWORD 'secure_password';"
psql postgres -c "GRANT ALL PRIVILEGES ON DATABASE github_pg_query TO github_user;"

# Set DATABASE_URL
export DATABASE_URL="postgresql://github_user:secure_password@localhost:5432/github_pg_query"
```

That's it! You should now have a working local setup for querying GitHub repositories and storing them in PostgreSQL.