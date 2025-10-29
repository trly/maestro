# Failure Analysis

Maestro's failure analysis feature aggregates multiple failed executions or validations and uses Amp SDK's `read_thread` tool to categorize common failure patterns.

## Overview

When executions or validations fail across multiple repositories, the analysis feature:

1. Collects thread URLs from failed executions/validations
2. Creates an analysis prompt instructing Amp to use `read_thread` for each thread
3. Uses Amp SDK to execute the analysis (Amp automatically fetches thread content)
4. Stores results linked to the prompt revision

**No OAuth credentials required** - leverages Amp's built-in `read_thread` tool instead of direct API integration.

## Domain Model

```rust
struct Analysis {
    id: String,
    revision_id: String,
    analysis_type: AnalysisType,    // "execution" or "validation"
    status: AnalysisStatus,          // "pending", "completed", "failed"
    analysis_prompt: String,
    analysis_result: Option<String>,
    amp_thread_url: Option<String>,
    amp_session_id: Option<String>,
    error_message: Option<String>,
    created_at: i64,
    updated_at: i64,
    completed_at: Option<i64>,
}
```

### Analysis Types

```rust
enum AnalysisType {
    Execution,   // Analyzes failed execution threads
    Validation,  // Analyzes failed validation threads
}
```

### Analysis Status

```rust
enum AnalysisStatus {
    Pending,    // Analysis created but not started
    Completed,  // Analysis finished successfully
    Failed,     // Analysis encountered an error
}
```

## Database Schema

### Tables

**analyses** - Main analysis records:

```sql
CREATE TABLE analyses (
    id TEXT PRIMARY KEY,
    revision_id TEXT NOT NULL,
    type TEXT NOT NULL CHECK (type IN ('execution', 'validation')),
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'completed', 'failed')),
    analysis_prompt TEXT NOT NULL,
    analysis_result TEXT,
    amp_thread_url TEXT,
    amp_session_id TEXT,
    error_message TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    completed_at INTEGER,
    FOREIGN KEY (revision_id) REFERENCES prompt_revisions(id) ON DELETE CASCADE
);
```

**analysis_executions** - Join table linking analyses to executions:

```sql
CREATE TABLE analysis_executions (
    analysis_id TEXT NOT NULL,
    execution_id TEXT NOT NULL,
    PRIMARY KEY (analysis_id, execution_id),
    FOREIGN KEY (analysis_id) REFERENCES analyses(id) ON DELETE CASCADE,
    FOREIGN KEY (execution_id) REFERENCES executions(id) ON DELETE CASCADE
);
```

### Indexes

```sql
CREATE INDEX idx_analyses_revision_type_created ON analyses (revision_id, type, created_at DESC);
CREATE INDEX idx_analyses_status_created ON analyses (status, created_at DESC);
CREATE INDEX idx_analysis_execs_execution ON analysis_executions (execution_id);
CREATE INDEX idx_analysis_execs_analysis ON analysis_executions (analysis_id);
```

## Amp SDK Integration

Maestro uses the same Amp SDK execution flow as regular prompt executions. The analysis prompt instructs Amp to use its built-in `read_thread` tool to fetch and analyze thread content.

**No direct API integration** - Amp SDK handles all thread fetching automatically via the `read_thread` tool.

**Authentication** - Uses the same `amp_token` as execution agent (stored in system keyring).

## Workflow

### 1. Trigger Analysis

User clicks "Analyze failed executions" or "Analyze failed validations" in the RevisionDetail UI.

**Frontend:**

```typescript
async function handleAnalyzeExecutions() {
	const failed = executions.filter((e) => e.status === "failed")
	const analysisId = await createAnalysis(
		revision.id,
		"execution",
		failed.map((e) => e.id)
	)
	await runAnalysis(analysisId)
}
```

### 2. Create Analysis Record

**Command:** `create_analysis(revision_id, analysis_type, execution_ids)`

**Backend:**

```rust
// Create analysis with pending status
let analysis = store.create_analysis(id, revision_id, analysis_type, prompt)?;

// Link executions
store.add_analysis_executions(&id, &execution_ids)?;
```

### 3. Run Analysis (Background)

**Command:** `run_analysis(analysis_id)`

**Backend (spawns tokio task):**

