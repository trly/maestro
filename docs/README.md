# Maestro Documentation

## Quick Start

New to Maestro? Start here:

1. **[Architecture Overview](./architecture.md)** - Understand the system design and core domains
2. **[SSH Authentication](./ssh-authentication.md)** - Set up SSH for repository access (required)
3. **[IPC Guide](./ipc-guide.md)** - Learn type-safe backend communication
4. **[Reactivity Guide](./reactivity.md)** - Svelte 5 runes mode patterns

## Core Domain Guides

### [Architecture Overview](./architecture.md)

**Read first for system-wide understanding**

Complete architectural overview covering:

- Technology stack (Tauri + SvelteKit)
- Core domain models (Repositories, Prompt Sets, Revisions, Executions)
- Data flow architecture (IPC, Event Bus, Reactivity)
- Security architecture (Keyring, SSH)
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

**When to read:** Working with failure analysis, Amp V2 API, or OAuth2 integration

Aggregating and analyzing failures:

- Analysis domain model and workflow
- Amp V2 API integration with OAuth2
- Thread message fetching and formatting
- Background analysis execution
- Frontend trigger UI and result display
- Security and performance considerations

### [Change Tracking](./change-tracking.md)

**When to read:** Working with diffs, file changes, or change statistics

How Maestro tracks code changes:

- Diff architecture (committed vs worktree)
- Backend diff module (`diff.rs`)
- Frontend diff store with caching
- Diff statistics calculation
- File-level diff viewing

### [CI Tracking](./ci-tracking.md)

**When to read:** Working with CI integration or GitHub API

GitHub CI/CD integration:

- CI status states (pending, passed, failed, not_configured)
- Workflow from commit → push → CI check
- Status aggregation (GitHub Actions + Commit Statuses)
- Polling strategy and rate limits
- UI components (badges, status display)

## Infrastructure Guides

### [IPC Guide](./ipc-guide.md)

**When to read:** Adding new Tauri commands or backend features

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
- Manual event handling
- Backend event emission

### [Reactivity Guide](./reactivity.md)

**When to read:** Building reactive UI components with Svelte 5

Comprehensive Svelte 5 patterns:

- Core primitives (`$state`, `$derived`, `$props`)
- Maestro-specific patterns (event bus integration)
- Anti-patterns to avoid
- Working with bits-ui components
- Migration from Svelte 4 stores

## Setup & Configuration

### [Settings & Configuration](./settings.md)

**When to read:** Configuring editors, terminals, or CI preferences

User settings and preferences:

- Available settings (editor, terminal, CI threshold)
- Settings store initialization
- Editor and terminal detection
- Opening worktrees with configured tools
- Best practices and fallback behavior

### [SSH Authentication](./ssh-authentication.md)

**When to read:** During initial setup or troubleshooting git operations

SSH setup for GitHub repositories:

- SSH key generation and configuration
- Adding keys to GitHub and ssh-agent
- How Maestro uses SSH internally
- Troubleshooting common issues

### [Distribution](./distribution.md)

**When to read:** Preparing releases or distributing the app

App distribution guide:

- Apple Developer account requirements
- Code signing and notarization
- GitHub Actions CI/CD setup
- Manual build and distribution
- Troubleshooting signing issues

### [Sourcegraph Integration](./sourcegraph-integration.md)

**When to read:** Working with repository search or Sourcegraph API

Sourcegraph repository search integration:

- GraphQL client architecture
- Configuration and token storage
- Search query syntax and examples
- Frontend/backend API usage
- Error handling patterns

## Additional References

### [Change Tracking Stats](./change-tracking-stats.md)

Detailed dive into statistics calculation:

- On-demand stats retrieval
- Frontend caching strategy
- Why stats aren't stored in database
- Display in ExecutionRow component

## Related Files

- **[AGENTS.md](../AGENTS.md)** - Commands, architecture, and conventions for AI agents
- **[README.md](../README.md)** - Project overview and getting started

## Documentation Standards

When updating docs:

1. **Verify accuracy** - Check against current codebase
2. **Include examples** - Show TypeScript/Rust code snippets
3. **Link related docs** - Help users discover related concepts
4. **Update this index** - Keep README.md in sync
5. **Follow domain structure** - Organize by core domain (not implementation detail)
