// Integration tests for the GBF Discord Bot
// These tests verify that different modules work together correctly

// Database integration tests
mod database_tests;

// Event handlers integration tests
mod event_handlers_tests;

// Utilities integration tests
mod utils_test;

// Recruitment v2 integration tests (button-based recruitment)
mod recruitment_v2_tests;

// Facade integration tests (facade→service→repository結合テスト)
mod facades;