```rust
tokio::spawn(async move {
    // 1. Fetch all thread messages
    for execution in executions {
        let thread_url = match analysis_type {
            AnalysisType::Execution => execution.thread_url,
            AnalysisType::Validation => execution.validation_thread_url,
        };
        let thread_id = extract_thread_id(thread_url)?;
        let messages = get_thread_messages(&thread_id).await?;
        all_threads.push(format_messages_for_analysis(&messages));
    }

    // 2. Create analysis prompt
    let prompt = format!(
        "Analyze the following failed {} threads and categorize common failure patterns:\n\n{}",
        analysis_type, all_threads.join("\n\n---\n\n")
    );

    // 4. Execute via Amp SDK
    let output = execute_analysis_with_amp(&prompt).await?;

    // 5. Update analysis record
    store.update_analysis_result(
        analysis_id,
        output.result,
        output.thread_url,
        output.session_id,
        now_ms()
    )?;
});
```

### 4. View Results

Analysis results stored in database and linked to revision:

```typescript
const analyses = await getAnalysesByRevision(revision.id, "execution")
// Display results in UI
```

## Frontend Integration

### Trigger Buttons

Located in [RevisionDetail.svelte](file:///Users/trly/src/github.com/trly/maestro/src/lib/components/ui/RevisionDetail.svelte) column headers:

```svelte
<!-- Execution column header -->
{#if hasFailedExecutions}
	<UiTooltip content="Analyze failed executions">
		{#snippet children({ props })}
			<button
				{...props}
				onclick={onAnalyzeExecutions}
				class="text-purple-600 hover:text-purple-700"
			>
				<ScanSearch class="w-3.5 h-3.5" />
			</button>
		{/snippet}
	</UiTooltip>
{/if}
```

### IPC Commands

```typescript
// Create analysis
const analysisId = await createAnalysis(
  revisionId: string,
  analysisType: 'execution' | 'validation',
  executionIds: string[]
) -> Promise<string>

// Run analysis (background)
await runAnalysis(analysisId: string) -> Promise<void>

// Retrieve results
const analysis = await getAnalysis(analysisId: string) -> Promise<Analysis>
const analyses = await getAnalysesByRevision(
  revisionId: string,
  analysisType?: 'execution' | 'validation'
) -> Promise<Analysis[]>
```

## Backend Architecture

### Module Structure

```
src-tauri/src/
├── commands/
│   └── analysis.rs           # Analysis commands
└── db/
    └── store.rs              # Analysis CRUD operations
```

### Key Components

**Analysis Commands** (`src-tauri/src/commands/analysis.rs`):

- `create_analysis()` - Creates analysis record
- `run_analysis()` - Spawns background task
- `get_analysis()` - Retrieves analysis by ID
- `get_analyses_by_revision()` - Lists analyses for revision

**Store Methods** (`src-tauri/src/db/store.rs`):

- `create_analysis()` - Insert analysis record
- `update_analysis_status()` - Update status/error
- `update_analysis_result()` - Store result/completion
- `add_analysis_executions()` - Link executions
- `get_analysis_executions()` - Retrieve linked executions

## Query Patterns

### Get Latest Analysis for Revision

```typescript
const analyses = await getAnalysesByRevision(revision.id, "execution")
const latest = analyses.sort((a, b) => b.createdAt - a.createdAt)[0]
```

### Get All Executions in Analysis

```rust
let executions = store.get_analysis_executions(&analysis_id)?;
```

### Get Analyses by Status

```sql
SELECT * FROM analyses
WHERE status = 'completed'
ORDER BY created_at DESC;
```

## Security Considerations

1. **Amp Token**
   - Stored in system keyring (same as execution agent)
   - Never exposed in logs or UI
   - Passed to amp-executor.ts via AMP_API_KEY environment variable

2. **Thread Access**
   - Only threads from workspace executions are analyzed
   - Amp SDK's read_thread tool handles authentication automatically
   - User must have access to threads in their workspace

## Performance

### Thread Fetching

- Amp SDK handles thread fetching via read_thread tool
- Multiple threads read sequentially by Amp agent
- No manual pagination required

### Background Execution

- Analysis runs in tokio background task
- Non-blocking for UI
- Database updates atomic per status change

## Future Enhancements

Potential improvements:

1. **Real-time Updates**
   - Event bus integration for analysis status
   - Progress updates during thread fetching

2. **Analysis History**
   - UI to view past analyses per revision
   - Compare analysis results over time

3. **Analysis Templates**
   - Customizable analysis prompts
   - Save common analysis patterns

4. **Batch Analysis**
   - Analyze across multiple revisions
   - Compare failure patterns across prompt changes

5. **Analysis Results UI**
   - Dedicated view for analysis results
   - Categorized failure display
   - Export to markdown/JSON

## Troubleshooting

### "Amp token not configured"

**Cause:** `amp_token` missing from keyring

**Solution:**

1. Go to Settings → Agents
2. Enter Amp API Token
3. Generate token at: https://ampcode.com/settings/profile

### "Failed to read thread"

**Possible causes:**

- Invalid Amp token
- Thread URL malformed
- Thread not accessible in workspace
- Network connectivity issues

**Solution:**

1. Verify Amp token in Settings → Agents
2. Check thread URL format: `https://ampcode.com/threads/T-{uuid}`
3. Ensure you have access to the thread in your Amp workspace
4. Check amp-executor.ts output for detailed errors

### "Analysis failed"

**Cause:** Error during Amp execution or thread fetching

**Solution:**

1. Check `error_message` field in analysis record
2. Verify Amp SDK is available (`bun run` works)
3. Check if executions have valid thread URLs

## UI Components

### AnalysisResult Component

Displays individual analysis results with status, timestamps, and results:

```svelte
<!-- src/lib/components/ui/AnalysisResult.svelte -->
<AnalysisResult
	{analysis}
	onDelete={() => deleteAnalysis(analysis.id)}
	onRerun={() => rerunAnalysis(analysis)}
/>
```

**Features:**

- Status badges with icons (completed, failed, running, pending)
- Creation and completion timestamps
- External link to Amp thread
- Error message display for failed analyses
- Formatted analysis results
- Delete button (trash icon) to remove analysis
- Re-run button (rotate icon) to re-execute completed/failed analyses

**Status Display:**

- **Completed**: Green badge with CheckCircle2 icon
- **Failed**: Red badge with AlertCircle icon and error message
- **Running**: Blue badge with animated Loader2 spinner
- **Pending**: Gray badge with Clock icon

**Actions:**

- **Delete**: Available for all analyses, removes analysis record from database
- **Re-run**: Available for completed/failed analyses, re-executes the same analysis with same execution set

### Integration in RevisionDetail

Analyses are displayed between the prompt console and executions table:

```svelte
<!-- Analysis Results Section -->
{#if analyses.length > 0}
	<div class="flex-shrink-0 border-b border-border/20 bg-card">
		<div class="px-4 py-3">
			<h3 class="text-sm font-semibold text-card-foreground mb-3">Failure Analysis</h3>
			<div class="space-y-3">
				{#each analyses as analysis (analysis.id)}
					<AnalysisResult
						{analysis}
						onDelete={() => handleDeleteAnalysis(analysis.id)}
						onRerun={() => handleRerunAnalysis(analysis)}
					/>
				{/each}
			</div>
		</div>
	</div>
{/if}
```

### Data Flow

1. **Trigger**: User clicks ScanSearch icon in column header
2. **Handler**: Parent component calls `handleAnalyzeExecutions()` or `handleAnalyzeValidations()`
3. **Creation**: `createAnalysis()` creates record, `runAnalysis()` spawns background task
4. **Refresh**: Analyses refetched after creation via `getAnalysesByRevision()`
5. **Display**: RevisionDetail renders AnalysisResult components
6. **Updates**: Results appear when background task completes
7. **Management**: Delete and re-run actions handled via parent component handlers

## Implementation Reference

**Backend:**

- `src-tauri/src/commands/analysis.rs` - Analysis commands
- `src-tauri/src/db/store.rs` - Analysis CRUD
- `src-tauri/src/db/migrations.rs` - Migration 13
- `src/lib/amp-executor.ts` - Amp SDK execution (shared with regular executions)

**Frontend:**

- `src/lib/ipc.ts` - IPC wrappers (`createAnalysis`, `runAnalysis`, `getAnalysesByRevision`, `deleteAnalysis`)
- `src/lib/types.ts` - TypeScript types
- `src/lib/components/ui/AnalysisResult.svelte` - Result display component with delete/rerun actions
- `src/lib/components/ui/RevisionDetail.svelte` - Trigger UI and results container
- `src/routes/promptsets/[id]/+page.svelte` - Handler implementation (`handleDeleteAnalysis`, `handleRerunAnalysis`) and data fetching

## Related Documentation

- **[Architecture](./architecture.md)** - Overall system design
- **[Executions](./executions.md)** - Execution lifecycle
- **[IPC Guide](./ipc-guide.md)** - API reference
- **[Settings](./settings.md)** - Amp token configuration
