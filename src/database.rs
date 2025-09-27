use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};

use crate::{AppError, QueryMetadata, Repository, Result};

/// Database operations manager for PostgreSQL
#[derive(Clone)]
pub struct DatabaseManager {
    pool: PgPool,
}

impl DatabaseManager {
    /// Create a new database manager with connection pool
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPool::connect(database_url)
            .await
            .map_err(|e| AppError::Database(e))?;

        let manager = Self { pool };

        // Initialize the query_history table
        manager.initialize_query_history_table().await?;

        Ok(manager)
    }

    /// Get a reference to the connection pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Generate a timestamped table name in the format repos_YYYYMMDDHHMMSS
    pub fn generate_table_name() -> String {
        let now = Utc::now();
        format!("repos_{}", now.format("%Y%m%d%H%M%S"))
    }

    /// Create the query_history table if it doesn't exist
    async fn initialize_query_history_table(&self) -> Result<()> {
        // Create the table
        let create_table_sql = r#"
            CREATE TABLE IF NOT EXISTS query_history (
                id UUID PRIMARY KEY,
                search_query TEXT NOT NULL,
                table_name VARCHAR(50) NOT NULL,
                result_count BIGINT NOT NULL DEFAULT 0,
                executed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                duration_ms BIGINT NOT NULL DEFAULT 0,
                success BOOLEAN NOT NULL DEFAULT FALSE,
                error_message TEXT
            )
        "#;

        sqlx::query(create_table_sql)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::table_creation("query_history", e.to_string()))?;

        // Create indexes separately
        let indexes = [
            "CREATE INDEX IF NOT EXISTS idx_query_history_executed_at ON query_history(executed_at)",
            "CREATE INDEX IF NOT EXISTS idx_query_history_table_name ON query_history(table_name)",
            "CREATE INDEX IF NOT EXISTS idx_query_history_success ON query_history(success)",
        ];

        for index_sql in indexes {
            sqlx::query(index_sql)
                .execute(&self.pool)
                .await
                .map_err(|e| AppError::table_creation("query_history", e.to_string()))?;
        }

        Ok(())
    }

    /// Create a dynamic table for storing repository data
    pub async fn create_repository_table(&self, table_name: &str) -> Result<()> {
        // Create the table first
        let create_table_sql = format!(
            r#"
            CREATE TABLE IF NOT EXISTS {} (
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
                topics TEXT[] DEFAULT '{{}}',
                has_issues BOOLEAN NOT NULL DEFAULT FALSE,
                has_projects BOOLEAN NOT NULL DEFAULT FALSE,
                has_wiki BOOLEAN NOT NULL DEFAULT FALSE,
                has_pages BOOLEAN NOT NULL DEFAULT FALSE,
                has_downloads BOOLEAN NOT NULL DEFAULT FALSE,
                fetched_at TIMESTAMPTZ DEFAULT NOW()
            )
            "#,
            table_name
        );

        sqlx::query(&create_table_sql)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::table_creation(table_name, e.to_string()))?;

        // Create indexes separately
        let indexes = [
            format!("CREATE INDEX IF NOT EXISTS idx_{}_github_id ON {}(github_id)", table_name, table_name),
            format!("CREATE INDEX IF NOT EXISTS idx_{}_full_name ON {}(full_name)", table_name, table_name),
            format!("CREATE INDEX IF NOT EXISTS idx_{}_language ON {}(language)", table_name, table_name),
            format!("CREATE INDEX IF NOT EXISTS idx_{}_stargazers ON {}(stargazers_count DESC)", table_name, table_name),
            format!("CREATE INDEX IF NOT EXISTS idx_{}_created_at ON {}(created_at)", table_name, table_name),
            format!("CREATE INDEX IF NOT EXISTS idx_{}_owner_login ON {}(owner_login)", table_name, table_name),
        ];

        for index_sql in indexes {
            sqlx::query(&index_sql)
                .execute(&self.pool)
                .await
                .map_err(|e| AppError::table_creation(table_name, e.to_string()))?;
        }

        Ok(())
    }

    /// Insert repositories into the specified table with conflict handling
    pub async fn insert_repositories(
        &self,
        table_name: &str,
        repositories: &[Repository],
    ) -> Result<i64> {
        if repositories.is_empty() {
            return Ok(0);
        }

        let mut inserted_count = 0i64;

        // Use a transaction for batch insertion
        let mut tx = self.pool.begin().await?;

        for repo in repositories {
            // Validate repository data before insertion
            repo.validate()?;

            let topics_array: Vec<String> = repo.topics.clone();

            let sql = format!(
                r#"
                INSERT INTO {} (
                    github_id, full_name, name, description, html_url, clone_url, ssh_url,
                    size_kb, stargazers_count, watchers_count, forks_count, open_issues_count,
                    language, default_branch, visibility, private, fork, archived, disabled,
                    created_at, updated_at, pushed_at,
                    owner_id, owner_login, owner_type, owner_avatar_url, owner_html_url, owner_site_admin,
                    license_key, license_name, license_spdx_id, license_url,
                    topics, has_issues, has_projects, has_wiki, has_pages, has_downloads
                ) VALUES (
                    $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19,
                    $20, $21, $22, $23, $24, $25, $26, $27, $28, $29, $30, $31, $32, $33, $34, $35, $36, $37, $38
                )
                ON CONFLICT (github_id) DO UPDATE SET
                    full_name = EXCLUDED.full_name,
                    name = EXCLUDED.name,
                    description = EXCLUDED.description,
                    html_url = EXCLUDED.html_url,
                    clone_url = EXCLUDED.clone_url,
                    ssh_url = EXCLUDED.ssh_url,
                    size_kb = EXCLUDED.size_kb,
                    stargazers_count = EXCLUDED.stargazers_count,
                    watchers_count = EXCLUDED.watchers_count,
                    forks_count = EXCLUDED.forks_count,
                    open_issues_count = EXCLUDED.open_issues_count,
                    language = EXCLUDED.language,
                    default_branch = EXCLUDED.default_branch,
                    visibility = EXCLUDED.visibility,
                    private = EXCLUDED.private,
                    fork = EXCLUDED.fork,
                    archived = EXCLUDED.archived,
                    disabled = EXCLUDED.disabled,
                    updated_at = EXCLUDED.updated_at,
                    pushed_at = EXCLUDED.pushed_at,
                    owner_login = EXCLUDED.owner_login,
                    owner_type = EXCLUDED.owner_type,
                    owner_avatar_url = EXCLUDED.owner_avatar_url,
                    owner_html_url = EXCLUDED.owner_html_url,
                    owner_site_admin = EXCLUDED.owner_site_admin,
                    license_key = EXCLUDED.license_key,
                    license_name = EXCLUDED.license_name,
                    license_spdx_id = EXCLUDED.license_spdx_id,
                    license_url = EXCLUDED.license_url,
                    topics = EXCLUDED.topics,
                    has_issues = EXCLUDED.has_issues,
                    has_projects = EXCLUDED.has_projects,
                    has_wiki = EXCLUDED.has_wiki,
                    has_pages = EXCLUDED.has_pages,
                    has_downloads = EXCLUDED.has_downloads,
                    fetched_at = NOW()
                "#,
                table_name
            );

            let result = sqlx::query(&sql)
                .bind(repo.id)
                .bind(&repo.full_name)
                .bind(&repo.name)
                .bind(&repo.description)
                .bind(&repo.html_url)
                .bind(&repo.clone_url)
                .bind(&repo.ssh_url)
                .bind(repo.size)
                .bind(repo.stargazers_count)
                .bind(repo.watchers_count)
                .bind(repo.forks_count)
                .bind(repo.open_issues_count)
                .bind(&repo.language)
                .bind(&repo.default_branch)
                .bind(&repo.visibility)
                .bind(repo.private)
                .bind(repo.fork)
                .bind(repo.archived)
                .bind(repo.disabled)
                .bind(repo.created_at)
                .bind(repo.updated_at)
                .bind(repo.pushed_at)
                .bind(repo.owner.id)
                .bind(&repo.owner.login)
                .bind(&repo.owner.owner_type)
                .bind(&repo.owner.avatar_url)
                .bind(&repo.owner.html_url)
                .bind(repo.owner.site_admin)
                .bind(repo.license.as_ref().map(|l| &l.key))
                .bind(repo.license.as_ref().map(|l| &l.name))
                .bind(repo.license.as_ref().map(|l| l.spdx_id.as_ref()).flatten())
                .bind(repo.license.as_ref().map(|l| l.url.as_ref()).flatten())
                .bind(&topics_array)
                .bind(repo.has_issues)
                .bind(repo.has_projects)
                .bind(repo.has_wiki)
                .bind(repo.has_pages)
                .bind(repo.has_downloads)
                .execute(&mut *tx)
                .await?;

            inserted_count += result.rows_affected() as i64;
        }

        tx.commit().await?;
        Ok(inserted_count)
    }

    /// Save query metadata to the query_history table
    pub async fn save_query_metadata(&self, metadata: &QueryMetadata) -> Result<()> {
        let sql = r#"
            INSERT INTO query_history (
                id, search_query, table_name, result_count, executed_at, 
                duration_ms, success, error_message
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (id) DO UPDATE SET
                result_count = EXCLUDED.result_count,
                duration_ms = EXCLUDED.duration_ms,
                success = EXCLUDED.success,
                error_message = EXCLUDED.error_message
        "#;

        sqlx::query(sql)
            .bind(metadata.id)
            .bind(&metadata.search_query)
            .bind(&metadata.table_name)
            .bind(metadata.result_count)
            .bind(metadata.executed_at)
            .bind(metadata.duration_ms)
            .bind(metadata.success)
            .bind(&metadata.error_message)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Get query history with optional filtering
    pub async fn get_query_history(
        &self,
        limit: Option<i64>,
        success_only: bool,
    ) -> Result<Vec<QueryMetadata>> {
        let mut sql = "SELECT * FROM query_history".to_string();
        
        if success_only {
            sql.push_str(" WHERE success = true");
        }
        
        sql.push_str(" ORDER BY executed_at DESC");
        
        if let Some(limit) = limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        let rows = sqlx::query(&sql).fetch_all(&self.pool).await?;

        let mut results = Vec::new();
        for row in rows {
            let metadata = QueryMetadata {
                id: row.get("id"),
                search_query: row.get("search_query"),
                table_name: row.get("table_name"),
                result_count: row.get("result_count"),
                executed_at: row.get("executed_at"),
                duration_ms: row.get("duration_ms"),
                success: row.get("success"),
                error_message: row.get("error_message"),
            };
            results.push(metadata);
        }

        Ok(results)
    }

    /// Get table statistics
    pub async fn get_table_stats(&self, table_name: &str) -> Result<TableStats> {
        // Check if table exists first
        let table_exists_sql = r#"
            SELECT EXISTS (
                SELECT FROM information_schema.tables 
                WHERE table_schema = 'public' 
                AND table_name = $1
            )
        "#;

        let exists: bool = sqlx::query_scalar(table_exists_sql)
            .bind(table_name)
            .fetch_one(&self.pool)
            .await?;

        if !exists {
            return Err(AppError::Database(sqlx::Error::RowNotFound));
        }

        let stats_sql = format!(
            r#"
            SELECT 
                COUNT(*) as total_repositories,
                COUNT(DISTINCT language) as unique_languages,
                COUNT(DISTINCT owner_login) as unique_owners,
                AVG(stargazers_count) as avg_stars,
                MAX(stargazers_count) as max_stars,
                MIN(created_at) as oldest_repo,
                MAX(created_at) as newest_repo
            FROM {}
            "#,
            table_name
        );

        let row = sqlx::query(&stats_sql).fetch_one(&self.pool).await?;

        Ok(TableStats {
            table_name: table_name.to_string(),
            total_repositories: row.get::<i64, _>("total_repositories"),
            unique_languages: row.get::<i64, _>("unique_languages"),
            unique_owners: row.get::<i64, _>("unique_owners"),
            avg_stars: row.get::<Option<f64>, _>("avg_stars").unwrap_or(0.0),
            max_stars: row.get::<i64, _>("max_stars"),
            oldest_repo: row.get::<Option<DateTime<Utc>>, _>("oldest_repo"),
            newest_repo: row.get::<Option<DateTime<Utc>>, _>("newest_repo"),
        })
    }

    /// List all repository tables
    pub async fn list_repository_tables(&self) -> Result<Vec<String>> {
        let sql = r#"
            SELECT table_name 
            FROM information_schema.tables 
            WHERE table_schema = 'public' 
            AND table_name LIKE 'repos_%'
            ORDER BY table_name DESC
        "#;

        let rows = sqlx::query(sql).fetch_all(&self.pool).await?;
        let tables = rows
            .into_iter()
            .map(|row| row.get::<String, _>("table_name"))
            .collect();

        Ok(tables)
    }

    /// Drop a repository table (for cleanup/testing)
    pub async fn drop_table(&self, table_name: &str) -> Result<()> {
        // Validate table name to prevent SQL injection
        if !table_name.starts_with("repos_") || !table_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(AppError::validation("table_name", "Invalid table name format"));
        }

        let sql = format!("DROP TABLE IF EXISTS {}", table_name);
        sqlx::query(&sql).execute(&self.pool).await?;
        Ok(())
    }

    /// Close the database connection pool
    pub async fn close(&self) {
        self.pool.close().await;
    }
}

