# Maestro Documentation

## Quick Start

New to Maestro? Start here:

1. **[Architecture Overview](./architecture.md)** - Understand the system design and core domains
2. **[Component Architecture](./components.md)** - Frontend component organization and patterns
3. **[SSH Authentication](./ssh-authentication.md)** - Set up SSH for repository access (required)
4. **[IPC Guide](./ipc-guide.md)** - Learn type-safe backend communication
5. **[Reactivity Guide](./reactivity.md)** - Svelte 5 runes mode patterns

## Critical: Styling Guidelines

**ALWAYS use semantic tokens, NEVER raw colors:**

When writing any UI code, you **MUST** use semantic tokens like `text-success`, `bg-primary`, `text-destructive` instead of raw Tailwind colors like `text-green-600`, `bg-blue-500`, `text-red-600`.

See [AGENTS.md Status Icon/Color Mappings](../AGENTS.md#styling--theming) for complete reference.

## Core Domain Guides

### [Architecture Overview](./architecture.md)

**Read first for system-wide understanding**

Complete architectural overview covering:

- Technology stack (VCS integration, AI execution SDK)
- Core domain models (Repositories, Prompt Sets, Revisions, Executions)
- Data flow architecture (IPC, Event Bus, Reactivity)
- Security architecture (Platform keyring, SSH)
- File system layout
- Concurrency & safety patterns

### [Prompt Sets](./prompt-sets.md)

**When to read:** Working with prompt sets, revisions, or repositories

Covers the organizational layer:

- Creating and managing prompt sets
- Prompt revision DAG structure
- Repository management
- Executing revisions against repositories
- Query API and UI components

### [Executions & Validations](./executions.md)

**When to read:** Working with execution lifecycle, validation, or commits

Complete execution lifecycle:

- Worktree creation and isolation
- Real-time status tracking
- Validation flow
- Commit and push operations
- Cancellation and resume
- Bulk operations
- Error handling and reconciliation

### [Failure Analysis](./analyses.md)

**When to read:** Working with failure analysis or Amp V2 API integration

Aggregating and analyzing failures:

- Analysis domain model and workflow
- Amp V2 API integration
- Thread message fetching and formatting
- Background analysis execution
- Frontend trigger UI and result display
- Security and performance considerations

### [Change Tracking](./change-tracking.md)

**When to read:** Working with diffs, file changes, or change statistics

How Maestro tracks code changes:

- Worktree diff vs. committed diff concepts
- Change lifecycle
- Frontend API for accessing diffs
- Statistics calculation
- File-level diff viewing

### [CI Tracking](./ci-tracking.md)

**When to read:** Working with CI integration or VCS provider APIs

VCS CI/CD integration:

- CI status states (pending, passed, failed, not_configured)
- Workflow from commit → push → CI check
- VCS provider support (GitHub, GitLab)
- Polling strategy and rate limits
- UI components (badges, status display)

## Frontend Guides

### [Component Architecture](./components.md)

**When to read:** Building or modifying UI components

Frontend component organization:

- Directory structure (`executions/`, `ui/`)
- Execution display components (Table, Filters, List, Row)
- Reusable UI primitives (BulkActionBar, IconButton, FilterInput)
- Data flow patterns (props, event bus, optimistic UI)
- Performance optimizations (virtual scrolling, derived state)
- Common patterns (selection, loading states)

### [Reactivity Guide](./reactivity.md)

**When to read:** Working with Svelte 5 runes and reactive state

Critical Svelte 5 patterns:

- Props pattern (never destructure!)
- Runes (`$state`, `$derived`, `$effect`, `$props`)
- Composables pattern
- bits-ui integration
- Top 5 reactivity footguns to avoid

## Infrastructure Guides

### [IPC Guide](./ipc-guide.md)

**When to read:** Adding new backend commands or features

Centralized type-safe IPC layer:

- Common patterns and examples
- Error handling with `TauriIPCError`
- Complete API reference (repositories, prompt sets, executions, tokens)
- Migration from direct `invoke()` calls

### [Execution Event Bus](./execution-event-bus.md)

**When to read:** Building UI that needs real-time execution updates

Real-time event system:

- Subscribing to execution events
- Supported events (session, status, validation, commit, progress, CI)
- Reactive execution state patterns
- Event store access patterns
- Backend event types

## Setup & Configuration

### [Settings & Configuration](./settings.md)

**When to read:** Configuring editors, terminals, or CI preferences

User settings and preferences:

- Available settings (editor, terminal, CI threshold)
- Settings store initialization
- Editor and terminal detection
- Opening worktrees with configured tools
- Secure token storage via platform keyring
- Best practices and fallback behavior

### [SSH Authentication](./ssh-authentication.md)

**When to read:** During initial setup or troubleshooting git operations

SSH setup for VCS repositories:

- SSH key generation and configuration
- Adding keys to GitHub/GitLab and ssh-agent
- How Maestro uses SSH authentication
- Troubleshooting common issues
- Multiple SSH keys configuration

### [Sourcegraph Integration](./sourcegraph-integration.md)

**When to read:** Working with repository search or Sourcegraph API

Sourcegraph repository search integration:

- GraphQL client architecture
- Configuration and token storage
- Search query syntax and examples
- Frontend/backend API usage
- Error handling patterns

### [GitLab Integration Readiness](./gitlab-integration-readiness.md)

**When to read:** Planning or implementing GitLab provider support

GitLab integration preparation:

- Provider abstraction architecture
- Required configuration structures
- CI and Git provider contexts
- Implementation checklist
- Code locations requiring updates

## Build & Distribution

### [Distribution](./distribution.md)

**When to read:** Preparing releases or distributing the app

App distribution guide:

- Apple Developer account requirements
- Code signing and notarization
- GitHub Actions CI/CD setup
- Manual build and distribution
- Troubleshooting signing issues

## Examples & References

### [Analysis Example](./analysis-example.md)

**When to read:** Understanding practical analysis workflow

Real-world analysis example:

- Creating and running analyses
- Working with analysis results
- Frontend/backend interaction patterns

## Related Files

- **[AGENTS.md](../AGENTS.md)** - Commands, architecture, and conventions for AI agents
- **[README.md](../README.md)** - Project overview and getting started

## Known Issues & Fixes

### macOS Code Signing with OpenSSL

**Problem**: Installed app crashes on launch with "Library not loaded: /opt/homebrew/*/libssl.3.dylib" even after running `xattr -d com.apple.quarantine`.

**Root Cause**: Dynamic linking to Homebrew's OpenSSL creates Team ID mismatch between app and library.

**Solution**: OpenSSL is now statically linked via:
- `git2` uses `vendored-openssl` and `vendored-libgit2` features
- `reqwest` uses `native-tls-vendored` feature  
- `build.rs` sets `OPENSSL_STATIC=1` for macOS builds

Rebuild with `make build` to generate a properly signed, self-contained installer.

## Documentation Standards

When updating docs:

1. **Use high-level abstractions** - Avoid implementation details like specific Rust crates or file paths
2. **Verify accuracy** - Check against current codebase
3. **Include examples** - Show usage patterns and API calls
4. **Link related docs** - Help users discover related concepts
5. **Update this index** - Keep README.md in sync
6. **Follow domain structure** - Organize by core domain concepts, not implementation details