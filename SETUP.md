# Setup Guide

This guide walks you through setting up the GitHub PostgreSQL Query tool from scratch.

## Prerequisites

### System Requirements
- **Operating System**: macOS, Linux, or Windows (with WSL)
- **Rust**: Version 1.70 or later
- **PostgreSQL**: Version 12 or later
- **Git**: For cloning the repository

### Hardware Requirements
- **RAM**: Minimum 2GB, recommended 4GB+
- **Disk Space**: 1GB for the application, additional space for PostgreSQL data
- **Network**: Internet connection for GitHub API access

## Step 1: Install Dependencies

### Install Rust
```bash
# Install Rust using rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Reload your shell or run:
source ~/.cargo/env

# Verify installation
rustc --version
cargo --version
```

### Install PostgreSQL

#### macOS (using Homebrew)
```bash
# Install Homebrew if not already installed
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install PostgreSQL
brew install postgresql

# Start PostgreSQL service
brew services start postgresql

# Create a database user (optional)
createuser -s postgres
```

#### Ubuntu/Debian
```bash
# Update package list
sudo apt update

# Install PostgreSQL
sudo apt install postgresql postgresql-contrib

# Start PostgreSQL service
sudo systemctl start postgresql
sudo systemctl enable postgresql

# Switch to postgres user and create database
sudo -u postgres psql
```

#### CentOS/RHEL/Fedora
```bash
# Install PostgreSQL
sudo dnf install postgresql postgresql-server postgresql-contrib

# Initialize database
sudo postgresql-setup --initdb

# Start and enable service
sudo systemctl start postgresql
sudo systemctl enable postgresql
```

#### Docker (Alternative)
```bash
# Run PostgreSQL in Docker
docker run --name postgres \
  -e POSTGRES_PASSWORD=password \
  -e POSTGRES_DB=github_pg_query \
  -p 5432:5432 \
  -d postgres:15

# Verify it's running
docker ps
```

### Verify PostgreSQL Installation
```bash
# Check if PostgreSQL is running
pg_isready -h localhost -p 5432

# Connect to PostgreSQL (adjust credentials as needed)
psql -h localhost -U postgres -d postgres
```

## Step 2: Create GitHub Personal Access Token

