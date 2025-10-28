# CI Integration

Maestro integrates with GitHub's CI/CD systems to validate that pushed commits pass all configured checks.

## Overview

After committing and pushing changes from an execution, Maestro automatically:

1. Sets CI status to "pending"
2. Provides a link to GitHub's checks page
3. Allows manual refresh of CI status via the UI

CI validation provides an additional quality gate beyond prompt execution and validation threads.

## CI Status States

- **Pending** (🔵) - CI jobs are queued or running
- **Passed** (✅) - All CI checks passed
- **Failed** (❌) - One or more CI checks failed
- **Skipped** (⚪) - No CI configured for this commit

## Workflow

### 1. Commit Changes

After an execution completes, commit the changes:

- Click the commit button on the execution
- Amp will generate an appropriate commit message
- Commit status shows as "committed" with SHA

### 2. Push to Remote

Once committed, push the changes:

- Click the push button (purple upload icon)
- Branch is pushed to GitHub using SSH authentication
- CI status automatically set to "pending"

### 3. Check CI Status

Maestro provides two ways to check CI:

- **Automatic**: After push, CI status is set to pending
- **Manual refresh**: Click the CI badge to refresh status

## CI Status Checking

Maestro checks both:

- **GitHub Actions** (Checks API) - Modern GitHub workflows
- **Commit Statuses** (Status API) - Legacy CI systems (Jenkins, CircleCI, etc.)

### Status Aggregation Rules

- If any check **fails** → Overall status: Failed
- If any check is **pending** → Overall status: Pending
- If all checks **pass** → Overall status: Passed
- If no checks found → Overall status: Skipped

## UI Components

### ExecutionRow

- **Commit column**: Shows commit SHA (green checkmark when committed)
- **CI column**: Shows CI status badge
  - Click badge to open GitHub checks page (if available)
  - Click badge to refresh status (if no URL yet)
- **Actions**: Push button appears after commit

### CiStatusBadge

Displays current CI status with appropriate icon and color:

- Pending: Blue spinner (animated)
- Passed: Green checkmark
- Failed: Red X
- Skipped: Gray minus circle

## Backend Architecture

### Database Schema

```sql
-- Migration 11
ALTER TABLE executions ADD COLUMN ci_status TEXT;
ALTER TABLE executions ADD COLUMN ci_checked_at INTEGER;
ALTER TABLE executions ADD COLUMN ci_url TEXT;
```

### Rust Modules

- **`ci/github_provider.rs`**: Octocrab integration for GitHub API
- **`ci/status_checker.rs`**: Polling logic with exponential backoff
- **`commands/ci.rs`**: Tauri commands for CI operations
- **`git/service.rs`**: `push_branch()` for pushing commits

### Commands

- `push_commit(execution_id, force)` - Push branch to remote, start CI checking
- `start_ci_check(execution_id)` - Manually start CI polling
- `refresh_ci_status(execution_id)` - One-time CI status check

### Events

- `execution:ci` - Real-time CI status updates via event bus
  - Payload: `{ executionId, ciStatus, ciUrl? }`

## Prerequisites

### GitHub Token

Configure your GitHub token in the Settings page (stored securely in system keyring):

1. Navigate to Settings
2. Enter your GitHub Personal Access Token
3. Token is securely stored in your system keyring

Token must have:

- `repo` scope (for private repositories)
- `checks:read` scope (optional, for richer CI details)

**Note**: Legacy support for `MAESTRO_GITHUB_TOKEN` environment variable has been removed. Use Settings instead.

### SSH Authentication

Push requires SSH authentication configured:

1. SSH key added to ssh-agent
2. Public key added to GitHub account
3. SSH_AUTH_SOCK environment variable set

See [ssh-authentication.md](./ssh-authentication.md) for setup details.

## Rate Limits

GitHub API has rate limits:

- **Authenticated**: 5,000 requests/hour
- **Unauthenticated**: 60 requests/hour

Maestro handles rate limits gracefully:

- Uses exponential backoff (10s → 120s)
- Respects `X-RateLimit-Reset` headers
- Falls back to "skipped" if polling exhausted

## Polling Strategy

When CI status is checked, Maestro uses exponential backoff:

- Attempt 1: 10 seconds
- Attempt 2: 20 seconds
- Attempt 3: 40 seconds
- Attempt 4: 80 seconds
- Attempt 5: 120 seconds

Total polling window: ~5 minutes

If still pending after 5 minutes, status remains "pending" and can be manually refreshed.

## Limitations

- **GitHub only**: Currently only GitHub Actions and commit statuses are supported
- **No auto-polling**: Background polling is disabled (manual refresh available)
- **No webhooks**: Uses polling instead of real-time webhooks
- **No job details**: Shows aggregate status, not individual job details

## Future Enhancements

Potential improvements:

- Background polling with proper store architecture
- Support for GitLab CI, CircleCI webhooks
- Job-level detail view
- Re-run failed CI jobs from Maestro
- Webhook integration for real-time updates
- Policy enforcement (block merge if CI fails)
