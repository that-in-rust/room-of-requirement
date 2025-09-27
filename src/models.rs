use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::errors::{AppError, Result};

/// Repository data structure matching GitHub API response
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Repository {
    /// GitHub repository ID
    pub id: i64,
    
    /// Repository full name (owner/repo)
    pub full_name: String,
    
    /// Repository name
    pub name: String,
    
    /// Repository description (can be null)
    pub description: Option<String>,
    
    /// Repository HTML URL
    pub html_url: String,
    
    /// Clone URL (HTTPS)
    pub clone_url: String,
    
    /// SSH URL
    pub ssh_url: String,
    
    /// Repository size in KB
    pub size: i64,
    
    /// Star count
    pub stargazers_count: i64,
    
    /// Watchers count
    pub watchers_count: i64,
    
    /// Forks count
    pub forks_count: i64,
    
    /// Open issues count
    pub open_issues_count: i64,
    
    /// Primary language
    pub language: Option<String>,
    
    /// Default branch
    pub default_branch: String,
    
    /// Repository visibility (public/private)
    pub visibility: String,
    
    /// Is the repository private
    pub private: bool,
    
    /// Is the repository a fork
    pub fork: bool,
    
    /// Is the repository archived
    pub archived: bool,
    
    /// Is the repository disabled
    pub disabled: bool,
    
    /// Repository creation date
    pub created_at: DateTime<Utc>,
    
    /// Repository last update date
    pub updated_at: DateTime<Utc>,
    
    /// Repository last push date
    pub pushed_at: Option<DateTime<Utc>>,
    
    /// Repository owner information
    pub owner: RepositoryOwner,
    
    /// Repository license information
    pub license: Option<RepositoryLicense>,
    
    /// Repository topics/tags
    pub topics: Vec<String>,
    
    /// Has issues enabled
    pub has_issues: bool,
    
    /// Has projects enabled
    pub has_projects: bool,
    
    /// Has wiki enabled
    pub has_wiki: bool,
    
    /// Has pages enabled
    pub has_pages: bool,
    
    /// Has downloads enabled
    pub has_downloads: bool,
}

/// Repository owner information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepositoryOwner {
    /// Owner ID
    pub id: i64,
    
    /// Owner login/username
    pub login: String,
    
    /// Owner type (User, Organization)
    #[serde(rename = "type")]
    pub owner_type: String,
    
    /// Owner avatar URL
    pub avatar_url: String,
    
    /// Owner HTML URL
    pub html_url: String,
    
    /// Is site admin
    pub site_admin: bool,
}

/// Repository license information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepositoryLicense {
    /// License key (e.g., "mit", "apache-2.0")
    pub key: String,
    
    /// License name
    pub name: String,
    
    /// License SPDX ID
    pub spdx_id: Option<String>,
    
    /// License URL
    pub url: Option<String>,
}

/// GitHub API search response wrapper
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchResponse {
    /// Total count of repositories found
    pub total_count: i64,
    
    /// Whether the search was incomplete (due to timeout)
    pub incomplete_results: bool,
    
    /// Array of repository items
    pub items: Vec<Repository>,
}

/// Query metadata for tracking search history
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryMetadata {
    /// Unique query ID
    pub id: uuid::Uuid,
    
    /// The search query string
    pub search_query: String,
    
    /// Generated table name for this query
    pub table_name: String,
    
    /// Number of results returned
    pub result_count: i64,
    
    /// When the query was executed
    pub executed_at: DateTime<Utc>,
    
    /// Query execution duration in milliseconds
    pub duration_ms: i64,
    
    /// Whether the query was successful
    pub success: bool,
    
    /// Error message if query failed
    pub error_message: Option<String>,
}

