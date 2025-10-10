# Maestro Documentation

## Quick Start

New to Maestro? Start here:

1. **[SSH Authentication](ssh-authentication.md)** - Set up SSH for repository access (required for first-time setup)
2. **[IPC Guide](ipc-guide.md)** - Learn how to call backend commands from the frontend
3. **[Reactivity Guide](reactivity.md)** - Svelte 5 runes mode patterns and best practices

## Architecture Guides

### [IPC Guide](ipc-guide.md)
**When to read:** When adding new features that need backend communication

Centralized IPC layer for type-safe Tauri command invocations. Covers:
- Common patterns and examples
- Error handling
- API reference for all commands
- Migration from direct `invoke()` calls

### [Execution Event Bus](execution-event-bus.md)
**When to read:** When building UI that needs real-time execution updates

Real-time event system for execution state changes. Covers:
- Subscribing to execution events
- Live progress updates
- State management patterns

### [Diff Architecture](diff-architecture.md)
**When to read:** When working with file diffs or git operations

Unified diff retrieval system for committed and uncommitted changes. Covers:
- Backend diff module (`diff.rs`)
- Frontend diff store with caching
- How to fetch and display diffs

### [Diff Stats](diff-stats.md)
**When to read:** When working with execution statistics or change tracking

How file change statistics are calculated and displayed. Covers:
- Stats calculation during execution and after commit
- Database storage and retrieval
- Frontend display in ExecutionRow
- Why stats are calculated multiple times

### [Reactivity Guide](reactivity.md)
**When to read:** When building reactive UI components with Svelte 5

Comprehensive guide to Svelte 5 runes mode and reactive patterns. Covers:
- Core reactivity primitives (`$state`, `$derived`, `$props`)
- Event bus integration patterns
- Anti-patterns to avoid
- Migration from Svelte 4 stores
- Working with bits-ui components

## Setup & Configuration

### [SSH Authentication](ssh-authentication.md)
**When to read:** During initial setup or when troubleshooting git operations

Complete guide to SSH setup for GitHub repositories. Covers:
- SSH key generation and setup
- Adding keys to GitHub and ssh-agent
- Common troubleshooting steps
- How Maestro uses SSH internally

## Related Files

- **[AGENTS.md](../AGENTS.md)** - Commands, architecture, and conventions for AI agents
- **[README.md](../README.md)** - Project overview and getting started
