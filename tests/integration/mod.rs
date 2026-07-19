// Integration tests for the GBF Discord Bot
// These tests verify that different modules work together correctly

// Database integration tests
mod database_tests;

// Utilities integration tests
mod utils_test;

// Recruitment v2 integration tests (button-based recruitment)
mod recruitment_v2_tests;

// Facade integration tests (facade→service→repository結合テスト)
mod facades;

// メッセージキー整合性テスト（DBなし）
mod message_consistency_test;

// unwrap再混入ガード（Task13）
mod unwrap_guard_test;
