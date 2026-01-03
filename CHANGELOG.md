# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.3] - 2026-01-03

### Added

- Comprehensive integration test suite with Docker infrastructure
  - Core tests: oneshot queries, streaming, multi-turn conversations
  - Error handling tests: connection, timeout, cancellation
  - Concurrent session tests: parallel queries, resource isolation
  - Hook tests: PreToolUse, PostToolUse lifecycle events
  - Budget/cost verification tests
  - Context and session management tests
- Property-based tests using `proptest` crate (19 tests)
- MCP unit tests (29 tests for types, builders, tool execution)
- Mock transport for deterministic unit testing
- Serialization round-trip tests for core types
- Edge case tests (unicode, empty values, boundary conditions)
- New examples:
  - `error_handling.rs` - Error handling patterns
  - `streaming_progress.rs` - Progress indicators for streaming
- Docker integration test infrastructure
  - `Dockerfile.integration-tests` for isolated test environment
  - `docker-compose.integration-tests.yml` for easy test execution
  - OAuth token support for authenticated testing
- GitHub Actions CI with optional integration test job
- Improved documentation with more rustdoc examples in `lib.rs`
- TESTING.md guide for running integration tests

### Changed

- Consolidated redundant test files into comprehensive test modules
- Improved test naming with descriptive assertions

### Fixed

- Removed unused imports and dead code warnings
- Fixed test assertions for `num_turns` behavior

## [0.1.2] - 2025-01-02

### Added

- CI/CD workflow with GitHub Actions
- Automatic publishing to crates.io on version bump
- CHANGELOG.md following Keep a Changelog format

## [0.1.1] - 2025-01-02

### Fixed

- Removed unnecessary `unsafe impl Send for QueryStream` - the type is automatically Send
- Changed `into_stream()` to return `Result` instead of panicking with `.expect()`
- Fixed MCP calculator example to use correct `ToolInputSchema` and `ToolResult` APIs

## [0.1.0] - 2025-01-02

### Added

- Initial release of the Claude Agents SDK for Rust
- `query()` function for one-shot queries returning async streams
- `ClaudeClient` for bidirectional streaming communication
- Full type definitions for Claude Code CLI messages
- Tool permission callbacks (`CanUseTool`)
- Hook system for pre/post tool use events
- MCP server configuration support
- Comprehensive error types with `thiserror`
- Builder pattern for `ClaudeAgentOptions`
- Support for all Claude CLI options (model, system prompt, permissions, etc.)

[Unreleased]: https://github.com/jimmystridh/claude-agents-sdk/compare/v0.1.3...HEAD
[0.1.3]: https://github.com/jimmystridh/claude-agents-sdk/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/jimmystridh/claude-agents-sdk/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/jimmystridh/claude-agents-sdk/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/jimmystridh/claude-agents-sdk/releases/tag/v0.1.0
