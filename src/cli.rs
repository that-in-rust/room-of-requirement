//! # Command Line Interface Module
//! 
//! This module provides CLI argument parsing, configuration management,
//! and progress indication functionality for the GitHub PostgreSQL Query tool.
//! 
//! ## Key Components
//! 
//! - [`CliConfig`]: Configuration structure with validation
//! - [`ProgressIndicator`]: User-friendly progress feedback
//! - Environment variable handling and validation
//! - Comprehensive error reporting with actionable suggestions

use clap::{Arg, ArgMatches, Command};
use std::env;
use std::io::{self, Write};

use crate::{AppError, Result};

/// CLI configuration structure containing all parsed and validated arguments.
/// 
/// This structure holds all configuration needed to execute a GitHub search
/// query, including authentication tokens, database connection details,
/// search parameters, and execution options.
/// 
/// # Fields
/// 
/// * `search_query` - GitHub repository search query string
/// * `github_token` - GitHub API authentication token
/// * `database_url` - PostgreSQL connection string
/// * `per_page` - Number of results per page (1-100)
/// * `page` - Page number to retrieve (starts from 1)
/// * `verbose` - Enable detailed progress output
/// * `dry_run` - Validate configuration without executing
/// 
/// # Example
/// 
/// ```rust
/// let config = CliConfig::parse()?;
/// println!("Searching for: {}", config.search_query);
/// ```
#[derive(Debug, Clone)]
pub struct CliConfig {
    /// GitHub search query
    pub search_query: String,
    /// GitHub API token
    pub github_token: String,
    /// PostgreSQL database URL
    pub database_url: String,
    /// Number of results per page (1-100)
    pub per_page: u32,
    /// Page number to retrieve
    pub page: u32,
    /// Verbose output flag
    pub verbose: bool,
    /// Dry run mode (validate only, don't execute)
    pub dry_run: bool,
}

/// Progress indicator for providing user-friendly feedback during operations.
/// 
/// This structure manages progress indication with different output modes:
/// - **Verbose mode**: Shows detailed step-by-step progress
/// - **Normal mode**: Shows concise progress with status updates
/// 
/// # Features
/// 
/// - Start/update/complete progress indication
/// - Success, error, warning, and info message types
/// - Automatic formatting with emoji indicators
/// - Verbose and normal output modes
/// 
/// # Example
/// 
/// ```rust
/// let progress = ProgressIndicator::new("Connecting to database".to_string(), verbose);
/// progress.start();
/// progress.update("Establishing connection pool");
/// progress.success("Database connected successfully");
/// ```
pub struct ProgressIndicator {
    message: String,
    verbose: bool,
}

impl ProgressIndicator {
    /// Create a new progress indicator
    pub fn new(message: String, verbose: bool) -> Self {
        Self { message, verbose }
    }

    /// Start the progress indicator
    pub fn start(&self) {
        if self.verbose {
            println!("🔄 {}", self.message);
        } else {
            print!("🔄 {}... ", self.message);
            io::stdout().flush().unwrap_or(());
        }
    }

    /// Update progress with a status message
    pub fn update(&self, status: &str) {
        if self.verbose {
            println!("   ↳ {}", status);
        }
    }

    /// Complete the progress indicator with success
    pub fn success(&self, message: &str) {
        if self.verbose {
            println!("✅ {}", message);
        } else {
            println!("✅ {}", message);
        }
    }

    /// Complete the progress indicator with failure
    pub fn error(&self, message: &str) {
        if self.verbose {
            println!("❌ {}", message);
        } else {
            println!("❌ {}", message);
        }
    }

    /// Show a warning message
    pub fn warning(&self, message: &str) {
        println!("⚠️  {}", message);
    }

    /// Show an info message
    pub fn info(&self, message: &str) {
        if self.verbose {
            println!("ℹ️  {}", message);
        }
    }
}

impl CliConfig {
    /// Parses command line arguments and environment variables.
    /// 
    /// This method creates a complete CLI configuration by:
    /// 1. Parsing command line arguments using clap
    /// 2. Reading environment variables for tokens and database URL
    /// 3. Validating all configuration values
    /// 4. Applying defaults where appropriate
    /// 
    /// # Returns
    /// 
    /// * `Ok(CliConfig)` - Successfully parsed and validated configuration
    /// * `Err(AppError)` - Configuration parsing or validation failed
    /// 
    /// # Environment Variables
    /// 
    /// - `GITHUB_TOKEN`: GitHub personal access token (required if not provided via --github-token)
    /// - `DATABASE_URL`: PostgreSQL connection string (required if not provided via --database-url)
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let config = CliConfig::parse()?;
    /// println!("Query: {}", config.search_query);
    /// ```
    pub fn parse() -> Result<Self> {
        let matches = Self::build_cli().get_matches();
        Self::from_matches(&matches)
    }

