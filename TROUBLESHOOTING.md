# Troubleshooting Guide

This guide covers common issues and their solutions when using the GitHub PostgreSQL Query tool.

## Environment Setup Issues

### GitHub Token Problems

#### Issue: "GITHUB_TOKEN environment variable is not set"
```
❌ Environment variable GITHUB_TOKEN is not set
```

**Solution:**
1. Go to [GitHub Settings > Personal Access Tokens](https://github.com/settings/tokens)
2. Click "Generate new token (classic)"
3. Select the `public_repo` scope for public repositories
4. Copy the generated token
5. Set the environment variable:
   ```bash
   export GITHUB_TOKEN="your_token_here"
   ```
6. Or use the command line option:
   ```bash
   github-pg-query "your query" --github-token "your_token_here"
   ```

#### Issue: "GitHub token appears to be too short"
```
❌ Authentication failed: GitHub token appears to be too short (minimum 10 characters)
```

**Solution:**
- Verify you copied the complete token
- GitHub tokens are typically 40+ characters long
- Check for extra spaces or newlines in the token

#### Issue: "Invalid or expired GitHub token"
```
❌ GitHub API authentication failed: Invalid or expired GitHub token
```

**Solutions:**
1. **Check token validity:**
   ```bash
   curl -H "Authorization: token $GITHUB_TOKEN" https://api.github.com/user
   ```
2. **Generate a new token** if the current one is expired
3. **Verify token permissions** - ensure it has `public_repo` scope
4. **Check for typos** in the token

### Database Connection Issues

#### Issue: "DATABASE_URL environment variable is not set"
```
❌ Environment variable DATABASE_URL is not set
```

**Solution:**
Set the database URL with proper format:
```bash
export DATABASE_URL="postgresql://username:password@hostname:port/database_name"
```

Examples:
```bash
# Local PostgreSQL with default port
export DATABASE_URL="postgresql://postgres:password@localhost:5432/github_data"

# Remote PostgreSQL
export DATABASE_URL="postgresql://user:pass@db.example.com:5432/mydb"

# PostgreSQL with SSL
export DATABASE_URL="postgresql://user:pass@localhost:5432/db?sslmode=require"
```

#### Issue: "Connection refused" or "Could not connect to server"
```
❌ Database error: Connection refused (os error 61)
```

**Solutions:**
1. **Check if PostgreSQL is running:**
   ```bash
   # Check if PostgreSQL is listening
   pg_isready -h localhost -p 5432
   
   # Check running processes
   ps aux | grep postgres
   ```

2. **Start PostgreSQL:**
   ```bash
   # macOS with Homebrew
   brew services start postgresql
   
   # Linux with systemd
   sudo systemctl start postgresql
   
   # Linux with service command
   sudo service postgresql start
   
   # Docker
   docker run --name postgres -e POSTGRES_PASSWORD=password -p 5432:5432 -d postgres
   ```

3. **Verify connection details:**
   ```bash
   # Test connection manually
   psql "postgresql://username:password@localhost:5432/database_name"
   ```

#### Issue: "Database does not exist"
```
❌ Database error: database "github_data" does not exist
```

**Solution:**
Create the database:
```bash
# Connect to PostgreSQL as superuser
psql -U postgres

# Create database
CREATE DATABASE github_data;

# Grant permissions (if needed)
GRANT ALL PRIVILEGES ON DATABASE github_data TO your_username;
```

#### Issue: "Authentication failed for user"
```
❌ Database error: password authentication failed for user "username"
```

**Solutions:**
1. **Verify credentials** in the DATABASE_URL
2. **Check PostgreSQL authentication configuration:**
   ```bash
   # Find pg_hba.conf location
   psql -U postgres -c "SHOW hba_file;"
   
   # Edit pg_hba.conf to allow password authentication
   # Change 'peer' or 'ident' to 'md5' for local connections
   ```
3. **Reset password:**
   ```bash
   # As PostgreSQL superuser
   psql -U postgres
   ALTER USER username PASSWORD 'new_password';
   ```

## GitHub API Issues

### Rate Limiting

#### Issue: "GitHub API rate limit exceeded"
```
❌ GitHub API rate limit exceeded: 2023-12-01 15:30:00 UTC
```

**Solutions:**
1. **Wait for rate limit reset** (shown in error message)
2. **Use authenticated requests** (higher limits):
   - Unauthenticated: 60 requests/hour
   - Authenticated: 5,000 requests/hour
3. **Check current rate limit:**
   ```bash
   curl -H "Authorization: token $GITHUB_TOKEN" https://api.github.com/rate_limit
   ```
4. **Use smaller page sizes** to make fewer requests
5. **Implement delays** between requests in scripts

#### Issue: "Search API rate limit exceeded"
```
❌ GitHub search API rate limit exceeded (30 requests per minute)
```

**Solutions:**
1. **Wait 60 seconds** before retrying
2. **Use more specific queries** to get better results with fewer requests
3. **Batch your searches** with appropriate delays

### Query Issues

#### Issue: "Invalid GitHub search query"
```
❌ Invalid search query: "invalid syntax" - The search is longer than 256 characters
```

**Solutions:**
1. **Shorten the query** to under 256 characters
2. **Use GitHub's search syntax** - see [GitHub Search Documentation](https://docs.github.com/en/search-github/searching-on-github/searching-for-repositories)
3. **Valid query examples:**
   ```bash
   # Language search
   "language:rust"
   
   # Star count
   "stars:>1000"
   
   # User repositories
   "user:octocat"
   
   # Combined criteria
   "language:python stars:>100 created:>2023-01-01"
   ```

#### Issue: "No repositories found"
```
⚠️  No repositories matched the search query
```

**Solutions:**
1. **Broaden your search criteria**
2. **Check for typos** in the query
3. **Verify the query works** on GitHub's web interface
4. **Try simpler queries** first

## Application Issues

### CLI Argument Issues

#### Issue: "Argument parsing failed"
```
❌ Configuration error: Argument parsing failed: error: The following required arguments were not provided: <QUERY>
```

**Solution:**
Provide a search query as the first argument:
```bash
github-pg-query "language:rust stars:>100"
```

#### Issue: "Invalid value for '--per-page'"
```
❌ Configuration error: Invalid value '150' for '--per-page <COUNT>': 150 is not in 1..=100
```

**Solution:**
Use a value between 1 and 100:
```bash
github-pg-query "rust" --per-page 50
```

### Performance Issues

#### Issue: Slow database operations
**Symptoms:**
- Long delays during repository insertion
- Timeouts during table creation

**Solutions:**
1. **Check database performance:**
   ```sql
   -- Check active connections
   SELECT count(*) FROM pg_stat_activity;
   
   -- Check table sizes
   SELECT schemaname, tablename, pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) as size
   FROM pg_tables WHERE schemaname = 'public';
   ```

2. **Optimize PostgreSQL configuration:**
   ```sql
   -- Increase work memory for large operations
   SET work_mem = '256MB';
   
   -- Increase maintenance work memory
   SET maintenance_work_mem = '1GB';
   ```

3. **Use connection pooling** (already implemented in the tool)

4. **Monitor disk space:**
   ```bash
   df -h
   ```

#### Issue: Memory usage growing during large queries
**Solutions:**
1. **Use smaller page sizes:**
   ```bash
   github-pg-query "large query" --per-page 10
   ```
2. **Process results in batches** rather than all at once
3. **Monitor system resources:**
   ```bash
   top -p $(pgrep github-pg-query)
   ```

## Network Issues

### Connectivity Problems

#### Issue: "Connection timeout" or "Network unreachable"
```
❌ HTTP request failed: connection timeout
```

**Solutions:**
1. **Check internet connectivity:**
   ```bash
   ping github.com
   curl -I https://api.github.com
   ```

2. **Check proxy settings** if behind corporate firewall:
   ```bash
   export HTTP_PROXY=http://proxy.company.com:8080
   export HTTPS_PROXY=http://proxy.company.com:8080
   ```

3. **Verify GitHub API accessibility:**
   ```bash
   curl https://api.github.com/zen
   ```

4. **Check firewall rules** that might block outbound connections

### SSL/TLS Issues

#### Issue: "SSL certificate verification failed"
```
❌ HTTP request failed: SSL certificate verification failed
```

**Solutions:**
1. **Update system certificates:**
   ```bash
   # macOS
   brew install ca-certificates
   
   # Ubuntu/Debian
   sudo apt-get update && sudo apt-get install ca-certificates
   
   # CentOS/RHEL
   sudo yum update ca-certificates
   ```

2. **Check system time** (SSL certificates are time-sensitive):
   ```bash
   date
   # Sync if necessary
   sudo ntpdate -s time.nist.gov
   ```

## Data Issues

### Table Management

#### Issue: "Table already exists"
```
❌ Database error: relation "repos_20231201143022" already exists
```

**This should not happen** due to timestamped table names, but if it does:

**Solutions:**
1. **Check system clock synchronization:**
   ```bash
   date
   timedatectl status  # Linux
   ```

2. **Manual cleanup:**
   ```sql
   -- List existing tables
   SELECT tablename FROM pg_tables WHERE schemaname = 'public' AND tablename LIKE 'repos_%';
   
   -- Drop specific table if needed
   DROP TABLE IF EXISTS repos_20231201143022;
   ```

3. **Restart the application** to generate a new timestamp

#### Issue: "Disk full" during large operations
```
❌ Database error: could not extend file: No space left on device
```

**Solutions:**
1. **Check disk space:**
   ```bash
   df -h
   ```

2. **Clean up old tables:**
   ```sql
   -- Find old repository tables
   SELECT tablename, pg_size_pretty(pg_total_relation_size(tablename)) as size
   FROM pg_tables 
   WHERE schemaname = 'public' AND tablename LIKE 'repos_%'
   ORDER BY tablename;
   
   -- Drop old tables (be careful!)
   DROP TABLE repos_20231101120000;  -- Example old table
   ```

3. **Archive old data** before dropping tables

## Debugging Tips

### Enable Verbose Output
Always use `--verbose` when troubleshooting:
```bash
github-pg-query "your query" --verbose
```

### Use Dry Run Mode
Test configuration without executing:
```bash
github-pg-query "your query" --dry-run
```

### Check Logs
The application outputs detailed error information. Capture it:
```bash
github-pg-query "your query" 2>&1 | tee debug.log
```

### Test Components Individually

1. **Test GitHub connectivity:**
   ```bash
   curl -H "Authorization: token $GITHUB_TOKEN" \
        "https://api.github.com/search/repositories?q=rust&per_page=1"
   ```

2. **Test database connectivity:**
   ```bash
   psql "$DATABASE_URL" -c "SELECT version();"
   ```

3. **Run examples:**
   ```bash
   cargo run --example github_client_demo
   cargo run --example database_demo
   ```

## Getting Additional Help

### Collect Debug Information
When reporting issues, include:

1. **Version information:**
   ```bash
   github-pg-query --version
   cargo --version
   psql --version
   ```

2. **Environment details:**
   ```bash
   echo "OS: $(uname -a)"
   echo "GitHub token length: ${#GITHUB_TOKEN}"
   echo "Database URL (masked): ${DATABASE_URL//:*@/:***@}"
   ```

3. **Error output** with `--verbose` flag

4. **Steps to reproduce** the issue

### Resources

- [GitHub API Documentation](https://docs.github.com/en/rest)
- [GitHub Search Syntax](https://docs.github.com/en/search-github/searching-on-github/searching-for-repositories)
- [PostgreSQL Documentation](https://www.postgresql.org/docs/)
- [Rust Documentation](https://doc.rust-lang.org/)

### Common Solutions Summary

| Issue Type | Quick Fix | Command |
|------------|-----------|---------|
| Missing GitHub token | Set environment variable | `export GITHUB_TOKEN="your_token"` |
| Database not running | Start PostgreSQL | `brew services start postgresql` |
| Invalid query | Check GitHub syntax | Visit GitHub search page to test |
| Rate limited | Wait or use auth token | Check `curl -H "Authorization: token $TOKEN" https://api.github.com/rate_limit` |
| Connection refused | Check network/firewall | `ping github.com` |
| Disk full | Clean up old tables | `df -h` and drop old `repos_*` tables |

Remember: Most issues are related to environment setup (GitHub token, database connection) or network connectivity. Start with the basics and work your way up to more complex debugging.