impl Repository {
    /// Validate repository data according to business rules
    pub fn validate(&self) -> Result<()> {
        // Validate required fields
        if self.full_name.is_empty() {
            return Err(AppError::validation("full_name", "cannot be empty"));
        }
        
        if self.name.is_empty() {
            return Err(AppError::validation("name", "cannot be empty"));
        }
        
        if self.html_url.is_empty() {
            return Err(AppError::validation("html_url", "cannot be empty"));
        }
        
        if self.clone_url.is_empty() {
            return Err(AppError::validation("clone_url", "cannot be empty"));
        }
        
        if self.ssh_url.is_empty() {
            return Err(AppError::validation("ssh_url", "cannot be empty"));
        }
        
        if self.default_branch.is_empty() {
            return Err(AppError::validation("default_branch", "cannot be empty"));
        }
        
        if self.visibility.is_empty() {
            return Err(AppError::validation("visibility", "cannot be empty"));
        }
        
        // Validate URL formats
        if !self.html_url.starts_with("https://github.com/") {
            return Err(AppError::validation("html_url", "must be a valid GitHub URL"));
        }
        
        if !self.clone_url.starts_with("https://github.com/") || !self.clone_url.ends_with(".git") {
            return Err(AppError::validation("clone_url", "must be a valid GitHub clone URL"));
        }
        
        if !self.ssh_url.starts_with("git@github.com:") || !self.ssh_url.ends_with(".git") {
            return Err(AppError::validation("ssh_url", "must be a valid GitHub SSH URL"));
        }
        
        // Validate visibility values
        if !["public", "private", "internal"].contains(&self.visibility.as_str()) {
            return Err(AppError::validation("visibility", "must be 'public', 'private', or 'internal'"));
        }
        
        // Validate numeric fields are non-negative
        if self.size < 0 {
            return Err(AppError::validation("size", "cannot be negative"));
        }
        
        if self.stargazers_count < 0 {
            return Err(AppError::validation("stargazers_count", "cannot be negative"));
        }
        
        if self.watchers_count < 0 {
            return Err(AppError::validation("watchers_count", "cannot be negative"));
        }
        
        if self.forks_count < 0 {
            return Err(AppError::validation("forks_count", "cannot be negative"));
        }
        
        if self.open_issues_count < 0 {
            return Err(AppError::validation("open_issues_count", "cannot be negative"));
        }
        
        // Validate owner
        self.owner.validate()?;
        
        // Validate license if present
        if let Some(ref license) = self.license {
            license.validate()?;
        }
        
        Ok(())
    }
    
    /// Generate a sanitized table name based on the repository full name
    pub fn generate_table_name_suffix(&self) -> String {
        self.full_name
            .to_lowercase()
            .replace(['/', '-'], "_")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_')
            .collect()
    }
}

impl RepositoryOwner {
    /// Validate repository owner data
    pub fn validate(&self) -> Result<()> {
        if self.login.is_empty() {
            return Err(AppError::validation("owner.login", "cannot be empty"));
        }
        
        if self.avatar_url.is_empty() {
            return Err(AppError::validation("owner.avatar_url", "cannot be empty"));
        }
        
        if self.html_url.is_empty() {
            return Err(AppError::validation("owner.html_url", "cannot be empty"));
        }
        
        if !["User", "Organization", "Bot"].contains(&self.owner_type.as_str()) {
            return Err(AppError::validation("owner.type", "must be 'User', 'Organization', or 'Bot'"));
        }
        
        // Validate URL format
        if !self.html_url.starts_with("https://github.com/") {
            return Err(AppError::validation("owner.html_url", "must be a valid GitHub URL"));
        }
        
        Ok(())
    }
}

impl RepositoryLicense {
    /// Validate repository license data
    pub fn validate(&self) -> Result<()> {
        if self.key.is_empty() {
            return Err(AppError::validation("license.key", "cannot be empty"));
        }
        
        if self.name.is_empty() {
            return Err(AppError::validation("license.name", "cannot be empty"));
        }
        
        Ok(())
    }
}

