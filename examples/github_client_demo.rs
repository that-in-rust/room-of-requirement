use github_pg_query::{GitHubClient, AppError};
use std::env;

/// Demonstration of the GitHub API client functionality
/// 
/// This example shows how to:
/// 1. Create a GitHub client with authentication
/// 2. Validate the token
/// 3. Check rate limit status
/// 4. Search for repositories
/// 
/// To run this example:
/// ```bash
/// export GITHUB_TOKEN="your_github_token_here"
/// cargo run --example github_client_demo
/// ```
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get GitHub token from environment
    let token = env::var("GITHUB_TOKEN")
        .map_err(|_| AppError::environment("GITHUB_TOKEN"))?;

    println!("🚀 GitHub API Client Demo");
    println!("========================");

    // Create GitHub client
    println!("\n1. Creating GitHub client...");
    let client = GitHubClient::new(token)?;
    println!("✅ Client created successfully");

    // Validate token
    println!("\n2. Validating GitHub token...");
    match client.validate_token().await {
        Ok(()) => println!("✅ Token is valid"),
        Err(e) => {
            println!("❌ Token validation failed: {}", e);
            return Err(e.into());
        }
    }

    // Check rate limit status
    println!("\n3. Checking rate limit status...");
    match client.get_rate_limit().await {
        Ok(status) => {
            println!("✅ Rate limit status:");
            println!("   Limit: {}", status.limit);
            println!("   Remaining: {}", status.remaining);
            println!("   Reset at: {}", status.reset_at.format("%Y-%m-%d %H:%M:%S UTC"));
        }
        Err(e) => {
            println!("⚠️  Could not get rate limit status: {}", e);
        }
    }

    // Search for repositories
    println!("\n4. Searching for Rust repositories...");
    let query = "language:rust stars:>1000";
    
    match client.search_repositories(query, Some(5), Some(1)).await {
        Ok(results) => {
            println!("✅ Search completed successfully");
            println!("   Total repositories found: {}", results.total_count);
            println!("   Showing first {} results:", results.items.len());
            
            for (i, repo) in results.items.iter().enumerate() {
                println!("\n   {}. {}", i + 1, repo.full_name);
                println!("      ⭐ Stars: {}", repo.stargazers_count);
                println!("      🍴 Forks: {}", repo.forks_count);
                println!("      📝 Description: {}", 
                    repo.description.as_deref().unwrap_or("No description"));
                println!("      🔗 URL: {}", repo.html_url);
            }
        }
        Err(e) => {
            println!("❌ Search failed: {}", e);
            return Err(e.into());
        }
    }

    // Demonstrate error handling with invalid query
    println!("\n5. Testing error handling with invalid query...");
    let invalid_query = ""; // Empty query should fail
    
    match client.search_repositories(invalid_query, None, None).await {
        Ok(_) => println!("⚠️  Unexpected success with empty query"),
        Err(e) => {
            println!("✅ Error handling working correctly:");
            println!("   Error: {}", e);
        }
    }

    println!("\n🎉 Demo completed successfully!");
    Ok(())
}