### Generate Token
1. Go to [GitHub Settings > Personal Access Tokens](https://github.com/settings/tokens)
2. Click "Generate new token (classic)"
3. Give it a descriptive name (e.g., "GitHub PG Query Tool")
4. Set expiration (recommend 90 days or no expiration for development)
5. Select scopes:
   - ✅ `public_repo` - Access public repositories
   - ✅ `read:org` - Read organization membership (optional, for org searches)
6. Click "Generate token"
7. **Copy the token immediately** (you won't see it again)

### Token Permissions Explained
- **`public_repo`**: Required for searching public repositories
- **`repo`**: Only needed if you want to search private repositories you have access to
- **`read:org`**: Helpful for organization-based searches
- **No other permissions needed** for basic functionality

### Verify Token
```bash
# Test your token (replace YOUR_TOKEN with actual token)
curl -H "Authorization: token YOUR_TOKEN" https://api.github.com/user

# Check rate limits
curl -H "Authorization: token YOUR_TOKEN" https://api.github.com/rate_limit
```

## Step 3: Set Up Database

### Create Database
```bash
# Connect to PostgreSQL as superuser
psql -U postgres

# Create database for the application
CREATE DATABASE github_pg_query;

# Create a dedicated user (optional but recommended)
CREATE USER github_user WITH PASSWORD 'secure_password';

# Grant permissions
GRANT ALL PRIVILEGES ON DATABASE github_pg_query TO github_user;

# Exit psql
\q
```

### Test Database Connection
```bash
# Test connection with new user
psql -h localhost -U github_user -d github_pg_query

# Or test with postgres user
psql -h localhost -U postgres -d github_pg_query
```

## Step 4: Install the Application

### Clone Repository
```bash
# Clone the repository
git clone <repository-url>
cd github-pg-query

# Verify you have the source code
ls -la
```

### Build Application
```bash
# Build in release mode for better performance
cargo build --release

# The binary will be at target/release/github-pg-query
ls -la target/release/github-pg-query
```

### Optional: Install Globally
```bash
# Install to ~/.cargo/bin (make sure it's in your PATH)
cargo install --path .

# Or copy to system location
sudo cp target/release/github-pg-query /usr/local/bin/

# Verify installation
github-pg-query --version
```

## Step 5: Configure Environment Variables

### Set Environment Variables

#### Option 1: Export in Shell
```bash
# Set GitHub token
export GITHUB_TOKEN="your_github_token_here"

# Set database URL (adjust credentials and database name)
export DATABASE_URL="postgresql://github_user:secure_password@localhost:5432/github_pg_query"

# Verify they're set
echo $GITHUB_TOKEN | cut -c1-10  # Shows first 10 characters
echo $DATABASE_URL
```

#### Option 2: Create .env File
```bash
# Create .env file in the project directory
cat > .env << EOF
GITHUB_TOKEN=your_github_token_here
DATABASE_URL=postgresql://github_user:secure_password@localhost:5432/github_pg_query
EOF

# Make sure .env is in .gitignore (it should be already)
echo ".env" >> .gitignore
```

#### Option 3: Shell Profile (Persistent)
```bash
# Add to ~/.bashrc, ~/.zshrc, or ~/.profile
echo 'export GITHUB_TOKEN="your_github_token_here"' >> ~/.bashrc
echo 'export DATABASE_URL="postgresql://github_user:secure_password@localhost:5432/github_pg_query"' >> ~/.bashrc

# Reload shell configuration
source ~/.bashrc
```

### Environment Variable Formats

#### GitHub Token Formats
```bash
# Classic personal access token (starts with ghp_)
GITHUB_TOKEN="ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"

# Fine-grained personal access token (starts with github_pat_)
GITHUB_TOKEN="github_pat_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"

# GitHub App token (if using GitHub App)
GITHUB_TOKEN="ghs_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
```

#### Database URL Formats
```bash
# Basic format
DATABASE_URL="postgresql://username:password@hostname:port/database_name"

# Local PostgreSQL with default port
DATABASE_URL="postgresql://postgres:password@localhost:5432/github_pg_query"

# Remote PostgreSQL
DATABASE_URL="postgresql://user:pass@db.example.com:5432/mydb"

# With SSL (recommended for production)
DATABASE_URL="postgresql://user:pass@localhost:5432/db?sslmode=require"

# With connection pool settings
DATABASE_URL="postgresql://user:pass@localhost:5432/db?max_connections=20"
```

## Step 6: Test Installation

### Run Dry Run Test
```bash
# Test configuration without executing a query
./target/release/github-pg-query "language:rust" --dry-run

# Expected output:
# ✅ Dry run completed successfully - configuration is valid
```

### Run First Query
```bash
# Execute a simple query
./target/release/github-pg-query "language:rust stars:>1000" --per-page 5 --verbose

# This should:
# 1. Connect to GitHub API
# 2. Connect to PostgreSQL
# 3. Create a timestamped table
# 4. Fetch and store 5 repositories
# 5. Show success message with table name
```

### Verify Database Results
```bash
# Connect to database
psql "$DATABASE_URL"

# List tables (should see a repos_YYYYMMDDHHMMSS table)
\dt

# Check the data
SELECT full_name, stargazers_count, language FROM repos_20231201143022 LIMIT 5;

# Check query history
SELECT * FROM query_history ORDER BY executed_at DESC LIMIT 5;

# Exit
\q
```

## Step 7: Optional Configuration

### Create Shell Alias
```bash
# Add to ~/.bashrc or ~/.zshrc for convenience
echo 'alias ghpg="github-pg-query"' >> ~/.bashrc
source ~/.bashrc

# Now you can use:
ghpg "language:python stars:>100" --verbose
```

### Set Up Log Directory
```bash
# Create directory for logs
mkdir -p ~/logs/github-pg-query

# Create wrapper script with logging
cat > ~/bin/github-pg-query-logged << 'EOF'
#!/bin/bash
LOG_FILE="$HOME/logs/github-pg-query/$(date +%Y%m%d_%H%M%S).log"
github-pg-query "$@" 2>&1 | tee "$LOG_FILE"
EOF

chmod +x ~/bin/github-pg-query-logged
```

### Configure PostgreSQL for Performance
```sql
-- Connect to PostgreSQL as superuser
psql -U postgres

-- Optimize for the application's usage patterns
ALTER SYSTEM SET shared_buffers = '256MB';
ALTER SYSTEM SET work_mem = '64MB';
ALTER SYSTEM SET maintenance_work_mem = '256MB';
ALTER SYSTEM SET effective_cache_size = '1GB';

-- Reload configuration
SELECT pg_reload_conf();
```

## Troubleshooting Setup Issues

### Common Issues and Solutions

#### "Command not found: github-pg-query"
```bash
# Check if binary exists
ls -la target/release/github-pg-query

# Use full path
./target/release/github-pg-query --version

# Or add to PATH
export PATH="$PATH:$(pwd)/target/release"
```

#### "GITHUB_TOKEN environment variable is not set"
```bash
# Check if variable is set
echo $GITHUB_TOKEN

# Set it temporarily
export GITHUB_TOKEN="your_token_here"

# Or use command line option
github-pg-query "query" --github-token "your_token_here"
```

#### "Connection refused" Database Error
```bash
# Check if PostgreSQL is running
pg_isready -h localhost -p 5432

# Start PostgreSQL
brew services start postgresql  # macOS
sudo systemctl start postgresql  # Linux

# Check connection
psql -h localhost -U postgres -c "SELECT version();"
```

#### "Authentication failed" Database Error
```bash
# Check database URL format
echo $DATABASE_URL

# Test connection manually
psql "$DATABASE_URL" -c "SELECT 1;"

# Reset password if needed
psql -U postgres -c "ALTER USER github_user PASSWORD 'new_password';"
```

#### "Invalid or expired GitHub token"
```bash
# Test token manually
curl -H "Authorization: token $GITHUB_TOKEN" https://api.github.com/user

# Check token permissions
curl -H "Authorization: token $GITHUB_TOKEN" https://api.github.com/user/repos

# Generate new token if needed (see Step 2)
```

### Verification Checklist

Before considering setup complete, verify:

- [ ] Rust is installed and working (`rustc --version`)
- [ ] PostgreSQL is running (`pg_isready`)
- [ ] Database exists and is accessible (`psql "$DATABASE_URL"`)
- [ ] GitHub token is valid (`curl -H "Authorization: token $TOKEN" https://api.github.com/user`)
- [ ] Application builds successfully (`cargo build --release`)
- [ ] Environment variables are set (`echo $GITHUB_TOKEN $DATABASE_URL`)
- [ ] Dry run passes (`github-pg-query "test" --dry-run`)
- [ ] First query succeeds (`github-pg-query "language:rust" --per-page 1`)
- [ ] Data appears in database (`psql "$DATABASE_URL" -c "\dt"`)

## Next Steps

After successful setup:

1. **Read the Examples**: Check `EXAMPLES.md` for query examples
2. **Explore the API**: Try different GitHub search queries
3. **Analyze Data**: Use SQL to analyze the collected repository data
4. **Automate Collection**: Create scripts for regular data collection
5. **Monitor Usage**: Keep track of GitHub API rate limits

## Production Deployment

For production use, consider:

### Security
- Use environment variables or secret management systems
- Restrict database user permissions
- Use SSL connections for database
- Rotate GitHub tokens regularly

### Performance
- Optimize PostgreSQL configuration
- Monitor disk space usage
- Set up connection pooling
- Consider read replicas for analysis

### Monitoring
- Set up logging and monitoring
- Track API rate limit usage
- Monitor database performance
- Set up alerts for failures

### Backup
- Regular database backups
- Document recovery procedures
- Test backup restoration

## Getting Help

If you encounter issues during setup:

1. **Check the logs**: Use `--verbose` flag for detailed output
2. **Verify each step**: Go through the checklist above
3. **Check documentation**: Review `README.md` and `TROUBLESHOOTING.md`
4. **Test components individually**: Test GitHub API and database separately
5. **Check system resources**: Ensure adequate disk space and memory

For additional help, include this information when reporting issues:
- Operating system and version
- Rust version (`rustc --version`)
- PostgreSQL version (`psql --version`)
- Error messages (with `--verbose` output)
- Steps to reproduce the issue