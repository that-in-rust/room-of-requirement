use github_pg_query::{DatabaseManager, QueryMetadata, Repository, RepositoryOwner, RepositoryLicense, TableStats};
use testcontainers::{clients::Cli, images::postgres::Postgres, Container};
use chrono::Utc;
use proptest::prelude::*;
use serial_test::serial;
use std::collections::HashMap;

// Test container setup
fn setup_test_database() -> (Cli, Container<'static, Postgres>) {
    let docker = Cli::default();
    let postgres_image = Postgres::default()
        .with_db_name("test_db")
        .with_user("test_user")
        .with_password("test_password");
    
    let container = docker.run(postgres_image);
    (docker, container)
}

async fn create_database_manager(container: &Container<'_, Postgres>) -> DatabaseManager {
    let port = container.get_host_port_ipv4(5432);
    let database_url = format!(
        "postgresql://test_user:test_password@localhost:{}/test_db",
        port
    );
    
    DatabaseManager::new(&database_url)
        .await
        .expect("Failed to create database manager")
}

fn create_test_repository(id: i64, full_name: &str) -> Repository {
    let name = full_name.split('/').last().unwrap_or("repo").to_string();
    let now = Utc::now();
    
    Repository {
        id,
        full_name: full_name.to_string(),
        name,
        description: Some("Test repository".to_string()),
        html_url: format!("https://github.com/{}", full_name),
        clone_url: format!("https://github.com/{}.git", full_name),
        ssh_url: format!("git@github.com:{}.git", full_name),
        size: 1024,
        stargazers_count: 42,
        watchers_count: 15,
        forks_count: 8,
        open_issues_count: 3,
        language: Some("Rust".to_string()),
        default_branch: "main".to_string(),
        visibility: "public".to_string(),
        private: false,
        fork: false,
        archived: false,
        disabled: false,
        created_at: now,
        updated_at: now,
        pushed_at: Some(now),
        owner: RepositoryOwner {
            id: id + 1000,
            login: full_name.split('/').next().unwrap_or("testuser").to_string(),
            owner_type: "User".to_string(),
            avatar_url: "https://github.com/images/error/octocat_happy.gif".to_string(),
            html_url: format!(
                "https://github.com/{}",
                full_name.split('/').next().unwrap_or("testuser")
            ),
            site_admin: false,
        },
        license: Some(RepositoryLicense {
            key: "mit".to_string(),
            name: "MIT License".to_string(),
            spdx_id: Some("MIT".to_string()),
            url: Some("https://api.github.com/licenses/mit".to_string()),
        }),
        topics: vec!["rust".to_string(), "cli".to_string()],
        has_issues: true,
        has_projects: true,
        has_wiki: false,
        has_pages: false,
        has_downloads: true,
    }
}

#[tokio::test]
#[serial]
async fn test_database_manager_initialization() {
    let (_docker, container) = setup_test_database();
    let db = create_database_manager(&container).await;

    // Test that the database manager was created successfully
    assert!(!db.pool().is_closed());

    // Test that query_history table exists
    let history = db.get_query_history(Some(1), false).await;
    assert!(history.is_ok());
}

#[tokio::test]
#[serial]
async fn test_table_name_generation() {
    let table_name1 = DatabaseManager::generate_table_name();
    
    // Small delay to ensure different timestamps
    tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
    
    let table_name2 = DatabaseManager::generate_table_name();

    // Both should have correct format
    assert!(table_name1.starts_with("repos_"));
    assert!(table_name2.starts_with("repos_"));
    assert_eq!(table_name1.len(), 20);
    assert_eq!(table_name2.len(), 20);

    // Should be different (unless generated in same second)
    assert!(table_name1 <= table_name2);
}

#[tokio::test]
#[serial]
async fn test_repository_table_lifecycle() {
    let (_docker, container) = setup_test_database();
    let db = create_database_manager(&container).await;
    let table_name = format!("repos_test_{}", fastrand::u64(..));

    // Test table creation
    let result = db.create_repository_table(&table_name).await;
    assert!(result.is_ok(), "Failed to create table: {:?}", result);

    // Verify table appears in list
    let tables = db.list_repository_tables().await.unwrap();
    assert!(tables.contains(&table_name));

    // Test table stats for empty table
    let stats = db.get_table_stats(&table_name).await.unwrap();
    assert_eq!(stats.total_repositories, 0);
    assert_eq!(stats.table_name, table_name);

    // Cleanup
    let result = db.drop_table(&table_name).await;
    assert!(result.is_ok());

    // Verify table is gone
    let tables = db.list_repository_tables().await.unwrap();
    assert!(!tables.contains(&table_name));
}

