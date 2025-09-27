use chrono::Utc;
use github_pg_query::{
    DatabaseManager, QueryMetadata, Repository, RepositoryLicense, RepositoryOwner,
};

/// Example demonstrating database operations
/// 
/// This example shows how to:
/// 1. Create a database manager
/// 2. Generate timestamped table names
/// 3. Create repository tables
/// 4. Insert repository data with conflict handling
/// 5. Track query metadata
/// 6. Get table statistics
/// 
/// Note: This example requires a PostgreSQL database to be running.
/// Set the DATABASE_URL environment variable to connect to your database.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get database URL from environment or use default
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:password@localhost/github_pg_query".to_string());

    println!("Connecting to database: {}", database_url);

    // Create database manager
    let db = match DatabaseManager::new(&database_url).await {
        Ok(db) => {
            println!("✅ Successfully connected to database");
            db
        }
        Err(e) => {
            eprintln!("❌ Failed to connect to database: {}", e);
            eprintln!("Make sure PostgreSQL is running and DATABASE_URL is set correctly");
            return Err(e.into());
        }
    };

    // Generate a timestamped table name
    let table_name = DatabaseManager::generate_table_name();
    println!("📅 Generated table name: {}", table_name);

    // Create repository table
    println!("🔨 Creating repository table...");
    db.create_repository_table(&table_name).await?;
    println!("✅ Repository table created successfully");

    // Create sample repository data
    let repositories = vec![
        create_sample_repository(1, "rust-lang/rust"),
        create_sample_repository(2, "tokio-rs/tokio"),
        create_sample_repository(3, "serde-rs/serde"),
    ];

    println!("📝 Inserting {} repositories...", repositories.len());

    // Insert repositories
    let inserted_count = db.insert_repositories(&table_name, &repositories).await?;
    println!("✅ Inserted {} repositories", inserted_count);

    // Create and save query metadata
    let mut metadata = QueryMetadata::new(
        "rust language:rust stars:>1000".to_string(),
        table_name.clone(),
    );
    metadata.mark_success(repositories.len() as i64, 1500);

    println!("💾 Saving query metadata...");
    db.save_query_metadata(&metadata).await?;
    println!("✅ Query metadata saved");

    // Get table statistics
    println!("📊 Getting table statistics...");
    let stats = db.get_table_stats(&table_name).await?;
    println!("📈 Table Statistics:");
    println!("  - Total repositories: {}", stats.total_repositories);
    println!("  - Unique languages: {}", stats.unique_languages);
    println!("  - Unique owners: {}", stats.unique_owners);
    println!("  - Average stars: {:.1}", stats.avg_stars);
    println!("  - Max stars: {}", stats.max_stars);

    // List all repository tables
    println!("📋 Listing all repository tables...");
    let tables = db.list_repository_tables().await?;
    println!("Found {} repository tables:", tables.len());
    for table in &tables {
        println!("  - {}", table);
    }

    // Get query history
    println!("📚 Getting recent query history...");
    let history = db.get_query_history(Some(5), false).await?;
    println!("Recent queries:");
    for query in &history {
        let status = if query.success { "✅" } else { "❌" };
        println!("  {} {} -> {} ({} results, {}ms)", 
                 status, query.search_query, query.table_name, 
                 query.result_count, query.duration_ms);
    }

    // Test conflict handling by inserting the same repositories again
    println!("🔄 Testing conflict handling (inserting same repositories)...");
    let updated_count = db.insert_repositories(&table_name, &repositories).await?;
    println!("✅ Updated {} repositories (no duplicates created)", updated_count);

    // Verify table still has the same number of repositories
    let updated_stats = db.get_table_stats(&table_name).await?;
    println!("📊 Verified: Still {} repositories in table", updated_stats.total_repositories);

    // Cleanup (optional - comment out if you want to keep the data)
    println!("🧹 Cleaning up test table...");
    db.drop_table(&table_name).await?;
    println!("✅ Test table dropped");

    println!("🎉 Database operations demo completed successfully!");

    Ok(())
}

fn create_sample_repository(id: i64, full_name: &str) -> Repository {
    let parts: Vec<&str> = full_name.split('/').collect();
    let owner_login = parts[0];
    let repo_name = parts[1];

    Repository {
        id,
        full_name: full_name.to_string(),
        name: repo_name.to_string(),
        description: Some(format!("Sample repository: {}", full_name)),
        html_url: format!("https://github.com/{}", full_name),
        clone_url: format!("https://github.com/{}.git", full_name),
        ssh_url: format!("git@github.com:{}.git", full_name),
        size: 1024 + (id * 100), // Vary the size
        stargazers_count: 1000 + (id * 500), // Vary the stars
        watchers_count: 100 + (id * 50),
        forks_count: 50 + (id * 25),
        open_issues_count: 10 + (id * 5),
        language: Some("Rust".to_string()),
        default_branch: "main".to_string(),
        visibility: "public".to_string(),
        private: false,
        fork: false,
        archived: false,
        disabled: false,
        created_at: Utc::now() - chrono::Duration::days(365 * (id as i64)), // Vary creation dates
        updated_at: Utc::now() - chrono::Duration::days(30),
        pushed_at: Some(Utc::now() - chrono::Duration::days(7)),
        owner: RepositoryOwner {
            id: id + 1000,
            login: owner_login.to_string(),
            owner_type: "User".to_string(),
            avatar_url: format!("https://github.com/{}.png", owner_login),
            html_url: format!("https://github.com/{}", owner_login),
            site_admin: false,
        },
        license: Some(RepositoryLicense {
            key: "mit".to_string(),
            name: "MIT License".to_string(),
            spdx_id: Some("MIT".to_string()),
            url: Some("https://api.github.com/licenses/mit".to_string()),
        }),
        topics: vec![
            "rust".to_string(),
            "async".to_string(),
            "performance".to_string(),
        ],
        has_issues: true,
        has_projects: true,
        has_wiki: true,
        has_pages: false,
        has_downloads: true,
    }
}