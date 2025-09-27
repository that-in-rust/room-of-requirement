# GitHub Search Query Examples

This document provides comprehensive examples of GitHub search queries that work with the GitHub PostgreSQL Query tool.

## Table of Contents

- [Basic Searches](#basic-searches)
- [Language-Based Searches](#language-based-searches)
- [Popularity Searches](#popularity-searches)
- [User and Organization Searches](#user-and-organization-searches)
- [Date-Based Searches](#date-based-searches)
- [Topic-Based Searches](#topic-based-searches)
- [Repository Features](#repository-features)
- [Complex Combined Queries](#complex-combined-queries)
- [Pagination Examples](#pagination-examples)
- [Expected Output Examples](#expected-output-examples)

## Basic Searches

### Simple Text Search
```bash
# Search for repositories containing "rust" in name or description
github-pg-query "rust"

# Search for "machine learning" (use quotes for multi-word terms)
github-pg-query "machine learning"

# Search for exact phrase
github-pg-query "\"web framework\""
```

### Repository Name Search
```bash
# Repositories with "api" in the name
github-pg-query "api in:name"

# Repositories with "cli" in name or description
github-pg-query "cli in:name,description"

# Repositories with "docker" in README
github-pg-query "docker in:readme"
```

## Language-Based Searches

### Single Language
```bash
# All Rust repositories
github-pg-query "language:rust"

# All Python repositories
github-pg-query "language:python"

# All JavaScript repositories
github-pg-query "language:javascript"

# All TypeScript repositories
github-pg-query "language:typescript"

# All Go repositories
github-pg-query "language:go"
```

### Language with Additional Criteria
```bash
# Popular Rust repositories
github-pg-query "language:rust stars:>1000"

# Recent Python projects
github-pg-query "language:python created:>2023-01-01"

# Large JavaScript projects
github-pg-query "language:javascript size:>10000"

# Active TypeScript projects
github-pg-query "language:typescript pushed:>2023-10-01"
```

### Multiple Languages
```bash
# Rust or Go repositories
github-pg-query "language:rust OR language:go"

# Web technologies
github-pg-query "language:javascript OR language:typescript OR language:html"
```

## Popularity Searches

### Star-Based Searches
```bash
# Repositories with more than 1000 stars
github-pg-query "stars:>1000"

# Repositories with 100-1000 stars
github-pg-query "stars:100..1000"

# Repositories with exactly 500 stars
github-pg-query "stars:500"

# Most popular repositories (>10k stars)
github-pg-query "stars:>10000"

# Highly starred Rust projects
github-pg-query "language:rust stars:>5000"
```

### Fork-Based Searches
```bash
# Repositories with many forks
github-pg-query "forks:>100"

# Repositories with moderate fork count
github-pg-query "forks:10..100"

# Popular Python libraries (high fork count)
github-pg-query "language:python forks:>500"
```

### Size-Based Searches
```bash
# Large repositories (>100MB)
github-pg-query "size:>100000"

# Small repositories (<1MB)
github-pg-query "size:<1000"

# Medium-sized projects
github-pg-query "size:1000..50000"
```

## User and Organization Searches

### User Repositories
```bash
# All repositories from a specific user
github-pg-query "user:octocat"

# User's Rust repositories
github-pg-query "user:octocat language:rust"

# User's popular repositories
github-pg-query "user:torvalds stars:>100"

# Multiple users
github-pg-query "user:octocat OR user:defunkt"
```

### Organization Repositories
```bash
# All repositories from an organization
github-pg-query "org:rust-lang"

# Organization's popular projects
github-pg-query "org:microsoft stars:>1000"

# Multiple organizations
github-pg-query "org:google OR org:facebook"

# Organization's specific language projects
github-pg-query "org:hashicorp language:go"
```

### Owner Type Searches
```bash
# Only user repositories (not organizations)
github-pg-query "type:user language:rust"

# Only organization repositories
github-pg-query "type:org language:python"
```

## Date-Based Searches

### Creation Date
```bash
# Repositories created after 2023
github-pg-query "created:>2023-01-01"

# Repositories created in 2023
github-pg-query "created:2023-01-01..2023-12-31"

# Repositories created in the last month
github-pg-query "created:>2023-11-01"

# Recent Rust projects
github-pg-query "language:rust created:>2023-06-01"
```

### Last Updated
```bash
# Recently updated repositories
github-pg-query "pushed:>2023-11-01"

# Repositories updated this year
github-pg-query "pushed:>2023-01-01"

# Active Python projects
github-pg-query "language:python pushed:>2023-10-01"

# Stale repositories (not updated in 2 years)
github-pg-query "pushed:<2022-01-01"
```

### Specific Date Ranges
```bash
# Repositories from 2022
github-pg-query "created:2022-01-01..2022-12-31"

# Recently active popular projects
github-pg-query "stars:>1000 pushed:>2023-10-01"

# New popular Rust projects
github-pg-query "language:rust created:>2023-01-01 stars:>100"
```

## Topic-Based Searches

### Single Topics
```bash
# Machine learning repositories
github-pg-query "topic:machine-learning"

# Web framework repositories
github-pg-query "topic:web-framework"

# API-related repositories
github-pg-query "topic:api"

# CLI tool repositories
github-pg-query "topic:cli"

# Database-related repositories
github-pg-query "topic:database"
```

### Multiple Topics
```bash
# Machine learning in Python
github-pg-query "topic:machine-learning language:python"

# Web frameworks in Rust
github-pg-query "topic:web-framework language:rust"

# API and REST together
github-pg-query "topic:api topic:rest"

# DevOps tools
github-pg-query "topic:devops topic:automation"
```

### Popular Topics
```bash
# Popular machine learning projects
github-pg-query "topic:machine-learning stars:>1000"

# Recent blockchain projects
github-pg-query "topic:blockchain created:>2023-01-01"

# Active web development projects
github-pg-query "topic:web-development pushed:>2023-10-01"
```

## Repository Features

### Repository Characteristics
```bash
# Repositories with issues enabled
github-pg-query "has:issues"

# Repositories with wiki
github-pg-query "has:wiki"

# Repositories with GitHub Pages
github-pg-query "has:pages"

# Repositories with projects enabled
github-pg-query "has:projects"

# Repositories with downloads
github-pg-query "has:downloads"
```

### License-Based Searches
```bash
# MIT licensed repositories
github-pg-query "license:mit"

# Apache licensed repositories
github-pg-query "license:apache-2.0"

# GPL licensed repositories
github-pg-query "license:gpl"

# Popular MIT licensed Rust projects
github-pg-query "language:rust license:mit stars:>500"
```

### Repository Status
```bash
# Non-fork repositories only
github-pg-query "fork:false"

# Fork repositories only
github-pg-query "fork:true"

# Archived repositories
github-pg-query "archived:true"

# Non-archived repositories
github-pg-query "archived:false"

# Private repositories (requires appropriate token permissions)
github-pg-query "is:private"

# Public repositories
github-pg-query "is:public"
```

## Complex Combined Queries

### Web Development
```bash
# Popular React projects
github-pg-query "react language:javascript stars:>1000"

# Modern web frameworks
github-pg-query "topic:web-framework created:>2022-01-01 stars:>100"

# Full-stack JavaScript projects
github-pg-query "language:javascript topic:fullstack stars:>500"

# Vue.js ecosystem
github-pg-query "vue OR vuejs language:javascript stars:>200"
```

### Data Science and ML
```bash
# Popular Python ML libraries
github-pg-query "language:python topic:machine-learning stars:>2000"

# Recent data science projects
github-pg-query "topic:data-science created:>2023-01-01 language:python"

# Deep learning frameworks
github-pg-query "topic:deep-learning topic:neural-network stars:>1000"

# Jupyter notebook projects
github-pg-query "topic:jupyter-notebook language:python"
```

### DevOps and Infrastructure
```bash
# Kubernetes tools
github-pg-query "kubernetes topic:devops language:go"

# Docker-related projects
github-pg-query "docker topic:containerization stars:>100"

# Infrastructure as Code
github-pg-query "topic:infrastructure topic:terraform OR topic:ansible"

# Monitoring and observability
github-pg-query "topic:monitoring topic:observability language:go"
```

### Mobile Development
```bash
# React Native projects
github-pg-query "react-native language:javascript stars:>100"

# Flutter projects
github-pg-query "flutter language:dart stars:>50"

# iOS Swift projects
github-pg-query "language:swift topic:ios stars:>200"

# Android Kotlin projects
github-pg-query "language:kotlin topic:android stars:>100"
```

### Blockchain and Crypto
```bash
# Ethereum smart contracts
github-pg-query "ethereum language:solidity stars:>50"

# Blockchain projects in Rust
github-pg-query "language:rust topic:blockchain stars:>100"

# DeFi protocols
github-pg-query "topic:defi topic:ethereum stars:>200"

# Cryptocurrency projects
github-pg-query "topic:cryptocurrency created:>2023-01-01"
```

## Pagination Examples

### Basic Pagination
```bash
# First page (default)
github-pg-query "language:rust stars:>1000"

# Second page
github-pg-query "language:rust stars:>1000" --page 2

# Third page with 50 results per page
github-pg-query "language:rust stars:>1000" --per-page 50 --page 3

# Large page size (maximum 100)
github-pg-query "popular repositories" --per-page 100
```

### Systematic Data Collection
```bash
# Collect first 500 results (5 pages of 100 each)
for page in {1..5}; do
    github-pg-query "language:python stars:>1000" --per-page 100 --page $page
    sleep 2  # Respect rate limits
done

# Collect with smaller pages to avoid timeouts
for page in {1..10}; do
    github-pg-query "large query" --per-page 50 --page $page
done
```

## Expected Output Examples

### Successful Query Output
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

### Verbose Output Example
```bash
github-pg-query "language:rust stars:>1000" --verbose
```

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

### Dry Run Output
```bash
github-pg-query "language:rust" --dry-run
```

```
🔄 Dry run validation... 
   ↳ Validating GitHub token
   ↳ GitHub token is valid
   ↳ Validating database connection
   ↳ Database connection is valid
   ↳ Validating search query format
   ↳ Search query format is valid
✅ All validations passed

✅ Dry run completed successfully - configuration is valid
```

### No Results Output
```bash
github-pg-query "language:nonexistent-language"
```

```
🔄 Initializing GitHub client... ✅ GitHub client initialized
🔄 Connecting to database... ✅ Database connection established
🔄 Creating table: repos_20231201143023... ✅ Table repos_20231201143023 created
🔄 Searching GitHub: 'language:nonexistent-language'... ✅ Found 0 repositories (total: 0, page: 1)
⚠️  No repositories found... ⚠️  No repositories matched the search query

🎉 Search completed successfully!
   Table name: repos_20231201143023
   Results: 0 repositories
   Total time: 2.15s
```

## Database Query Examples

After running the tool, you can query the generated tables:

### Basic Queries
```sql
-- Count repositories by language
SELECT language, COUNT(*) as count 
FROM repos_20231201143022 
WHERE language IS NOT NULL 
GROUP BY language 
ORDER BY count DESC;

-- Top repositories by stars
SELECT full_name, stargazers_count, description 
FROM repos_20231201143022 
ORDER BY stargazers_count DESC 
LIMIT 10;

-- Repositories by owner type
SELECT owner_type, COUNT(*) as count 
FROM repos_20231201143022 
GROUP BY owner_type;
```

### Advanced Queries
```sql
-- Average stars by language
SELECT language, 
       COUNT(*) as repo_count,
       AVG(stargazers_count) as avg_stars,
       MAX(stargazers_count) as max_stars
FROM repos_20231201143022 
WHERE language IS NOT NULL 
GROUP BY language 
HAVING COUNT(*) > 5
ORDER BY avg_stars DESC;

-- Most active repositories (recent pushes)
SELECT full_name, stargazers_count, pushed_at
FROM repos_20231201143022 
WHERE pushed_at > NOW() - INTERVAL '30 days'
ORDER BY stargazers_count DESC;

-- Repositories with specific topics
SELECT full_name, topics, stargazers_count
FROM repos_20231201143022 
WHERE 'web-framework' = ANY(topics)
ORDER BY stargazers_count DESC;
```

## Performance Tips

### Optimizing Queries
```bash
# Use specific criteria to reduce result set
github-pg-query "language:rust stars:>1000" --per-page 30

# Instead of very broad queries
github-pg-query "rust" --per-page 100  # May timeout or hit limits
```

### Rate Limit Management
```bash
# Check current rate limit before large operations
curl -H "Authorization: token $GITHUB_TOKEN" https://api.github.com/rate_limit

# Use smaller page sizes for better reliability
github-pg-query "popular query" --per-page 20

# Add delays between requests in scripts
github-pg-query "query1" && sleep 2 && github-pg-query "query2"
```

### Database Performance
```sql
-- Create indexes for common queries
CREATE INDEX idx_repos_language ON repos_20231201143022(language);
CREATE INDEX idx_repos_stars ON repos_20231201143022(stargazers_count DESC);
CREATE INDEX idx_repos_created ON repos_20231201143022(created_at);

-- Analyze table for better query planning
ANALYZE repos_20231201143022;
```

## Best Practices

1. **Start Small**: Test queries with small result sets first
2. **Use Dry Run**: Always validate configuration with `--dry-run`
3. **Be Specific**: Use specific criteria to get relevant results
4. **Respect Limits**: Be mindful of GitHub API rate limits
5. **Monitor Progress**: Use `--verbose` for detailed feedback
6. **Plan Pagination**: For large datasets, plan your pagination strategy
7. **Clean Up**: Regularly clean up old repository tables to save space

## Common Query Patterns

### Research and Analysis
```bash
# Trending technologies
github-pg-query "created:>2023-01-01 stars:>100" --per-page 100

# Popular libraries in ecosystem
github-pg-query "language:rust topic:web-framework stars:>50"

# Active projects for contribution
github-pg-query "language:python has:issues pushed:>2023-10-01 stars:10..1000"
```

### Competitive Analysis
```bash
# Similar projects
github-pg-query "topic:your-domain language:your-language stars:>100"

# Market leaders
github-pg-query "topic:web-framework stars:>5000"

# Emerging projects
github-pg-query "topic:ai created:>2023-06-01 stars:>50"
```

### Learning and Education
```bash
# Beginner-friendly projects
github-pg-query "good-first-issues:>5 language:rust stars:100..1000"

# Educational repositories
github-pg-query "topic:tutorial topic:learning language:python"

# Example projects
github-pg-query "topic:example topic:demo stars:>10"
```

Remember to always respect GitHub's terms of service and rate limits when using these queries!