#[tokio::test]
#[serial]
async fn test_repository_insertion_and_conflict_handling() {
    let (_docker, container) = setup_test_database();
    let db = create_database_manager(&container).await;
    let table_name = format!("repos_test_{}", fastrand::u64(..));

    // Create table
    db.create_repository_table(&table_name).await.unwrap();

    // Create test repositories
    let repos = vec![
        create_test_repository(12345, "rust-lang/rust"),
        create_test_repository(67890, "tokio-rs/tokio"),
        create_test_repository(11111, "serde-rs/serde"),
    ];

    // Test initial insertion
    let inserted_count = db.insert_repositories(&table_name, &repos).await.unwrap();
    assert_eq!(inserted_count, 3);

    // Verify table stats
    let stats = db.get_table_stats(&table_name).await.unwrap();
    assert_eq!(stats.total_repositories, 3);

    // Test conflict handling - modify and re-insert
    let mut updated_repos = repos.clone();
    updated_repos[0].stargazers_count = 99999;
    updated_repos[0].description = Some("Updated description".to_string());

    let updated_count = db.insert_repositories(&table_name, &updated_repos).await.unwrap();
    assert_eq!(updated_count, 3); // Should update existing records

    // Verify still only 3 repositories
    let stats = db.get_table_stats(&table_name).await.unwrap();
    assert_eq!(stats.total_repositories, 3);

    // Test empty insertion
    let empty_count = db.insert_repositories(&table_name, &[]).await.unwrap();
    assert_eq!(empty_count, 0);

    // Cleanup
    db.drop_table(&table_name).await.unwrap();
}

#[tokio::test]
#[serial]
async fn test_query_metadata_operations() {
    let (_docker, container) = setup_test_database();
    let db = create_database_manager(&container).await;

    // Create test metadata
    let mut metadata1 = QueryMetadata::new(
        "rust language:rust stars:>1000".to_string(),
        "repos_20231201120000".to_string(),
    );
    metadata1.mark_success(150, 2500);

    let mut metadata2 = QueryMetadata::new(
        "javascript language:javascript".to_string(),
        "repos_20231201130000".to_string(),
    );
    metadata2.mark_failure("API rate limit exceeded".to_string(), 1000);

    // Save metadata
    db.save_query_metadata(&metadata1).await.unwrap();
    db.save_query_metadata(&metadata2).await.unwrap();

    // Test retrieving all history
    let all_history = db.get_query_history(None, false).await.unwrap();
    assert!(all_history.len() >= 2);

    // Test retrieving successful queries only
    let success_history = db.get_query_history(None, true).await.unwrap();
    let successful_queries: Vec<_> = success_history
        .iter()
        .filter(|h| h.id == metadata1.id)
        .collect();
    assert_eq!(successful_queries.len(), 1);

    // Test limited results
    let limited_history = db.get_query_history(Some(1), false).await.unwrap();
    assert_eq!(limited_history.len(), 1);

    // Verify metadata content
    let found_metadata = all_history
        .iter()
        .find(|h| h.id == metadata1.id)
        .expect("Metadata not found");

    assert_eq!(found_metadata.search_query, metadata1.search_query);
    assert_eq!(found_metadata.table_name, metadata1.table_name);
    assert_eq!(found_metadata.result_count, metadata1.result_count);
    assert_eq!(found_metadata.duration_ms, metadata1.duration_ms);
    assert_eq!(found_metadata.success, metadata1.success);
    assert_eq!(found_metadata.error_message, metadata1.error_message);
}

