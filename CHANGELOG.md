# Changelog

All notable changes to Maestro will be documented in this file.

## [0.4.12] - 2025-11-04

### Changed

- Agent: Use lightweight tags for version bumps
- Agent: Improved bump_version tool formatting

## [0.4.11] - 2025-11-04

### Changed

- Removed fake code signing configuration
- Updated copyright information

## [0.4.10] - 2025-11-04

### Added

- HTTPS authentication with PAT fallback for Git operations

### Fixed

- Retry logic now starts fresh session when no thread exists

### Changed

- Improved UX with documentation, settings, and error handling

## [0.4.9] - 2025-11-04

### Fixed

- Release workflow: Use HOMEBREW_TAP_TOKEN for cross-repository dispatch to homebrew tap

## [0.4.8] - 2025-11-04

### Changed

- Release workflow now triggers Homebrew tap update via repository_dispatch instead of direct push

## [0.4.7] - 2025-11-03

### Fixed

- CI: Build both app and dmg bundles on macOS to preserve .app for zipping

## [0.4.6] - 2025-11-03

### Added

- ZIP distribution format for macOS alongside DMG (better Gatekeeper experience)
- SHA256 checksums generated during CI build and included as artifacts

### Changed

- Migrated Homebrew cask to separate tap repository (trly/homebrew-maestro)
- Simplified Homebrew installation: `brew tap trly/maestro && brew install maestro`
- Release workflow now auto-updates tap repository with version and SHA256
- Removed cask update logic from bump_version tool

### Fixed

- Homebrew cask location moved to correct tap structure

## [0.4.5] - 2025-11-03

### Added

- Homebrew tap for macOS installation

## [0.4.4] - 2025-11-03

### Fixed

- CI: Fixed Rust linting failures by replacing mise-action with dtolnay/rust-toolchain
- CI: Added explicit rustfmt and clippy components to Rust toolchain setup
- CI: Fixed build step working directory for cargo tauri build

### Changed

- CI: Replaced mise-action with direct version pinning for all tools
- CI: Use taiki-e/install-action for faster Tauri CLI installation
- CI: Optimized Rust cache placement and added OS-specific cache keys

## [0.4.3] - 2025-11-03

### Fixed

- Replace bun with pnpm in Tauri build commands

### Changed

- CI workflow optimized to eliminate duplicate compilation steps

## [0.4.2] - 2025-11-03

### Changed

- CI runtime versions pinned with mise

## [0.4.0] - 2025-11-03

### Changed

- Simplified Amp integration to use system `amp` CLI with `--stream-json` flag instead of custom executor binary

## [0.3.0] - 2025-10-31

### Added

- Standalone executor binary for improved sidecar architecture
- First-run dialog for initial setup
- Help option to launch getting started dialog

### Changed

- CI workflow optimized with validated Tauri patterns and improved caching
- Replaced custom binary search with 'which' crate

### Fixed

- CI linting now blocks builds on failure
- CI workflow missing bun commands
- Amp-executor builds before Rust cache runs
- Copy/paste functionality restored

### Security

- Hardened shell plugin

## [0.2.8] - 2025-10-29

### Fixed

- Statically link OpenSSL to prevent macOS installer crashes

## [0.2.7] - 2025-10-29

### Fixed

- Configure ad-hoc code signing for macOS builds to address quarantine issues

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
