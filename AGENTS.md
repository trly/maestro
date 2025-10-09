# Maestro - Agent Guide

## Commands

- **Dev**: `bun run dev` - Start both API server (port 3000) and Vite dev server (port 5173)
- **Build**: `bun run build` - Build frontend with Vite
- **Preview**: `bun run preview` - Run production server (serves API + static files)
- **Test**: `bun test` - Run tests (use `bun test <file>` for single file)
- **Install**: `bun install` - Install dependencies

## Environment Variables

- **VITE_MAESTRO_GITHUB_TOKEN**: GitHub Personal Access Token (required)
  - Generate at: https://github.com/settings/tokens
  - Required scopes: `repo`
  - Copy `.env.example` to `.env` and add your token
- **VITE_MAESTRO_CLONE_DIR**: Directory for cloning repositories (optional)
  - Default: `$HOME/maestro/repos`
  - Used by execution engine to clone repos

## Architecture

- **Tech stack**: Svelte 5 SPA + Bun backend, Tailwind 4, TypeScript, Vite
- **Backend**: server.ts - Bun HTTP server with `/api/*` routes, uses `bun:sqlite`
- **Frontend**: src/App.svelte - Svelte SPA built with Vite
- **Database**: SQLite via `bun:sqlite` with migration system
- **Purpose**: Orchestrator UI for running AI prompts across multiple repositories

### Current Features

- **Repository Selection**: GitHub integration for selecting repositories
- **Prompt Sets**: Create named sets of repositories to execute prompts against
- **Prompt Revisions**: Version control for prompts with parent-child relationships
- **Execution Engine**: Uses @sourcegraph/amp-sdk to execute prompts in cloned repos
- **Execution Tracking**: Monitor execution status (pending/running/completed/failed)
- **Branch Management**: Auto-creates branches per execution (`maestro/{promptsetId:8}/{revisionId:8}/{executionId:8}`) using 8-char short hashes
- **Prompt Diffing**: Visual diff between prompt revisions
- **Branch Viewer**: View all cloned repos and maestro branches with associated prompt set/revision/execution metadata
- **Repository Sync**: Clean up repos on disk not in DB and unused repos from DB (not in prompt sets)

### Project Structure

```
src/
├── lib/
│   ├── components/
│   │   ├── RepositorySelector.svelte  # GitHub repo selection UI
│   │   ├── PromptDiff.svelte          # Prompt revision diff viewer
│   │   └── BranchesView.svelte        # View all repos and branches
│   ├── db/
│   │   ├── store.ts                   # SQLite data access layer
│   │   └── migrations.ts              # DB schema migrations
│   ├── providers/
│   │   ├── github.ts                  # GitHub API integration
│   │   ├── types.ts                   # Provider types
│   │   └── index.ts                   # Provider exports
│   ├── api.ts                         # Frontend API client
│   ├── executor.ts                    # Amp SDK execution logic
│   ├── maestroScanner.ts              # Scan clone dir for branches
│   └── types.ts                       # Core type definitions
├── App.svelte                         # Main UI component
└── main.ts                            # App entry point
server.ts                              # API server
```

### API Endpoints

- `POST /api/repositories` - Create/find repository
- `GET /api/repositories?id=...` - Get repository by ID
- `GET /api/repositories?provider=...&providerId=...` - Find repository
- `POST /api/promptsets` - Create prompt set
- `GET /api/promptsets/:id` - Get prompt set
- `GET /api/promptsets/:id/revisions` - List revisions
- `GET /api/promptsets/:id/executions` - List executions
- `POST /api/revisions` - Create revision
- `GET /api/revisions/:id` - Get revision
- `GET /api/revisions/:id/executions` - List executions for revision
- `POST /api/revisions/:id/execute` - Execute revision across all repos
- `POST /api/executions` - Create execution
- `GET /api/executions/:id` - Get execution
- `PATCH /api/executions/:id` - Update execution status
- `GET /api/maestro/branches?refresh=1` - Scan and list all repos/branches with metadata
- `POST /api/maestro/sync` - Sync repositories (delete from disk if not in DB, delete from DB if not in prompt sets)

### Data Model

- **Repository**: Stores repo metadata (provider, providerId, name)
- **PromptSet**: Named collection of repositories
- **PromptRevision**: Versioned prompt text with parent pointer
- **Execution**: Individual prompt run against a repository (tracks status, thread URL)

## Code Style

- **Runtime**: Always use Bun, not Node.js (see .cursor/rules/use-bun-instead-of-node-vite-npm-pnpm.mdc)
- **Components**: Svelte 5 syntax with `<script lang="ts">`, use runes for reactivity
- **Styling**: Tailwind utility classes directly in components
- **TypeScript**: Strict typing, explicit types in function signatures
- **Formatting**: Tabs for indentation, descriptive variable names
- **Events**: Use `onclick` not `on:click` (Svelte 5 convention)