#[tokio::test]
#[serial]
async fn test_table_statistics() {
    let (_docker, container) = setup_test_database();
    let db = create_database_manager(&container).await;
    let table_name = format!("repos_test_{}", fastrand::u64(..));

    // Create table
    db.create_repository_table(&table_name).await.unwrap();

    // Create diverse test repositories
    let repos = vec![
        {
            let mut repo = create_test_repository(1, "user1/rust-project");
            repo.stargazers_count = 1000;
            repo.language = Some("Rust".to_string());
            repo
        },
        {
            let mut repo = create_test_repository(2, "user2/js-project");
            repo.stargazers_count = 500;
            repo.language = Some("JavaScript".to_string());
            repo
        },
        {
            let mut repo = create_test_repository(3, "user1/python-project");
            repo.stargazers_count = 2000;
            repo.language = Some("Python".to_string());
            repo
        },
        {
            let mut repo = create_test_repository(4, "user3/go-project");
            repo.stargazers_count = 750;
            repo.language = Some("Go".to_string());
            repo
        },
    ];

    // Insert repositories
    db.insert_repositories(&table_name, &repos).await.unwrap();

    // Get and verify statistics
    let stats = db.get_table_stats(&table_name).await.unwrap();

    assert_eq!(stats.table_name, table_name);
    assert_eq!(stats.total_repositories, 4);
    assert_eq!(stats.unique_languages, 4);
    assert_eq!(stats.unique_owners, 3);
    assert_eq!(stats.max_stars, 2000);
    assert_eq!(stats.avg_stars, 1312.5);

    // Verify timestamp fields are populated
    assert!(stats.oldest_repo.is_some());
    assert!(stats.newest_repo.is_some());

    // Cleanup
    db.drop_table(&table_name).await.unwrap();
}

#[tokio::test]
#[serial]
async fn test_nonexistent_table_operations() {
    let (_docker, container) = setup_test_database();
    let db = create_database_manager(&container).await;

    // Test stats for nonexistent table
    let result = db.get_table_stats("nonexistent_table").await;
    assert!(result.is_err());

    // Test insertion into nonexistent table
    let repos = vec![create_test_repository(1, "test/repo")];
    let result = db.insert_repositories("nonexistent_table", &repos).await;
    assert!(result.is_err());
}

