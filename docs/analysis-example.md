# Failure Analysis with read_thread Tool

## How It Works

Maestro now uses the `read_thread` tool (available in Amp SDK) to extract failure information from execution threads.

### Flow

1. **Collect Thread URLs**: Maestro gathers thread URLs from failed executions
2. **Create Analysis Prompt**: Builds a prompt instructing Amp to use `read_thread` for each URL
3. **Execute with SDK**: Uses `@sourcegraph/amp-sdk` to run the analysis
4. **Extract Results**: Amp automatically uses `read_thread` tool to fetch thread content and analyzes failures

### Example Prompt Generated

```
You are analyzing failed execution threads to identify common failure patterns.

STEP 1: For EACH thread URL listed below, use the read_thread tool to extract:
- The original task/goal that was attempted
- All error messages and stack traces
- The failure cause and context
- Any relevant tool outputs or file contents

Thread URLs to analyze:
- https://ampcode.com/threads/T-abc123 (exec-001)
- https://ampcode.com/threads/T-def456 (exec-002)
- https://ampcode.com/threads/T-ghi789 (exec-003)

STEP 2: After reading ALL threads, create a comprehensive analysis that includes:
1. A markdown table categorizing failure patterns with columns: Pattern | Count | Example Thread | Root Cause
2. Specific, actionable suggestions to modify the original prompt to prevent these failures
3. Any common environmental or setup issues discovered

IMPORTANT:
- You MUST use read_thread for every URL listed above
- Do NOT write any files to disk
- Return your complete analysis as markdown
- Be specific about which execution IDs exhibited which patterns
```

### Implementation

The analysis runs via `amp-executor.ts` using the SDK:

```typescript
import { execute } from "@sourcegraph/amp-sdk"

for await (const message of execute({
	prompt: analysisPrompt, // Contains instructions to use read_thread
	options: {
		dangerouslyAllowAll: true,
		cwd: tempDir,
	},
})) {
	if (message.type === "result" && !message.is_error) {
		return message.result // Contains failure analysis
	}
}
```

### Benefits

- ✅ No OAuth credentials needed
- ✅ No manual HTTP API integration
- ✅ Leverages Amp's built-in `read_thread` tool
- ✅ Thread content extraction handled by Amp
- ✅ Simpler, more maintainable code
