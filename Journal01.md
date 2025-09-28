# GitHub PostgreSQL Query Tool - Session Journal 01

**Date:** September 27, 2024  
**Session Duration:** ~2 hours  
**Objective:** Set up local database and query GitHub repositories for C/C++ and Microsoft projects

## Session Overview

This session focused on setting up a local PostgreSQL database for the GitHub PostgreSQL Query tool and executing several repository queries to collect data about C/C++ repositories and Microsoft organization projects.

## Setup Phase

### 1. Local Database Setup
- **Challenge:** User needed local database setup instead of relying on external database
- **Solution:** Created comprehensive local database setup scripts
- **Approach:** Attempted Docker setup first, fell back to native PostgreSQL on macOS

#### Files Created:
- `docker-compose.yml` - Docker PostgreSQL setup
- `init.sql` - Database initialization script
- `scripts/setup-local-db.sh` - Automated Docker setup script
- `scripts/setup-native-postgres.sh` - Native macOS PostgreSQL setup
- `scripts/db-manage.sh` - Database management utilities
- `QUICKSTART.md` - 5-minute setup guide

### 2. PostgreSQL Installation (macOS)
```bash
# Installed PostgreSQL 15 via Homebrew
brew install postgresql@15
brew services start postgresql@15

# Created database and user
createdb github_pg_query
psql postgres -c "CREATE USER github_user WITH PASSWORD 'secure_password';"
psql postgres -c "GRANT ALL PRIVILEGES ON DATABASE github_pg_query TO github_user;"
```

### 3. Environment Configuration
Created `.env` file with:
```bash
DATABASE_URL=postgresql://github_user:secure_password@localhost:5432/github_pg_query
GITHUB_TOKEN=github_pat_***REDACTED_FOR_SECURITY***
```

## Technical Issues Resolved

### 1. Multi-Statement SQL Issue
**Problem:** SQLx doesn't support multiple SQL statements in a single query
**Location:** `src/database.rs` - both `initialize_query_history_table()` and `create_repository_table()`
**Solution:** Split CREATE TABLE and CREATE INDEX statements into separate queries

**Before:**
```sql
CREATE TABLE IF NOT EXISTS query_history (...);
CREATE INDEX IF NOT EXISTS idx_query_history_executed_at ON query_history(executed_at);
CREATE INDEX IF NOT EXISTS idx_query_history_table_name ON query_history(table_name);
```

**After:**
```rust
// Create table first
sqlx::query(create_table_sql).execute(&self.pool).await?;

// Create indexes separately
for index_sql in indexes {
    sqlx::query(&index_sql).execute(&self.pool).await?;
}
```

### 2. Database Permissions Issue
**Problem:** `permission denied for schema public`
**Solution:** 
```sql
GRANT CREATE ON SCHEMA public TO github_user;
GRANT USAGE ON SCHEMA public TO github_user;
GRANT ALL PRIVILEGES ON TABLE query_history TO github_user;
ALTER TABLE query_history OWNER TO github_user;
```

### 3. GitHub Search Query Syntax
**Problem:** Invalid query syntax `"language:c OR language:cpp stars:>=10000 size:<500"`
**Solution:** Split into separate queries for C and C++ languages

## Queries Executed

### 1. C Repositories Query
```bash
./target/release/github-pg-query "language:c stars:>=10000 size:<500" --per-page 100 --verbose
```
**Results:**
- Table: `repos_20250927212750`
- Found: 5 repositories
- Query ID: `bd51f695-2a7d-40e0-9ca9-600d0665a815`
- Execution time: 0.83s

### 2. C++ Repositories Query
```bash
./target/release/github-pg-query "language:cpp stars:>=10000 size:<500" --per-page 100 --verbose
```
**Results:**
- Table: `repos_20250927214939`
- Found: 3 repositories
- Query ID: `d087655f-03af-4efb-a463-e611dad02f61`
- Execution time: 0.87s

### 3. Microsoft Organization Query (Multiple Pages)

#### Page 1:
```bash
./target/release/github-pg-query "org:microsoft size:>100000" --per-page 100 --verbose
```
**Results:**
- Table: `repos_20250927220142`
- Found: 100 repositories (total available: 541)
- Query ID: `fa8ec9d4-83fe-4c81-acf9-d8ca34f48b6f`
- Execution time: 3.49s

