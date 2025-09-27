use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use github_pg_query::{
    DatabaseManager, Repository, RepositoryOwner, RepositoryLicense, 
    QueryMetadata, GitHubClient, SearchResponse
};
use testcontainers::{clients::Cli, images::postgres::Postgres};
use chrono::Utc;
use tokio::runtime::Runtime;
use std::time::Duration;

// Helper function to create test repositories
fn create_test_repositories(count: usize) -> Vec<Repository> {
    (0..count)
        .map(|i| {
            let now = Utc::now();
            Repository {
                id: i as i64,
                full_name: format!("user{}/repo{}", i % 100, i),
                name: format!("repo{}", i),
                description: Some(format!("Test repository {}", i)),
                html_url: format!("https://github.com/user{}/repo{}", i % 100, i),
                clone_url: format!("https://github.com/user{}/repo{}.git", i % 100, i),
                ssh_url: format!("git@github.com:user{}/repo{}.git", i % 100, i),
                size: (i * 1024) as i64,
                stargazers_count: (i * 10) as i64,
                watchers_count: (i * 5) as i64,
                forks_count: (i * 2) as i64,
                open_issues_count: (i % 50) as i64,
                language: Some(match i % 5 {
                    0 => "Rust".to_string(),
                    1 => "JavaScript".to_string(),
                    2 => "Python".to_string(),
                    3 => "Go".to_string(),
                    _ => "TypeScript".to_string(),
                }),
                default_branch: "main".to_string(),
                visibility: "public".to_string(),
                private: false,
                fork: i % 10 == 0,
                archived: false,
                disabled: false,
                created_at: now,
                updated_at: now,
                pushed_at: Some(now),
                owner: RepositoryOwner {
                    id: (i % 100) as i64,
                    login: format!("user{}", i % 100),
                    owner_type: if i % 10 == 0 { "Organization".to_string() } else { "User".to_string() },
                    avatar_url: format!("https://avatars.githubusercontent.com/u/{}?v=4", i % 100),
                    html_url: format!("https://github.com/user{}", i % 100),
                    site_admin: false,
                },
                license: if i % 3 == 0 {
                    Some(RepositoryLicense {
                        key: "mit".to_string(),
                        name: "MIT License".to_string(),
                        spdx_id: Some("MIT".to_string()),
                        url: Some("https://api.github.com/licenses/mit".to_string()),
                    })
                } else {
                    None
                },
                topics: vec![
                    format!("topic{}", i % 20),
                    format!("category{}", i % 10),
                ],
                has_issues: true,
                has_projects: i % 4 == 0,
                has_wiki: i % 6 == 0,
                has_pages: i % 8 == 0,
                has_downloads: true,
            }
        })
        .collect()
}

// Setup test database for benchmarks
async fn setup_benchmark_database() -> DatabaseManager {
    let docker = Cli::default();
    let postgres_image = Postgres::default()
        .with_db_name("benchmark_db")
        .with_user("bench_user")
        .with_password("bench_password");
    
    let container = docker.run(postgres_image);
    let port = container.get_host_port_ipv4(5432);
    let database_url = format!(
        "postgresql://bench_user:bench_password@localhost:{}/benchmark_db",
        port
    );
    
    DatabaseManager::new(&database_url)
        .await
        .expect("Failed to create benchmark database manager")
}

fn bench_repository_validation(c: &mut Criterion) {
    let repositories = create_test_repositories(1000);
    
    c.bench_function("repository_validation", |b| {
        b.iter(|| {
            for repo in &repositories {
                black_box(repo.validate()).unwrap();
            }
        })
    });
}

fn bench_repository_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("repository_serialization");
    
    for size in [10, 100, 1000].iter() {
        let repositories = create_test_repositories(*size);
        let search_response = SearchResponse {
            total_count: *size as i64,
            incomplete_results: false,
            items: repositories,
        };
        
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(
            BenchmarkId::new("serialize", size),
            &search_response,
            |b, response| {
                b.iter(|| {
                    black_box(serde_json::to_string(response)).unwrap()
                })
            },
        );
        
        let json = serde_json::to_string(&search_response).unwrap();
        group.bench_with_input(
            BenchmarkId::new("deserialize", size),
            &json,
            |b, json_str| {
                b.iter(|| {
                    black_box(serde_json::from_str::<SearchResponse>(json_str)).unwrap()
                })
            },
        );
    }
    
    group.finish();
}

fn bench_database_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let db = rt.block_on(setup_benchmark_database());
    
    let mut group = c.benchmark_group("database_operations");
    group.measurement_time(Duration::from_secs(30));
    
    for batch_size in [10, 50, 100, 500].iter() {
        let repositories = create_test_repositories(*batch_size);
        let table_name = format!("bench_repos_{}", fastrand::u64(..));
        
        // Setup table
        rt.block_on(db.create_repository_table(&table_name)).unwrap();
        
        group.throughput(Throughput::Elements(*batch_size as u64));
        group.bench_with_input(
            BenchmarkId::new("insert_repositories", batch_size),
            &repositories,
            |b, repos| {
                b.to_async(&rt).iter(|| async {
                    let table_name = format!("bench_repos_{}", fastrand::u64(..));
                    db.create_repository_table(&table_name).await.unwrap();
                    
                    let result = black_box(
                        db.insert_repositories(&table_name, repos).await
                    ).unwrap();
                    
                    db.drop_table(&table_name).await.unwrap();
                    result
                })
            },
        );
        
        // Cleanup
        rt.block_on(db.drop_table(&table_name)).unwrap();
    }
    
    group.finish();
}

