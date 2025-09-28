# PostgreSQL Command Reference & Database Operations

## 🎯 The Essence

**PostgreSQL database successfully set up, populated with 308 repositories, and managed through comprehensive command toolkit.**

- **Database Location**: `/opt/homebrew/var/postgresql@15/` (9.1MB)
- **Data Collected**: 308 repositories across C/C++ and Microsoft projects
- **Management**: Complete command reference for all operations

---

## 🚀 Quick Command Reference

### Essential Operations
```bash
# Database status and connection
./scripts/db-manage.sh status
./scripts/db-manage.sh connect

# View your data
./scripts/db-manage.sh history    # Query history
./scripts/db-manage.sh tables     # All tables
./scripts/db-manage.sh stats      # Database statistics
```

### Core PostgreSQL Commands
```bash
# Service management
brew services start postgresql@15
brew services stop postgresql@15
brew services restart postgresql@15

# Direct connection
psql -U github_user -d github_pg_query
psql -U neetipatni -d postgres    # Superuser access
```

---

## 📊 Database Architecture Overview

### Query Tracking System
```sql
-- Central audit table
SELECT search_query, table_name, result_count, executed_at 
FROM query_history ORDER BY executed_at DESC;

-- Current data inventory
Total Tables: 7 (6 successful queries + 1 failed)
Total Repositories: 308
Database Size: 9.1MB
```

### Data Distribution
- **Microsoft repositories**: 300 (3 tables, ~696KB)
- **C repositories**: 5 (1 table, 144KB)
- **C++ repositories**: 3 (1 table, 144KB)
- **Query metadata**: 6 records (80KB)

---

## 🛠️ Complete Command Inventory

### Database Setup Commands
```bash
# Initial setup (automated)
./scripts/setup-local-db.sh        # Docker setup
./scripts/setup-native-postgres.sh # Native macOS setup

# Manual setup
createdb github_pg_query
psql postgres -c "CREATE USER github_user WITH PASSWORD 'secure_password';"
psql postgres -c "GRANT ALL PRIVILEGES ON DATABASE github_pg_query TO github_user;"
psql postgres -c "GRANT CREATE ON SCHEMA public TO github_user;"
psql postgres -c "GRANT USAGE ON SCHEMA public TO github_user;"
```

### Database Management Commands
```bash
# Service operations
brew services list | grep postgres  # Check running services
pg_isready -h localhost -p 5432     # Test connectivity

# Database information
psql -U neetipatni -d postgres -c "SHOW data_directory;"
psql -U github_user -d github_pg_query -c "\dt"  # List tables
psql -U github_user -d github_pg_query -c "\du"  # List users
psql -U github_user -d github_pg_query -c "\l"   # List databases
```

### Data Analysis Commands
```sql
-- Query history analysis
SELECT 
    executed_at,
    search_query,
    table_name,
    result_count,
    duration_ms,
    success
FROM query_history 
ORDER BY executed_at DESC;

-- Repository data by size
SELECT 
    full_name,
    size_kb,
    ROUND(size_kb/1024.0, 2) as size_mb,
    stargazers_count,
    language
FROM repos_20250927220142 
ORDER BY size_kb DESC 
LIMIT 10;

-- Database size analysis
SELECT 
    schemaname,
    tablename,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) as size
FROM pg_tables 
WHERE schemaname = 'public'
ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC;
```

### Backup and Recovery Commands
```bash
# Create backup
pg_dump -U github_user github_pg_query > backup_$(date +%Y%m%d_%H%M%S).sql

# Restore from backup
psql -U github_user -d github_pg_query < backup_file.sql

# Full cluster backup
cp -R /opt/homebrew/var/postgresql@15/ /path/to/backup/location

# Database size monitoring
du -sh /opt/homebrew/var/postgresql@15/
psql -U neetipatni -d github_pg_query -c "
SELECT 
    datname as database_name,
    pg_size_pretty(pg_database_size(datname)) as size
FROM pg_database 
WHERE datname = 'github_pg_query';"
```

---

## 📁 Database Migration Attempt (Session 02)

### Migration Objective
**Goal**: Move database from `/opt/homebrew/var/postgresql@15/` to `/Users/neetipatni/desktop/PensieveDB01`

