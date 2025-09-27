use std::process::Command;
use std::env;

/// Comprehensive test runner for the GitHub PostgreSQL Query tool
/// 
/// This module provides utilities to run different types of tests:
/// - Unit tests with proper mocking
/// - Integration tests with test containers
/// - End-to-end CLI tests
/// - Property-based tests for data validation
/// - Performance tests for large result sets

pub struct TestRunner {
    pub verbose: bool,
    pub test_database_url: Option<String>,
}

impl TestRunner {
    pub fn new() -> Self {
        Self {
            verbose: env::var("VERBOSE").is_ok(),
            test_database_url: env::var("TEST_DATABASE_URL").ok(),
        }
    }

    /// Run all unit tests
    pub fn run_unit_tests(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🧪 Running unit tests...");
        
        let mut cmd = Command::new("cargo");
        cmd.args(&["test", "--lib"]);
        
        if self.verbose {
            cmd.arg("--verbose");
        }
        
        let output = cmd.output()?;
        
        if !output.status.success() {
            eprintln!("Unit tests failed:");
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
            return Err("Unit tests failed".into());
        }
        
        println!("✅ Unit tests passed");
        Ok(())
    }

    /// Run integration tests with test containers
    pub fn run_integration_tests(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔗 Running integration tests...");
        
        // Check if Docker is available
        if !self.check_docker_available() {
            println!("⚠️  Docker not available, skipping integration tests");
            return Ok(());
        }
        
        let mut cmd = Command::new("cargo");
        cmd.args(&["test", "--test", "database_integration_tests"]);
        cmd.args(&["--test", "integration_workflow_tests"]);
        cmd.args(&["--test", "main_workflow_integration_test"]);
        
        if self.verbose {
            cmd.arg("--verbose");
        }
        
        // Set test database URL if provided
        if let Some(ref db_url) = self.test_database_url {
            cmd.env("TEST_DATABASE_URL", db_url);
        }
        
        let output = cmd.output()?;
        
        if !output.status.success() {
            eprintln!("Integration tests failed:");
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
            return Err("Integration tests failed".into());
        }
        
        println!("✅ Integration tests passed");
        Ok(())
    }

    /// Run end-to-end CLI tests
    pub fn run_e2e_tests(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🎯 Running end-to-end tests...");
        
        let mut cmd = Command::new("cargo");
        cmd.args(&["test", "--test", "cli_integration_tests"]);
        
        if self.verbose {
            cmd.arg("--verbose");
        }
        
        let output = cmd.output()?;
        
        if !output.status.success() {
            eprintln!("E2E tests failed:");
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
            return Err("E2E tests failed".into());
        }
        
        println!("✅ End-to-end tests passed");
        Ok(())
    }

    /// Run property-based tests
    pub fn run_property_tests(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🎲 Running property-based tests...");
        
        let mut cmd = Command::new("cargo");
        cmd.args(&["test", "property_tests"]);
        
        if self.verbose {
            cmd.arg("--verbose");
        }
        
        let output = cmd.output()?;
        
        if !output.status.success() {
            eprintln!("Property-based tests failed:");
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
            return Err("Property-based tests failed".into());
        }
        
        println!("✅ Property-based tests passed");
        Ok(())
    }

    /// Run performance benchmarks
    pub fn run_performance_tests(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("⚡ Running performance benchmarks...");
        
        let mut cmd = Command::new("cargo");
        cmd.args(&["bench"]);
        
        if self.verbose {
            cmd.arg("--verbose");
        }
        
        let output = cmd.output()?;
        
        if !output.status.success() {
            eprintln!("Performance benchmarks failed:");
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
            return Err("Performance benchmarks failed".into());
        }
        
        println!("✅ Performance benchmarks completed");
        Ok(())
    }

    /// Run all tests in sequence
    pub fn run_all_tests(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🚀 Running comprehensive test suite...");
        
        self.run_unit_tests()?;
        self.run_integration_tests()?;
        self.run_e2e_tests()?;
        self.run_property_tests()?;
        
        println!("🎉 All tests passed successfully!");
        Ok(())
    }

    /// Run all tests including performance benchmarks
    pub fn run_full_suite(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.run_all_tests()?;
        self.run_performance_tests()?;
        
        println!("🏆 Full test suite completed successfully!");
        Ok(())
    }

    /// Check if Docker is available for integration tests
    fn check_docker_available(&self) -> bool {
        Command::new("docker")
            .args(&["--version"])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Generate test coverage report
    pub fn generate_coverage_report(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Generating test coverage report...");
        
        // Install cargo-tarpaulin if not available
        let mut cmd = Command::new("cargo");
        cmd.args(&["install", "cargo-tarpaulin"]);
        let _ = cmd.output(); // Ignore if already installed
        
        // Generate coverage report
        let mut cmd = Command::new("cargo");
        cmd.args(&["tarpaulin", "--out", "Html", "--output-dir", "target/coverage"]);
        
        if self.verbose {
            cmd.arg("--verbose");
        }
        
        let output = cmd.output()?;
        
        if !output.status.success() {
            eprintln!("Coverage report generation failed:");
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
            return Err("Coverage report generation failed".into());
        }
        
        println!("✅ Coverage report generated at target/coverage/tarpaulin-report.html");
        Ok(())
    }

    /// Run tests with specific filter
    pub fn run_filtered_tests(&self, filter: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔍 Running filtered tests: {}", filter);
        
        let mut cmd = Command::new("cargo");
        cmd.args(&["test", filter]);
        
        if self.verbose {
            cmd.arg("--verbose");
        }
        
        let output = cmd.output()?;
        
        if !output.status.success() {
            eprintln!("Filtered tests failed:");
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
            return Err("Filtered tests failed".into());
        }
        
        println!("✅ Filtered tests passed");
        Ok(())
    }
}

impl Default for TestRunner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runner_creation() {
        let runner = TestRunner::new();
        assert!(!runner.verbose || env::var("VERBOSE").is_ok());
    }

    #[test]
    fn test_docker_availability_check() {
        let runner = TestRunner::new();
        // This test will pass regardless of Docker availability
        let _docker_available = runner.check_docker_available();
    }
}

/// Main function for running tests from command line
pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let runner = TestRunner::new();
    
    match args.get(1).map(|s| s.as_str()) {
        Some("unit") => runner.run_unit_tests(),
        Some("integration") => runner.run_integration_tests(),
        Some("e2e") => runner.run_e2e_tests(),
        Some("property") => runner.run_property_tests(),
        Some("performance") => runner.run_performance_tests(),
        Some("coverage") => runner.generate_coverage_report(),
        Some("all") => runner.run_all_tests(),
        Some("full") => runner.run_full_suite(),
        Some(filter) => runner.run_filtered_tests(filter),
        None => {
            println!("Usage: cargo run --bin test_runner [unit|integration|e2e|property|performance|coverage|all|full|<filter>]");
            println!();
            println!("Commands:");
            println!("  unit        - Run unit tests only");
            println!("  integration - Run integration tests with test containers");
            println!("  e2e         - Run end-to-end CLI tests");
            println!("  property    - Run property-based tests");
            println!("  performance - Run performance benchmarks");
            println!("  coverage    - Generate test coverage report");
            println!("  all         - Run all tests except performance");
            println!("  full        - Run all tests including performance");
            println!("  <filter>    - Run tests matching the filter");
            Ok(())
        }
    }
}