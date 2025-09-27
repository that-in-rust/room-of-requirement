use thiserror::Error;

/// Comprehensive error hierarchy for the GitHub PostgreSQL query tool
#[derive(Error, Debug)]
pub enum AppError {
    #[error("GitHub API error: {message}")]
    GitHubApi { message: String },

    #[error("GitHub API rate limit exceeded: {reset_time}")]
    RateLimit { reset_time: String },

    #[error("GitHub API authentication failed: {reason}")]
    Authentication { reason: String },

    #[error("Invalid GitHub search query: {query} - {reason}")]
    InvalidQuery { query: String, reason: String },

    #[error("Database connection error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Database table creation failed: {table_name} - {reason}")]
    TableCreation { table_name: String, reason: String },

    #[error("Repository data validation failed: {field} - {reason}")]
    Validation { field: String, reason: String },

    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON serialization/deserialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Environment variable missing or invalid: {var_name}")]
    Environment { var_name: String },

    #[error("Configuration error: {message}")]
    Configuration { message: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Timeout error: operation took longer than {timeout_seconds} seconds")]
    Timeout { timeout_seconds: u64 },

    #[error("Internal error: {message}")]
    Internal { message: String },
}

/// Result type alias for convenience
pub type Result<T> = std::result::Result<T, AppError>;

impl AppError {
    /// Create a new GitHub API error
    pub fn github_api(message: impl Into<String>) -> Self {
        Self::GitHubApi {
            message: message.into(),
        }
    }

    /// Create a new rate limit error
    pub fn rate_limit(reset_time: impl Into<String>) -> Self {
        Self::RateLimit {
            reset_time: reset_time.into(),
        }
    }

    /// Create a new authentication error
    pub fn authentication(reason: impl Into<String>) -> Self {
        Self::Authentication {
            reason: reason.into(),
        }
    }

    /// Create a new invalid query error
    pub fn invalid_query(query: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidQuery {
            query: query.into(),
            reason: reason.into(),
        }
    }

    /// Create a new table creation error
    pub fn table_creation(table_name: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::TableCreation {
            table_name: table_name.into(),
            reason: reason.into(),
        }
    }

    /// Create a new validation error
    pub fn validation(field: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::Validation {
            field: field.into(),
            reason: reason.into(),
        }
    }

    /// Create a new environment error
    pub fn environment(var_name: impl Into<String>) -> Self {
        Self::Environment {
            var_name: var_name.into(),
        }
    }

    /// Create a new configuration error
    pub fn configuration(message: impl Into<String>) -> Self {
        Self::Configuration {
            message: message.into(),
        }
    }

    /// Create a new timeout error
    pub fn timeout(timeout_seconds: u64) -> Self {
        Self::Timeout { timeout_seconds }
    }

    /// Create a new internal error
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }
}