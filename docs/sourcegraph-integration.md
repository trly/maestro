# Sourcegraph Integration

Maestro integrates with Sourcegraph's GraphQL API to search for repositories across codebases, enabling dynamic repository discovery and bulk prompt execution.

## Overview

The Sourcegraph integration allows users to:

- Search repositories by query patterns (e.g., `org:mycompany`, `lang:typescript`)
- Filter by repository metadata (cloned, indexed, private/public)
- Retrieve repository metadata (description, language, stars, fork status)
- Get code host information for cloning (GitHub, GitLab, etc.)

## Architecture

### Backend Module

**Location:** `src-tauri/src/sourcegraph/mod.rs`

```rust
pub struct SourcegraphClient {
    endpoint: String,
    access_token: String,
    http_client: reqwest::Client,
}

pub async fn search_repositories(
    &self,
    query: &str,
    limit: i32,
) -> Result<RepositorySearchResult>
```

**Key Features:**

- GraphQL API client using `reqwest`
- Automatic error handling for HTTP and GraphQL errors
- Pagination support via `hasNextPage` field
- Only returns cloned repositories by default

### IPC Command

**Location:** `src-tauri/src/commands/sourcegraph.rs`

```rust
#[tauri::command]
pub async fn search_sourcegraph_repositories(
    query: String,
    limit: Option<i32>,
) -> Result<RepositorySearchResult, String>
```

**Token Retrieval:**

- `sourcegraph_endpoint` - Instance URL (e.g., `https://sourcegraph.com`)
- `sourcegraph_token` - Access token for API authentication
- Both retrieved from system keyring at command execution time

### Frontend API

**Location:** `src/lib/ipc.ts`

```typescript
export interface SourcegraphRepository {
	id: string
	name: string
	description: string | null
	url: string
	language: string | null
	stars: number
	isPrivate: boolean
	isFork: boolean
	isArchived: boolean
	externalRepository: {
		serviceType: string // "github", "gitlab", etc.
		serviceId: string
	}
}

export interface RepositorySearchResult {
	repositories: SourcegraphRepository[]
	totalCount: number
	hasNextPage: boolean
}

export async function searchSourcegraphRepositories(
	query: string,
	limit?: number
): Promise<RepositorySearchResult>
```

## Configuration

### Settings UI

Users configure Sourcegraph integration in Settings:

1. **Sourcegraph Instance** - Base URL of Sourcegraph instance
   - Example: `https://sourcegraph.com`
   - Example: `https://sourcegraph.company.com` (self-hosted)

2. **Sourcegraph Access Token** - API authentication token
   - Generated at: `<instance>/user/settings/tokens`
   - Requires read access to repositories

Both values are stored securely in the system keyring.

### Token Storage

```rust
// Backend - retrieve from keyring
let endpoint = tokens::get_token_value("sourcegraph_endpoint")?
    .ok_or_else(|| "Sourcegraph endpoint not configured")?;

let access_token = tokens::get_token_value("sourcegraph_token")?
    .ok_or_else(|| "Sourcegraph access token not configured")?;
```

## GraphQL Query

The backend uses the following GraphQL query:

```graphql
query SearchRepositories($query: String!, $first: Int!) {
	repositories(query: $query, first: $first, cloned: true) {
		nodes {
			id
			name
			description
			url
			language
			stars
			isPrivate
			isFork
			isArchived
			externalRepository {
				serviceType
				serviceID
			}
		}
		totalCount
		pageInfo {
			hasNextPage
		}
	}
}
```

**Key Parameters:**

- `query` - Search query string (supports Sourcegraph search syntax)
- `first` - Number of results to return (default: 50)
- `cloned: true` - Only return repositories that have been cloned to the Sourcegraph instance

## Usage Examples

### Basic Repository Search

```typescript
import * as ipc from "$lib/ipc"

// Search by organization
const result = await ipc.searchSourcegraphRepositories("org:mycompany", 50)

console.log(`Found ${result.totalCount} repositories`)
result.repositories.forEach((repo) => {
	console.log(`${repo.name} - ${repo.language}`)
})
```

### Filter by Language

```typescript
// Search for TypeScript repositories in an org
const result = await ipc.searchSourcegraphRepositories("org:mycompany lang:typescript", 100)
```

### Get Code Host Information

```typescript
const result = await ipc.searchSourcegraphRepositories("my-repo")

result.repositories.forEach((repo) => {
	const codeHost = repo.externalRepository.serviceType
	console.log(`${repo.name} is hosted on ${codeHost}`)

	// Clone URL format depends on code host
	if (codeHost === "github") {
		const cloneUrl = `git@github.com:${repo.externalRepository.serviceId}.git`
		console.log(`Clone: ${cloneUrl}`)
	}
})
```

### Error Handling

```typescript
try {
	const result = await ipc.searchSourcegraphRepositories("query")
	// Handle results
} catch (error) {
	if (error.message.includes("not configured")) {
		console.error("Sourcegraph credentials not set - go to Settings")
	} else {
		console.error("Search failed:", error)
	}
}
```

## Search Query Syntax

Sourcegraph supports powerful search syntax:

- **Organization**: `org:mycompany`
- **Language**: `lang:typescript`
- **Private repos**: `visibility:private`
- **Forks**: `fork:yes` or `fork:no`
- **Archived**: `archived:yes` or `archived:no`
- **Name pattern**: `repo:.*backend.*`
- **Combine**: `org:mycompany lang:go -archived`

See [Sourcegraph search reference](https://sourcegraph.com/docs/code-search/queries) for complete syntax.

## Future Enhancements

Potential improvements:

1. **Pagination Support** - Fetch all results across multiple pages
2. **Repository Caching** - Cache search results to reduce API calls
3. **Advanced Filters** - UI for building complex search queries
4. **Bulk Import** - Import all matching repositories to Maestro
5. **Auto-refresh** - Periodic sync of repository metadata
6. **Rate Limiting** - Respect Sourcegraph API rate limits

## Implementation Reference

**Backend:**

- `src-tauri/src/sourcegraph/mod.rs` - GraphQL client
- `src-tauri/src/commands/sourcegraph.rs` - IPC command
- `src-tauri/Cargo.toml` - Dependencies (`reqwest` with JSON support)

**Frontend:**

- `src/lib/ipc.ts` - Type-safe IPC wrapper
- `src/lib/components/Settings.svelte` - Configuration UI
- `src/lib/tokenStore.ts` - Token management

## Related Documentation

- **[Settings](./settings.md)** - User settings and configuration
- **[Architecture](./architecture.md)** - System architecture overview
- **[IPC Guide](./ipc-guide.md)** - IPC command reference