impl QueryMetadata {
    /// Create new query metadata
    pub fn new(search_query: String, table_name: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            search_query,
            table_name,
            result_count: 0,
            executed_at: Utc::now(),
            duration_ms: 0,
            success: false,
            error_message: None,
        }
    }
    
    /// Mark query as completed successfully
    pub fn mark_success(&mut self, result_count: i64, duration_ms: i64) {
        self.result_count = result_count;
        self.duration_ms = duration_ms;
        self.success = true;
        self.error_message = None;
    }
    
    /// Mark query as failed
    pub fn mark_failure(&mut self, error_message: String, duration_ms: i64) {
        self.duration_ms = duration_ms;
        self.success = false;
        self.error_message = Some(error_message);
    }
    
    /// Generate timestamped table name in the format repos_YYYYMMDDHHMMSS
    pub fn generate_table_name() -> String {
        let now = Utc::now();
        format!("repos_{}", now.format("%Y%m%d%H%M%S"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_repository() -> Repository {
        Repository {
            id: 123456789,
            full_name: "octocat/Hello-World".to_string(),
            name: "Hello-World".to_string(),
            description: Some("This your first repo!".to_string()),
            html_url: "https://github.com/octocat/Hello-World".to_string(),
            clone_url: "https://github.com/octocat/Hello-World.git".to_string(),
            ssh_url: "git@github.com:octocat/Hello-World.git".to_string(),
            size: 108,
            stargazers_count: 80,
            watchers_count: 9,
            forks_count: 9,
            open_issues_count: 0,
            language: Some("C".to_string()),
            default_branch: "master".to_string(),
            visibility: "public".to_string(),
            private: false,
            fork: false,
            archived: false,
            disabled: false,
            created_at: "2011-01-26T19:01:12Z".parse().unwrap(),
            updated_at: "2011-01-26T19:14:43Z".parse().unwrap(),
            pushed_at: Some("2011-01-26T19:06:43Z".parse().unwrap()),
            owner: RepositoryOwner {
                id: 1,
                login: "octocat".to_string(),
                owner_type: "User".to_string(),
                avatar_url: "https://github.com/images/error/octocat_happy.gif".to_string(),
                html_url: "https://github.com/octocat".to_string(),
                site_admin: false,
            },
            license: Some(RepositoryLicense {
                key: "mit".to_string(),
                name: "MIT License".to_string(),
                spdx_id: Some("MIT".to_string()),
                url: Some("https://api.github.com/licenses/mit".to_string()),
            }),
            topics: vec!["octocat".to_string(), "atom".to_string(), "electron".to_string(), "api".to_string()],
            has_issues: true,
            has_projects: true,
            has_wiki: true,
            has_pages: false,
            has_downloads: true,
        }
    }
    
    #[test]
    fn test_repository_validation_success() {
        let repo = create_test_repository();
        assert!(repo.validate().is_ok());
    }
    
    #[test]
    fn test_repository_validation_empty_full_name() {
        let mut repo = create_test_repository();
        repo.full_name = "".to_string();
        
        let result = repo.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("full_name"));
    }
    
    #[test]
    fn test_repository_validation_invalid_html_url() {
        let mut repo = create_test_repository();
        repo.html_url = "https://example.com/repo".to_string();
        
        let result = repo.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("html_url"));
    }
    
    #[test]
    fn test_repository_validation_invalid_visibility() {
        let mut repo = create_test_repository();
        repo.visibility = "invalid".to_string();
        
        let result = repo.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("visibility"));
    }
    
    #[test]
    fn test_repository_validation_negative_counts() {
        let mut repo = create_test_repository();
        repo.stargazers_count = -1;
        
        let result = repo.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("stargazers_count"));
    }
    
    #[test]
    fn test_repository_owner_validation_success() {
        let repo = create_test_repository();
        assert!(repo.owner.validate().is_ok());
    }
    
    #[test]
    fn test_repository_owner_validation_empty_login() {
        let mut repo = create_test_repository();
        repo.owner.login = "".to_string();
        
        let result = repo.owner.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("owner.login"));
    }
    
    #[test]
    fn test_repository_owner_validation_invalid_type() {
        let mut repo = create_test_repository();
        repo.owner.owner_type = "Invalid".to_string();
        
        let result = repo.owner.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("owner.type"));
    }
    
    #[test]
    fn test_repository_license_validation_success() {
        let repo = create_test_repository();
        assert!(repo.license.as_ref().unwrap().validate().is_ok());
    }
    
    #[test]
    fn test_repository_license_validation_empty_key() {
        let mut repo = create_test_repository();
        repo.license.as_mut().unwrap().key = "".to_string();
        
        let result = repo.license.as_ref().unwrap().validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("license.key"));
    }
    
    #[test]
    fn test_generate_table_name_suffix() {
        let repo = create_test_repository();
        let suffix = repo.generate_table_name_suffix();
        assert_eq!(suffix, "octocat_hello_world");
    }
    
    #[test]
    fn test_query_metadata_new() {
        let metadata = QueryMetadata::new("rust language:rust".to_string(), "repos_20231201120000".to_string());
        
        assert_eq!(metadata.search_query, "rust language:rust");
        assert_eq!(metadata.table_name, "repos_20231201120000");
        assert_eq!(metadata.result_count, 0);
        assert!(!metadata.success);
        assert!(metadata.error_message.is_none());
    }
    
    #[test]
    fn test_query_metadata_mark_success() {
        let mut metadata = QueryMetadata::new("test".to_string(), "test_table".to_string());
        metadata.mark_success(100, 1500);
        
        assert_eq!(metadata.result_count, 100);
        assert_eq!(metadata.duration_ms, 1500);
        assert!(metadata.success);
        assert!(metadata.error_message.is_none());
    }
    
    #[test]
    fn test_query_metadata_mark_failure() {
        let mut metadata = QueryMetadata::new("test".to_string(), "test_table".to_string());
        metadata.mark_failure("API error".to_string(), 500);
        
        assert_eq!(metadata.duration_ms, 500);
        assert!(!metadata.success);
        assert_eq!(metadata.error_message, Some("API error".to_string()));
    }
    
    #[test]
    fn test_generate_table_name_format() {
        let table_name = QueryMetadata::generate_table_name();
        assert!(table_name.starts_with("repos_"));
        assert_eq!(table_name.len(), 20); // "repos_" + 14 digits (YYYYMMDDHHMMSS)
    }
    
    #[test]
    fn test_serde_serialization() {
        let repo = create_test_repository();
        
        // Test serialization
        let json = serde_json::to_string(&repo).unwrap();
        assert!(!json.is_empty());
        
        // Test deserialization
        let deserialized: Repository = serde_json::from_str(&json).unwrap();
        assert_eq!(repo, deserialized);
    }
}