    /// Parse from provided arguments (for testing)
    pub fn parse_from<I, T>(args: I) -> Result<Self>
    where
        I: IntoIterator<Item = T>,
        T: Into<std::ffi::OsString> + Clone,
    {
        let matches = Self::build_cli().try_get_matches_from(args)
            .map_err(|e| AppError::configuration(format!("Argument parsing failed: {}", e)))?;
        Self::from_matches(&matches)
    }

    /// Build the CLI command structure
    fn build_cli() -> Command {
        Command::new("github-pg-query")
            .version("0.1.0")
            .author("Developer")
            .about("Execute GitHub API search queries and store results in PostgreSQL tables")
            .long_about(
                "A simple tool that executes GitHub API search queries and stores the results \
                in timestamped PostgreSQL tables. Supports any valid GitHub repository search \
                syntax and provides progress indicators and error handling."
            )
            .arg(
                Arg::new("query")
                    .help("GitHub search query (e.g., 'rust language:rust', 'stars:>1000')")
                    .long_help(
                        "GitHub repository search query using GitHub's search syntax. Examples:\n\
                        • 'rust language:rust' - Rust repositories\n\
                        • 'stars:>1000' - Repositories with more than 1000 stars\n\
                        • 'user:octocat' - Repositories owned by octocat\n\
                        • 'created:>2023-01-01' - Repositories created after 2023-01-01\n\
                        • 'topic:machine-learning' - Repositories tagged with machine-learning"
                    )
                    .required(true)
                    .value_name("QUERY")
                    .index(1)
            )
            .arg(
                Arg::new("per-page")
                    .help("Number of results per page (1-100)")
                    .long("per-page")
                    .short('p')
                    .value_name("COUNT")
                    .default_value("30")
                    .value_parser(clap::value_parser!(u32).range(1..=100))
            )
            .arg(
                Arg::new("page")
                    .help("Page number to retrieve (starts from 1)")
                    .long("page")
                    .value_name("NUMBER")
                    .default_value("1")
                    .value_parser(clap::value_parser!(u32).range(1..))
            )
            .arg(
                Arg::new("verbose")
                    .help("Enable verbose output with detailed progress information")
                    .long("verbose")
                    .short('v')
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("dry-run")
                    .help("Validate configuration and query without executing the search")
                    .long("dry-run")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("github-token")
                    .help("GitHub API token (overrides GITHUB_TOKEN environment variable)")
                    .long("github-token")
                    .value_name("TOKEN")
            )
            .arg(
                Arg::new("database-url")
                    .help("PostgreSQL database URL (overrides DATABASE_URL environment variable)")
                    .long("database-url")
                    .value_name("URL")
            )
    }

    /// Create CliConfig from parsed arguments
    fn from_matches(matches: &ArgMatches) -> Result<Self> {
        // Get search query (required argument)
        let search_query = matches
            .get_one::<String>("query")
            .ok_or_else(|| AppError::configuration("Search query is required"))?
            .clone();

        // Validate search query
        Self::validate_search_query(&search_query)?;

        // Get GitHub token from argument or environment
        let github_token = matches
            .get_one::<String>("github-token")
            .cloned()
            .or_else(|| env::var("GITHUB_TOKEN").ok())
            .ok_or_else(|| AppError::environment("GITHUB_TOKEN"))?;

        // Validate GitHub token
        Self::validate_github_token(&github_token)?;

        // Get database URL from argument or environment
        let database_url = matches
            .get_one::<String>("database-url")
            .cloned()
            .or_else(|| env::var("DATABASE_URL").ok())
            .ok_or_else(|| AppError::environment("DATABASE_URL"))?;

        // Validate database URL
        Self::validate_database_url(&database_url)?;

        // Get other arguments with defaults
        let per_page = *matches.get_one::<u32>("per-page").unwrap_or(&30);
        let page = *matches.get_one::<u32>("page").unwrap_or(&1);
        let verbose = matches.get_flag("verbose");
        let dry_run = matches.get_flag("dry-run");

        Ok(Self {
            search_query,
            github_token,
            database_url,
            per_page,
            page,
            verbose,
            dry_run,
        })
    }

