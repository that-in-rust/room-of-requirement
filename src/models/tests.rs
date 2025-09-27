use super::*;
use proptest::prelude::*;
use chrono::Utc;

// Property-based test generators
prop_compose! {
    fn arb_repository_owner()(
        id in 1i64..i64::MAX,
        login in "[a-zA-Z0-9_-]{1,39}",
        owner_type in prop::sample::select(vec!["User", "Organization", "Bot"]),
        site_admin in any::<bool>()
    ) -> RepositoryOwner {
        RepositoryOwner {
            id,
            login: login.clone(),
            owner_type: owner_type.to_string(),
            avatar_url: format!("https://avatars.githubusercontent.com/u/{}?v=4", id),
            html_url: format!("https://github.com/{}", login),
            site_admin,
        }
    }
}

prop_compose! {
    fn arb_repository_license()(
        key in "[a-z0-9-]{2,20}",
        name in "[A-Za-z0-9 .-]{5,50}",
        spdx_id in prop::option::of("[A-Z0-9-]{2,20}"),
        url in prop::option::of("https://[a-z./-]{10,100}")
    ) -> RepositoryLicense {
        RepositoryLicense {
            key,
            name,
            spdx_id,
            url,
        }
    }
}

prop_compose! {
    fn arb_repository()(
        id in 1i64..i64::MAX,
        full_name in "[a-zA-Z0-9_-]{1,39}/[a-zA-Z0-9_.-]{1,100}",
        description in prop::option::of("[a-zA-Z0-9 .,!?-]{0,500}"),
        size in 0i64..1_000_000,
        stargazers_count in 0i64..1_000_000,
        watchers_count in 0i64..1_000_000,
        forks_count in 0i64..100_000,
        open_issues_count in 0i64..10_000,
        language in prop::option::of(prop::sample::select(vec![
            "Rust", "JavaScript", "Python", "Go", "Java", "C++", "TypeScript"
        ])),
        default_branch in prop::sample::select(vec!["main", "master", "develop"]),
        visibility in prop::sample::select(vec!["public", "private", "internal"]),
        private in any::<bool>(),
        fork in any::<bool>(),
        archived in any::<bool>(),
        disabled in any::<bool>(),
        owner in arb_repository_owner(),
        license in prop::option::of(arb_repository_license()),
        topics in prop::collection::vec("[a-z0-9-]{2,20}", 0..10),
        has_issues in any::<bool>(),
        has_projects in any::<bool>(),
        has_wiki in any::<bool>(),
        has_pages in any::<bool>(),
        has_downloads in any::<bool>()
    ) -> Repository {
        let name = full_name.split('/').last().unwrap_or("repo").to_string();
        let now = Utc::now();
        
        Repository {
            id,
            full_name: full_name.clone(),
            name,
            description,
            html_url: format!("https://github.com/{}", full_name),
            clone_url: format!("https://github.com/{}.git", full_name),
            ssh_url: format!("git@github.com:{}.git", full_name),
            size,
            stargazers_count,
            watchers_count,
            forks_count,
            open_issues_count,
            language: language.map(|s| s.to_string()),
            default_branch: default_branch.to_string(),
            visibility: visibility.to_string(),
            private,
            fork,
            archived,
            disabled,
            created_at: now,
            updated_at: now,
            pushed_at: Some(now),
            owner,
            license,
            topics,
            has_issues,
            has_projects,
            has_wiki,
            has_pages,
            has_downloads,
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_repository_validation_comprehensive() {
        let _repo = create_valid_test_repository();
        
        // Test all validation rules
        let validation_tests = vec![
            // (field_modifier, expected_error_field)
            (Box::new(|r: &mut Repository| r.full_name = "".to_string()) as Box<dyn Fn(&mut Repository)>, "full_name"),
            (Box::new(|r: &mut Repository| r.name = "".to_string()), "name"),
            (Box::new(|r: &mut Repository| r.html_url = "".to_string()), "html_url"),
            (Box::new(|r: &mut Repository| r.clone_url = "".to_string()), "clone_url"),
            (Box::new(|r: &mut Repository| r.ssh_url = "".to_string()), "ssh_url"),
            (Box::new(|r: &mut Repository| r.default_branch = "".to_string()), "default_branch"),
            (Box::new(|r: &mut Repository| r.visibility = "".to_string()), "visibility"),
            (Box::new(|r: &mut Repository| r.html_url = "https://example.com/repo".to_string()), "html_url"),
            (Box::new(|r: &mut Repository| r.clone_url = "https://example.com/repo.git".to_string()), "clone_url"),
            (Box::new(|r: &mut Repository| r.ssh_url = "git@example.com:repo.git".to_string()), "ssh_url"),
            (Box::new(|r: &mut Repository| r.visibility = "invalid".to_string()), "visibility"),
            (Box::new(|r: &mut Repository| r.size = -1), "size"),
            (Box::new(|r: &mut Repository| r.stargazers_count = -1), "stargazers_count"),
            (Box::new(|r: &mut Repository| r.watchers_count = -1), "watchers_count"),
            (Box::new(|r: &mut Repository| r.forks_count = -1), "forks_count"),
            (Box::new(|r: &mut Repository| r.open_issues_count = -1), "open_issues_count"),
        ];

        for (modifier, expected_field) in validation_tests {
            let mut test_repo = create_valid_test_repository();
            modifier(&mut test_repo);
            
            let result = test_repo.validate();
            assert!(result.is_err(), "Expected validation error for field: {}", expected_field);
            
            let error_msg = result.unwrap_err().to_string();
            assert!(error_msg.contains(expected_field), 
                "Error message should contain field '{}': {}", expected_field, error_msg);
        }
    }

    #[test]
    fn test_repository_owner_validation_comprehensive() {
        let validation_tests = vec![
            (Box::new(|o: &mut RepositoryOwner| o.login = "".to_string()) as Box<dyn Fn(&mut RepositoryOwner)>, "owner.login"),
            (Box::new(|o: &mut RepositoryOwner| o.avatar_url = "".to_string()), "owner.avatar_url"),
            (Box::new(|o: &mut RepositoryOwner| o.html_url = "".to_string()), "owner.html_url"),
            (Box::new(|o: &mut RepositoryOwner| o.owner_type = "Invalid".to_string()), "owner.type"),
            (Box::new(|o: &mut RepositoryOwner| o.html_url = "https://example.com/user".to_string()), "owner.html_url"),
        ];

        for (modifier, expected_field) in validation_tests {
            let mut owner = create_valid_test_owner();
            modifier(&mut owner);
            
            let result = owner.validate();
            assert!(result.is_err(), "Expected validation error for field: {}", expected_field);
            
            let error_msg = result.unwrap_err().to_string();
            assert!(error_msg.contains(expected_field), 
                "Error message should contain field '{}': {}", expected_field, error_msg);
        }
    }

    #[test]
    fn test_repository_license_validation() {
        let mut license = create_valid_test_license();
        
        // Test empty key
        license.key = "".to_string();
        assert!(license.validate().is_err());
        
        // Test empty name
        license = create_valid_test_license();
        license.name = "".to_string();
        assert!(license.validate().is_err());
        
        // Test valid license
        license = create_valid_test_license();
        assert!(license.validate().is_ok());
    }

    #[test]
    fn test_query_metadata_lifecycle() {
        let mut metadata = QueryMetadata::new(
            "rust language:rust".to_string(),
            "repos_20231201120000".to_string()
        );

        // Test initial state
        assert_eq!(metadata.search_query, "rust language:rust");
        assert_eq!(metadata.table_name, "repos_20231201120000");
        assert_eq!(metadata.result_count, 0);
        assert!(!metadata.success);
        assert!(metadata.error_message.is_none());
        assert_eq!(metadata.duration_ms, 0);

        // Test success marking
        metadata.mark_success(100, 1500);
        assert_eq!(metadata.result_count, 100);
        assert!(metadata.success);
        assert_eq!(metadata.duration_ms, 1500);
        assert!(metadata.error_message.is_none());

        // Test failure marking
        let mut failure_metadata = QueryMetadata::new("test".to_string(), "test_table".to_string());
        failure_metadata.mark_failure("Test error".to_string(), 500);
        assert_eq!(failure_metadata.result_count, 0);
        assert!(!failure_metadata.success);
        assert_eq!(failure_metadata.duration_ms, 500);
        assert_eq!(failure_metadata.error_message, Some("Test error".to_string()));
    }

    #[test]
    fn test_generate_table_name_suffix() {
        let test_cases = vec![
            ("octocat/Hello-World", "octocat_hello_world"),
            ("rust-lang/rust", "rust_lang_rust"),
            ("user/repo-with-dashes", "user_repo_with_dashes"),
            ("ORG/UPPERCASE", "org_uppercase"),
            ("user/repo.with.dots", "user_repowithdots"),
            ("user/repo@with#special$chars", "user_repowithspecialchars"),
        ];

        for (input, expected) in test_cases {
            let repo = Repository {
                full_name: input.to_string(),
                ..create_valid_test_repository()
            };
            assert_eq!(repo.generate_table_name_suffix(), expected);
        }
    }

    #[test]
    fn test_search_response_serialization() {
        let repo = create_valid_test_repository();
        let search_response = SearchResponse {
            total_count: 1,
            incomplete_results: false,
            items: vec![repo.clone()],
        };

        // Test serialization
        let json = serde_json::to_string(&search_response).unwrap();
        assert!(!json.is_empty());

        // Test deserialization
        let deserialized: SearchResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.total_count, 1);
        assert!(!deserialized.incomplete_results);
        assert_eq!(deserialized.items.len(), 1);
        assert_eq!(deserialized.items[0], repo);
    }

    #[test]
    fn test_table_name_generation_format() {
        let table_name = QueryMetadata::generate_table_name();
        
        // Should start with "repos_"
        assert!(table_name.starts_with("repos_"));
        
        // Should have correct length (repos_ + YYYYMMDDHHMMSS)
        assert_eq!(table_name.len(), 20);
        
        // Should contain only valid characters
        assert!(table_name.chars().all(|c| c.is_alphanumeric() || c == '_'));
        
        // Should be parseable as a timestamp
        let timestamp_part = &table_name[6..]; // Remove "repos_" prefix
        assert_eq!(timestamp_part.len(), 14);
        assert!(timestamp_part.chars().all(|c| c.is_ascii_digit()));
    }

    fn create_valid_test_repository() -> Repository {
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
            owner: create_valid_test_owner(),
            license: Some(create_valid_test_license()),
            topics: vec!["octocat".to_string(), "atom".to_string()],
            has_issues: true,
            has_projects: true,
            has_wiki: true,
            has_pages: false,
            has_downloads: true,
        }
    }

    fn create_valid_test_owner() -> RepositoryOwner {
        RepositoryOwner {
            id: 1,
            login: "octocat".to_string(),
            owner_type: "User".to_string(),
            avatar_url: "https://github.com/images/error/octocat_happy.gif".to_string(),
            html_url: "https://github.com/octocat".to_string(),
            site_admin: false,
        }
    }

    fn create_valid_test_license() -> RepositoryLicense {
        RepositoryLicense {
            key: "mit".to_string(),
            name: "MIT License".to_string(),
            spdx_id: Some("MIT".to_string()),
            url: Some("https://api.github.com/licenses/mit".to_string()),
        }
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;

    proptest! {
        #[test]
        fn test_repository_validation_invariants(repo in arb_repository()) {
            // All generated repositories should be valid
            prop_assert!(repo.validate().is_ok());
        }

        #[test]
        fn test_repository_serialization_roundtrip(repo in arb_repository()) {
            let json = serde_json::to_string(&repo)?;
            let deserialized: Repository = serde_json::from_str(&json)?;
            prop_assert_eq!(repo, deserialized);
        }

        #[test]
        fn test_search_response_serialization_roundtrip(
            repos in prop::collection::vec(arb_repository(), 0..10),
            total_count in 0i64..1_000_000,
            incomplete_results in any::<bool>()
        ) {
            let search_response = SearchResponse {
                total_count,
                incomplete_results,
                items: repos.clone(),
            };

            let json = serde_json::to_string(&search_response)?;
            let deserialized: SearchResponse = serde_json::from_str(&json)?;
            
            prop_assert_eq!(search_response.total_count, deserialized.total_count);
            prop_assert_eq!(search_response.incomplete_results, deserialized.incomplete_results);
            prop_assert_eq!(search_response.items, deserialized.items);
        }

        #[test]
        fn test_table_name_suffix_generation_invariants(
            full_name in "[a-zA-Z0-9_-]{1,39}/[a-zA-Z0-9_.-]{1,100}"
        ) {
            let repo = Repository {
                full_name: full_name.clone(),
                ..create_valid_test_repository()
            };
            
            let suffix = repo.generate_table_name_suffix();
            
            // Should only contain valid PostgreSQL identifier characters
            prop_assert!(suffix.chars().all(|c| c.is_alphanumeric() || c == '_'));
            
            // Should not be empty
            prop_assert!(!suffix.is_empty());
            
            // Should be lowercase
            prop_assert_eq!(suffix.clone(), suffix.to_lowercase());
        }

        #[test]
        fn test_query_metadata_uuid_uniqueness(
            query1 in "[a-zA-Z0-9 ]{1,100}",
            query2 in "[a-zA-Z0-9 ]{1,100}",
            table1 in "repos_[0-9]{14}",
            table2 in "repos_[0-9]{14}"
        ) {
            let metadata1 = QueryMetadata::new(query1, table1);
            let metadata2 = QueryMetadata::new(query2, table2);
            
            // UUIDs should always be unique
            prop_assert_ne!(metadata1.id, metadata2.id);
        }
    }

    fn create_valid_test_repository() -> Repository {
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
            topics: vec!["octocat".to_string(), "atom".to_string()],
            has_issues: true,
            has_projects: true,
            has_wiki: true,
            has_pages: false,
            has_downloads: true,
        }
    }
}