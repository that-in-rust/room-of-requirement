# Database Migration Journal - Session 02

**Date:** September 27, 2024  
**Operation:** Move PostgreSQL database from Homebrew location to custom directory  
**Objective:** Relocate database to `/Users/neetipatni/desktop/PensieveDB01`

## Pre-Migration State

### Current Database Location
- **Original Path:** `/opt/homebrew/var/postgresql@15/`
- **Database Size:** 9.1MB (`github_pg_query` database)
- **Total PostgreSQL Cluster:** 72MB
- **Installation Method:** Homebrew PostgreSQL@15

### Current Database Contents
```sql
-- Tables and sizes before migration
repos_20250927220408 | 232 kB  (Microsoft repos page 2)
repos_20250927220430 | 232 kB  (Microsoft repos page 3)  
repos_20250927220142 | 232 kB  (Microsoft repos page 1)
repos_20250927214939 | 144 kB  (C++ repos)
repos_20250927212750 | 144 kB  (C repos)
query_history        | 80 kB   (Query tracking)
repos_20250927210726 | 72 kB   (Failed query table)
```

### Data Inventory
- **Total Repositories:** 308 repositories
- **Query History Records:** 6 queries tracked
- **Tables Created:** 7 tables (6 successful + 1 failed)
- **Time Period:** September 27, 2024 (single session)

## Migration Process

### Step 1: Stop PostgreSQL Service**Status
:** ✅ PostgreSQL service stopped successfully

### Step 2: Create Backup Before Migration**St
atus:** ✅ Complete backup created (71MB)
**Backup Location:** `/Users/neetipatni/desktop/PensieveDB01/postgresql_backup_original`

### Step 3: Move Database Files to New Location**
Status:** ✅ Database files moved successfully
**New Location:** `/Users/neetipatni/desktop/PensieveDB01/postgresql@15`

### Step 4: Create Symbolic Link for Homebrew Compatibility**Statu
s:** ✅ Symbolic link created successfully
**Link:** `/opt/homebrew/var/postgresql@15` → `/Users/neetipatni/desktop/PensieveDB01/postgresql@15`

### Step 5: Restart PostgreSQL Service*
*Status:** ✅ PostgreSQL service restarted successfully

### Step 6: Verify Database Integrity and Accessibility