    /// Validate GitHub search query
    fn validate_search_query(query: &str) -> Result<()> {
        if query.trim().is_empty() {
            return Err(AppError::invalid_query(query, "Query cannot be empty"));
        }

        if query.len() > 256 {
            return Err(AppError::invalid_query(
                query,
                "Query too long (maximum 256 characters)"
            ));
        }

        // Check for potentially problematic characters
        if query.contains('\0') {
            return Err(AppError::invalid_query(
                query,
                "Query contains null characters"
            ));
        }

        Ok(())
    }

    /// Validate GitHub token format
    fn validate_github_token(token: &str) -> Result<()> {
        if token.trim().is_empty() {
            return Err(AppError::environment("GITHUB_TOKEN cannot be empty"));
        }

        if token.len() < 10 {
            return Err(AppError::authentication(
                "GitHub token appears to be too short (minimum 10 characters)"
            ));
        }

        if token.len() > 255 {
            return Err(AppError::authentication(
                "GitHub token appears to be too long (maximum 255 characters)"
            ));
        }

        // Check for whitespace in token
        if token.contains(char::is_whitespace) {
            return Err(AppError::authentication(
                "GitHub token contains whitespace characters"
            ));
        }

        Ok(())
    }

    /// Validate PostgreSQL database URL format
    fn validate_database_url(url: &str) -> Result<()> {
        if url.trim().is_empty() {
            return Err(AppError::environment("DATABASE_URL cannot be empty"));
        }

        if !url.starts_with("postgres://") && !url.starts_with("postgresql://") {
            return Err(AppError::configuration(
                "DATABASE_URL must start with 'postgres://' or 'postgresql://'"
            ));
        }

        // Basic URL validation - check for required components
        if !url.contains('@') {
            return Err(AppError::configuration(
                "DATABASE_URL must contain authentication information (user@host)"
            ));
        }

        if !url.contains('/') || url.matches('/').count() < 3 {
            return Err(AppError::configuration(
                "DATABASE_URL must contain a database name"
            ));
        }

        Ok(())
    }

    /// Display configuration summary
    pub fn display_summary(&self) {
        let progress = ProgressIndicator::new("Configuration".to_string(), self.verbose);
        
        progress.info("Configuration Summary:");
        progress.info(&format!("  Search Query: {}", self.search_query));
        progress.info(&format!("  Results per page: {}", self.per_page));
        progress.info(&format!("  Page number: {}", self.page));
        progress.info(&format!("  Verbose mode: {}", self.verbose));
        progress.info(&format!("  Dry run mode: {}", self.dry_run));
        progress.info(&format!("  GitHub token: {}***", &self.github_token[..3.min(self.github_token.len())]));
        
        // Mask sensitive parts of database URL
        let masked_db_url = self.mask_database_url();
        progress.info(&format!("  Database URL: {}", masked_db_url));
    }

    /// Mask sensitive information in database URL for display
    fn mask_database_url(&self) -> String {
        if let Some(at_pos) = self.database_url.find('@') {
            if let Some(colon_pos) = self.database_url[..at_pos].rfind(':') {
                let mut masked = self.database_url.clone();
                masked.replace_range(colon_pos + 1..at_pos, "***");
                return masked;
            }
        }
        // Fallback: just show the protocol and host
        if let Some(at_pos) = self.database_url.find('@') {
            format!("{}@{}", &self.database_url[..at_pos.min(10)], "***")
        } else {
            "***".to_string()
        }
    }

    /// Validate environment variables are accessible
    pub fn validate_environment() -> Result<()> {
        let progress = ProgressIndicator::new("Environment validation".to_string(), false);
        progress.start();

        // Check if GITHUB_TOKEN is set
        match env::var("GITHUB_TOKEN") {
            Ok(token) => {
                if token.trim().is_empty() {
                    progress.error("GITHUB_TOKEN environment variable is empty");
                    return Err(AppError::environment("GITHUB_TOKEN is empty"));
                }
                progress.update("GITHUB_TOKEN found");
            }
            Err(_) => {
                progress.warning("GITHUB_TOKEN environment variable not set");
                progress.info("You can set it with: export GITHUB_TOKEN=your_token_here");
                progress.info("Or provide it via --github-token argument");
            }
        }

        // Check if DATABASE_URL is set
        match env::var("DATABASE_URL") {
            Ok(url) => {
                if url.trim().is_empty() {
                    progress.error("DATABASE_URL environment variable is empty");
                    return Err(AppError::environment("DATABASE_URL is empty"));
                }
                progress.update("DATABASE_URL found");
            }
            Err(_) => {
                progress.warning("DATABASE_URL environment variable not set");
                progress.info("You can set it with: export DATABASE_URL=postgresql://user:pass@host:port/dbname");
                progress.info("Or provide it via --database-url argument");
            }
        }

        progress.success("Environment validation completed");
        Ok(())
    }

