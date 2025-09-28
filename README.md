# GitHub PostgreSQL Query Tool

A powerful Rust-based tool that executes GitHub API search queries and stores results in PostgreSQL for analysis and research.

## 🚀 Quick Start

**Get running in 5 minutes:**

1. **Setup Database**:
   ```bash
   # One-command setup (creates local PostgreSQL)
   ./scripts/setup-local-db.sh
   ```

2. **Add GitHub Token**:
   ```bash
   # Edit .env file (created by setup script)
   GITHUB_TOKEN=your_github_token_here
   ```

3. **Build & Run**:
   ```bash
   cargo build --release
   ./target/release/github-pg-query "language:rust stars:>1000" --verbose
   ```

## 🎯 What It Does

- **Queries GitHub API** with any search syntax
- **Stores results** in timestamped PostgreSQL tables
- **Tracks everything** in a query history mega-table
- **Handles pagination** automatically
- **Provides analytics** on collected data

## 📊 Recent Session Results

**Session 01 (Sept 27, 2024):**
- ✅ 5 C repositories (10k+ stars, <500 lines)
- ✅ 3 C++ repositories (10k+ stars, <500 lines)  
- ✅ 300 Microsoft repositories (>100MB)
- ✅ All tracked in `query_history` table

## 🗃️ Database Architecture

### Query History (`query_history`)
Central tracking table for all queries:
```sql
SELECT search_query, table_name, result_count, executed_at 
FROM query_history ORDER BY executed_at DESC;
```

### Result Tables (`repos_YYYYMMDDHHMMSS`)
Timestamped tables with full repository data:
- Repository metadata & statistics
- Owner information & licensing
- Language, topics, and timestamps

## 🛠️ Database Management

```bash
./scripts/db-manage.sh status      # Check database status
./scripts/db-manage.sh connect     # Connect to database  
./scripts/db-manage.sh history     # Show query history
./scripts/db-manage.sh stats       # Database statistics
./scripts/db-manage.sh tables      # List all tables
```

## 📝 Usage Examples

```bash
# Popular repositories by language
./target/release/github-pg-query "language:python stars:>10000"

# Organization repositories
./target/release/github-pg-query "org:microsoft size:>100000" --per-page 100

# Topic-based search
./target/release/github-pg-query "topic:machine-learning"

# Size constraints
./target/release/github-pg-query "language:c stars:>=10000 size:<500"

# Multiple pages
./target/release/github-pg-query "org:google" --page 2 --per-page 100
```

## 🔧 Setup Options

### Option A: Docker (Recommended)
```bash
./scripts/setup-local-db.sh
```

### Option B: Native PostgreSQL (macOS)
```bash
./scripts/setup-native-postgres.sh
```

### Option C: Manual Setup
```bash
createdb github_pg_query
psql postgres -c "CREATE USER github_user WITH PASSWORD 'secure_password';"
```

## 📋 Requirements

- **Rust** 1.70+
- **PostgreSQL** 12+ (or Docker)
- **GitHub Token** with `public_repo` scope

## 🏗️ Architecture

- **Language**: Rust (async/await with tokio)
- **Database**: PostgreSQL with SQLx
- **API**: GitHub REST API v3
- **CLI**: clap with comprehensive options

## 📚 Documentation

- **[QUICKSTART.md](QUICKSTART.md)** - 5-minute setup guide
- **[Journal01.md](Journal01.md)** - Session logs and insights
- **[API.md](API.md)** - Detailed API documentation
- **[zzArchive/](zzArchive/)** - Additional documentation

## 🎯 Key Features

✅ **Audit Trail** - Every query tracked with metadata  
✅ **Performance Metrics** - Execution times and success rates  
✅ **Data Integrity** - Validation before database insertion  
✅ **Flexible Queries** - Full GitHub search syntax support  
✅ **Pagination Support** - Handle large result sets  
✅ **Error Handling** - Comprehensive error reporting  

## 🔍 Command Line Options

```
github-pg-query [OPTIONS] <QUERY>

Arguments:
  <QUERY>  GitHub search query

Options:
  -p, --per-page <COUNT>     Results per page (1-100) [default: 30]
      --page <NUMBER>        Page number [default: 1]
  -v, --verbose              Enable verbose output
      --dry-run              Validate without executing
      --history              Show query history
  -h, --help                 Print help
```

## 🌟 Example Queries

```bash
# Language-based searches
"language:rust stars:>5000"
"language:python created:>2023-01-01"

# Organization searches  
"org:microsoft"
"user:octocat language:javascript"

# Topic searches
"topic:machine-learning language:python"
"topic:web-framework stars:>1000"

# Complex queries
"language:rust topic:web-framework stars:>500"
"created:>2023-01-01 stars:>100 language:typescript"
```

---

**Latest Session:** 308 repositories collected across C/C++ and Microsoft projects  
**Database Status:** ✅ Operational with full audit trail  
**Next:** CSV exports and advanced analytics