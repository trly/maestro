# Maestro

AI-powered orchestrator for running prompts across multiple repositories using [Amp](https://ampcode.com).

**Built with SvelteKit + Tauri 2.0**

## Quick Start

### Prerequisites

- [Bun](https://bun.sh) (JavaScript runtime & package manager)
- [Rust](https://rustup.rs) (for Tauri backend)
- SSH key added to GitHub (for private repos - see [SSH setup](docs/ssh-authentication.md))
- Amp API token (for AI executions)

### Installation

1. **Install dependencies:**

   ```bash
   bun install
   ```

2. **Install Rust** (if not already installed):

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

3. **Set up SSH authentication** (required for private repos):

   ```bash
   # Generate SSH key if you don't have one
   ssh-keygen -t ed25519 -C "your_email@example.com"

   # Add to ssh-agent
   ssh-add ~/.ssh/id_ed25519

   # Add public key to GitHub: https://github.com/settings/keys
   ```

   See [docs/ssh-authentication.md](docs/ssh-authentication.md) for detailed instructions.

4. **Start development server:**
   ```bash
   bun run dev
   ```

### Production Installation

1. **Build the installer:**

   ```bash
   bun run build
   ```

2. **Install the app:**

   **macOS:**

   ```bash
   # Option 1: Install from DMG (recommended)
   open src-tauri/target/release/bundle/dmg/*.dmg
   # Drag Maestro.app to Applications folder

   # Option 2: Run directly from bundle
   open src-tauri/target/release/bundle/macos/Maestro.app
   ```

   **Linux (Debian/Ubuntu):**

   ```bash
   sudo dpkg -i src-tauri/target/release/bundle/deb/*.deb
   # Launch from application menu or run:
   maestro
   ```

## Commands

| Command         | Description                         |
| --------------- | ----------------------------------- |
| `bun run dev`   | Start Tauri app in development mode |
| `bun run build` | Build production installer          |
| `bun run check` | Run TypeScript type checking        |
| `cargo test`    | Run Rust tests (from `src-tauri/`)  |

## Tech Stack

- **Frontend**: SvelteKit (Svelte 5) with adapter-static
- **Backend**: Tauri 2.0 (Rust)
- **Database**: rusqlite (SQLite)
- **Git**: git2-rs (native operations)
- **AI**: @sourcegraph/amp-sdk
- **UI**: bits-ui + lucide-svelte + Tailwind 4

## Architecture

### High-Level Overview

Maestro orchestrates AI-powered code changes across multiple repositories:

1. **Prompt Sets** - Organize related prompts and target repositories
2. **Prompt Revisions** - Version control for prompt iterations
3. **Executions** - Isolated worktree environments for each repository
4. **Validations** - Automated testing of AI-generated changes
5. **Failure Analysis** - Aggregate and analyze failures using Amp V2 API

### Data Flow

```
User creates Prompt Set → Selects repositories → Executes Revision
                                                          ↓
                                    Backend clones repos (SSH) & creates worktrees
                                                          ↓
                                    Amp AI agent runs in isolated worktree
                                                          ↓
                                    Changes tracked via git diffs
                                                          ↓
                                    User reviews → commits → worktree cleaned up
```

### File System Layout

Maestro uses platform-specific app data directories:

- **macOS**: `~/Library/Application Support/dev.trly.maestro/`
- **Linux**: `~/.local/share/maestro/`
- **Windows**: `%APPDATA%\dev.trly.maestro\`
- **Custom**: Set `MAESTRO_CONFIG` environment variable to override

```
{app_data_dir}/
├── maestro.db              # SQLite database
├── repos/                  # Admin clones (permanent)
│   └── owner/
│       └── repo/.git/
└── executions/            # Worktrees (ephemeral)
    └── {promptsetId}/
        └── {executionId}/
```

### Key Patterns

- **UUID Strategy**: Full UUIDs in DB/storage, 8-char hashes for display (`toShortHash()`)
- **Git Branches**: `maestro/{promptsetId:8}/{revisionId:8}/{executionId:8}`
- **IPC Layer**: Typed wrappers in `src/lib/ipc.ts` (never direct `invoke()`)
- **Event Bus**: Centralized execution events via `src/lib/stores/executionBus.ts`
- **Diff Access**: Unified backend module (`src-tauri/src/git/diff.rs`) + frontend store

### Type Safety

- **Status Enums**: `ExecutionStatus`, `ValidationStatus`, `CommitStatus` (Rust enums serialized to lowercase)
- **Type Mirroring**: TypeScript interfaces match Rust structs via serde
- **IPC Types**: Automatic camelCase conversion with `#[serde(rename_all = "camelCase")]`

## Documentation

- [AGENTS.md](AGENTS.md) - Codebase guide for AI agents
- [docs/](docs/) - Detailed technical documentation
  - [Architecture](docs/architecture.md) - System design and core domains
  - [IPC Guide](docs/ipc-guide.md) - Command reference and patterns
  - [Event Bus](docs/execution-event-bus.md) - Event handling architecture
  - [Change Tracking](docs/change-tracking.md) - Diff access patterns
  - [Failure Analysis](docs/analyses.md) - Aggregating failures with Amp V2 API
  - [CI Tracking](docs/ci-tracking.md) - GitHub CI integration
  - [SSH Authentication](docs/ssh-authentication.md) - Private repo setup

## Development Workflow

### Adding Features

1. **Backend (Rust)**:
   - Add command handler in `src-tauri/src/commands/`
   - Register in `src-tauri/src/lib.rs`
   - Add typed wrapper in `src/lib/ipc.ts`

2. **Frontend (TypeScript/Svelte)**:
   - Use IPC wrappers from `src/lib/ipc.ts`
   - Use bits-ui primitives for UI components
   - Subscribe to events via `executionBus.ts`

3. **Always run before committing**:
   ```bash
   bun run check    # TypeScript
   cargo test       # Rust (from src-tauri/)
   ```

### Common Patterns

**Tauri IPC:**

```typescript
import { getExecution, executePrompt } from "$lib/ipc"
const execution = await getExecution(id)
```

**Execution Events:**

```typescript
import { subscribeToExecutions } from "$lib/stores/executionBus"
subscribeToExecutions() // Call once at app init
```

**Diff Access:**

```typescript
import { fetchDiff } from "$lib/stores/diffStore"
const files = await fetchDiff(executionId)
```

## Troubleshooting

### Keychain Access Prompts in Dev Mode

**Issue:** macOS prompts for keychain access every time you run `bun run dev`, even after selecting "Always Allow".

**Cause:** In development mode, the Tauri app isn't code-signed, so macOS treats each run as potentially different.

**Workarounds:**

- **Accept the prompt** - It's annoying but harmless during development
- **Manual keychain config** - Open Keychain Access.app → find `dev.trly.maestro` entries → double-click → add the dev binary to "Always allow access"

**Built apps:** This issue does **not** affect installed apps from `bun run build`. The "Always Allow" setting persists in production builds.

## Contributing

See [AGENTS.md](AGENTS.md) for detailed code conventions and architectural patterns.
