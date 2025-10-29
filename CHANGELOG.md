# Changelog

All notable changes to Maestro will be documented in this file.

## [0.2.6] - 2025-10-29

### Fixed

- Remove manual codesign step from CI, rely on Tauri's automatic ad-hoc signing

## [0.2.5] - 2025-10-29

### Changed

- CI now self-signs macOS application for distribution
- Updated architecture documentation

## [0.2.4] - 2025-10-29

### Fixed

- Release workflow now waits for CI to complete instead of just checking for existing runs

## [0.2.3] - 2025-10-29

### Fixed

- Conditionally import PredefinedMenuItem for macOS only

### Changed

- CI workflow now reuses existing artifacts when releasing tags

## [0.2.2] - 2025-10-29

### Changed

- Updated application icon with transparent background

## [0.2.1] - 2025-10-29

### Added

- Production logging with file rotation and environment controls

## [0.2.0] - 2025-10-29

### Added

- Resizable panels with PaneForge integration
- Synchronized scrolling for prompt console
- Improved sidebar layout with centered icons and tooltips

### Changed

- Replaced ampcode.com API with Amp `read_thread` tool for analysis
- Extracted shared table logic into reusable TableState class
- Optimized CI caching strategy to eliminate tauri-cli rebuild

### Fixed

- Preserve all UI settings when updating individual panel sizes
- Remove unnecessary bg-card from RepositorySelector

## [0.1.0] - 2025-10-29

### Added

- GitLab provider support with health checks
- Not pushed CI status for uncommitted changes
- Prettier formatting and tidy command

### Changed

- Refactored Git and CI logic using provider traits for better abstraction
- Updated settings page layout

### Fixed

- CI workflow now installs rustfmt component

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