/// Statistics for a repository table
#[derive(Debug, Clone)]
pub struct TableStats {
    pub table_name: String,
    pub total_repositories: i64,
    pub unique_languages: i64,
    pub unique_owners: i64,
    pub avg_stars: f64,
    pub max_stars: i64,
    pub oldest_repo: Option<DateTime<Utc>>,
    pub newest_repo: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_table_name() {
        let table_name = DatabaseManager::generate_table_name();
        assert!(table_name.starts_with("repos_"));
        assert_eq!(table_name.len(), 20); // "repos_" + 14 digits (YYYYMMDDHHMMSS)
        
        // Verify it contains only valid characters
        assert!(table_name.chars().all(|c| c.is_alphanumeric() || c == '_'));
    }

    #[test]
    fn test_table_stats_creation() {
        let stats = TableStats {
            table_name: "repos_20231201120000".to_string(),
            total_repositories: 100,
            unique_languages: 5,
            unique_owners: 25,
            avg_stars: 42.5,
            max_stars: 1000,
            oldest_repo: Some(Utc::now()),
            newest_repo: Some(Utc::now()),
        };
        
        assert_eq!(stats.table_name, "repos_20231201120000");
        assert_eq!(stats.total_repositories, 100);
        assert_eq!(stats.unique_languages, 5);
        assert_eq!(stats.unique_owners, 25);
        assert_eq!(stats.avg_stars, 42.5);
        assert_eq!(stats.max_stars, 1000);
    }

    #[test]
    fn test_table_name_format_consistency() {
        // Generate multiple table names and verify they're all different
        // (unless generated in the same second)
        let name1 = DatabaseManager::generate_table_name();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let name2 = DatabaseManager::generate_table_name();
        
        // Both should have correct format
        assert!(name1.starts_with("repos_"));
        assert!(name2.starts_with("repos_"));
        assert_eq!(name1.len(), 20);
        assert_eq!(name2.len(), 20);
        
        // They should be different (unless generated in same second)
        // This is a weak test but validates the timestamp component
        assert!(name1 <= name2); // Timestamps should be monotonic
    }
}