#### Page 2:
```bash
./target/release/github-pg-query "org:microsoft size:>100000" --per-page 100 --page 2 --verbose
```
**Results:**
- Table: `repos_20250927220408`
- Found: 100 repositories
- Query ID: `8f5a93f8-750c-4120-95db-f57a894e1ea6`
- Execution time: 3.41s

#### Page 3:
```bash
./target/release/github-pg-query "org:microsoft size:>100000" --per-page 100 --page 3 --verbose
```
**Results:**
- Table: `repos_20250927220430`
- Found: 100 repositories
- Query ID: `f8f88360-7181-48ec-8cc6-dd8aea2adc87`
- Execution time: 4.57s

## Data Analysis Insights

### Microsoft Repositories by Size (Top 10)
From the database query showing largest Microsoft repositories:

1. **microsoft/PhiCookBook** - 8,791.73 MB (Jupyter Notebook)
2. **microsoft/TypeScript** - 2,789.73 MB (TypeScript) - 106,192 stars
3. **microsoft/mcp-for-beginners** - 2,114.99 MB (Jupyter Notebook)
4. **microsoft/fluentui-charting-contrib** - 1,857.21 MB (TypeScript)
5. **microsoft/onnxruntime** - 1,364.24 MB (C++) - 17,970 stars
6. **microsoft/openjdk-jdk25u** - 1,285.36 MB (Java)
7. **microsoft/automated-brain-explanations** - 1,222.47 MB (Jupyter Notebook)
8. **microsoft/openjdk-jdk21u** - 1,136.45 MB (Java)
9. **microsoft/windows-rs** - 1,131.73 MB (Rust) - 11,600 stars
10. **microsoft/vscode** - 1,041.39 MB (TypeScript) - 177,056 stars

## Database Schema

### Tables Created:
1. **query_history** - Tracks all executed queries
2. **repos_YYYYMMDDHHMMSS** - Timestamped tables for each query result

### Key Fields:
- Repository metadata (name, description, URLs)
- Statistics (stars, forks, size, language)
- Owner information
- Timestamps and licensing info

## Files Modified/Created

### Core Application:
- `src/database.rs` - Fixed multi-statement SQL issues
- `.env` - Environment configuration
- `.gitignore` - Updated with database-specific exclusions

### Setup Scripts:
- `scripts/setup-local-db.sh` - Docker-based setup
- `scripts/setup-native-postgres.sh` - Native macOS setup
- `scripts/db-manage.sh` - Database management utilities

### Documentation:
- `QUICKSTART.md` - Quick setup guide
- `docker-compose.yml` - Docker configuration
- `init.sql` - Database initialization

## Next Steps Planned

1. **CSV Export:** Export Microsoft repository data to `/Users/neetipatni/desktop/PensieveDB01`
2. **Data Analysis:** Analyze repository patterns and trends
3. **Additional Queries:** Potentially fetch remaining Microsoft repositories (up to 500 total)

## Lessons Learned

1. **SQLx Limitations:** Multi-statement queries require separation
2. **PostgreSQL Permissions:** Proper user permissions crucial for table creation
3. **GitHub API Constraints:** Search syntax must be precise, OR operators not supported in complex queries
4. **Pagination Strategy:** Large result sets require multiple API calls with pagination

## Technical Stack Confirmed

- **Database:** PostgreSQL 15 (Homebrew)
- **Language:** Rust with SQLx
- **API:** GitHub REST API v3
- **Environment:** macOS with native PostgreSQL

## Performance Metrics

- **C repositories:** 5 results in 0.83s
- **C++ repositories:** 3 results in 0.87s  
- **Microsoft repos (100 each):** 3.41-4.57s per page
- **Total Microsoft repos collected:** 300 repositories
- **Database operations:** All successful after permission fixes

---

**Session Status:** ✅ Successful  
**Database Status:** ✅ Operational  
**Data Collected:** 308 repositories total (5 C + 3 C++ + 300 Microsoft)  
**Next Session:** CSV export and data analysis