    /// Display help for setting up environment variables
    pub fn display_setup_help() {
        println!("\n📋 Setup Instructions:");
        println!();
        println!("1. GitHub Token:");
        println!("   • Go to https://github.com/settings/tokens");
        println!("   • Generate a new token with 'public_repo' scope");
        println!("   • Set the environment variable:");
        println!("     export GITHUB_TOKEN=your_token_here");
        println!();
        println!("2. PostgreSQL Database:");
        println!("   • Ensure PostgreSQL is running and accessible");
        println!("   • Create a database for storing repository data");
        println!("   • Set the environment variable:");
        println!("     export DATABASE_URL=postgresql://user:password@localhost:5432/dbname");
        println!();
        println!("3. Example Usage:");
        println!("   github-pg-query 'rust language:rust stars:>100'");
        println!("   github-pg-query 'user:octocat' --per-page 50 --verbose");
        println!();
    }

    /// Display actionable error message with suggestions
    pub fn display_error(error: &AppError) {
        let progress = ProgressIndicator::new("Error".to_string(), false);
        
        match error {
            AppError::Environment { var_name } => {
                progress.error(&format!("Environment variable {} is not set", var_name));
                println!();
                if var_name == "GITHUB_TOKEN" {
                    println!("💡 To fix this:");
                    println!("   1. Go to https://github.com/settings/tokens");
                    println!("   2. Generate a new token with 'public_repo' scope");
                    println!("   3. Run: export GITHUB_TOKEN=your_token_here");
                    println!("   4. Or use: --github-token your_token_here");
                } else if var_name == "DATABASE_URL" {
                    println!("💡 To fix this:");
                    println!("   1. Ensure PostgreSQL is running");
                    println!("   2. Create a database for the application");
                    println!("   3. Run: export DATABASE_URL=postgresql://user:pass@host:port/dbname");
                    println!("   4. Or use: --database-url postgresql://...");
                }
            }
            AppError::Authentication { reason } => {
                progress.error(&format!("Authentication failed: {}", reason));
                println!();
                println!("💡 To fix this:");
                println!("   1. Check that your GitHub token is valid");
                println!("   2. Ensure the token has 'public_repo' scope");
                println!("   3. Try generating a new token if the current one is expired");
            }
            AppError::InvalidQuery { query, reason } => {
                progress.error(&format!("Invalid search query: {}", reason));
                println!("   Query: {}", query);
                println!();
                println!("💡 Valid query examples:");
                println!("   • 'rust language:rust' - Rust repositories");
                println!("   • 'stars:>1000' - Popular repositories");
                println!("   • 'user:octocat' - User's repositories");
                println!("   • 'created:>2023-01-01' - Recent repositories");
            }
            AppError::Database(db_error) => {
                progress.error(&format!("Database error: {}", db_error));
                println!();
                println!("💡 To fix this:");
                println!("   1. Ensure PostgreSQL is running and accessible");
                println!("   2. Check that the database exists and credentials are correct");
                println!("   3. Verify network connectivity to the database server");
                println!("   4. Check database permissions for the user");
            }
            AppError::Configuration { message } => {
                progress.error(&format!("Configuration error: {}", message));
                println!();
                println!("💡 Run with --help to see all available options");
            }
            _ => {
                progress.error(&format!("{}", error));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_search_query_valid() {
        assert!(CliConfig::validate_search_query("rust language:rust").is_ok());
        assert!(CliConfig::validate_search_query("stars:>1000").is_ok());
        assert!(CliConfig::validate_search_query("user:octocat").is_ok());
    }

    #[test]
    fn test_validate_search_query_empty() {
        assert!(CliConfig::validate_search_query("").is_err());
        assert!(CliConfig::validate_search_query("   ").is_err());
    }

    #[test]
    fn test_validate_search_query_too_long() {
        let long_query = "a".repeat(300);
        assert!(CliConfig::validate_search_query(&long_query).is_err());
    }

    #[test]
    fn test_validate_search_query_null_character() {
        assert!(CliConfig::validate_search_query("test\0query").is_err());
    }

    #[test]
    fn test_validate_github_token_valid() {
        assert!(CliConfig::validate_github_token("ghp_1234567890abcdef").is_ok());
        assert!(CliConfig::validate_github_token("github_pat_1234567890").is_ok());
    }

    #[test]
    fn test_validate_github_token_empty() {
        assert!(CliConfig::validate_github_token("").is_err());
        assert!(CliConfig::validate_github_token("   ").is_err());
    }

    #[test]
    fn test_validate_github_token_too_short() {
        assert!(CliConfig::validate_github_token("short").is_err());
    }

    #[test]
    fn test_validate_github_token_too_long() {
        let long_token = "a".repeat(300);
        assert!(CliConfig::validate_github_token(&long_token).is_err());
    }

    #[test]
    fn test_validate_github_token_whitespace() {
        assert!(CliConfig::validate_github_token("token with spaces").is_err());
        assert!(CliConfig::validate_github_token("token\nwith\nnewlines").is_err());
    }

    #[test]
    fn test_validate_database_url_valid() {
        assert!(CliConfig::validate_database_url("postgresql://user:pass@localhost:5432/dbname").is_ok());
        assert!(CliConfig::validate_database_url("postgres://user:pass@host.com:5432/db").is_ok());
    }

    #[test]
    fn test_validate_database_url_empty() {
        assert!(CliConfig::validate_database_url("").is_err());
        assert!(CliConfig::validate_database_url("   ").is_err());
    }

    #[test]
    fn test_validate_database_url_wrong_protocol() {
        assert!(CliConfig::validate_database_url("mysql://user:pass@host/db").is_err());
        assert!(CliConfig::validate_database_url("http://example.com").is_err());
    }

    #[test]
    fn test_validate_database_url_missing_auth() {
        assert!(CliConfig::validate_database_url("postgresql://localhost:5432/db").is_err());
    }

    #[test]
    fn test_validate_database_url_missing_database() {
        assert!(CliConfig::validate_database_url("postgresql://user:pass@localhost:5432").is_err());
    }

    #[test]
    fn test_mask_database_url() {
        let config = CliConfig {
            search_query: "test".to_string(),
            github_token: "token".to_string(),
            database_url: "postgresql://user:password@localhost:5432/dbname".to_string(),
            per_page: 30,
            page: 1,
            verbose: false,
            dry_run: false,
        };

        let masked = config.mask_database_url();
        assert!(masked.contains("***"));
        assert!(!masked.contains("password"));
    }

    #[test]
    fn test_parse_from_args_valid() {
        let args = vec![
            "github-pg-query",
            "rust language:rust",
            "--per-page", "50",
            "--page", "2",
            "--verbose",
            "--github-token", "test_token_1234567890",
            "--database-url", "postgresql://user:pass@localhost:5432/test"
        ];

        let config = CliConfig::parse_from(args).unwrap();
        assert_eq!(config.search_query, "rust language:rust");
        assert_eq!(config.per_page, 50);
        assert_eq!(config.page, 2);
        assert!(config.verbose);
        assert_eq!(config.github_token, "test_token_1234567890");
    }

    #[test]
    fn test_parse_from_args_missing_query() {
        let args = vec![
            "github-pg-query",
            "--github-token", "test_token_1234567890",
            "--database-url", "postgresql://user:pass@localhost:5432/test"
        ];

        assert!(CliConfig::parse_from(args).is_err());
    }

    #[test]
    fn test_parse_from_args_invalid_per_page() {
        let args = vec![
            "github-pg-query",
            "test query",
            "--per-page", "150", // Too high
            "--github-token", "test_token_1234567890",
            "--database-url", "postgresql://user:pass@localhost:5432/test"
        ];

        assert!(CliConfig::parse_from(args).is_err());
    }

    #[test]
    fn test_progress_indicator() {
        let progress = ProgressIndicator::new("Test operation".to_string(), true);
        
        // These should not panic
        progress.start();
        progress.update("Step 1");
        progress.success("Completed");
        progress.error("Failed");
        progress.warning("Warning message");
        progress.info("Info message");
    }
}