# CI Integration

Maestro integrates with VCS CI/CD systems to validate that pushed commits pass all configured checks.

## Overview

After committing and pushing changes from an execution, Maestro automatically:

1. Sets CI status to "pending"
2. Provides a link to the VCS provider's checks page
3. Allows manual refresh of CI status via the UI

CI validation provides an additional quality gate beyond prompt execution and validation threads.

## CI Status States

- **Pending** (🔵) - CI jobs are queued or running
- **Passed** (✅) - All CI checks passed
- **Failed** (❌) - One or more CI checks failed
- **Skipped** (⚪) - CI polling exhausted or timed out
- **Not Configured** - No CI configured for this repository

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

Maestro integrates with multiple CI systems:

- **GitHub**: GitHub Actions and Commit Statuses
- **GitLab**: Pipeline status
- **Legacy CI**: Jenkins, CircleCI, and other external systems via commit statuses

### Status Aggregation Rules

- If any check **fails** → Overall status: Failed
- If any check is **pending** → Overall status: Pending
- If all checks **pass** → Overall status: Passed
- If no checks found → Overall status: Not Configured

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

Executions store CI status information:

- `ci_status` - Current CI check status
- `ci_checked_at` - Timestamp of last check
- `ci_url` - Link to CI results page

### Core Modules

- **CI Provider Integration** - VCS-specific API clients (GitHub, GitLab)
- **Status Checker** - Polling logic with exponential backoff
- **Git Service** - Branch push operations
- **CI Commands** - IPC interface for CI operations

### IPC Commands

- `push_commit(execution_id, force)` - Push branch to remote, start CI checking
- `start_ci_check(execution_id)` - Manually start CI polling
- `refresh_ci_status(execution_id)` - One-time CI status check

### Events

- `execution:ci` - Real-time CI status updates via event bus
  - Payload: `{ executionId, ciStatus, ciUrl? }`

## Prerequisites

### VCS Provider Tokens

Configure your VCS provider tokens in the Settings page (stored securely in system keyring):

1. Navigate to Settings
2. Enter your provider tokens (GitHub, GitLab, etc.)
3. Tokens are securely stored in your system keyring

**GitHub Token** must have:

- `repo` scope (for API access and HTTPS git operations)
- `workflow` scope (for push operations)
- `checks:read` scope (optional, for richer CI details)

**GitLab Token** must have:

- `api` scope (for API access and pipeline status)
- `write_repository` scope (for HTTPS git operations)

### Git Authentication

Maestro uses HTTPS with Personal Access Tokens for push operations. SSH is optional but preferred when available.

See [ssh-authentication.md](./ssh-authentication.md) for authentication setup details.

## Rate Limits

VCS provider APIs have rate limits that Maestro handles gracefully:

**GitHub:**

- Authenticated: 5,000 requests/hour
- Unauthenticated: 60 requests/hour

**GitLab:**

- Varies by instance configuration (typically 600-2,000 requests/minute)

Maestro's rate limit handling:

- Uses exponential backoff (10s → 120s)
- Respects rate limit headers
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

- **Manual refresh only**: Background polling is disabled (manual refresh available)
- **No webhooks**: Uses polling instead of real-time webhooks
- **Aggregate status**: Shows overall status, not individual job details
- **Provider-specific features**: Some advanced CI features may not be available across all providers

## Future Enhancements

Potential improvements:

- Background polling with proper store architecture
- Support for GitLab CI, CircleCI webhooks
- Job-level detail view
- Re-run failed CI jobs from Maestro
- Webhook integration for real-time updates
- Policy enforcement (block merge if CI fails)
