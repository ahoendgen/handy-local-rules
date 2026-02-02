# Changelog

All notable changes to handy-local-rules will be documented in this file.

## [0.1.0] - 2026-02-02

### Features

- OpenAI-compatible Chat Completion API for text transformation
- Rule engine with regex, function, and shell rule types
- Hot-reload: rules are automatically reloaded when files change
- Web dashboard for testing rules and monitoring status
- Swagger UI for interactive API documentation
- CLI commands: `serve`, `transform`, `validate`, `list-rules`, `status`, `setup`, `dashboard`, `logs`
- `logs` command with `-f` follow mode for continuous monitoring
- macOS launchd service support via Makefile
- German punctuation rules (Satzzeichen)
- Text cleanup rules (whitespace, duplicate punctuation)

### Rules

- German: Convert spoken punctuation to symbols (Punkt → ., Komma → ,, etc.)
- Cleanup: Normalize whitespace, remove duplicate punctuation
- Dev commands: AC → "add and commit", SAC → "/add-and-commit"
