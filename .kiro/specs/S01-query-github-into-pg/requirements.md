# Requirements Document20250927162247

## Introduction

Simple GitHub repository discovery tool that fetches top repositories and stores them in timestamped PostgreSQL tables.

## Requirements

### Requirement 1

**User Story:** As a researcher, I want to fetch top GitHub repositories by stars/size/date and store them in PostgreSQL, so that I can build a database of repositories for analysis.

#### Acceptance Criteria

1. WHEN I run the tool with search criteria THEN the system SHALL fetch top 1000 repositories from GitHub API
2. WHEN repositories are fetched THEN the system SHALL store them in a timestamped table (TableYYYYMMDDHHSS format)
3. WHEN a query completes THEN the system SHALL record query metadata in a MegaTable with SQL query, row count, and timestamp
4. WHEN using GitHub API THEN the system SHALL securely handle API tokens from environment variables
5. WHEN API rate limits are hit THEN the system SHALL handle errors gracefully

### Requirement 2

**User Story:** As a researcher, I want to search repositories by different criteria, so that I can get different types of repository datasets.

#### Acceptance Criteria

1. WHEN I specify "stars" criteria THEN the system SHALL search repositories ordered by star count
2. WHEN I specify "size" criteria THEN the system SHALL search repositories ordered by repository size
3. WHEN I specify "date" criteria THEN the system SHALL search repositories ordered by creation date
4. WHEN search completes THEN the system SHALL store basic repository metadata (name, stars, forks, language, url, description)

### Requirement 3

**User Story:** As a researcher, I want a simple command-line interface, so that I can easily run repository searches.

#### Acceptance Criteria

1. WHEN I run the bash script THEN the system SHALL provide a simple interface to specify search criteria
2. WHEN the tool runs THEN the system SHALL show progress and completion status
3. WHEN errors occur THEN the system SHALL display clear error messages
4. WHEN the tool completes THEN the system SHALL show the table name and row count