fn bench_table_statistics(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let db = rt.block_on(setup_benchmark_database());
    
    let mut group = c.benchmark_group("table_statistics");
    
    for repo_count in [100, 500, 1000, 2000].iter() {
        let repositories = create_test_repositories(*repo_count);
        let table_name = format!("bench_stats_{}", fastrand::u64(..));
        
        // Setup table with data
        rt.block_on(async {
            db.create_repository_table(&table_name).await.unwrap();
            db.insert_repositories(&table_name, &repositories).await.unwrap();
        });
        
        group.throughput(Throughput::Elements(*repo_count as u64));
        group.bench_with_input(
            BenchmarkId::new("get_table_stats", repo_count),
            &table_name,
            |b, table| {
                b.to_async(&rt).iter(|| async {
                    black_box(db.get_table_stats(table).await).unwrap()
                })
            },
        );
        
        // Cleanup
        rt.block_on(db.drop_table(&table_name)).unwrap();
    }
    
    group.finish();
}

fn bench_query_metadata_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let db = rt.block_on(setup_benchmark_database());
    
    let mut group = c.benchmark_group("query_metadata");
    
    // Benchmark metadata creation
    group.bench_function("create_metadata", |b| {
        b.iter(|| {
            black_box(QueryMetadata::new(
                "rust language:rust".to_string(),
                "repos_20231201120000".to_string(),
            ))
        })
    });
    
    // Benchmark metadata save/retrieve cycle
    group.bench_function("save_and_retrieve_metadata", |b| {
        b.to_async(&rt).iter(|| async {
            let mut metadata = QueryMetadata::new(
                format!("query_{}", fastrand::u64(..)),
                format!("table_{}", fastrand::u64(..)),
            );
            metadata.mark_success(100, 1500);
            
            black_box(db.save_query_metadata(&metadata).await).unwrap();
            
            let history = black_box(db.get_query_history(Some(1), false).await).unwrap();
            assert!(!history.is_empty());
        })
    });
    
    group.finish();
}

fn bench_concurrent_database_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let db = rt.block_on(setup_benchmark_database());
    
    let mut group = c.benchmark_group("concurrent_operations");
    group.measurement_time(Duration::from_secs(45));
    
    for concurrency in [2, 4, 8].iter() {
        group.bench_with_input(
            BenchmarkId::new("concurrent_inserts", concurrency),
            concurrency,
            |b, &concurrency_level| {
                b.to_async(&rt).iter(|| async {
                    let mut handles = vec![];
                    
                    for i in 0..concurrency_level {
                        let db_clone = db.clone();
                        let handle = tokio::spawn(async move {
                            let repositories = create_test_repositories(50);
                            let table_name = format!("bench_concurrent_{}_{}", i, fastrand::u64(..));
                            
                            db_clone.create_repository_table(&table_name).await.unwrap();
                            let result = db_clone.insert_repositories(&table_name, &repositories).await.unwrap();
                            db_clone.drop_table(&table_name).await.unwrap();
                            
                            result
                        });
                        handles.push(handle);
                    }
                    
                    let mut total_inserted = 0;
                    for handle in handles {
                        total_inserted += handle.await.unwrap();
                    }
                    
                    black_box(total_inserted)
                })
            },
        );
    }
    
    group.finish();
}

fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    
    for size in [1000, 5000, 10000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(
            BenchmarkId::new("create_repositories_in_memory", size),
            size,
            |b, &repo_count| {
                b.iter(|| {
                    let repositories = black_box(create_test_repositories(repo_count));
                    
                    // Simulate some processing
                    let total_stars: i64 = repositories
                        .iter()
                        .map(|r| r.stargazers_count)
                        .sum();
                    
                    black_box(total_stars)
                })
            },
        );
    }
    
    group.finish();
}

fn bench_search_response_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("search_response_processing");
    
    for size in [10, 100, 1000].iter() {
        let repositories = create_test_repositories(*size);
        let search_response = SearchResponse {
            total_count: *size as i64,
            incomplete_results: false,
            items: repositories,
        };
        
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(
            BenchmarkId::new("validate_all_repositories", size),
            &search_response,
            |b, response| {
                b.iter(|| {
                    for repo in &response.items {
                        black_box(repo.validate()).unwrap();
                    }
                })
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("calculate_statistics", size),
            &search_response,
            |b, response| {
                b.iter(|| {
                    let total_stars: i64 = response.items.iter().map(|r| r.stargazers_count).sum();
                    let total_forks: i64 = response.items.iter().map(|r| r.forks_count).sum();
                    let languages: std::collections::HashSet<_> = response.items
                        .iter()
                        .filter_map(|r| r.language.as_ref())
                        .collect();
                    
                    black_box((total_stars, total_forks, languages.len()))
                })
            },
        );
    }
    
    group.finish();
}

fn bench_table_name_generation(c: &mut Criterion) {
    c.bench_function("table_name_generation", |b| {
        b.iter(|| {
            black_box(DatabaseManager::generate_table_name())
        })
    });
    
    c.bench_function("query_metadata_table_name_generation", |b| {
        b.iter(|| {
            black_box(QueryMetadata::generate_table_name())
        })
    });
}

criterion_group!(
    benches,
    bench_repository_validation,
    bench_repository_serialization,
    bench_database_operations,
    bench_table_statistics,
    bench_query_metadata_operations,
    bench_concurrent_database_operations,
    bench_memory_usage,
    bench_search_response_processing,
    bench_table_name_generation
);

criterion_main!(benches);