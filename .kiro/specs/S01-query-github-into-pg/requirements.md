# Requirements Document

## Introduction

Simple tool that executes GitHub API search queries and stores results in PostgreSQL tables.

## Requirements

### Requirement 1

**User Story:** As a developer, I want to run any valid GitHub API search query and store the results in PostgreSQL, so that I can analyze repository data.

#### Acceptance Criteria

1. WHEN I provide a GitHub search query THEN the system SHALL execute it against the GitHub API
2. WHEN results are fetched THEN the system SHALL store them in a timestamped table (repos_YYYYMMDDHHMMSS format)
3. WHEN a query completes THEN the system SHALL record query metadata (query string, result count, timestamp)
4. WHEN using GitHub API THEN the system SHALL handle authentication via environment variable
5. WHEN API errors occur THEN the system SHALL display clear error messages

### Requirement 2

**User Story:** As a developer, I want to use standard GitHub search syntax, so that I can leverage existing GitHub search knowledge.

#### Acceptance Criteria

1. WHEN I provide a search query THEN the system SHALL accept any valid GitHub repository search syntax
2. WHEN search executes THEN the system SHALL store results in a timestamped table name as above - the columns can be different for the table based on the query
3. WHEN search completes THEN the system SHALL show the table name and result count

### Requirement 3

**User Story:** As a developer, I want a simple command-line interface, so that I can quickly run searches.

#### Acceptance Criteria

1. WHEN I run the tool with a query THEN the system SHALL execute the search and store results
2. WHEN the tool runs THEN the system SHALL show progress and completion status
3. WHEN errors occur THEN the system SHALL display actionable error messages