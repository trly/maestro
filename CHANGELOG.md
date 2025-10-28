# Changelog

All notable changes to Maestro will be documented in this file.

## [0.0.9] - 2025-10-24

### Changed

- Optimized CI/CD to reuse build artifacts from commit CI runs instead of re-running CI on tag push

## [0.0.8] - 2025-10-24

### Changed

- Refactored CI workflow to separate lint, build, and test jobs with platform matrix

## [0.0.7] - 2025-10-24

### Fixed

- Fixed bump_version tool to update all three version files (package.json, tauri.conf.json, Cargo.toml)

### Changed

- CI now defaults to pre-release for all releases
- Added .envrc for amp toolbox integration
- Updated release workflow to build platform-specific bundles (deb, rpm, AppImage for Linux; dmg for macOS)