#[tokio::test]
#[serial]
async fn test_table_name_validation() {
    let (_docker, container) = setup_test_database();
    let db = create_database_manager(&container).await;

    // Test invalid table names for drop operation
    let invalid_names = vec![
        "invalid-table-name",
        "users_table",
        "repos_; DROP TABLE users; --",
        "",
    ];

    for invalid_name in invalid_names {
        let result = db.drop_table(invalid_name).await;
        assert!(result.is_err(), "Should reject invalid table name: {}", invalid_name);
    }

    // Test valid table name
    let valid_name = "repos_20231201120000";
    let result = db.drop_table(valid_name).await;
    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_repository_validation_during_insertion() {
    let (_docker, container) = setup_test_database();
    let db = create_database_manager(&container).await;
    let table_name = format!("repos_test_{}", fastrand::u64(..));

    // Create table
    db.create_repository_table(&table_name).await.unwrap();

    // Create invalid repository
    let mut invalid_repo = create_test_repository(1, "test/repo");
    invalid_repo.full_name = "".to_string();

    let repos = vec![invalid_repo];

    // Should fail validation
    let result = db.insert_repositories(&table_name, &repos).await;
    assert!(result.is_err());

    // Verify no data was inserted
    let stats = db.get_table_stats(&table_name).await.unwrap();
    assert_eq!(stats.total_repositories, 0);

    // Cleanup
    db.drop_table(&table_name).await.unwrap();
}

#[tokio::test]
#[serial]
async fn test_concurrent_operations() {
    let (_docker, container) = setup_test_database();
    let db = create_database_manager(&container).await;
    let table_name = format!("repos_test_{}", fastrand::u64(..));

    // Create table
    db.create_repository_table(&table_name).await.unwrap();

    // Create multiple tasks that insert repositories concurrently
    let mut handles = vec![];

    for i in 0..5 {
        let db_clone = db.clone();
        let table_name_clone = table_name.clone();
        
        let handle = tokio::spawn(async move {
            let repos = vec![
                create_test_repository(i * 1000 + 1, &format!("user{}/repo1", i)),
                create_test_repository(i * 1000 + 2, &format!("user{}/repo2", i)),
            ];
            
            db_clone.insert_repositories(&table_name_clone, &repos).await.unwrap()
        });
        
        handles.push(handle);
    }

    // Wait for all tasks to complete
    let mut total_inserted = 0;
    for handle in handles {
        let inserted = handle.await.unwrap();
        total_inserted += inserted;
    }

    assert_eq!(total_inserted, 10);

    // Verify final state
    let stats = db.get_table_stats(&table_name).await.unwrap();
    assert_eq!(stats.total_repositories, 10);

    // Cleanup
    db.drop_table(&table_name).await.unwrap();
}

#[tokio::test]
#[serial]
async fn test_large_batch_insertion() {
    let (_docker, container) = setup_test_database();
    let db = create_database_manager(&container).await;
    let table_name = format!("repos_test_{}", fastrand::u64(..));

    // Create table
    db.create_repository_table(&table_name).await.unwrap();

    // Create a large batch of repositories
    let mut repos = Vec::new();
    for i in 0..100 {
        repos.push(create_test_repository(i, &format!("user{}/repo{}", i % 10, i)));
    }

    // Insert large batch
    let start = std::time::Instant::now();
    let inserted_count = db.insert_repositories(&table_name, &repos).await.unwrap();
    let duration = start.elapsed();

    assert_eq!(inserted_count, 100);
    println!("Inserted 100 repositories in {:?}", duration);

    // Verify stats
    let stats = db.get_table_stats(&table_name).await.unwrap();
    assert_eq!(stats.total_repositories, 100);
    assert_eq!(stats.unique_owners, 10);

    // Cleanup
    db.drop_table(&table_name).await.unwrap();
}

#[tokio::test]
#[serial]
async fn test_query_metadata_update_on_conflict() {
    let (_docker, container) = setup_test_database();
    let db = create_database_manager(&container).await;

    // Create metadata
    let mut metadata = QueryMetadata::new(
        "test query".to_string(),
        "test_table".to_string(),
    );
    metadata.mark_success(50, 1000);

    // Save initial metadata
    db.save_query_metadata(&metadata).await.unwrap();

    // Update metadata
    metadata.mark_success(100, 2000);

    // Save updated metadata (should update, not insert new)
    db.save_query_metadata(&metadata).await.unwrap();

    // Verify only one record exists with updated values
    let history = db.get_query_history(None, false).await.unwrap();
    let matching_records: Vec<_> = history.iter().filter(|h| h.id == metadata.id).collect();
    
    assert_eq!(matching_records.len(), 1);
    assert_eq!(matching_records[0].result_count, 100);
    assert_eq!(matching_records[0].duration_ms, 2000);
}

// Property-based tests
proptest! {
    #[test]
    fn test_table_name_generation_properties(
        _seed in 0u64..1000
    ) {
        let table_name = DatabaseManager::generate_table_name();
        
        // Should always start with "repos_"
        prop_assert!(table_name.starts_with("repos_"));
        
        // Should always have correct length
        prop_assert_eq!(table_name.len(), 20);
        
        // Should only contain valid characters
        prop_assert!(table_name.chars().all(|c| c.is_alphanumeric() || c == '_'));
        
        // Timestamp part should be all digits
        let timestamp_part = &table_name[6..];
        prop_assert!(timestamp_part.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_table_stats_invariants(
        total_repos in 0i64..1000,
        unique_languages in 0i64..50,
        unique_owners in 0i64..100,
        max_stars in 0i64..1_000_000
    ) {
        let stats = TableStats {
            table_name: "test_table".to_string(),
            total_repositories: total_repos,
            unique_languages,
            unique_owners,
            avg_stars: max_stars as f64 / 2.0,
            max_stars,
            oldest_repo: Some(Utc::now()),
            newest_repo: Some(Utc::now()),
        };

        // Basic invariants
        prop_assert!(stats.total_repositories >= 0);
        prop_assert!(stats.unique_languages >= 0);
        prop_assert!(stats.unique_owners >= 0);
        prop_assert!(stats.max_stars >= 0);
        prop_assert!(stats.avg_stars >= 0.0);
        
        // Logical constraints
        if stats.total_repositories > 0 {
            prop_assert!(stats.unique_owners <= stats.total_repositories);
            prop_assert!(stats.unique_languages <= stats.total_repositories);
        }
    }
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

#[tokio::test]
#[serial]
async fn test_database_connection_error_handling() {
    // Test with invalid database URL
    let invalid_url = "postgresql://invalid:invalid@nonexistent:5432/invalid";
    let result = DatabaseManager::new(invalid_url).await;
    assert!(result.is_err());
}

#[tokio::test]
#[serial]
async fn test_database_manager_clone() {
    let (_docker, container) = setup_test_database();
    let db1 = create_database_manager(&container).await;
    let db2 = db1.clone();

    // Both should work independently
    let tables1 = db1.list_repository_tables().await.unwrap();
    let tables2 = db2.list_repository_tables().await.unwrap();
    
    assert_eq!(tables1, tables2);
}