### Migration Process Executed
```bash
# 1. Stop service
brew services stop postgresql@15

# 2. Create backup
cp -R /opt/homebrew/var/postgresql@15/ /Users/neetipatni/desktop/PensieveDB01/postgresql_backup_original

# 3. Move database files
mv /opt/homebrew/var/postgresql@15/ /Users/neetipatni/desktop/PensieveDB01/postgresql@15

# 4. Create symbolic link
ln -s /Users/neetipatni/desktop/PensieveDB01/postgresql@15 /opt/homebrew/var/postgresql@15

# 5. Restart service
brew services start postgresql@15
```

### Migration Result
- **Status**: ⚠️ Partially successful
- **Issue**: PostgreSQL@14 was running instead of PostgreSQL@15
- **Resolution**: Restored from backup to original location
- **Current Location**: `/opt/homebrew/var/postgresql@15/` (original Homebrew location)
- **Backup Created**: Complete 71MB backup preserved in PensieveDB01

---

## 🔧 Troubleshooting Commands

### Connection Issues
```bash
# Check if PostgreSQL is running
pg_isready -h localhost -p 5432

# Check which PostgreSQL versions are running
brew services list | grep postgres

# Check data directory
psql -U neetipatni -d postgres -c "SHOW data_directory;"

# Test connection with different users
psql -U github_user -d github_pg_query -c "SELECT 1;"
psql -U neetipatni -d postgres -c "SELECT version();"
```

### Permission Issues
```sql
-- Grant necessary permissions
GRANT ALL PRIVILEGES ON DATABASE github_pg_query TO github_user;
GRANT CREATE ON SCHEMA public TO github_user;
GRANT USAGE ON SCHEMA public TO github_user;
ALTER TABLE query_history OWNER TO github_user;
GRANT ALL PRIVILEGES ON TABLE query_history TO github_user;
```

### Service Management Issues
```bash
# Stop all PostgreSQL services
brew services stop postgresql@14
brew services stop postgresql@15

# Check PostgreSQL processes
ps aux | grep postgres

# Restart specific version
brew services restart postgresql@15
```

---

## 📈 Performance and Monitoring

### Database Statistics
```sql
-- Overall database stats
SELECT 
    COUNT(*) as total_tables,
    pg_size_pretty(pg_database_size('github_pg_query')) as database_size
FROM information_schema.tables 
WHERE table_schema = 'public' 
AND table_name LIKE 'repos_%';

-- Query performance tracking
SELECT 
    COUNT(*) as total_queries,
    COUNT(*) FILTER (WHERE success) as successful_queries,
    COUNT(*) FILTER (WHERE NOT success) as failed_queries,
    AVG(duration_ms) as avg_duration_ms,
    SUM(result_count) as total_repositories_fetched
FROM query_history;

-- Table size analysis
SELECT 
    table_name,
    (SELECT COUNT(*) FROM information_schema.columns WHERE table_name = t.table_name) as column_count
FROM information_schema.tables t
WHERE table_schema = 'public' 
AND table_name LIKE 'repos_%'
ORDER BY table_name;
```

---

## 🗂️ File Locations and Paths

### Key Directories
- **Database Data**: `/opt/homebrew/var/postgresql@15/`
- **Backup Location**: `/Users/neetipatni/desktop/PensieveDB01/postgresql_backup_original`
- **PostgreSQL Binary**: `/opt/homebrew/opt/postgresql@15/bin/`
- **Configuration**: `/opt/homebrew/var/postgresql@15/postgresql.conf`

### Environment Variables
```bash
# Required for PATH
export PATH="/opt/homebrew/opt/postgresql@15/bin:$PATH"

# Database connection
export DATABASE_URL="postgresql://github_user:secure_password@localhost:5432/github_pg_query"
```

---

## 📝 Session Summary

### What Was Accomplished
- ✅ **Database Setup**: Native PostgreSQL@15 installation via Homebrew
- ✅ **Data Collection**: 308 repositories successfully stored
- ✅ **Query Tracking**: Complete audit trail in query_history table
- ✅ **Management Tools**: Comprehensive db-manage.sh script created
- ✅ **Backup Strategy**: Full backup created and preserved
- ⚠️ **Migration Attempt**: Attempted but reverted due to version conflicts

### Current State
- **Database**: Operational at original Homebrew location
- **Size**: 9.1MB (github_pg_query database)
- **Tables**: 7 tables with 308 repositories
- **Backup**: 71MB complete backup in PensieveDB01
- **Access**: Full command toolkit available

### Next Steps
- CSV export to PensieveDB01 directory
- Advanced analytics on collected data
- Potential future migration with proper version management